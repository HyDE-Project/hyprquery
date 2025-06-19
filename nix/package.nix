{
  lib,
  stdenv,
  cmake,
  pkg-config,
  spdlog,
  nlohmann_json,
  cli11,
  hyprlang,
  version,
  src,
}:
stdenv.mkDerivation {
  pname = "hyprquery";
  version = lib.strings.removeSuffix "\n" version;

  inherit src;

  nativeBuildInputs = [
    cmake
    pkg-config
  ];

  buildInputs = [
    spdlog
    nlohmann_json
    cli11
    hyprlang
  ];

  prePatch = ''
    # Use the nix-optimized CMakeLists.txt
    cp nix/CMakeLists.txt CMakeLists.txt
  '';

  cmakeFlags = [
    "-DCMAKE_BUILD_TYPE=Release"
    "-DCMAKE_EXPORT_COMPILE_COMMANDS=ON"
    "-DUSE_SYSTEM_DEPS=ON"
  ];

  installPhase = ''
    runHook preInstall

    mkdir -p $out/bin
    cp ../bin/hyq $out/bin/

    # Install man page if it exists
    if [ -d "../man" ]; then
      mkdir -p $out/share/man/man1
      cp ../man/*.1 $out/share/man/man1/ || true
    fi

    runHook postInstall
  '';

  meta = with lib; {
    description = "A command-line utility for querying configuration values from Hyprland";
    homepage = "https://github.com/HyDE-Project/hyprquery";
    license = licenses.mit;
    maintainers = [ richen604 ];
    platforms = platforms.linux;
    mainProgram = "hyq";
  };
}
