# Legacy shell.nix for compatibility
# Use `nix develop` with the flake instead for better experience

let
  pkgs = import <nixpkgs> { };
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    # Build tools
    cmake
    pkg-config
    clang-tools

    # Dependencies
    spdlog
    nlohmann_json
    cli11
    hyprlang

    # Development tools
    gdb
    valgrind
  ];

  shellHook = ''
    echo "🔧 HyprQuery development environment (legacy shell.nix)"
    echo "💡 Consider using 'nix develop' with the flake for better experience"
    echo "📦 Available packages: cmake, pkg-config, spdlog, nlohmann_json, cli11, hyprlang"
    echo "🛠️  Available tools: clang-tools, gdb, valgrind"
  '';
}
