#include "ConfigUtils.hpp"
#include "ExportEnv.hpp"
#include "ExportJson.hpp"
#include "KeybindHandler.hpp"
#include "Logger.hpp"
#include "SourceHandler.hpp"

#include <CLI/CLI.hpp>
#include <filesystem>
#include <functional>
#include <hyprlang.hpp>
#include <memory>
#include <nlohmann/json.hpp>
#include <regex>

static Hyprlang::CConfig *pConfig = nullptr;

void prepareConfig(const std::vector<hyprquery::QueryInput> &queries,
                   std::string &configFilePath,
                   Hyprlang::SConfigOptions &options,
                   std::vector<std::string> &dynamicVars) {
  for (const auto &q : queries) {
    if (q.isDynamicVariable) {
      std::string envVarName = q.query;
      if (!envVarName.empty() && envVarName[0] == '$') {
        envVarName = envVarName.substr(1);
      }
      if (!getenv(envVarName.c_str()) &&
          setenv(envVarName.c_str(), "", 0) == 0) {
        hyprquery::Logger::debug("[env-export] Pre-exported (blank) {}",
                                 envVarName);
      }
    }
  }

  dynamicVars.resize(queries.size());
  pConfig = new Hyprlang::CConfig(configFilePath.c_str(), options);

  for (size_t i = 0; i < queries.size(); ++i) {
    if (queries[i].isDynamicVariable) {
      dynamicVars[i] = "__hyprquery_capture_" + std::to_string(i);
      pConfig->addConfigValue(dynamicVars[i].c_str(), (Hyprlang::STRING) "");
    } else {
      dynamicVars[i] = queries[i].query;
      pConfig->addConfigValue(queries[i].query.c_str(), (Hyprlang::STRING) "");
    }
  }

  pConfig->commence();
}

std::vector<hyprquery::QueryResult>
executeQueries(const std::vector<hyprquery::QueryInput> &queries,
               const std::vector<std::string> &dynamicVars) {
  std::vector<hyprquery::QueryResult> results;
  results.reserve(queries.size());
  for (size_t i = 0; i < queries.size(); ++i) {
    hyprquery::QueryResult result;
    result.key = queries[i].query;

    std::string lookupKey = queries[i].query;

    if (queries[i].isDynamicVariable) {
      const std::string varName = queries[i].query.substr(1);
      const std::string captureKey = dynamicVars[i];

      const std::string line = captureKey + " = $" + varName;
      const auto dynResult = pConfig->parseDynamic(line.c_str());
      if (dynResult.error) {
        hyprquery::Logger::debug("[capture] Failed to capture ${}: {}", varName,
                                 dynResult.getError());
      }

      lookupKey = captureKey;
    }

    std::any value = pConfig->getConfigValue(lookupKey.c_str());
    result.value = hyprquery::ConfigUtils::convertValueToString(value);
    result.type = hyprquery::ConfigUtils::getValueTypeName(value);

    if (!queries[i].expectedType.empty()) {
      if (hyprquery::normalizeType(result.type) !=
          hyprquery::normalizeType(queries[i].expectedType)) {
        result.value = "";
        result.type = "NULL";
      }
    }
    if (!queries[i].expectedRegex.empty()) {
      try {
        std::regex rx(queries[i].expectedRegex);
        if (!std::regex_match(result.value, rx)) {
          result.value = "";
          result.type = "NULL";
        }
      } catch (const std::regex_error &) {
        result.value = "";
        result.type = "NULL";
      }
    }
    results.push_back(result);
  }
  return results;
}

void outputResults(const std::vector<hyprquery::QueryResult> &results,
                   const std::string &exportFormat,
                   const std::string &delimiter,
                   const std::vector<hyprquery::QueryInput> &queries) {
  if (exportFormat == "json") {
    hyprquery::exportJson(results);
  } else if (exportFormat == "env") {
    std::vector<hyprquery::QueryInput> patchedQueries = queries;
    for (auto &q : patchedQueries) {
      if (!q.query.empty() && q.query[0] == '$') {
        q.query = q.query.substr(1);
      }
    }
    hyprquery::exportEnv(results, patchedQueries);
  } else {
    for (size_t i = 0; i < results.size(); ++i) {
      std::cout << (results[i].type == "NULL" ? "" : results[i].value);
      if (i + 1 < results.size())
        std::cout << delimiter;
    }
    std::cout << std::endl;
  }
}

