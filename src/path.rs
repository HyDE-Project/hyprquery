//! Path normalization and glob pattern resolution.
//!
//! This module handles path manipulation for configuration files:
//! - Tilde (`~`) expansion to home directory
//! - Environment variable expansion
//! - Relative to absolute path conversion
//! - Glob pattern matching for source directives

use std::path::{Path, PathBuf};

use crate::error::AppError;

/// Normalize and expand a file path
///
/// Handles:
/// - Environment variable expansion
/// - Tilde expansion for home directory
/// - Relative to absolute path conversion
/// - Path canonicalization
///
/// # Arguments
///
/// * `path` - Path string to normalize
///
/// # Returns
///
/// Normalized path as PathBuf
///
/// # Errors
///
/// Returns error if path cannot be resolved
pub fn normalize_path(path: &str) -> Result<PathBuf, AppError> {
    let mut processed = path.to_string();

    if (processed.starts_with('"') && processed.ends_with('"'))
        || (processed.starts_with('\'') && processed.ends_with('\''))
    {
        processed = processed[1..processed.len() - 1].to_string();
    }

    let expanded = shellexpand::full(&processed)
        .map_err(|e| AppError::path_resolution(&e.to_string()))?
        .to_string();

    let path_buf = PathBuf::from(&expanded);

    let absolute = if path_buf.is_relative() {
        std::env::current_dir()?.join(&path_buf)
    } else {
        path_buf
    };

    if absolute.exists() {
        absolute
            .canonicalize()
            .map_err(|e| AppError::path_resolution(&e.to_string()))
    } else {
        Ok(absolute)
    }
}

/// Resolve paths with glob pattern support
///
/// # Arguments
///
/// * `pattern` - Path pattern possibly containing glob wildcards
/// * `base_dir` - Base directory for relative paths
///
/// # Returns
///
/// Vector of resolved paths
///
/// # Errors
///
/// Returns error if glob pattern is invalid
pub fn resolve_glob(pattern: &str, base_dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    let expanded = shellexpand::full(pattern)
        .map_err(|e| AppError::path_resolution(&e.to_string()))?
        .to_string();

    let full_pattern = if expanded.starts_with('/') || expanded.starts_with('~') {
        expanded
    } else {
        base_dir.join(&expanded).display().to_string()
    };

    let paths: Vec<PathBuf> = glob::glob(&full_pattern)?.filter_map(Result::ok).collect();

    if paths.is_empty() {
        let fallback = PathBuf::from(&full_pattern);
        if fallback.parent().map(|p| p.exists()).unwrap_or(false) {
            return Ok(vec![fallback]);
        }
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_normalize_absolute_path() {
        let result = normalize_path("/tmp");
        assert!(result.is_ok());
        assert!(result.unwrap().is_absolute());
    }

    #[test]
    fn test_normalize_relative_path() {
        let result = normalize_path(".");
        assert!(result.is_ok());
        assert!(result.unwrap().is_absolute());
    }

    #[test]
    fn test_normalize_quoted_path() {
        let result = normalize_path("\"/tmp\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_normalize_single_quoted_path() {
        let result = normalize_path("'/tmp'");
        assert!(result.is_ok());
    }

    #[test]
    fn test_normalize_tilde_path() {
        let result = normalize_path("~");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.is_absolute());
        assert!(!path.to_string_lossy().contains('~'));
    }

    #[test]
    fn test_resolve_glob_no_match() {
        let result = resolve_glob("/nonexistent/*.conf", Path::new("/tmp"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_glob_with_files() {
        let temp_dir = std::env::temp_dir().join("hyprquery_test");
        let _ = fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("test.conf");
        let _ = fs::write(&test_file, "test");

        let result = resolve_glob("*.conf", &temp_dir);
        assert!(result.is_ok());
        let paths = result.unwrap();
        assert!(!paths.is_empty());

        let _ = fs::remove_file(test_file);
        let _ = fs::remove_dir(temp_dir);
    }

    #[test]
    fn test_resolve_glob_absolute_pattern() {
        let result = resolve_glob("/tmp/*.nonexistent", Path::new("/"));
        assert!(result.is_ok());
    }
}
