use serde_json::{Value, json};

use crate::query::{QueryInput, QueryResult};

/// Export results as JSON to stdout
///
/// # Arguments
///
/// * `results` - Vector of query results to export
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

/// Export results as environment variables to stdout
///
/// # Arguments
///
/// * `results` - Vector of query results to export
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

/// Export results as plain text with delimiter
///
/// # Arguments
///
/// * `results` - Vector of query results to export
/// * `delimiter` - Delimiter between values
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
