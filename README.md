# HyprQuery (hyq)

A command-line utility for querying configuration values from Hyprland and hyprland-related configuration files using the hyprlang-rs parsing library.

## Features

- Query any value from Hyprland configuration files
- Support for variables and nested includes via `source` directives
- Load schema files to provide default values
- JSON output format for integration with other tools
- Environment variable expansion in file paths
- Type and regex filtering for query results

## Installation

### Dependencies

- Rust 1.82+ (Edition 2024)

### Building from Source

```bash
git clone https://github.com/HyDE-Project/hyprquery.git
cd hyprquery

cargo build --release

sudo cp target/release/hyprquery /usr/local/bin/hyq
```

### From crates.io

```bash
cargo install hyprquery
```

## Usage

Basic syntax:

```
hyq [OPTIONS] --query KEY CONFIG_FILE
```

### Examples

Query a simple value:

```bash
hyq --query "general:border_size" ~/.config/hypr/hyprland.conf
```

Query with JSON output:

```bash
hyq --export json --query "decoration:blur:enabled" ~/.config/hypr/hyprland.conf
```

Query with a schema file:

```bash
hyq --schema ~/.config/hypr/schema.json --query "general:gaps_in" ~/.config/hypr/hyprland.conf
```

Follow source directives:

```bash
hyq -s --query "general:border_size" ~/.config/hypr/hyprland.conf
```

Query with type and regex hints:

```bash
hyq --query "general:border_size[INT][^[0-9]+$]" ~/.config/hypr/hyprland.conf
```

Query a variable:

```bash
hyq --query '$terminal' ~/.config/hypr/hyprland.conf
```

Multiple queries:

```bash
hyq -Q "general:border_size" -Q "general:gaps_in" -Q "decoration:rounding" ~/.config/hypr/hyprland.conf
```

Export as environment variables:

```bash
hyq --export env -Q "general:border_size" -Q "general:gaps_in" ~/.config/hypr/hyprland.conf
```

### Options

- `-Q, --query KEY`: Query to execute (format: `query[expectedType][expectedRegex]`)
- `--schema PATH`: Load a schema file with default values
- `--allow-missing`: Don't fail if the value is missing
- `--get-defaults`: Get default keys from schema
- `--strict`: Enable strict mode validation
- `--export FORMAT`: Export format: `json` or `env`
- `-s, --source`: Follow source directives in config files
- `-D, --delimiter`: Delimiter for plain output (default: newline)
- `--debug`: Enable debug logging

## Query Format

Queries support optional type and regex hints:

```
query[expectedType][expectedRegex]
```

- `query`: The configuration key path (e.g., `general:border_size`)
- `expectedType`: Expected value type (`INT`, `FLOAT`, `STRING`, `VEC2`, `COLOR`)
- `expectedRegex`: Regex pattern to match the value

If the value doesn't match the expected type or regex, the result will be `NULL`.

## Schema Files

Schema files define the format and default values for configuration options. They are JSON files that can be derived from [Hyprland's ConfigDescriptions.hpp](https://github.com/hyprwm/Hyprland/blob/main/src/config/ConfigDescriptions.hpp).

```json
{
  "hyprlang_schema": [
    {
      "value": "general:border_size",
      "type": "INT",
      "data": {
        "default": 2
      }
    },
    {
      "value": "general:gaps_in",
      "type": "INT",
      "data": {
        "default": 5
      }
    }
  ]
}
```

## License

[GPL-3.0 License](LICENSE)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
