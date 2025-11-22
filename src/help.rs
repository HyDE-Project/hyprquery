//! Custom colorful help output for hydequery.
//!
//! This module provides a beautifully formatted, colored help display
//! with clear explanations, examples, and usage patterns.

/// ANSI color codes for terminal output.
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";

    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";
}

use colors::*;

/// Print the complete help message with colors and formatting.
pub fn print_help() {
    print_header();
    print_usage();
    print_arguments();
    print_options();
    print_query_format();
    print_examples();
    print_exit_codes();
    print_footer();
}

/// Print the application header with logo.
fn print_header() {
    println!(
        r#"
{CYAN}{BOLD}╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   {MAGENTA}██╗  ██╗██╗   ██╗██████╗ ██████╗  ██████╗ ██╗   ██╗███████╗{CYAN}  ║
║   {MAGENTA}██║  ██║╚██╗ ██╔╝██╔══██╗██╔══██╗██╔═══██╗██║   ██║██╔════╝{CYAN}  ║
║   {MAGENTA}███████║ ╚████╔╝ ██████╔╝██████╔╝██║   ██║██║   ██║█████╗  {CYAN}  ║
║   {MAGENTA}██╔══██║  ╚██╔╝  ██╔═══╝ ██╔══██╗██║▄▄ ██║██║   ██║██╔══╝  {CYAN}  ║
║   {MAGENTA}██║  ██║   ██║   ██║     ██║  ██║╚██████╔╝╚██████╔╝███████╗{CYAN}  ║
║   {MAGENTA}╚═╝  ╚═╝   ╚═╝   ╚═╝     ╚═╝  ╚═╝ ╚══▀▀═╝  ╚═════╝ ╚══════╝{CYAN}  ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝{RESET}

{WHITE}{BOLD}Hydequery{RESET} - {DIM}High-performance configuration parser for Hyprland{RESET}
"#
    );
}

/// Print usage section.
fn print_usage() {
    println!(
        "{YELLOW}{BOLD}USAGE:{RESET}
    {GREEN}hydequery{RESET} {CYAN}<CONFIG_FILE>{RESET} {MAGENTA}-Q{RESET} {BLUE}<QUERY>{RESET} [{DIM}OPTIONS{RESET}]
    {GREEN}hydequery{RESET} {CYAN}<CONFIG_FILE>{RESET} {MAGENTA}-Q{RESET} {BLUE}<QUERY1>{RESET} {MAGENTA}-Q{RESET} {BLUE}<QUERY2>{RESET} ...
"
    );
}

/// Print arguments section.
fn print_arguments() {
    println!(
        "{YELLOW}{BOLD}ARGUMENTS:{RESET}
    {CYAN}<CONFIG_FILE>{RESET}    Path to Hyprland configuration file
                    {DIM}Supports: ~, $HOME, environment variables{RESET}
"
    );
}

