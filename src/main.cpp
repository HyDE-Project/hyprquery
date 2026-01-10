#include "ConfigUtils.hpp"
#include "ExportEnv.hpp"
#include "ExportJson.hpp"
#include "SourceHandler.hpp"
#include <CLI/CLI.hpp>
#include <filesystem>
#include <functional>
#include <hyprlang.hpp>
#include <nlohmann/json.hpp>
#include <regex>
#include <spdlog/spdlog.h>

using hyprquery::exportEnv;
using hyprquery::exportJson;

static Hyprlang::CConfig *pConfig = nullptr;

void prepareConfig(const std::vector<hyprquery::QueryInput> &queries,
                   std::string &configFilePath,
                   Hyprlang::SConfigOptions &options,
                   std::vector<std::string> &dynamicVars, bool debugLogging) {

  for (const auto &q : queries) {
    if (q.isDynamicVariable) {
      std::string envVarName = q.query;
      if (!envVarName.empty() && envVarName[0] == '$') {
        envVarName = envVarName.substr(1);
      }

      if (!getenv(envVarName.c_str()) &&
          setenv(envVarName.c_str(), "", 0) == 0) {
        if (debugLogging)
          spdlog::debug(std::string("[env-export] Pre-exported (blank) ") +
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
               const std::vector<std::string> &dynamicVars, bool debugLogging) {
  std::vector<hyprquery::QueryResult> results;
  for (size_t i = 0; i < queries.size(); ++i) {
    hyprquery::QueryResult result;
    result.key = queries[i].query;

    std::string lookupKey = queries[i].query;

    if (queries[i].isDynamicVariable) {
      const std::string varName = queries[i].query.substr(1);
      const std::string captureKey = dynamicVars[i];

      const std::string line = captureKey + " = $" + varName;
      const auto dynResult = pConfig->parseDynamic(line.c_str());
      if (debugLogging && dynResult.error) {
        spdlog::debug(std::string("[capture] Failed to capture $") + varName +
                      ": " + dynResult.getError());
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
  app.add_option(
         "--query,-Q", rawQueries,
         "Query to execute (format: query[expectedType][expectedRegex], can be "
         "specified multiple times)")
      ->required()
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
  CLI11_PARSE(app, argc, argv);
  configFilePath = hyprquery::ConfigUtils::normalizePath(configFilePath);
  auto resolvedPaths = hyprquery::SourceHandler::resolvePath(configFilePath);
  if (resolvedPaths.empty()) {
    std::cerr << "Error: Could not resolve configuration file path: "
              << configFilePath << std::endl;
    return 1;
  }
  configFilePath = resolvedPaths.front().string();
  if (!std::filesystem::exists(configFilePath)) {
    std::cerr << "Error: Configuration file does not exist: " << configFilePath
              << std::endl;
    return 1;
  }
  hyprquery::SourceHandler::setConfigDir(
      std::filesystem::path(configFilePath).parent_path().string());
  if (!schemaFilePath.empty()) {
    schemaFilePath = hyprquery::ConfigUtils::normalizePath(schemaFilePath);
    auto resolvedSchemaPath =
        hyprquery::SourceHandler::resolvePath(schemaFilePath);
    if (!resolvedSchemaPath.empty()) {
      schemaFilePath = resolvedSchemaPath.front().string();
    }
    if (!std::filesystem::exists(schemaFilePath)) {
      std::cerr << "Error: Schema file does not exist: " << schemaFilePath
                << std::endl;
      return 1;
    }
  }
  Hyprlang::SConfigOptions options;
  options = {.verifyOnly = static_cast<bool>(getDefaultKeys ? 1 : 0),
             .allowMissingConfig = static_cast<bool>(1)};
  std::vector<hyprquery::QueryInput> queries =
      hyprquery::parseQueryInputs(rawQueries);
  std::vector<std::string> dynamicVars;
  prepareConfig(queries, configFilePath, options, dynamicVars, debugLogging);
  if (!schemaFilePath.empty()) {
    hyprquery::ConfigUtils::addConfigValuesFromSchema(*pConfig, schemaFilePath);
  }
  if (followSource) {
    if (debugLogging)
      spdlog::debug("Registering source handler");
    hyprquery::SourceHandler::registerHandler(pConfig);
  }
  if (debugLogging) {
    spdlog::set_level(spdlog::level::debug);
    spdlog::flush_on(spdlog::level::debug);
  } else {
    spdlog::set_level(spdlog::level::off);
  }
  const auto PARSERESULT = pConfig->parse();
  if (PARSERESULT.error) {
    if (debugLogging)
      spdlog::debug(std::string("Parse error: ") + PARSERESULT.getError());
    if (strictMode) {
      return 1;
    }
  }
  std::vector<hyprquery::QueryResult> results =
      executeQueries(queries, dynamicVars, debugLogging);
  int nullCount = 0;
  for (const auto &r : results) {
    if (r.type == "NULL")
      nullCount++;
  }
  outputResults(results, exportFormat, delimiter, queries);
  return nullCount > 0 ? 1 : 0;
}
