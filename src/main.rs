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
//! hyprquery /path/to/config.conf -Q 'general:border_size'
//! hyprquery /path/to/config.conf -Q '$terminal' --export json
//! hyprquery /path/to/config.conf -Q 'gaps[INT][^\d+$]' --strict
//! ```

mod app;
mod cli;
mod error;
mod export;
mod path;
mod query;
mod schema;
mod source;
mod value;

use std::process;

fn main() {
    match app::run() {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
