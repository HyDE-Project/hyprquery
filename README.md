> [!WARNING]
>
> EXPERIMENTAL branch. porting to rust.

![HyDE Banner](https://raw.githubusercontent.com/HyDE-Project/HyDE/master/Source/assets/hyde_banner.png)

# Hyprquery

[![CI](https://github.com/HyDE-Project/hyprquery/actions/workflows/ci.yml/badge.svg)](https://github.com/HyDE-Project/hyprquery/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/HyDE-Project/hyprquery/graph/badge.svg)](https://codecov.io/gh/HyDE-Project/hyprquery)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://opensource.org/licenses/GPL-3.0)
[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)

**High-performance configuration parser for Hyprland**

A blazing-fast CLI tool written in Rust for querying values from Hyprland configuration files. Supports nested keys, dynamic variables, type filtering, regex matching, and multiple export formats.

## Features

- **Fast** - Optimized Rust implementation with minimal allocations (~1ms per query)
- **Flexible queries** - Support for nested keys, type filters, and regex patterns
- **Dynamic variables** - Query `$variable` values directly
- **Multiple formats** - Export as plain text, JSON, or environment variables
- **Source following** - Recursively parse `source = path` directives with cycle detection
- **Schema support** - Load default values from Hyprland schema files
- **Colorful help** - Beautiful, detailed `--help` with examples

## Installation

### From source

```bash
git clone https://github.com/HyDE-Project/hyprquery
cd hyprquery
cargo build --release
sudo cp target/release/hyq /usr/local/bin/
```

### Requirements

- Rust stable (2024 edition)
- Rust nightly (for formatting only)

## Usage

### Basic syntax

```bash
hyq <CONFIG_FILE> -Q <QUERY> [OPTIONS]
```

### Query format

```
key                    # Simple lookup
key[type]              # With type filter
key[type][regex]       # With type and regex filter
$variable              # Dynamic variable
```

**Types:** `INT`, `FLOAT`, `STRING`, `VEC2`, `COLOR`, `BOOL`

## Examples

### Basic query

```bash
hyq ~/.config/hypr/hyprland.conf -Q 'general:border_size'
# Output: 2
```

### Query variable

```bash
hyq config.conf -Q '$terminal'
# Output: kitty
```

### Multiple queries

```bash
hyq config.conf -Q 'general:gaps_in' -Q 'general:gaps_out'
# Output:
# 5
# 10
```

### With type filter

```bash
hyq config.conf -Q 'general:border_size[INT]'
# Output: 2
```

### With regex filter

```bash
hyq config.conf -Q 'decoration:rounding[INT][^[0-9]+$]'
# Output: 8
```

### JSON export

```bash
hyq config.conf -Q 'general:border_size' --export json
```

```json
{
  "key": "general:border_size",
  "value": "2",
  "type": "INT"
}
```

### Environment variables export

```bash
hyq config.conf -Q '$terminal' --export env
# Output: TERMINAL="kitty"
```

### Follow source directives

```bash
hyq config.conf -Q 'colors:background' -s
```

### Fetch and cache schema

```bash
hyq --fetch-schema
# Output: Schema cached at: ~/.cache/hyprquery/hyprland.json
```

### Use cached schema

```bash
hyq config.conf -Q 'general:layout' --schema auto
```

### Get all default keys from schema

```bash
hyq config.conf --schema auto --get-defaults
```

### With custom schema

```bash
hyq config.conf -Q 'general:layout' --schema hyprland.json
```

### Custom delimiter

```bash
hyq config.conf -Q 'a' -Q 'b' -D ','
# Output: val1,val2
```

## Options

| Option | Description |
|--------|-------------|
| `-Q, --query <QUERY>` | Query to execute (multiple allowed) |
| `--schema <PATH>` | Load schema file (use `auto` for cached) |
| `--fetch-schema` | Download and cache latest schema |
| `--get-defaults` | Output all keys from schema |
| `--allow-missing` | Don't fail on NULL values (exit 0) |
| `--strict` | Fail on config parse errors |
| `--export <FORMAT>` | Output format: `json`, `env` |
| `-s, --source` | Follow source directives recursively |
| `-D, --delimiter <STR>` | Delimiter for plain output (default: `\n`) |
| `--debug` | Enable debug logging to stderr |
| `-h, --help` | Show colorful help with examples |
| `-V, --version` | Show version |

## Exit codes

| Code | Description |
|------|-------------|
| `0` | All queries resolved successfully |
| `1` | One or more queries returned NULL, or error occurred |

## Configuration file format

Hydequery parses standard Hyprland configuration format:

```bash
# Variables
$terminal = kitty
$mod = SUPER

# Nested sections
general {
    border_size = 2
    gaps_in = 5
    gaps_out = 10
}

decoration {
    rounding = 8
    blur {
        enabled = true
        size = 3
    }
}

# Source directives (supports globs)
source = ~/.config/hypr/colors.conf
source = ~/.config/hypr/keybinds/*.conf
```

## Schema files

Schema files define default values for configuration options:

```json
{
  "hyprlang_schema": [
    {
      "value": "general:border_size",
      "type": "INT",
      "data": { "default": 2 }
    },
    {
      "value": "general:gaps_in",
      "type": "INT",
      "data": { "default": 5 }
    }
  ]
}
```

## Architecture

```
src/
├── main.rs      # Entry point
├── app.rs       # Core application logic
├── cli.rs       # CLI argument definitions
├── defaults.rs  # Schema defaults handling
├── error.rs     # Error handling (masterror)
├── export.rs    # Output formatters (JSON, env, plain)
├── fetch.rs     # Schema fetching and caching
├── filters.rs   # Type and regex filtering
├── help.rs      # Colorful help display
├── path.rs      # Path normalization and glob resolution
├── query.rs     # Query parsing
├── schema.rs    # Schema loading
├── source.rs    # Source directive handling
└── value.rs     # Config value conversion
```

## Performance

- **Binary size:** ~2.5 MB (stripped, LTO enabled)
- **Query time:** ~1ms
- **Memory:** Optimized with `Box<str>` and `&'static str`

## Dependencies

- [clap](https://crates.io/crates/clap) - CLI argument parsing
- [hyprlang](https://github.com/spinualexandru/hyprlang-rs) - Hyprland config parsing
- [masterror](https://crates.io/crates/masterror) - Error handling
- [serde_json](https://crates.io/crates/serde_json) - JSON serialization
- [regex](https://crates.io/crates/regex) - Pattern matching
- [glob](https://crates.io/crates/glob) - Glob pattern support
- [shellexpand](https://crates.io/crates/shellexpand) - Path expansion (~, $HOME)
- [ureq](https://crates.io/crates/ureq) - HTTP client for schema fetching
- [dirs](https://crates.io/crates/dirs) - Platform-specific directories

## Contributing

Contributions are welcome! Please follow [RustManifest](https://github.com/RAprogramm/RustManifest) development standards.

## License

GPL-3.0 - see [LICENSE](LICENSE) for details.

## Credits

Created for [HyDE](https://github.com/HyDE-Project) - Hyprland Desktop Environment.