/// Print options section.
fn print_options() {
    println!(
        "{YELLOW}{BOLD}OPTIONS:{RESET}
    {GREEN}-Q, --query{RESET} {BLUE}<QUERY>{RESET}      Query to execute {DIM}(required, multiple allowed){RESET}
                            Format: {CYAN}key[type][regex]{RESET}

    {GREEN}--schema{RESET} {BLUE}<PATH>{RESET}          Load schema file {DIM}(use \"{CYAN}auto{DIM}\" for cached){RESET}
    {GREEN}--fetch-schema{RESET}          Download and cache latest schema
    {GREEN}--get-defaults{RESET}          Output all keys from schema
    {GREEN}--allow-missing{RESET}         Don't fail on NULL values {DIM}(exit 0){RESET}
    {GREEN}--strict{RESET}                Fail on config parse errors
    {GREEN}--export{RESET} {BLUE}<FORMAT>{RESET}        Output format: {CYAN}json{RESET}, {CYAN}env{RESET}
    {GREEN}-s, --source{RESET}            Follow source directives recursively
    {GREEN}-D, --delimiter{RESET} {BLUE}<STR>{RESET}   Delimiter for plain output {DIM}(default: \\n){RESET}
    {GREEN}--debug{RESET}                 Enable debug logging to stderr

    {GREEN}-h, --help{RESET}              Show this help message
    {GREEN}-V, --version{RESET}           Show version information
"
    );
}

/// Print query format explanation.
fn print_query_format() {
    println!(
        "{YELLOW}{BOLD}QUERY FORMAT:{RESET}
    {CYAN}key{RESET}                     Simple key lookup
    {CYAN}key{RESET}{DIM}[type]{RESET}               With type filter
    {CYAN}key{RESET}{DIM}[type][regex]{RESET}        With type and regex filter
    {CYAN}$variable{RESET}               Dynamic variable lookup

    {WHITE}{BOLD}Types:{RESET} {BLUE}INT{RESET}, {BLUE}FLOAT{RESET}, {BLUE}STRING{RESET}, {BLUE}VEC2{RESET}, {BLUE}COLOR{RESET}, {BLUE}BOOL{RESET}

    {WHITE}{BOLD}Key Syntax:{RESET}
    {DIM}•{RESET} Nested keys use {CYAN}:{RESET} separator: {GREEN}general:border_size{RESET}
    {DIM}•{RESET} Categories: {GREEN}general{RESET}, {GREEN}decoration{RESET}, {GREEN}input{RESET}, {GREEN}animations{RESET}, etc.
"
    );
}

/// Print examples section.
fn print_examples() {
    println!(
        "{YELLOW}{BOLD}EXAMPLES:{RESET}

    {WHITE}Basic query:{RESET}
    {DIM}${RESET} {GREEN}hydequery{RESET} ~/.config/hypr/hyprland.conf {MAGENTA}-Q{RESET} {CYAN}'general:border_size'{RESET}
    {BLUE}2{RESET}

    {WHITE}Query variable:{RESET}
    {DIM}${RESET} {GREEN}hydequery{RESET} config.conf {MAGENTA}-Q{RESET} {CYAN}'$terminal'{RESET}
    {BLUE}kitty{RESET}

    {WHITE}Multiple queries:{RESET}
    {DIM}${RESET} {GREEN}hydequery{RESET} config.conf {MAGENTA}-Q{RESET} {CYAN}'general:gaps_in'{RESET} {MAGENTA}-Q{RESET} {CYAN}'general:gaps_out'{RESET}
    {BLUE}5{RESET}
    {BLUE}10{RESET}

    {WHITE}With type filter:{RESET}
    {DIM}${RESET} {GREEN}hydequery{RESET} config.conf {MAGENTA}-Q{RESET} {CYAN}'general:border_size[INT]'{RESET}
    {BLUE}2{RESET}

    {WHITE}With regex filter:{RESET}
    {DIM}${RESET} {GREEN}hydequery{RESET} config.conf {MAGENTA}-Q{RESET} {CYAN}'decoration:rounding[INT][^[0-9]+$]'{RESET}
    {BLUE}8{RESET}

    {WHITE}JSON export:{RESET}
    {DIM}${RESET} {GREEN}hydequery{RESET} config.conf {MAGENTA}-Q{RESET} {CYAN}'general:border_size'{RESET} {MAGENTA}--export{RESET} {CYAN}json{RESET}
    {BLUE}{{
      \"key\": \"general:border_size\",
      \"value\": \"2\",
      \"type\": \"INT\"
    }}{RESET}

    {WHITE}Environment variables:{RESET}
    {DIM}${RESET} {GREEN}hydequery{RESET} config.conf {MAGENTA}-Q{RESET} {CYAN}'$terminal'{RESET} {MAGENTA}--export{RESET} {CYAN}env{RESET}
    {BLUE}TERMINAL=\"kitty\"{RESET}

    {WHITE}Fetch and cache schema:{RESET}
    {DIM}${RESET} {GREEN}hydequery{RESET} {MAGENTA}--fetch-schema{RESET}
    {BLUE}Schema cached at: ~/.cache/hydequery/hyprland.json{RESET}

    {WHITE}Use cached schema:{RESET}
    {DIM}${RESET} {GREEN}hydequery{RESET} config.conf {MAGENTA}-Q{RESET} {CYAN}'general:layout'{RESET} {MAGENTA}--schema{RESET} {CYAN}auto{RESET}

    {WHITE}With custom schema:{RESET}
    {DIM}${RESET} {GREEN}hydequery{RESET} config.conf {MAGENTA}-Q{RESET} {CYAN}'general:layout'{RESET} {MAGENTA}--schema{RESET} {CYAN}hyprland.json{RESET}

    {WHITE}Follow source directives:{RESET}
    {DIM}${RESET} {GREEN}hydequery{RESET} config.conf {MAGENTA}-Q{RESET} {CYAN}'colors:background'{RESET} {MAGENTA}-s{RESET}

    {WHITE}Custom delimiter:{RESET}
    {DIM}${RESET} {GREEN}hydequery{RESET} config.conf {MAGENTA}-Q{RESET} {CYAN}'a'{RESET} {MAGENTA}-Q{RESET} {CYAN}'b'{RESET} {MAGENTA}-D{RESET} {CYAN}','{RESET}
    {BLUE}val1,val2{RESET}
"
    );
}

/// Print exit codes section.
fn print_exit_codes() {
    println!(
        "{YELLOW}{BOLD}EXIT CODES:{RESET}
    {GREEN}0{RESET}    All queries resolved successfully
    {YELLOW}1{RESET}    One or more queries returned NULL
    {RED}1{RESET}    Error occurred (config not found, parse error, etc.)
"
    );
}

/// Print footer with additional info.
fn print_footer() {
    println!(
        "{DIM}───────────────────────────────────────────────────────────────{RESET}
{WHITE}Repository:{RESET}  {CYAN}https://github.com/HyDE-Project/hydequery{RESET}
{WHITE}License:{RESET}     {CYAN}GPL-3.0{RESET}
{DIM}───────────────────────────────────────────────────────────────{RESET}
"
    );
}
