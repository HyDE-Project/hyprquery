cmake_minimum_required(VERSION 3.19)

project(hyprquery LANGUAGES CXX VERSION 0.1.0)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

include(GNUInstallDirs)

find_package(PkgConfig REQUIRED)
pkg_check_modules(XKBCOMMON REQUIRED xkbcommon)

find_package(PkgConfig REQUIRED)

include(FetchContent)

option(HYPRQUERY_DISTRO_BUILD "Use system-provided dependencies for distro packaging" OFF)

if(HYPRQUERY_DISTRO_BUILD)
    set(USE_SYSTEM_HYPRLANG ON CACHE BOOL "Use system-installed hyprlang instead of FetchContent" FORCE)
endif()


option(USE_SYSTEM_HYPRLANG "Use system-installed hyprlang instead of FetchContent" OFF)

if(USE_SYSTEM_HYPRLANG)
    pkg_check_modules(HYPRLANG REQUIRED hyprlang)
else()
    set(HYPRLANG_BUILD_TESTS OFF CACHE BOOL "" FORCE)
    set(HYPRLANG_BUILD_FUZZERS OFF CACHE BOOL "" FORCE)
    set(HYPRLANG_BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)
    FetchContent_Declare(
        hyprlang
        GIT_REPOSITORY https://github.com/hyprwm/hyprlang
        GIT_TAG v0.6.3  
    )
    FetchContent_MakeAvailable(hyprlang)
endif()

if(HYPRQUERY_DISTRO_BUILD)
    find_package(CLI11 REQUIRED)
else()
    find_package(CLI11 QUIET)
    if (NOT CLI11_FOUND)
        FetchContent_Declare(
            CLI11
            GIT_REPOSITORY https://github.com/CLIUtils/CLI11.git
            GIT_TAG        v2.5.0
        )
        FetchContent_MakeAvailable(CLI11)
    endif()
endif()

if(HYPRQUERY_DISTRO_BUILD)
    find_package(nlohmann_json REQUIRED)
else()
    find_package(nlohmann_json QUIET)
    if (NOT nlohmann_json_FOUND)
        FetchContent_Declare(
            nlohmann_json
            GIT_REPOSITORY https://github.com/nlohmann/json.git
            GIT_TAG        v3.12.0
        )
        FetchContent_MakeAvailable(nlohmann_json)
    endif()
endif()

set(SOURCES
    src/main.cpp
    src/ConfigUtils.cpp
    src/SourceHandler.cpp
    src/ExportJson.cpp
    src/ExportEnv.cpp
    src/KeybindHandler.cpp
    src/Logger.cpp
)

list(APPEND SOURCES src/XKBKeymapResolver.cpp)
add_executable(hyq ${SOURCES})
target_link_libraries(hyq PRIVATE ${XKBCOMMON_LIBRARIES})
target_include_directories(hyq PRIVATE ${XKBCOMMON_INCLUDE_DIRS})
target_compile_options(hyq PRIVATE ${XKBCOMMON_CFLAGS_OTHER})

if(USE_SYSTEM_HYPRLANG)
    target_include_directories(hyq PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src
        ${HYPRLANG_INCLUDE_DIRS}
    )
    target_link_libraries(hyq PRIVATE
        ${HYPRLANG_LIBRARIES}
        CLI11::CLI11
        nlohmann_json::nlohmann_json
    )
else()
    target_include_directories(hyq PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src
    )
    target_link_libraries(hyq PRIVATE
        hyprlang
        CLI11::CLI11
        nlohmann_json::nlohmann_json
    )
endif()

install(TARGETS hyq DESTINATION ${CMAKE_INSTALL_BINDIR})
install(FILES LICENSE DESTINATION ${CMAKE_INSTALL_DATADIR}/licenses/hyprquery)
install(FILES man/hyprquery DESTINATION ${CMAKE_INSTALL_MANDIR}/man1 RENAME hyq.1 OPTIONAL)