void cliOptions(CLI::App &app, std::vector<std::string> &rawQueries,
                std::string &configFilePath, std::string &schemaFilePath,
                bool &allowMissing, bool &getDefaultKeys, bool &strictMode,
                bool &followSource, bool &debugLogging, std::string &delimiter,
                std::string &exportFormat, bool &listBinds) {
  app.add_option(
         "--query,-Q", rawQueries,
         "Query to execute (format: query[expectedType][expectedRegex], can be "
         "specified multiple times)")
      ->take_all();
  app.add_option("config_file", configFilePath, "Configuration file")
      ->required();
  app.add_option("--schema", schemaFilePath, "Schema file");
  app.add_flag("--allow-missing", allowMissing, "Allow missing values");
  app.add_flag("--get-defaults", getDefaultKeys, "Get default keys");
  app.add_flag("--strict", strictMode, "Enable strict mode");
  app.add_option("--export", exportFormat, "Export format: json or env");
  app.add_flag("--source,-s", followSource, "Follow the source command");
  app.add_flag("--debug", debugLogging, "Enable debug logging");
  app.add_option("--delimiter,-D", delimiter,
                 "Delimiter for plain output (default: newline)");
  app.add_flag("--binds", listBinds, "List all binds in the config file");
}

bool noOperationRequested(bool listBinds,
                          const std::vector<std::string> &rawQueries) {
  return !listBinds && rawQueries.empty();
}

std::string resolveConfigPath(const std::string &configFilePath) {
  std::string normalized =
      hyprquery::ConfigUtils::normalizePath(configFilePath);
  auto resolvedPaths = hyprquery::SourceHandler::resolvePath(normalized);
  if (resolvedPaths.empty()) {
    std::cerr << "Error: Could not resolve configuration file path: "
              << normalized << std::endl;
    return "";
  }

  std::string resolved = resolvedPaths.front().string();
  if (!std::filesystem::exists(resolved)) {
    std::cerr << "Error: Configuration file does not exist: " << resolved
              << std::endl;
    return "";
  }
  return resolved;
}

std::string resolveSchemaPath(const std::string &schemaFilePath) {
  if (schemaFilePath.empty()) {
    return "";
  }

  std::string normalized =
      hyprquery::ConfigUtils::normalizePath(schemaFilePath);
  auto resolvedPaths = hyprquery::SourceHandler::resolvePath(normalized);
  if (!resolvedPaths.empty()) {
    normalized = resolvedPaths.front().string();
  }

  if (!std::filesystem::exists(normalized)) {
    std::cerr << "Error: Schema file does not exist: " << normalized
              << std::endl;
    return "";
  }
  return normalized;
}

void setConfDir(const std::string &configFilePath) {
  hyprquery::SourceHandler::setConfigDir(
      std::filesystem::path(configFilePath).parent_path().string());
}

std::unique_ptr<Hyprlang::CConfig>
buildBinds(const std::string &configFilePath) {
  Hyprlang::SConfigOptions options;
  options.allowMissingConfig = true;
  options.pathIsStream = false;
  return std::make_unique<Hyprlang::CConfig>(configFilePath.c_str(), options);
}

void regBindHandler(Hyprlang::CConfig *config) {
  Hyprlang::SHandlerOptions hOpts;
  hOpts.allowFlags = true;
  config->registerHandler(
      [](const char *command, const char *value) -> Hyprlang::CParseResult {
        return hyprquery::KeybindCollector::handleBind(command, value);
      },
      "bind", hOpts);
  config->registerHandler(
      [](const char *command, const char *value) -> Hyprlang::CParseResult {
        return hyprquery::KeybindCollector::handleUnbind(command, value);
      },
      "unbind", hOpts);
  hyprquery::Logger::debug(
      "Registered handler for 'bind', 'unbind', all bind* "
      "variants. All other keywords are ignored by default.");
}

void regSourceHandler(Hyprlang::CConfig *config, bool followSource,
                      const std::string &contextMessage) {
  if (!followSource) {
    return;
  }
  hyprquery::Logger::debug("{}", contextMessage);
  hyprquery::SourceHandler::registerHandler(config);
}

void outputBinds(const std::vector<hyprquery::BindEntry> &binds,
                 const std::string &exportFormat,
                 const std::string &delimiter) {
  if (exportFormat == "json") {
    hyprquery::exportBindsJson(binds);
  } else {
    for (const auto &bind : binds) {
      std::cout << bind.raw << delimiter;
    }
  }
}

