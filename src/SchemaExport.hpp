#pragma once

#include <string>

namespace hyprquery {

// Dumps all config values filtered by the provided schema.
//
// exportFormat: "json" (default flat array), "nested-json" (nested object),
//               "hypr" (hyprland sections), "lua" (Lua data table)
//
// Behavior:
//   Default (no extra flags): only outputs keys that are both in the schema
//     AND explicitly set in the config file.
//   --fallback: also outputs schema defaults for keys not found in the config.
//   --strict: exits with error if any schema key is missing from the config
//     or if a type mismatch is detected.
int runDumpMode(const std::string &configFilePath,
                const std::string &schemaFilePath,
                const std::string &exportFormat, bool strictMode,
                bool useFallback, bool followSource, bool debugLogging);

} // namespace hyprquery
