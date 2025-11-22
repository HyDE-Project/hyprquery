use std::{collections::HashSet, fs, path::PathBuf};

use clap::Parser;
use hyprlang::{Config, ConfigOptions};
use regex::Regex;

use crate::{
    cli::Args,
    error::AppError,
    export::{export_env, export_json, export_plain},
    path::normalize_path,
    query::{QueryResult, normalize_type, parse_query_inputs},
    schema,
    source::parse_sources_recursive,
    value::{config_value_to_string, config_value_type_name, hash_string}
};

/// Main application logic
pub fn run() -> Result<i32, AppError> {
    let args = Args::parse();

    let config_path = normalize_path(&args.config_file)?;
    if !config_path.exists() {
        return Err(AppError::config_not_found(
            &config_path.display().to_string()
        ));
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
        let mut content = fs::read_to_string(&config_path)?;

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
                return Err(AppError::config_parse(&e.to_string()));
            }
        }
    } else {
        let parse_result = config.parse_file(&config_path);
        if let Err(e) = parse_result {
            if args.debug {
                eprintln!("[debug] Parse error: {}", e);
            }
            if args.strict {
                return Err(AppError::config_parse(&e.to_string()));
            }
        }
    }

    if let Some(ref schema_path_str) = args.schema {
        let schema_path = normalize_path(schema_path_str)?;
        if !schema_path.exists() {
            return Err(AppError::schema_not_found(
                &schema_path.display().to_string()
            ));
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

/// Handle --get-defaults flag to output all schema keys
fn handle_get_defaults(args: &Args) -> Result<i32, AppError> {
    let schema_path = match &args.schema {
        Some(path) => normalize_path(path)?,
        None => {
            return Err(AppError::schema_not_found(
                "Schema file required for --get-defaults"
            ));
        }
    };

    if !schema_path.exists() {
        return Err(AppError::schema_not_found(
            &schema_path.display().to_string()
        ));
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

/// Apply type and regex filters to a value
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
        let rx = Regex::new(pattern)?;
        if !rx.is_match(&value) {
            return Ok((String::new(), "NULL"));
        }
    }

    Ok((value, type_str))
}
