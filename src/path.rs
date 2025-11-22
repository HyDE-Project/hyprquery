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
