cmake_minimum_required(VERSION 3.19)

project(hyprquery LANGUAGES CXX VERSION 0.1.0)

# Enable compile commands export for clangd and other tools
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Configure output directories
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${PROJECT_SOURCE_DIR}/bin)

# Use system dependencies (this is the Nix build)
set(USE_SYSTEM_DEPS ON)

# Find required packages
find_package(PkgConfig REQUIRED)
pkg_check_modules(SPDLOG REQUIRED spdlog)
pkg_check_modules(HYPRLANG REQUIRED hyprlang)
find_package(CLI11 REQUIRED)
find_package(nlohmann_json REQUIRED)

# Add source files
set(SOURCES
    src/main.cpp
    src/ConfigUtils.cpp
    src/SourceHandler.cpp
)

# Create executable
add_executable(hyq ${SOURCES})

# Include directories
target_include_directories(hyq PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${SPDLOG_INCLUDE_DIRS}
    ${HYPRLANG_INCLUDE_DIRS}
)

# Link libraries
target_link_libraries(hyq PRIVATE
    ${HYPRLANG_LIBRARIES}
    ${SPDLOG_LIBRARIES}
    CLI11::CLI11
    nlohmann_json::nlohmann_json
) 