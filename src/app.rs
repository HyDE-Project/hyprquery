//! Core application logic for hydequery.
//!
//! This module contains the main execution flow, including:
//! - Configuration file parsing and validation
//! - Query processing and value resolution
//! - Type and regex filtering
//! - Result formatting and output
//!
//! The [`run`] function serves as the primary entry point for the application.

use std::{collections::HashSet, fs, path::PathBuf};

use clap::Parser;
use hyprlang::{Config, ConfigOptions};
use masterror::AppError;
use regex::Regex;

use crate::{
    cli::Args,
    error,
    export::{export_env, export_json, export_plain},
    fetch,
    path::normalize_path,
    query::{QueryResult, normalize_type, parse_query_inputs},
    schema,
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
pub fn run() -> Result<i32, AppError> {
    let args = Args::parse();

    if args.fetch_schema {
        let path = fetch::fetch_schema()?;
        println!("Schema cached at: {}", path.display());
        return Ok(0);
    }

    let config_path = normalize_path(&args.config_file)?;
    if !config_path.exists() {
        return Err(error::config_not_found(&config_path.display().to_string()));
    }

    let config_dir = match config_path.parent() {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from(".")
    };

    if args.get_defaults {
        return handle_get_defaults(&args);
    }

    let queries = parse_query_inputs(&args.queries);
    let has_dynamic = queries.iter().any(|q| q.is_dynamic_variable);

    let mut options = ConfigOptions::default();
    if args.source {
        options.base_dir = Some(config_dir.clone());
    }

    let mut config = Config::with_options(options);

    if has_dynamic {
        let mut content = fs::read_to_string(&config_path).map_err(error::from_io)?;

        for query in &queries {
            if query.is_dynamic_variable {
                let dyn_key = format!("Dynamic_{}", hash_string(&query.query));
                let dyn_line = format!("\n{}={}\n", dyn_key, query.query);
                content.push_str(&dyn_line);

                if args.debug {
                    eprintln!("[debug] Injecting line: {}", dyn_line.trim());
                }
            }
        }

        let parse_result = config.parse(&content);
        if let Err(e) = parse_result {
            if args.debug {
                eprintln!("[debug] Parse error: {}", e);
            }
            if args.strict {
                return Err(error::config_parse(&e.to_string()));
            }
        }
    } else {
        let parse_result = config.parse_file(&config_path);
        if let Err(e) = parse_result {
            if args.debug {
                eprintln!("[debug] Parse error: {}", e);
            }
            if args.strict {
                return Err(error::config_parse(&e.to_string()));
            }
        }
    }

    if let Some(ref schema_path_str) = args.schema {
        let schema_path = if schema_path_str == "auto" {
            fetch::resolve_schema_path(schema_path_str)?
        } else {
            normalize_path(schema_path_str)?
        };
        if !schema_path.exists() {
            return Err(error::schema_not_found(&schema_path.display().to_string()));
        }
        schema::load_schema(&mut config, &schema_path)?;
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
            eprintln!("[debug] Looking up key: {}", lookup_key);
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

/// Handle the `--get-defaults` flag to output all schema keys.
///
/// Reads the schema file and outputs all defined configuration keys,
/// either as plain text (one per line) or as JSON array.
///
/// # Arguments
///
/// * `args` - Parsed command-line arguments
///
/// # Returns
///
/// Always returns `Ok(0)` on success.
///
/// # Errors
///
/// Returns an error if schema file is not specified or cannot be read.
fn handle_get_defaults(args: &Args) -> Result<i32, AppError> {
    let schema_path = match &args.schema {
        Some(path) => {
            if path == "auto" {
                fetch::resolve_schema_path(path)?
            } else {
                normalize_path(path)?
            }
        }
        None => {
            return Err(error::schema_not_found(
                "Schema file required for --get-defaults"
            ));
        }
    };

    if !schema_path.exists() {
        return Err(error::schema_not_found(&schema_path.display().to_string()));
    }

    let keys = schema::get_schema_keys(&schema_path)?;

    match args.export.as_deref() {
        Some("json") => {
            let json_keys: Vec<serde_json::Value> = keys
                .iter()
                .map(|k| serde_json::Value::String(k.clone()))
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&json_keys).unwrap_or_default()
            );
        }
        _ => {
            for key in keys {
                println!("{}", key);
            }
        }
    }

    Ok(0)
}

/// Apply type and regex filters to a configuration value.
///
/// Validates that the value matches the expected type and regex pattern.
/// If validation fails, returns an empty string with NULL type.
///
/// # Arguments
///
/// * `value` - The resolved configuration value
/// * `type_str` - The actual type of the value (INT, FLOAT, STRING, etc.)
/// * `expected_type` - Optional expected type to match against
/// * `expected_regex` - Optional regex pattern the value must match
///
/// # Returns
///
/// A tuple of (filtered_value, type_str). If filters don't match,
/// returns ("", "NULL").
///
/// # Errors
///
/// Returns an error if the regex pattern is invalid.
fn apply_filters(
    value: String,
    type_str: &'static str,
    expected_type: &Option<String>,
    expected_regex: &Option<String>
) -> Result<(String, &'static str), AppError> {
    if let Some(exp_type) = expected_type
        && normalize_type(type_str) != normalize_type(exp_type)
    {
        return Ok((String::new(), "NULL"));
    }

    if let Some(pattern) = expected_regex {
        let rx = Regex::new(pattern).map_err(error::from_regex)?;
        if !rx.is_match(&value) {
            return Ok((String::new(), "NULL"));
        }
    }

    Ok((value, type_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_filters_no_filters() {
        let result = apply_filters("Gruvbox-Retro".to_string(), "STRING", &None, &None);
        assert!(result.is_ok());
        let (value, type_str) = result.unwrap();
        assert_eq!(value, "Gruvbox-Retro");
        assert_eq!(type_str, "STRING");
    }

    #[test]
    fn test_apply_filters_type_match_int() {
        let result = apply_filters("2".to_string(), "INT", &Some("INT".to_string()), &None);
        assert!(result.is_ok());
        let (value, type_str) = result.unwrap();
        assert_eq!(value, "2");
        assert_eq!(type_str, "INT");
    }

    #[test]
    fn test_apply_filters_type_mismatch() {
        let result = apply_filters(
            "Gruvbox-Retro".to_string(),
            "STRING",
            &Some("INT".to_string()),
            &None
        );
        assert!(result.is_ok());
        let (value, type_str) = result.unwrap();
        assert_eq!(value, "");
        assert_eq!(type_str, "NULL");
    }

    #[test]
    fn test_apply_filters_regex_match_cursor_size() {
        let result = apply_filters("20".to_string(), "INT", &None, &Some(r"^\d+$".to_string()));
        assert!(result.is_ok());
        let (value, type_str) = result.unwrap();
        assert_eq!(value, "20");
        assert_eq!(type_str, "INT");
    }

    #[test]
    fn test_apply_filters_regex_no_match() {
        let result = apply_filters(
            "prefer-dark".to_string(),
            "STRING",
            &None,
            &Some(r"^prefer-light$".to_string())
        );
        assert!(result.is_ok());
        let (value, type_str) = result.unwrap();
        assert_eq!(value, "");
        assert_eq!(type_str, "NULL");
    }

    #[test]
    fn test_apply_filters_type_and_regex_match() {
        let result = apply_filters(
            "3".to_string(),
            "INT",
            &Some("INT".to_string()),
            &Some(r"^[0-9]+$".to_string())
        );
        assert!(result.is_ok());
        let (value, type_str) = result.unwrap();
        assert_eq!(value, "3");
        assert_eq!(type_str, "INT");
    }

    #[test]
    fn test_apply_filters_invalid_regex() {
        let result = apply_filters("value".to_string(), "STRING", &None, &Some("[".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_filters_border_size() {
        let result = apply_filters(
            "2".to_string(),
            "INT",
            &Some("INT".to_string()),
            &Some(r"^[1-5]$".to_string())
        );
        assert!(result.is_ok());
        let (value, _) = result.unwrap();
        assert_eq!(value, "2");
    }

    #[test]
    fn test_apply_filters_gaps_value() {
        let result = apply_filters("8".to_string(), "INT", &Some("int".to_string()), &None);
        assert!(result.is_ok());
        let (value, type_str) = result.unwrap();
        assert_eq!(value, "8");
        assert_eq!(type_str, "INT");
    }

    #[test]
    fn test_apply_filters_theme_name_regex() {
        let result = apply_filters(
            "Gruvbox-Plus-Dark".to_string(),
            "STRING",
            &None,
            &Some(r"^Gruvbox.*$".to_string())
        );
        assert!(result.is_ok());
        let (value, _) = result.unwrap();
        assert_eq!(value, "Gruvbox-Plus-Dark");
    }

    #[test]
    fn test_apply_filters_color_scheme() {
        let result = apply_filters(
            "prefer-dark".to_string(),
            "STRING",
            &Some("STRING".to_string()),
            &Some(r"^prefer-(dark|light)$".to_string())
        );
        assert!(result.is_ok());
        let (value, _) = result.unwrap();
        assert_eq!(value, "prefer-dark");
    }

    #[test]
    fn test_apply_filters_rounding_value() {
        let result = apply_filters("3".to_string(), "INT", &None, &Some(r"^\d$".to_string()));
        assert!(result.is_ok());
        let (value, _) = result.unwrap();
        assert_eq!(value, "3");
    }
}
