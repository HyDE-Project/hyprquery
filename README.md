# HyprQuery (hyq)

A command-line utility for querying configuration values from Hyprland and hyprland-related configuration files using the hyprlang parsing library.

## Features

- Query any value from Hyprland configuration files
- Support for variables and nested includes via `source` directives
- Load schema files to provide default values
- JSON output format for integration with other tools
- Environment variable expansion in file paths

## Installation

### Dependencies

- C++23 compatible compiler
- CMake 3.19+
- spdlog
- nlohmann_json
- CLI11
- hyprlang (automatically downloaded and built during compilation)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/HyDE-Project/hyprquery.git
cd hyprquery

# Create build directory
mkdir build && cd build

# Configure the project
cmake ..

# Build
make -j$(nproc)

# Install (optional)
sudo make install
```

### Build Options

- `CMAKE_EXPORT_COMPILE_COMMANDS=ON`: Generate compile_commands.json for IDE integration
- `CMAKE_BUILD_TYPE=Release|Debug`: Build in release or debug mode

Example:

```bash
# Build with compile commands for IDE integration
cmake -DCMAKE_EXPORT_COMPILE_COMMANDS=ON ..

# Build in release mode
cmake -DCMAKE_BUILD_TYPE=Release ..
```

### IDE Setup

For IDE integration with clangd, run the setup script to generate the compile_commands.json file:

```bash
# Make the script executable (if needed)
chmod +x scripts/setup-clangd.sh

# Run the script
./scripts/setup-clangd.sh
```

This will:

1. Generate the compile_commands.json in the build directory
2. Create a copy of the compile_commands.json in the project root
3. Allow clangd to find all headers and provide proper code intelligence

After running the script, restart your IDE or reload the clangd language server.

### Distribution Packages

Debian/Ubuntu:

```
# Coming soon
```

Arch Linux:

```
# Coming soon
```

Nix:

```bash
# Install temporarily
nix shell github:HyDE-Project/hyprquery

# Install permanently  
nix profile install github:HyDE-Project/hyprquery

# Development shell
nix develop

# For legacy Nix
nix-shell nix/shell.nix
```

See `nix/README.md` for detailed Nix installation options including NixOS and Home Manager configurations.

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
hyq --json --query "decoration:blur:enabled" ~/.config/hypr/hyprland.conf
```

Query with a schema file:

```bash
hyq --schema ~/.config/hypr/schema.json --query "general:gaps_in" ~/.config/hypr/hyprland.conf
```

Follow source directives:

```bash
hyq -s --query "general:border_size" ~/.config/hypr/hyprland.conf
```

### Options

- `--query KEY`: Specify the key to query from the config file
- `--schema PATH`: Load a schema file with default values
- `--allow-missing`: Don't fail if the value is missing
- `--get-defaults`: Get default keys from schema
- `--strict`: Enable strict mode validation
- `--json`, `-j`: Output result in JSON format
- `--source`, `-s`: Follow source directives in config files

### Environment Variables

- `LOG_LEVEL`: Set the log level (debug, info, warn, error, critical)

## Schema Files

Schema files define the format and default values for configuration options. They are JSON files an example is can be derived from [Hyprland's ConfigDescriptions.hpp](https://github .com/hyprwm/Hyprland/blob/main/src/config/ConfigDescriptions.hpp) and converted to JSON format for easier use. The structure is as follows:

```json
{
  "hyprlang_schema": [
    {
      "value": "general:border_size",
      "type": "INT",
      "data": {
        "default": 2,
        "min": 0,
        "max": 20
      }
    },
    {
      "value": "general:gaps_in",
      "type": "INT",
      "data": {
        "default": 5,
        "min": 0,
        "max": 50
      }
    }
  ]
}
```

## License

[MIT License](LICENSE)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
