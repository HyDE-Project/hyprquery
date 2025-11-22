//! Command-line interface definitions for hyprquery.
//!
//! This module defines the CLI argument structure using the `clap` derive API.
//! All command-line options and flags are documented and validated by clap.

use clap::Parser;

/// Command-line arguments for hyprquery.
///
/// Defines all available options, flags, and positional arguments.
/// Uses clap's derive API for automatic parsing and help generation.
#[derive(Parser, Debug)]
#[command(name = "hyprquery")]
#[command(version)]
#[command(about = "A configuration parser for hypr* config files")]
#[command(disable_help_flag = true)]
pub struct Args {
    /// Show help information
    #[arg(short = 'h', long = "help")]
    pub help:        bool,
    /// Configuration file path
    #[arg(required = true)]
    pub config_file: String,

    /// Query to execute (format: `query[expectedType][expectedRegex]`)
    #[arg(short = 'Q', long = "query", required = true, num_args = 1..)]
    pub queries: Vec<String>,

    /// Schema file path
    #[arg(long)]
    pub schema: Option<String>,

    /// Allow missing values (don't fail with exit code 1)
    #[arg(long)]
    pub allow_missing: bool,

    /// Get default keys from schema
    #[arg(long)]
    pub get_defaults: bool,

    /// Enable strict mode validation
    #[arg(long)]
    pub strict: bool,

    /// Export format: json or env
    #[arg(long)]
    pub export: Option<String>,

    /// Follow source directives in config files
    #[arg(short = 's', long)]
    pub source: bool,

    /// Enable debug logging
    #[arg(long)]
    pub debug: bool,

    /// Delimiter for plain output
    #[arg(short = 'D', long, default_value = "\n")]
    pub delimiter: String
}
