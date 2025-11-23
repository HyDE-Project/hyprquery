//! Schema defaults output functionality.
//!
//! This module handles the `--get-defaults` flag to output all schema keys.

use masterror::AppResult;
use serde_json::{Value, to_string_pretty};

use crate::{
    cli::Args, error::schema_not_found, fetch::resolve_schema_path, path::normalize_path,
    schema::get_schema_keys
};

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
pub fn handle_get_defaults(args: &Args) -> AppResult<i32> {
    let schema_path = match &args.schema {
        Some(path) => {
            if path == "auto" {
                resolve_schema_path(path)?
            } else {
                normalize_path(path)?
            }
        }
        None => {
            return Err(schema_not_found("Schema file required for --get-defaults"));
        }
    };

    if !schema_path.exists() {
        return Err(schema_not_found(&schema_path.display().to_string()));
    }

    let keys = get_schema_keys(&schema_path)?;

    match args.export.as_deref() {
        Some("json") => {
            let json_keys: Vec<Value> = keys.iter().map(|k| Value::String(k.clone())).collect();
            println!("{}", to_string_pretty(&json_keys).unwrap_or_default());
        }
        _ => {
            for key in keys {
                println!("{key}");
            }
        }
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write, path::PathBuf};

    use super::*;

    fn create_test_schema(name: &str, content: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir().join("hydequery_defaults_test");
        let _ = fs::create_dir_all(&temp_dir);
        let path = temp_dir.join(name);
        let mut file = fs::File::create(&path).unwrap();
        write!(file, "{}", content).unwrap();
        path
    }

    fn make_args_with_schema(schema_path: &str) -> Args {
        Args {
            help:          false,
            config_file:   Some("/tmp/dummy.conf".to_string()),
            queries:       vec![],
            schema:        Some(schema_path.to_string()),
            fetch_schema:  false,
            allow_missing: false,
            get_defaults:  true,
            strict:        false,
            export:        None,
            source:        false,
            debug:         false,
            delimiter:     "\n".to_string()
        }
    }

    #[test]
    fn test_handle_get_defaults_no_schema() {
        let args = Args {
            help:          false,
            config_file:   Some("/tmp/dummy.conf".to_string()),
            queries:       vec![],
            schema:        None,
            fetch_schema:  false,
            allow_missing: false,
            get_defaults:  true,
            strict:        false,
            export:        None,
            source:        false,
            debug:         false,
            delimiter:     "\n".to_string()
        };

        let result = handle_get_defaults(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_get_defaults_schema_not_found() {
        let args = make_args_with_schema("/nonexistent/schema.json");
        let result = handle_get_defaults(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_get_defaults_valid_schema() {
        let schema_content = r#"{
            "hyprlang_schema": [
                { "value": "general:border_size", "type": "INT", "data": { "default": 2 } },
                { "value": "general:gaps_in", "type": "INT", "data": { "default": 3 } },
                { "value": "decoration:rounding", "type": "INT", "data": { "default": 3 } }
            ]
        }"#;
        let path = create_test_schema("defaults.json", schema_content);

        let args = make_args_with_schema(path.to_str().unwrap());
        let result = handle_get_defaults(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_handle_get_defaults_json_export() {
        let schema_content = r#"{
            "hyprlang_schema": [
                { "value": "general:border_size", "type": "INT", "data": { "default": 2 } }
            ]
        }"#;
        let path = create_test_schema("defaults_json.json", schema_content);

        let mut args = make_args_with_schema(path.to_str().unwrap());
        args.export = Some("json".to_string());

        let result = handle_get_defaults(&args);
        assert!(result.is_ok());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_handle_get_defaults_multiple_keys() {
        let schema_content = r#"{
            "hyprlang_schema": [
                { "value": "general:border_size", "type": "INT", "data": { "default": 2 } },
                { "value": "general:gaps_in", "type": "INT", "data": { "default": 3 } },
                { "value": "general:gaps_out", "type": "INT", "data": { "default": 8 } },
                { "value": "decoration:rounding", "type": "INT", "data": { "default": 3 } },
                { "value": "decoration:active_opacity", "type": "FLOAT", "data": { "default": 1.0 } }
            ]
        }"#;
        let path = create_test_schema("defaults_multi.json", schema_content);

        let args = make_args_with_schema(path.to_str().unwrap());
        let result = handle_get_defaults(&args);
        assert!(result.is_ok());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_handle_get_defaults_empty_schema() {
        let schema_content = r#"{ "hyprlang_schema": [] }"#;
        let path = create_test_schema("defaults_empty.json", schema_content);

        let args = make_args_with_schema(path.to_str().unwrap());
        let result = handle_get_defaults(&args);
        assert!(result.is_ok());

        let _ = fs::remove_file(path);
    }
}
