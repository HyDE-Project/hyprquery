use clap::Parser;

/// A command-line utility for querying configuration values from Hyprland
/// configuration files
#[derive(Parser, Debug)]
#[command(name = "hyprquery")]
#[command(version)]
#[command(about = "A configuration parser for hypr* config files")]
pub struct Args {
    /// Configuration file path
    #[arg(required = true)]
    pub config_file: String,

    /// Query to execute (format: query[expectedType][expectedRegex])
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
