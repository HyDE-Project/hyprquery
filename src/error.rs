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
