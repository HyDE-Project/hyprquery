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

use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        help::print_help();
        process::exit(0);
    }

    match app::run() {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
