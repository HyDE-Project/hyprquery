use std::fmt::{Display, Formatter, Result as FmtResult};

use masterror::{AppError as MasterError, AppErrorKind};

/// Application error wrapper for hyprquery
#[derive(Debug)]
pub struct AppError {
    inner: MasterError
}

impl AppError {
    /// Configuration file not found
    pub fn config_not_found(path: &str) -> Self {
        Self {
            inner: MasterError::new(
                AppErrorKind::NotFound,
                format!("Configuration file not found: {path}")
            )
        }
    }

    /// Schema file not found
    pub fn schema_not_found(path: &str) -> Self {
        Self {
            inner: MasterError::new(
                AppErrorKind::NotFound,
                format!("Schema file not found: {path}")
            )
        }
    }

    /// Config parse error
    pub fn config_parse(msg: &str) -> Self {
        Self {
            inner: MasterError::new(
                AppErrorKind::BadRequest,
                format!("Failed to parse configuration: {msg}")
            )
        }
    }

    /// Schema parse error
    pub fn schema_parse(msg: &str) -> Self {
        Self {
            inner: MasterError::new(
                AppErrorKind::BadRequest,
                format!("Failed to parse schema: {msg}")
            )
        }
    }

    /// Invalid query format
    pub fn invalid_query(msg: &str) -> Self {
        Self {
            inner: MasterError::new(
                AppErrorKind::BadRequest,
                format!("Invalid query format: {msg}")
            )
        }
    }

    /// IO error
    pub fn io(msg: &str) -> Self {
        Self {
            inner: MasterError::new(AppErrorKind::Internal, format!("IO error: {msg}"))
        }
    }

    /// Path resolution error
    pub fn path_resolution(msg: &str) -> Self {
        Self {
            inner: MasterError::new(
                AppErrorKind::BadRequest,
                format!("Path resolution error: {msg}")
            )
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.inner)
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::io(&err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::schema_parse(&err.to_string())
    }
}

impl From<glob::PatternError> for AppError {
    fn from(err: glob::PatternError) -> Self {
        Self::path_resolution(&err.to_string())
    }
}

impl From<regex::Error> for AppError {
    fn from(err: regex::Error) -> Self {
        Self::invalid_query(&err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_not_found() {
        let err = AppError::config_not_found("/test/path");
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_schema_not_found() {
        let err = AppError::schema_not_found("/schema/path");
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_config_parse() {
        let err = AppError::config_parse("syntax error");
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_invalid_query() {
        let err = AppError::invalid_query("bad format");
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_io_error() {
        let err = AppError::io("read failed");
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_path_resolution() {
        let err = AppError::path_resolution("invalid path");
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        let msg = app_err.to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_from_glob_error() {
        let glob_err = glob::Pattern::new("[").unwrap_err();
        let app_err: AppError = glob_err.into();
        let msg = app_err.to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_from_regex_error() {
        let regex_err = regex::Regex::new("[").unwrap_err();
        let app_err: AppError = regex_err.into();
        let msg = app_err.to_string();
        assert!(!msg.is_empty());
    }
}
