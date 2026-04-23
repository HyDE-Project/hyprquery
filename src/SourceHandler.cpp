#include "SourceHandler.hpp"
#include "ConfigUtils.hpp"
#include "Logger.hpp"
#include <cstring>

#include <glob.h>
#include <memory>
#include <wordexp.h>

namespace hyprquery {

Hyprlang::CConfig *SourceHandler::s_pConfig = nullptr;
std::string SourceHandler::s_configDir = "";
bool SourceHandler::s_initialized = false;
bool SourceHandler::s_suppressNoMatchErrors = false;
bool SourceHandler::s_strictMode = false;

void SourceHandler::setConfigDir(const std::string &dir) { s_configDir = dir; }

std::string SourceHandler::getConfigDir() { return s_configDir; }

bool SourceHandler::isInitialized() { return s_initialized; }

std::string SourceHandler::expandEnvVars(const std::string &path) {

  if (!path.empty() && path[0] == '~') {
    const char *home = getenv("HOME");
    if (home)
      return std::string(home) + path.substr(1);
  }
  return path;
}

std::vector<std::filesystem::path>
SourceHandler::resolvePath(const std::string &filePath) {
  std::vector<std::filesystem::path> paths;

  std::string normalizedPath = ConfigUtils::normalizePath(filePath);

  Logger::debug("Normalized path: {}", normalizedPath);

  if (normalizedPath.empty()) {
    Logger::error("Path is empty after normalization: {}", filePath);
    return paths;
  }

  glob_t glob_result;
  memset(&glob_result, 0, sizeof(glob_t));

  int ret = glob(normalizedPath.c_str(), GLOB_TILDE | GLOB_BRACE, nullptr,
                 &glob_result);

  if (ret != 0) {
    if (ret == GLOB_NOMATCH) {
      Logger::warn("No matches found for path: {}", normalizedPath);
    } else {
      Logger::error("Glob error for pattern: {}", normalizedPath);
    }

    std::filesystem::path fallbackPath(normalizedPath);
    if (std::filesystem::exists(fallbackPath.parent_path())) {
      paths.push_back(fallbackPath);
    }
    globfree(&glob_result);
    return paths;
  }

  for (size_t i = 0; i < glob_result.gl_pathc; ++i) {
    std::string pathStr = glob_result.gl_pathv[i];
    std::filesystem::path fsPath(pathStr);
    if (fsPath.is_relative() || fsPath.string().find("./") == 0) {
      fsPath = std::filesystem::weakly_canonical(s_configDir / fsPath);
    } else {
      fsPath = std::filesystem::weakly_canonical(fsPath);
    }
    if (std::filesystem::exists(fsPath.parent_path())) {
      paths.push_back(fsPath);
    } else {
      Logger::warn("Directory does not exist: {}",
                   fsPath.parent_path().string());
    }
  }
  globfree(&glob_result);

  Logger::debug("Resolved paths: ");
  for (const auto &p : paths) {
    Logger::debug("PATHS: {}  ::: {}", s_configDir, p.string());
  }

  return paths;
}

Hyprlang::CParseResult SourceHandler::handleSource(const char *command,
                                                   const char *rawpath) {
  Hyprlang::CParseResult result;
  std::string path = rawpath;

  if (path.length() < 2) {
    result.setError("source= path too short or empty");
    return result;
  }

  std::unique_ptr<glob_t, void (*)(glob_t *)> glob_buf{
      static_cast<glob_t *>(calloc(1, sizeof(glob_t))), [](glob_t *g) {
        if (g) {
          globfree(g);
          free(g);
        }
      }};

  std::string absPath;
  if (Logger::isDebug()) {
    Logger::debug("[source] handleSource: rawpath='{}' s_configDir='{}'", path, s_configDir);
  }
  if (path[0] == '~') {
    const char *home = getenv("HOME");
    absPath = home ? std::string(home) + path.substr(1) : path;
    if (Logger::isDebug()) {
      Logger::debug("[source] handleSource: expanded '~' to absPath='{}'", absPath);
    }
  } else if (path[0] != '/') {
    absPath = s_configDir + "/" + path;
    if (Logger::isDebug()) {
      Logger::debug("[source] handleSource: relative path, absPath='{}'", absPath);
    }
  } else {
    absPath = path;
    if (Logger::isDebug()) {
      Logger::debug("[source] handleSource: absolute path, absPath='{}'", absPath);
    }
  }

  int r = glob(absPath.c_str(), GLOB_TILDE, nullptr, glob_buf.get());
  if (r != 0) {
    if (r == GLOB_NOMATCH) {
      if (s_strictMode) {
        std::string err = "source= globbing error: found no match";
        Logger::error("[source] handleSource: glob('{}') failed: {} (code={})", absPath, err, r);
        result.setError(err.c_str());
      } else {
        Logger::debug("[source] handleSource: glob('{}') found no match (source file not present, skipping)", absPath);
      }
      return result;
    }
    std::string err = std::string("source= globbing error: ") +
                      (r == GLOB_ABORTED ? "read error" : "out of memory");
    Logger::error("[source] handleSource: glob('{}') failed: {} (code={})", absPath, err, r);
    result.setError(err.c_str());
    return result;
  }

  std::string errorsFromParsing;

  for (size_t i = 0; i < glob_buf->gl_pathc; i++) {
    std::string value = glob_buf->gl_pathv[i];
    if (!std::filesystem::is_regular_file(value)) {
      if (std::filesystem::exists(value)) {
        Logger::warn("source= skipping non-file {}", value);
        continue;
      }
      std::string err = "source= file " + value + " doesn't exist!";
      Logger::error("{}", err);
      result.setError(err.c_str());
      return result;
    }

    // Always set s_configDir to the parent of the file being parsed, even for the first include
    std::string configDirBackup = s_configDir;
    s_configDir = std::filesystem::path(value).parent_path().string();
    if (Logger::isDebug()) {
      Logger::debug("[source] handleSource: set s_configDir='{}' for file='{}'", s_configDir, value);
    }

    auto parseResult = s_pConfig->parseFile(value.c_str());

    s_configDir = configDirBackup;

    if (parseResult.error && errorsFromParsing.empty())
      errorsFromParsing += parseResult.getError();
  }

  if (!errorsFromParsing.empty())
    result.setError(errorsFromParsing.c_str());
  return result;
}

void SourceHandler::registerHandler(Hyprlang::CConfig *config) {
  s_pConfig = config;
  s_initialized = true;

  Hyprlang::SHandlerOptions options;
  options.allowFlags = false;

  config->registerHandler(
      [](const char *command, const char *value) -> Hyprlang::CParseResult {
        return SourceHandler::handleSource(command, value);
      },
      "source", options);

  Logger::debug("Registered source handler");
}

void SourceHandler::setSuppressNoMatchErrors(bool suppress) {
    s_suppressNoMatchErrors = suppress;
}

void SourceHandler::setStrictMode(bool strict) {
    s_strictMode = strict;
}

} // namespace hyprquery