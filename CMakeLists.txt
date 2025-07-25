cmake_minimum_required(VERSION 3.19)

project(hyprquery LANGUAGES CXX VERSION 0.1.0)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${PROJECT_SOURCE_DIR}/bin)

find_package(PkgConfig REQUIRED)
pkg_check_modules(SPDLOG REQUIRED spdlog)

include(FetchContent)

option(USE_SYSTEM_HYPRLANG "Use system-installed hyprlang instead of FetchContent" OFF)

if(USE_SYSTEM_HYPRLANG)
    pkg_check_modules(HYPRLANG REQUIRED hyprlang)
else()
    set(HYPRLANG_BUILD_TESTS OFF CACHE BOOL "" FORCE)
    set(HYPRLANG_BUILD_FUZZERS OFF CACHE BOOL "" FORCE)
    FetchContent_Declare(
        hyprlang
        GIT_REPOSITORY https://github.com/hyprwm/hyprlang
        GIT_TAG v0.6.3  
    )
    FetchContent_MakeAvailable(hyprlang)
endif()

find_package(CLI11 QUIET)
if (NOT CLI11_FOUND)
    FetchContent_Declare(
        CLI11
        GIT_REPOSITORY https://github.com/CLIUtils/CLI11.git
        GIT_TAG        v2.5.0
    )
    FetchContent_MakeAvailable(CLI11)
endif()

find_package(nlohmann_json QUIET)
if (NOT nlohmann_json_FOUND)
    FetchContent_Declare(
        nlohmann_json
        GIT_REPOSITORY https://github.com/nlohmann/json.git
        GIT_TAG        v3.12.0
    )
    FetchContent_MakeAvailable(nlohmann_json)
endif()

set(SOURCES
    src/main.cpp
    src/ConfigUtils.cpp
    src/SourceHandler.cpp
    src/ExportJson.cpp
    src/ExportEnv.cpp
)

add_executable(hyq ${SOURCES})

if(USE_SYSTEM_HYPRLANG)
    target_include_directories(hyq PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src
        ${SPDLOG_INCLUDE_DIRS}
        ${HYPRLANG_INCLUDE_DIRS}
    )
    target_link_libraries(hyq PRIVATE
        ${HYPRLANG_LIBRARIES}
        ${SPDLOG_LIBRARIES}
        CLI11::CLI11
        nlohmann_json::nlohmann_json
    )
else()
    target_include_directories(hyq PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src
        ${SPDLOG_INCLUDE_DIRS}
    )
    target_link_libraries(hyq PRIVATE
        hyprlang
        ${SPDLOG_LIBRARIES}
        CLI11::CLI11
        nlohmann_json::nlohmann_json
    )
endif()
