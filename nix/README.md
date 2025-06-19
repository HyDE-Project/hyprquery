# Nix Build Configuration

This directory contains Nix-specific configuration files for building HyprQuery. The main flake is located in the project root.

## Files

- `package.nix` - Package derivation for HyprQuery
- `CMakeLists.txt` - Nix-optimized CMake configuration
- `shell.nix` - Legacy nix-shell support (optional)

## Usage

### Building the package
```bash
nix build .#hyprquery
```

### Installing as a package

**Temporary installation (current shell only):**
```bash
nix shell .#hyprquery
```

**Permanent installation to user profile:**
```bash
nix profile install .#hyprquery
```

**Installing from GitHub:**
```bash
# Temporary
nix shell github:HyDE-Project/hyprquery

# Permanent
nix profile install github:HyDE-Project/hyprquery
```

### Development shell
```bash
nix develop
```

### System-wide installation

**NixOS configuration:**
```nix
# In your flake.nix inputs:
inputs.hyprquery.url = "github:HyDE-Project/hyprquery";

# In your system packages:
environment.systemPackages = [
  inputs.hyprquery.packages.${pkgs.system}.default
];
```

**Home Manager:**
```nix
home.packages = [
  inputs.hyprquery.packages.${pkgs.system}.default
];
```

### Legacy nix-shell (if shell.nix exists)
```bash
nix-shell nix/shell.nix
```

## Features

- Uses system dependencies via Nix packages
- Includes development tools (clang-tools, gdb, valgrind)
- Automatic build directory setup
- Compile commands export for language servers
- Clean separation from source code
- Organized structure with dedicated nix directory

## Structure

The main `flake.nix` is in the project root and imports configuration from this directory:

- Root `flake.nix` - Main flake definition with inputs/outputs
- `nix/package.nix` - Package derivation
- `nix/CMakeLists.txt` - Nix-specific build configuration
- `nix/shell.nix` - Legacy compatibility 