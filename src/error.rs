//! Error types and handling for hydequery.
//!
//! This module provides error handling using the `masterror` crate's
//! builder pattern API. All errors are categorized by kind and include
//! descriptive messages for debugging.
//!
//! # Error Kinds
//!
//! - `NotFound` - Configuration or schema file not found
//! - `BadRequest` - Invalid input (parse errors, invalid queries)
//! - `Internal` - IO and other internal errors

use masterror::prelude::*;

/// Create a "configuration file not found" error.
///
/// # Arguments
///
/// * `path` - Path to the missing configuration file
pub fn config_not_found(path: &str) -> AppError {
    AppError::not_found(format!("Configuration file not found: {path}"))
}

/// Create a "schema file not found" error.
///
/// # Arguments
///
/// * `path` - Path to the missing schema file
pub fn schema_not_found(path: &str) -> AppError {
    AppError::not_found(format!("Schema file not found: {path}"))
}

/// Create a configuration parse error.
///
/// # Arguments
///
/// * `msg` - Parse error message
pub fn config_parse(msg: &str) -> AppError {
    AppError::bad_request(format!("Failed to parse configuration: {msg}"))
}

/// Create a schema parse error.
///
/// # Arguments
///
/// * `msg` - Parse error message
pub fn schema_parse(msg: &str) -> AppError {
    AppError::bad_request(format!("Failed to parse schema: {msg}"))
}

/// Create an invalid query error.
///
/// # Arguments
///
/// * `msg` - Error message describing the invalid query
pub fn invalid_query(msg: &str) -> AppError {
    AppError::bad_request(format!("Invalid query format: {msg}"))
}

/// Create an IO error.
///
/// # Arguments
///
/// * `msg` - IO error message
pub fn io_error(msg: &str) -> AppError {
    AppError::internal(format!("IO error: {msg}"))
}

/// Create a path resolution error.
///
/// # Arguments
///
/// * `msg` - Path resolution error message
pub fn path_resolution(msg: &str) -> AppError {
    AppError::bad_request(format!("Path resolution error: {msg}"))
}

/// Convert std::io::Error to AppError.
pub fn from_io(err: std::io::Error) -> AppError {
    io_error(&err.to_string())
}

/// Convert serde_json::Error to AppError.
pub fn from_json(err: serde_json::Error) -> AppError {
    schema_parse(&err.to_string())
}

/// Convert glob::PatternError to AppError.
pub fn from_glob(err: glob::PatternError) -> AppError {
    path_resolution(&err.to_string())
}

/// Convert regex::Error to AppError.
pub fn from_regex(err: regex::Error) -> AppError {
    invalid_query(&err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_not_found() {
        let err = config_not_found("/test/path");
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_schema_not_found() {
        let err = schema_not_found("/schema/path");
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_config_parse() {
        let err = config_parse("syntax error");
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_invalid_query() {
        let err = invalid_query("bad format");
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_io_error() {
        let err = io_error("read failed");
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_path_resolution() {
        let err = path_resolution("invalid path");
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err = from_io(io_err);
        assert!(!app_err.to_string().is_empty());
    }

    #[test]
    fn test_from_glob_error() {
        let glob_err = glob::Pattern::new("[").unwrap_err();
        let app_err = from_glob(glob_err);
        assert!(!app_err.to_string().is_empty());
    }

    #[test]
    fn test_from_regex_error() {
        let regex_err = regex::Regex::new("[").unwrap_err();
        let app_err = from_regex(regex_err);
        assert!(!app_err.to_string().is_empty());
    }
}
