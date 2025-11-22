//! Output formatting and export functions for hydequery.
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
