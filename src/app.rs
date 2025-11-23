//! Core application logic for hydequery.
//!
//! This module contains the main execution flow, including:
//! - Configuration file parsing and validation
//! - Query processing and value resolution
//! - Type and regex filtering
//! - Result formatting and output
//!
//! The [`run`] function serves as the primary entry point for the application.

use std::{collections::HashSet, fs::read_to_string, path::PathBuf};

use clap::Parser;
use hyprlang::{Config, ConfigOptions};
use masterror::AppResult;

use crate::{
    cli::Args,
    defaults::handle_get_defaults,
    error::{config_not_found, config_parse, from_io, schema_not_found, validation_error},
    export::{export_env, export_json, export_plain},
    fetch::{fetch_schema, resolve_schema_path},
    filters::apply_filters,
    path::normalize_path,
    query::{QueryResult, parse_query_inputs},
    schema::load_schema,
    source::parse_sources_recursive,
    value::{config_value_to_string, config_value_type_name, hash_string}
};

/// Execute the main application logic.
///
/// Parses command-line arguments, loads configuration files, processes queries,
/// and outputs results in the requested format.
///
/// # Returns
///
/// - `Ok(0)` - All queries resolved successfully
/// - `Ok(1)` - One or more queries returned NULL (unless `--allow-missing` is
///   set)
/// - `Err(AppError)` - Fatal error occurred during execution
///
/// # Errors
///
/// Returns an error if:
/// - Configuration file not found or cannot be parsed
/// - Schema file not found or invalid
/// - Invalid regex pattern in query
pub fn run() -> AppResult<i32> {
    let args = Args::parse();
    run_with_args(args)
}

