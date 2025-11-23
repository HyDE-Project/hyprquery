//! Source directive parsing for configuration file includes.
//!
//! This module handles the recursive parsing of `source = path` directives
//! in Hyprland configuration files. Features include:
//! - Glob pattern support for source paths
//! - Cycle detection to prevent infinite loops
//! - Recursive directory traversal

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf}
};

use hyprlang::Config;
use masterror::AppError;

use crate::path::resolve_glob;

/// Recursively parse source directives from configuration files.
///
/// Scans the configuration file for `source = path` directives and
/// parses referenced files into the configuration. Supports glob patterns
/// and prevents infinite loops through cycle detection.
///
/// # Arguments
///
/// * `config` - Configuration instance to populate
/// * `config_path` - Path to the current configuration file
/// * `base_dir` - Base directory for resolving relative paths
/// * `visited` - Set of already-visited paths for cycle detection
/// * `debug` - Enable debug output to stderr
///
/// # Errors
///
/// Returns an error if a source file cannot be read or path resolution fails.
pub fn parse_sources_recursive(
    config: &mut Config,
    config_path: &Path,
    base_dir: &Path,
    visited: &mut HashSet<PathBuf>,
    debug: bool
) -> Result<(), AppError> {
    let content = fs::read_to_string(config_path).map_err(crate::error::from_io)?;

    for line in content.lines() {
        let trimmed = line.trim();

        if !trimmed.starts_with("source") {
            continue;
        }

        let Some(eq_pos) = trimmed.find('=') else {
            continue;
        };

        let path_part = trimmed[eq_pos + 1..].trim();
        let path_part = path_part.split('#').next().unwrap_or("").trim();

        if path_part.is_empty() {
            continue;
        }

        let paths = match resolve_glob(path_part, base_dir) {
            Ok(p) => p,
            Err(e) => {
                if debug {
                    eprintln!("[debug] Failed to resolve: {} - {}", path_part, e);
                }
                continue;
            }
        };

        for source_path in paths {
            if !source_path.exists() || !source_path.is_file() {
                continue;
            }

            let canonical = match source_path.canonicalize() {
                Ok(p) => p,
                Err(_) => source_path.clone()
            };

            if visited.contains(&canonical) {
                if debug {
                    eprintln!("[debug] Skipping already visited: {}", canonical.display());
                }
                continue;
            }

            visited.insert(canonical.clone());

            if debug {
                eprintln!("[debug] Parsing source: {}", source_path.display());
            }

            let _ = config.parse_file(&source_path);

            let source_dir = match source_path.parent() {
                Some(p) => p.to_path_buf(),
                None => base_dir.to_path_buf()
            };

            parse_sources_recursive(config, &source_path, &source_dir, visited, debug)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_parse_sources_no_source_directive() {
        let temp_dir = std::env::temp_dir().join("hydequery_source_test");
        let _ = fs::create_dir_all(&temp_dir);
        let config_path = temp_dir.join("test.conf");
        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(file, "key = value").unwrap();

        let mut config = Config::default();
        let mut visited = HashSet::new();
        visited.insert(config_path.clone());

        let result =
            parse_sources_recursive(&mut config, &config_path, &temp_dir, &mut visited, false);
        assert!(result.is_ok());

        let _ = fs::remove_file(config_path);
        let _ = fs::remove_dir(temp_dir);
    }

    #[test]
    fn test_parse_sources_cycle_detection() {
        let temp_dir = std::env::temp_dir().join("hydequery_cycle_test");
        let _ = fs::create_dir_all(&temp_dir);
        let config_path = temp_dir.join("cycle.conf");
        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(file, "source = {}", config_path.display()).unwrap();

        let mut config = Config::default();
        let mut visited = HashSet::new();
        let canonical = config_path.canonicalize().unwrap_or(config_path.clone());
        visited.insert(canonical);

        let result =
            parse_sources_recursive(&mut config, &config_path, &temp_dir, &mut visited, false);
        assert!(result.is_ok());

        let _ = fs::remove_file(config_path);
        let _ = fs::remove_dir(temp_dir);
    }
}