int runBindsMode(const std::string &configFilePath,
                 const std::string &exportFormat, const std::string &delimiter,
                 bool strictMode, bool debugLogging, bool followSource) {
  hyprquery::SourceHandler::setSuppressNoMatchErrors(true);
  hyprquery::SourceHandler::setStrictMode(strictMode);

  std::string resolvedPath = resolveConfigPath(configFilePath);
  if (resolvedPath.empty()) {
    return 1;
  }

  setConfDir(resolvedPath);

  auto config = buildBinds(resolvedPath);
  regBindHandler(config.get());
  regSourceHandler(
      config.get(), followSource,
      "Registering source handler for 'source' keyword (main.cpp, --binds "
      "mode)");

  auto binds = hyprquery::KeybindHandler::parseKeybinds(
      resolvedPath, strictMode, followSource, config.get());
  outputBinds(binds, exportFormat, delimiter);

  return 0;
}

Hyprlang::SConfigOptions createQueryConfigOptions(bool getDefaultKeys) {
  Hyprlang::SConfigOptions options;
  options.verifyOnly = getDefaultKeys;
  options.allowMissingConfig = true;
  return options;
}

void configureDebugLogging(bool debugLogging) {
  hyprquery::Logger::setDebug(debugLogging);
}

bool parseConfigSucceeded(bool strictMode) {
  const auto parseResult = pConfig->parse();
  if (parseResult.error) {
    hyprquery::Logger::debug("Parse error: {}", parseResult.getError());
    if (strictMode) {
      return false;
    }
  }
  return true;
}

bool hasNull(const std::vector<hyprquery::QueryResult> &results) {
  for (const auto &r : results) {
    if (r.type == "NULL") {
      return true;
    }
  }
  return false;
}

int runQueryMode(const std::string &configFilePath,
                 const std::string &schemaFilePath,
                 const std::vector<std::string> &rawQueries,
                 const std::string &exportFormat, const std::string &delimiter,
                 bool getDefaultKeys, bool strictMode, bool debugLogging,
                 bool followSource) {
  hyprquery::SourceHandler::setSuppressNoMatchErrors(false);
  hyprquery::SourceHandler::setStrictMode(strictMode);

  std::string resolvedPath = resolveConfigPath(configFilePath);
  if (resolvedPath.empty()) {
    return 1;
  }

  setConfDir(resolvedPath);

  std::string resolvedSchema = resolveSchemaPath(schemaFilePath);
  if (!schemaFilePath.empty() && resolvedSchema.empty()) {
    return 1;
  }

  auto options = createQueryConfigOptions(getDefaultKeys);
  auto queries = hyprquery::parseQueryInputs(rawQueries);
  std::vector<std::string> dynamicVars;

  prepareConfig(queries, resolvedPath, options, dynamicVars);

  if (!resolvedSchema.empty()) {
    hyprquery::ConfigUtils::addConfigValuesFromSchema(*pConfig, resolvedSchema);
  }

  regSourceHandler(pConfig, followSource, "Registering source handler");
  configureDebugLogging(debugLogging);

  if (!parseConfigSucceeded(strictMode)) {
    return 1;
  }

  auto results = executeQueries(queries, dynamicVars);
  outputResults(results, exportFormat, delimiter, queries);

  return hasNull(results) ? 1 : 0;
}

int main(int argc, char **argv) {
  CLI::App app{"hyprquery - A configuration parser for hypr* config files"};
  std::vector<std::string> rawQueries;
  std::string configFilePath;
  std::string schemaFilePath;
  bool allowMissing = false;
  bool getDefaultKeys = false;
  bool strictMode = false;
  bool followSource = false;
  bool debugLogging = false;
  std::string delimiter = "\n";
  std::string exportFormat;
  bool listBinds = false;

  cliOptions(app, rawQueries, configFilePath, schemaFilePath, allowMissing,
             getDefaultKeys, strictMode, followSource, debugLogging, delimiter,
             exportFormat, listBinds);
  CLI11_PARSE(app, argc, argv);

  if (noOperationRequested(listBinds, rawQueries)) {
    std::cout << app.help() << std::endl;
    return 0;
  }

  if (listBinds) {
    return runBindsMode(configFilePath, exportFormat, delimiter, strictMode,
                        debugLogging, followSource);
  }

  return runQueryMode(configFilePath, schemaFilePath, rawQueries, exportFormat,
                      delimiter, getDefaultKeys, strictMode, debugLogging,
                      followSource);
}
