//! Output formatting and export functions for hyprquery.
//!
//! This module provides functions to format query results in various formats:
//! - Plain text with configurable delimiter
//! - JSON (single object or array)
//! - Environment variable assignments
//!
//! All output is written to stdout.

use serde_json::{Value, json};

use crate::query::{QueryInput, QueryResult};

/// Export results as JSON to stdout.
///
/// Outputs a single JSON object for single results, or an array for multiple.
/// NULL values are represented as JSON `null`.
///
/// # Arguments
///
/// * `results` - Query results to export
pub fn export_json(results: &[QueryResult]) {
    let json_results: Vec<Value> = results
        .iter()
        .map(|r| {
            json!({
                "key": &*r.key,
                "value": if r.value_type == "NULL" { Value::Null } else { Value::String(r.value.to_string()) },
                "type": r.value_type
            })
        })
        .collect();

    if results.len() == 1 {
        println!(
            "{}",
            serde_json::to_string_pretty(&json_results[0]).unwrap_or_default()
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&json_results).unwrap_or_default()
        );
    }
}

/// Export results as shell environment variable assignments.
///
/// Converts query keys to valid variable names by:
/// - Removing leading `$` from dynamic variables
/// - Replacing `:` with `_`
/// - Converting to uppercase
///
/// NULL values are skipped (no output).
///
/// # Arguments
///
/// * `results` - Query results to export
/// * `queries` - Original query inputs for variable naming
pub fn export_env(results: &[QueryResult], queries: &[QueryInput]) {
    for (i, result) in results.iter().enumerate() {
        let var_name = if i < queries.len() {
            let name = &queries[i].query;
            let name = name.strip_prefix('$').unwrap_or(name);
            name.replace(':', "_").to_uppercase()
        } else {
            result.key.replace(':', "_").to_uppercase()
        };

        if result.value_type != "NULL" {
            println!("{}=\"{}\"", var_name, result.value);
        }
    }
}

/// Export results as plain text with configurable delimiter.
///
/// Each value is output separated by the specified delimiter.
/// NULL values are represented as empty strings.
///
/// # Arguments
///
/// * `results` - Query results to export
/// * `delimiter` - String to insert between values (default: newline)
pub fn export_plain(results: &[QueryResult], delimiter: &str) {
    let output: Vec<&str> = results
        .iter()
        .map(|r| {
            if r.value_type == "NULL" {
                ""
            } else {
                &*r.value
            }
        })
        .collect();

    println!("{}", output.join(delimiter));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(key: &str, value: &str, value_type: &'static str) -> QueryResult {
        QueryResult {
            key: key.to_string().into_boxed_str(),
            value: value.to_string().into_boxed_str(),
            value_type
        }
    }

    fn make_query(query: &str, is_dynamic: bool) -> QueryInput {
        QueryInput {
            query:               query.to_string(),
            expected_type:       None,
            expected_regex:      None,
            is_dynamic_variable: is_dynamic
        }
    }

    #[test]
    fn test_export_json_single_border_size() {
        let results = vec![make_result("general:border_size", "2", "INT")];
        export_json(&results);
    }

    #[test]
    fn test_export_json_multiple_theme_vars() {
        let results = vec![
            make_result("$GTK_THEME", "Gruvbox-Retro", "STRING"),
            make_result("$ICON_THEME", "Gruvbox-Plus-Dark", "STRING"),
            make_result("$CURSOR_SIZE", "20", "INT"),
        ];
        export_json(&results);
    }

    #[test]
    fn test_export_json_null_missing_var() {
        let results = vec![make_result("$FONT", "", "NULL")];
        export_json(&results);
    }

    #[test]
    fn test_export_json_hyprland_settings() {
        let results = vec![
            make_result("general:gaps_in", "3", "INT"),
            make_result("general:gaps_out", "8", "INT"),
            make_result("decoration:rounding", "3", "INT"),
        ];
        export_json(&results);
    }

    #[test]
    fn test_export_env_gtk_theme() {
        let results = vec![make_result("$GTK_THEME", "Gruvbox-Retro", "STRING")];
        let queries = vec![make_query("$GTK_THEME", true)];
        export_env(&results, &queries);
    }

    #[test]
    fn test_export_env_cursor_settings() {
        let results = vec![
            make_result("$CURSOR_THEME", "Gruvbox-Retro", "STRING"),
            make_result("$CURSOR_SIZE", "20", "INT"),
        ];
        let queries = vec![
            make_query("$CURSOR_THEME", true),
            make_query("$CURSOR_SIZE", true),
        ];
        export_env(&results, &queries);
    }

    #[test]
    fn test_export_env_nested_general_settings() {
        let results = vec![make_result("general:border_size", "2", "INT")];
        let queries = vec![make_query("general:border_size", false)];
        export_env(&results, &queries);
    }

    #[test]
    fn test_export_env_null_font_skipped() {
        let results = vec![make_result("$FONT", "", "NULL")];
        let queries = vec![make_query("$FONT", true)];
        export_env(&results, &queries);
    }

    #[test]
    fn test_export_env_color_scheme() {
        let results = vec![make_result("$COLOR_SCHEME", "prefer-dark", "STRING")];
        let queries = vec![make_query("$COLOR_SCHEME", true)];
        export_env(&results, &queries);
    }

    #[test]
    fn test_export_env_fallback_to_result_key() {
        let results = vec![
            make_result("$GTK_THEME", "Gruvbox-Retro", "STRING"),
            make_result("general:layout", "dwindle", "STRING"),
        ];
        let queries = vec![make_query("$GTK_THEME", true)];
        export_env(&results, &queries);
    }

    #[test]
    fn test_export_plain_single_theme() {
        let results = vec![make_result("$GTK_THEME", "Gruvbox-Retro", "STRING")];
        export_plain(&results, "\n");
    }

    #[test]
    fn test_export_plain_multiple_gaps() {
        let results = vec![
            make_result("general:gaps_in", "3", "INT"),
            make_result("general:gaps_out", "8", "INT"),
        ];
        export_plain(&results, ",");
    }

    #[test]
    fn test_export_plain_with_missing_sddm() {
        let results = vec![
            make_result("$GTK_THEME", "Gruvbox-Retro", "STRING"),
            make_result("$SDDM_THEME", "", "NULL"),
        ];
        export_plain(&results, "\n");
    }

    #[test]
    fn test_export_plain_all_theme_vars() {
        let results = vec![
            make_result("$GTK_THEME", "Gruvbox-Retro", "STRING"),
            make_result("$ICON_THEME", "Gruvbox-Plus-Dark", "STRING"),
            make_result("$CURSOR_THEME", "Gruvbox-Retro", "STRING"),
            make_result("$CURSOR_SIZE", "20", "INT"),
            make_result("$COLOR_SCHEME", "prefer-dark", "STRING"),
        ];
        export_plain(&results, "\n");
    }
}