/// Execute application logic with provided arguments.
///
/// This is the core implementation that can be tested directly.
///
/// # Arguments
///
/// * `args` - Parsed command-line arguments
///
/// # Returns
///
/// - `Ok(0)` - All queries resolved successfully
/// - `Ok(1)` - One or more queries returned NULL (unless `--allow-missing` is
///   set)
/// - `Err(AppError)` - Fatal error occurred during execution
pub fn run_with_args(args: Args) -> AppResult<i32> {
    if args.fetch_schema {
        let path = fetch_schema()?;
        println!("Schema cached at: {}", path.display());
        return Ok(0);
    }

    let config_file = match &args.config_file {
        Some(path) => path,
        None => return Err(validation_error("Configuration file is required"))
    };

    let config_path = normalize_path(config_file)?;
    if !config_path.exists() {
        return Err(config_not_found(&config_path.display().to_string()));
    }

    let config_dir = match config_path.parent() {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from(".")
    };

    if args.get_defaults {
        return handle_get_defaults(&args);
    }

    if args.queries.is_empty() {
        return Err(validation_error(
            "No queries specified. Use -Q to specify queries"
        ));
    }

    let queries = parse_query_inputs(&args.queries);
    let has_dynamic = queries.iter().any(|q| q.is_dynamic_variable);

    let mut options = ConfigOptions::default();
    if args.source {
        options.base_dir = Some(config_dir.clone());
    }

    let mut config = Config::with_options(options);

    if has_dynamic {
        let mut content = read_to_string(&config_path).map_err(from_io)?;

        for query in &queries {
            if query.is_dynamic_variable {
                let dyn_key = format!("Dynamic_{}", hash_string(&query.query));
                let dyn_line = format!("\n{dyn_key}={}\n", query.query);
                content.push_str(&dyn_line);

                if args.debug {
                    eprintln!("[debug] Injecting line: {}", dyn_line.trim());
                }
            }
        }

        let parse_result = config.parse(&content);
        if let Err(e) = parse_result {
            if args.debug {
                eprintln!("[debug] Parse error: {e}");
            }
            if args.strict {
                return Err(config_parse(&e.to_string()));
            }
        }
    } else {
        let parse_result = config.parse_file(&config_path);
        if let Err(e) = parse_result {
            if args.debug {
                eprintln!("[debug] Parse error: {e}");
            }
            if args.strict {
                return Err(config_parse(&e.to_string()));
            }
        }
    }

    if let Some(ref schema_path_str) = args.schema {
        let schema_path = if schema_path_str == "auto" {
            resolve_schema_path(schema_path_str)?
        } else {
            normalize_path(schema_path_str)?
        };
        if !schema_path.exists() {
            return Err(schema_not_found(&schema_path.display().to_string()));
        }
        load_schema(&mut config, &schema_path)?;
    }

    if args.source {
        let mut visited = HashSet::new();
        visited.insert(config_path.clone());
        parse_sources_recursive(
            &mut config,
            &config_path,
            &config_dir,
            &mut visited,
            args.debug
        )?;
    }

    let mut results = Vec::with_capacity(queries.len());

    for query in &queries {
        let lookup_key = if query.is_dynamic_variable {
            format!("Dynamic_{}", hash_string(&query.query))
        } else {
            query.query.clone()
        };

        if args.debug {
            eprintln!("[debug] Looking up key: {lookup_key}");
        }

        let (value_str, type_str) = match config.get(&lookup_key) {
            Ok(value) => {
                let v = config_value_to_string(value);
                let t = config_value_type_name(value);

                if query.is_dynamic_variable && v == query.query {
                    (String::new(), "NULL")
                } else {
                    (v, t)
                }
            }
            Err(_) => {
                if query.is_dynamic_variable {
                    match config.get_variable(&query.query[1..]) {
                        Some(var_value) => (var_value.to_string(), "STRING"),
                        None => (String::new(), "NULL")
                    }
                } else {
                    (String::new(), "NULL")
                }
            }
        };

        let (final_value, final_type) = apply_filters(
            value_str,
            type_str,
            &query.expected_type,
            &query.expected_regex
        )?;

        results.push(QueryResult {
            key:        query.query.clone().into_boxed_str(),
            value:      final_value.into_boxed_str(),
            value_type: final_type
        });
    }

    let null_count = results.iter().filter(|r| r.value_type == "NULL").count();

    match args.export.as_deref() {
        Some("json") => export_json(&results),
        Some("env") => export_env(&results, &queries),
        _ => export_plain(&results, &args.delimiter)
    }

    if null_count > 0 && !args.allow_missing {
        Ok(1)
    } else {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use super::*;

    fn create_test_config(name: &str, content: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir().join("hydequery_app_test");
        let _ = fs::create_dir_all(&temp_dir);
        let path = temp_dir.join(name);
        let mut file = fs::File::create(&path).unwrap();
        write!(file, "{}", content).unwrap();
        path
    }

    fn make_args(config_file: &str, queries: Vec<&str>) -> Args {
        Args {
            help:          false,
            config_file:   Some(config_file.to_string()),
            queries:       queries.into_iter().map(String::from).collect(),
            schema:        None,
            fetch_schema:  false,
            allow_missing: false,
            get_defaults:  false,
            strict:        false,
            export:        None,
            source:        false,
            debug:         false,
            delimiter:     "\n".to_string()
        }
    }

    #[test]
    fn test_run_with_args_theme_variables() {
        let content = r#"
$GTK_THEME = Gruvbox-Retro
$ICON_THEME = Gruvbox-Plus-Dark
$CURSOR_SIZE = 20
"#;
        let path = create_test_config("theme_run.conf", content);
        let args = make_args(
            path.to_str().unwrap(),
            vec!["$GTK_THEME", "$ICON_THEME", "$CURSOR_SIZE"]
        );

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_general_settings() {
        let content = r#"
general {
    gaps_in = 3
    gaps_out = 8
    border_size = 2
}
"#;
        let path = create_test_config("general_run.conf", content);
        let args = make_args(
            path.to_str().unwrap(),
            vec!["general:gaps_in", "general:border_size"]
        );

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_missing_variable() {
        let content = "$GTK_THEME = Gruvbox-Retro";
        let path = create_test_config("missing_run.conf", content);
        let args = make_args(path.to_str().unwrap(), vec!["$FONT"]);

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_allow_missing() {
        let content = "$GTK_THEME = Gruvbox-Retro";
        let path = create_test_config("allow_missing_run.conf", content);
        let mut args = make_args(path.to_str().unwrap(), vec!["$FONT"]);
        args.allow_missing = true;

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_json_export() {
        let content = "$GTK_THEME = Gruvbox-Retro";
        let path = create_test_config("json_run.conf", content);
        let mut args = make_args(path.to_str().unwrap(), vec!["$GTK_THEME"]);
        args.export = Some("json".to_string());

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_env_export() {
        let content = r#"
$GTK_THEME = Gruvbox-Retro
$CURSOR_SIZE = 20
"#;
        let path = create_test_config("env_run.conf", content);
        let mut args = make_args(path.to_str().unwrap(), vec!["$GTK_THEME", "$CURSOR_SIZE"]);
        args.export = Some("env".to_string());

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_config_not_found() {
        let args = make_args("/nonexistent/path.conf", vec!["$GTK_THEME"]);
        let result = run_with_args(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_with_args_source_directive() {
        let temp_dir = std::env::temp_dir().join("hydequery_source_run");
        let _ = fs::create_dir_all(&temp_dir);

        let theme_path = temp_dir.join("theme.conf");
        let mut theme_file = fs::File::create(&theme_path).unwrap();
        writeln!(theme_file, "$GTK_THEME = Wallbash-Gtk").unwrap();

        let main_path = temp_dir.join("main.conf");
        let mut main_file = fs::File::create(&main_path).unwrap();
        writeln!(main_file, "source = {}", theme_path.display()).unwrap();

        let mut args = make_args(main_path.to_str().unwrap(), vec!["$GTK_THEME"]);
        args.source = true;

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(theme_path);
        let _ = fs::remove_file(main_path);
        let _ = fs::remove_dir(temp_dir);
    }

    #[test]
    fn test_run_with_args_custom_delimiter() {
        let content = r#"
$GTK_THEME = Gruvbox-Retro
$ICON_THEME = Gruvbox-Plus-Dark
"#;
        let path = create_test_config("delim_run.conf", content);
        let mut args = make_args(path.to_str().unwrap(), vec!["$GTK_THEME", "$ICON_THEME"]);
        args.delimiter = ",".to_string();

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_strict_mode() {
        let content = "$GTK_THEME = Gruvbox-Retro";
        let path = create_test_config("strict_run.conf", content);
        let mut args = make_args(path.to_str().unwrap(), vec!["$GTK_THEME"]);
        args.strict = true;

        let result = run_with_args(args);
        assert!(result.is_ok());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_debug_mode() {
        let content = "$GTK_THEME = Gruvbox-Retro";
        let path = create_test_config("debug_run.conf", content);
        let mut args = make_args(path.to_str().unwrap(), vec!["$GTK_THEME"]);
        args.debug = true;

        let result = run_with_args(args);
        assert!(result.is_ok());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_decoration_settings() {
        let content = r#"
decoration {
    rounding = 3
    blur {
        enabled = yes
        size = 4
    }
}
"#;
        let path = create_test_config("decoration_run.conf", content);
        let args = make_args(path.to_str().unwrap(), vec!["decoration:rounding"]);

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_color_scheme() {
        let content = "$COLOR_SCHEME = prefer-dark";
        let path = create_test_config("color_run.conf", content);
        let args = make_args(path.to_str().unwrap(), vec!["$COLOR_SCHEME"]);

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_cursor_theme() {
        let content = r#"
$CURSOR_THEME = Bibata-Modern-Ice
$CURSOR_SIZE = 24
"#;
        let path = create_test_config("cursor_run.conf", content);
        let args = make_args(
            path.to_str().unwrap(),
            vec!["$CURSOR_THEME", "$CURSOR_SIZE"]
        );

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_multiple_missing() {
        let content = "$GTK_THEME = Gruvbox-Retro";
        let path = create_test_config("multi_missing_run.conf", content);
        let args = make_args(path.to_str().unwrap(), vec!["$FONT", "$SDDM_THEME"]);

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_with_type_filter() {
        let content = r#"
general {
    border_size = 2
    gaps_in = 3
}
"#;
        let path = create_test_config("type_filter_run.conf", content);
        let args = make_args(path.to_str().unwrap(), vec!["general:border_size[INT]"]);

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_with_regex_filter() {
        let content = "$GTK_THEME = Gruvbox-Retro";
        let path = create_test_config("regex_filter_run.conf", content);
        let args = make_args(
            path.to_str().unwrap(),
            vec!["$GTK_THEME[STRING][^Gruvbox.*$]"]
        );

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_type_mismatch_returns_null() {
        let content = "$GTK_THEME = Gruvbox-Retro";
        let path = create_test_config("type_mismatch_run.conf", content);
        let args = make_args(path.to_str().unwrap(), vec!["$GTK_THEME[INT]"]);

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_regex_no_match_returns_null() {
        let content = "$GTK_THEME = Gruvbox-Retro";
        let path = create_test_config("regex_nomatch_run.conf", content);
        let args = make_args(
            path.to_str().unwrap(),
            vec!["$GTK_THEME[STRING][^Adwaita$]"]
        );

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_mixed_found_and_missing() {
        let content = r#"
$GTK_THEME = Gruvbox-Retro
$CURSOR_SIZE = 20
"#;
        let path = create_test_config("mixed_run.conf", content);
        let args = make_args(
            path.to_str().unwrap(),
            vec!["$GTK_THEME", "$FONT", "$CURSOR_SIZE"]
        );

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_nested_key_not_found() {
        let content = r#"
general {
    gaps_in = 3
}
"#;
        let path = create_test_config("nested_missing_run.conf", content);
        let args = make_args(path.to_str().unwrap(), vec!["general:nonexistent"]);

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_empty_config() {
        let content = "";
        let path = create_test_config("empty_run.conf", content);
        let args = make_args(path.to_str().unwrap(), vec!["$GTK_THEME"]);

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_source_with_comment() {
        let temp_dir = std::env::temp_dir().join("hydequery_source_comment");
        let _ = fs::create_dir_all(&temp_dir);

        let theme_path = temp_dir.join("theme.conf");
        let mut theme_file = fs::File::create(&theme_path).unwrap();
        writeln!(theme_file, "$GTK_THEME = Wallbash-Gtk").unwrap();

        let main_path = temp_dir.join("main.conf");
        let mut main_file = fs::File::create(&main_path).unwrap();
        writeln!(
            main_file,
            "source = {} # theme settings",
            theme_path.display()
        )
        .unwrap();

        let mut args = make_args(main_path.to_str().unwrap(), vec!["$GTK_THEME"]);
        args.source = true;

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(theme_path);
        let _ = fs::remove_file(main_path);
        let _ = fs::remove_dir(temp_dir);
    }

    #[test]
    fn test_run_with_args_parent_dir_fallback() {
        let content = "$GTK_THEME = Gruvbox-Retro";
        let path = create_test_config("parent_run.conf", content);
        let args = make_args(path.to_str().unwrap(), vec!["$GTK_THEME"]);

        let result = run_with_args(args);
        assert!(result.is_ok());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_get_defaults_no_schema() {
        let content = "$GTK_THEME = Gruvbox-Retro";
        let path = create_test_config("defaults_no_schema.conf", content);
        let mut args = make_args(path.to_str().unwrap(), vec!["$GTK_THEME"]);
        args.get_defaults = true;

        let result = run_with_args(args);
        assert!(result.is_err());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_schema_not_found() {
        let content = "$GTK_THEME = Gruvbox-Retro";
        let path = create_test_config("schema_missing.conf", content);
        let mut args = make_args(path.to_str().unwrap(), vec!["$GTK_THEME"]);
        args.schema = Some("/nonexistent/schema.json".to_string());

        let result = run_with_args(args);
        assert!(result.is_err());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_dynamic_variable_not_resolved() {
        let content = "";
        let path = create_test_config("dynamic_unresolved.conf", content);
        let args = make_args(path.to_str().unwrap(), vec!["$NONEXISTENT_VAR"]);

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_static_key_with_source() {
        let temp_dir = std::env::temp_dir().join("hydequery_static_source");
        let _ = fs::create_dir_all(&temp_dir);

        let theme_path = temp_dir.join("theme.conf");
        let mut theme_file = fs::File::create(&theme_path).unwrap();
        writeln!(theme_file, "general {{").unwrap();
        writeln!(theme_file, "    border_size = 2").unwrap();
        writeln!(theme_file, "}}").unwrap();

        let main_path = temp_dir.join("main.conf");
        let mut main_file = fs::File::create(&main_path).unwrap();
        writeln!(main_file, "source = {}", theme_path.display()).unwrap();

        let mut args = make_args(main_path.to_str().unwrap(), vec!["general:border_size"]);
        args.source = true;

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(theme_path);
        let _ = fs::remove_file(main_path);
        let _ = fs::remove_dir(temp_dir);
    }

    #[test]
    fn test_run_with_args_multiple_queries_all_found() {
        let content = r#"
$GTK_THEME = Gruvbox-Retro
$ICON_THEME = Gruvbox-Plus-Dark
$CURSOR_THEME = Bibata-Modern-Ice
$CURSOR_SIZE = 24
$COLOR_SCHEME = prefer-dark
"#;
        let path = create_test_config("all_found.conf", content);
        let args = make_args(
            path.to_str().unwrap(),
            vec![
                "$GTK_THEME",
                "$ICON_THEME",
                "$CURSOR_THEME",
                "$CURSOR_SIZE",
                "$COLOR_SCHEME",
            ]
        );

        let result = run_with_args(args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_type_filter_float() {
        let content = r#"
decoration {
    active_opacity = 0.95
}
"#;
        let path = create_test_config("float_type.conf", content);
        let args = make_args(
            path.to_str().unwrap(),
            vec!["decoration:active_opacity[FLOAT]"]
        );

        let result = run_with_args(args);
        assert!(result.is_ok());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_run_with_args_variable_value_equals_query() {
        let content = "$TEST = $TEST";
        let path = create_test_config("var_equals_query.conf", content);
        let args = make_args(path.to_str().unwrap(), vec!["$TEST"]);

        let result = run_with_args(args);
        assert!(result.is_ok());

        let _ = fs::remove_file(path);
    }
}
