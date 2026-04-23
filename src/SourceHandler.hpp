#pragma once

#include <filesystem>
#include <hyprlang.hpp>
#include <string>
#include <vector>

namespace hyprquery {

class SourceHandler {
public:
  static void registerHandler(Hyprlang::CConfig *config);

  static Hyprlang::CParseResult handleSource(const char *command,
                                             const char *value);

  static void setConfigDir(const std::string &dir);

  static std::string getConfigDir();

  static std::vector<std::filesystem::path>
  resolvePath(const std::string &path);
  static std::string expandEnvVars(const std::string &path);

  static bool isInitialized();

  static void setSuppressNoMatchErrors(bool suppress);
  static void setStrictMode(bool strict);

private:
  static Hyprlang::CConfig *s_pConfig;
  static std::string s_configDir;
  static bool s_initialized;
  static bool s_suppressNoMatchErrors;
  static bool s_strictMode;
};

} // namespace hyprquery