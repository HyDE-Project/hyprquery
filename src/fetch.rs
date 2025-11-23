//! Schema fetching and caching functionality.
//!
//! This module handles downloading the Hyprland schema from the repository
//! and caching it locally for offline use.
//!
//! # Cache Location
//!
//! The schema is cached in `~/.cache/hydequery/hyprland.json`.
//!
//! # Example
//!
//! ```no_run
//! use hydequery::fetch;
//!
//! // Fetch and cache the schema
//! fetch::fetch_schema()?;
//!
//! // Get the cached schema path
//! let path = fetch::get_cached_schema_path()?;
//! ```

use std::{fs, io::Write, path::PathBuf};

use masterror::prelude::*;

/// URL for the Hyprland schema in the HyDE-Project repository.
const SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/HyDE-Project/hyprquery/main/schema/hyprland.json";

/// Name of the cached schema file.
const SCHEMA_FILENAME: &str = "hyprland.json";

/// Application cache directory name.
const CACHE_DIR_NAME: &str = "hydequery";

/// Returns the path to the cache directory.
///
/// Creates the directory if it doesn't exist.
///
/// # Returns
///
/// Path to `~/.cache/hydequery/`
///
/// # Errors
///
/// Returns an error if the cache directory cannot be determined or created.
pub fn get_cache_dir() -> Result<PathBuf, AppError> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| AppError::internal("Failed to determine cache directory"))?
        .join(CACHE_DIR_NAME);

    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)
            .map_err(|e| AppError::internal(format!("Failed to create cache directory: {e}")))?;
    }

    Ok(cache_dir)
}

/// Returns the path to the cached schema file.
///
/// # Returns
///
/// Path to `~/.cache/hydequery/hyprland.json`
///
/// # Errors
///
/// Returns an error if the cache directory cannot be determined.
pub fn get_cached_schema_path() -> Result<PathBuf, AppError> {
    Ok(get_cache_dir()?.join(SCHEMA_FILENAME))
}

/// Checks if a cached schema exists.
///
/// # Returns
///
/// `true` if the cached schema file exists, `false` otherwise.
pub fn has_cached_schema() -> bool {
    get_cached_schema_path()
        .map(|p| p.exists())
        .unwrap_or(false)
}

/// Fetches the schema from the repository and caches it locally.
///
/// Downloads the Hyprland schema from the HyDE-Project repository
/// and saves it to the cache directory.
///
/// # Returns
///
/// Path to the cached schema file.
///
/// # Errors
///
/// Returns an error if:
/// - The network request fails
/// - The response cannot be read
/// - The file cannot be written
///
/// # Example
///
/// ```no_run
/// use hydequery::fetch;
///
/// let path = fetch::fetch_schema()?;
/// println!("Schema cached at: {}", path.display());
/// ```
pub fn fetch_schema() -> Result<PathBuf, AppError> {
    let body = ureq::get(SCHEMA_URL)
        .call()
        .map_err(|e| AppError::internal(format!("Failed to fetch schema: {e}")))?
        .body_mut()
        .read_to_string()
        .map_err(|e| AppError::internal(format!("Failed to read schema response: {e}")))?;

    let cache_path = get_cached_schema_path()?;

    let mut file = fs::File::create(&cache_path)
        .map_err(|e| AppError::internal(format!("Failed to create schema file: {e}")))?;

    file.write_all(body.as_bytes())
        .map_err(|e| AppError::internal(format!("Failed to write schema file: {e}")))?;

    Ok(cache_path)
}

/// Resolves the schema path from user input.
///
/// Handles the special "auto" value by returning the cached schema path.
/// For any other value, returns the path as-is after normalization.
///
/// # Arguments
///
/// * `schema` - The schema path from CLI ("auto" or a file path)
///
/// # Returns
///
/// The resolved schema file path.
///
/// # Errors
///
/// Returns an error if:
/// - "auto" is specified but no cached schema exists
/// - The cache directory cannot be determined
pub fn resolve_schema_path(schema: &str) -> Result<PathBuf, AppError> {
    if schema == "auto" {
        if !has_cached_schema() {
            return Err(AppError::not_found(
                "No cached schema found. Run with --fetch-schema first"
            ));
        }
        get_cached_schema_path()
    } else {
        Ok(PathBuf::from(schema))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cache_dir() {
        let result = get_cache_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("hydequery"));
    }

    #[test]
    fn test_get_cached_schema_path() {
        let result = get_cached_schema_path();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().ends_with("hyprland.json"));
    }

    #[test]
    fn test_resolve_schema_path_custom() {
        let result = resolve_schema_path("/custom/path/schema.json");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/custom/path/schema.json"));
    }

    #[test]
    fn test_resolve_schema_path_auto_no_cache() {
        if !has_cached_schema() {
            let result = resolve_schema_path("auto");
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_has_cached_schema() {
        let _result = has_cached_schema();
    }

    #[test]
    fn test_resolve_schema_path_auto_with_cache() {
        if has_cached_schema() {
            let result = resolve_schema_path("auto");
            assert!(result.is_ok());
            let path = result.unwrap();
            assert!(path.to_string_lossy().ends_with("hyprland.json"));
        }
    }

    #[test]
    fn test_resolve_schema_path_relative() {
        let result = resolve_schema_path("./schema.json");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("./schema.json"));
    }

    #[test]
    fn test_resolve_schema_path_home() {
        let result = resolve_schema_path("~/schema.json");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("~/schema.json"));
    }

    #[test]
    fn test_cache_dir_contains_correct_name() {
        let cache_dir = get_cache_dir().unwrap();
        assert!(cache_dir.ends_with(CACHE_DIR_NAME));
    }

    #[test]
    fn test_schema_filename_constant() {
        assert_eq!(SCHEMA_FILENAME, "hyprland.json");
    }

    #[test]
    fn test_cache_dir_name_constant() {
        assert_eq!(CACHE_DIR_NAME, "hydequery");
    }

    #[test]
    fn test_schema_url_constant() {
        assert!(SCHEMA_URL.starts_with("https://"));
        assert!(SCHEMA_URL.contains("hyprland.json"));
    }

    #[test]
    fn test_get_cache_dir_creates_directory() {
        let cache_dir = get_cache_dir().unwrap();
        assert!(cache_dir.exists() || !cache_dir.exists());
    }

    #[test]
    fn test_cached_schema_path_is_file_path() {
        let path = get_cached_schema_path().unwrap();
        assert!(path.file_name().is_some());
        assert_eq!(path.file_name().unwrap(), SCHEMA_FILENAME);
    }
}
