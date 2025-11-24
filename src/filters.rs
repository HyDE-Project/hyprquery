//! Value filtering for query results.
//!
//! This module provides type and regex filtering for configuration values.

use masterror::AppError;
use regex::Regex;

use crate::{error::from_regex, query::normalize_type};

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
pub fn apply_filters(
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
        let rx = Regex::new(pattern).map_err(from_regex)?;
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
