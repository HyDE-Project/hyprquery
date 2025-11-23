//! # Hyprquery
//!
//! A high-performance command-line utility for querying configuration values
//! from Hyprland configuration files.
//!
//! ## Features
//!
//! - Query nested configuration values using dot notation
//! - Support for dynamic variables (`$var`)
//! - Type and regex filtering with `query[type][regex]` syntax
//! - Multiple export formats: plain text, JSON, environment variables
//! - Recursive source directive following
//! - Schema-based default value support
//!
//! ## Usage
//!
//! ```bash
//! hydequery /path/to/config.conf -Q 'general:border_size'
//! hydequery /path/to/config.conf -Q '$terminal' --export json
//! hydequery /path/to/config.conf -Q 'gaps[INT][^\d+$]' --strict
//! ```

mod app;
mod cli;
mod defaults;
mod error;
mod export;
mod fetch;
mod filters;
mod help;
mod path;
mod query;
mod schema;
mod source;
mod value;

use std::{env::args, process::exit};

use crate::{app::run, help::print_help};

/// Check if help flag is present in arguments.
fn has_help_flag(args: &[String]) -> bool {
    args.iter().any(|a| a == "-h" || a == "--help")
}

/// Main entry point logic without process::exit.
///
/// Returns exit code: 0 for success, 1 for error.
fn run_main(args: &[String]) -> i32 {
    if has_help_flag(args) {
        print_help();
        return 0;
    }

    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error: {e}");
            1
        }
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    exit(run_main(&args));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_help_flag_short() {
        let args = vec!["hydequery".to_string(), "-h".to_string()];
        assert!(has_help_flag(&args));
    }

    #[test]
    fn test_has_help_flag_long() {
        let args = vec!["hydequery".to_string(), "--help".to_string()];
        assert!(has_help_flag(&args));
    }

    #[test]
    fn test_has_help_flag_none() {
        let args = vec![
            "hydequery".to_string(),
            "config.conf".to_string(),
            "-Q".to_string(),
            "$GTK_THEME".to_string(),
        ];
        assert!(!has_help_flag(&args));
    }

    #[test]
    fn test_has_help_flag_among_args() {
        let args = vec![
            "hydequery".to_string(),
            "config.conf".to_string(),
            "-h".to_string(),
        ];
        assert!(has_help_flag(&args));
    }

    #[test]
    fn test_run_main_with_help() {
        let args = vec!["hydequery".to_string(), "-h".to_string()];
        let code = run_main(&args);
        assert_eq!(code, 0);
    }

    #[test]
    fn test_run_main_with_long_help() {
        let args = vec!["hydequery".to_string(), "--help".to_string()];
        let code = run_main(&args);
        assert_eq!(code, 0);
    }
}
