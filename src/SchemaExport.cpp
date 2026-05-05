#include "SchemaExport.hpp"
#include "ConfigUtils.hpp"
#include "Logger.hpp"
#include "SourceHandler.hpp"

#include <any>
#include <filesystem>
#include <fstream>
#include <hyprlang.hpp>
#include <iostream>
#include <map>
#include <memory>
#include <nlohmann/json.hpp>
#include <sstream>
#include <string>
#include <vector>

namespace hyprquery {

// Sentinel values used to detect whether hyprlang actually parsed a value from
// the config file.  After parsing, if a registered value still equals its
// sentinel it was not present in the config.
static constexpr Hyprlang::INT   SENTINEL_INT    = 0x7FFFFFFE;
static constexpr Hyprlang::FLOAT SENTINEL_FLOAT  = -999999.125f;
static const std::string         SENTINEL_STRING = "__HYPRQUERY_SENTINEL__";

static bool isSentinel(const std::any &val, const std::string &schemaType) {
  if (schemaType == "INT" || schemaType == "BOOL") {
    if (val.type() == typeid(Hyprlang::INT))
      return std::any_cast<Hyprlang::INT>(val) == SENTINEL_INT;
  } else if (schemaType == "FLOAT") {
    if (val.type() == typeid(Hyprlang::FLOAT))
      return std::any_cast<Hyprlang::FLOAT>(val) == SENTINEL_FLOAT;
  } else if (schemaType == "STRING_SHORT" || schemaType == "STRING_LONG" ||
             schemaType == "GRADIENT"     || schemaType == "COLOR"        ||
             schemaType == "CHOICE") {
    if (val.type() == typeid(Hyprlang::STRING))
      return std::string(std::any_cast<Hyprlang::STRING>(val)) == SENTINEL_STRING;
  } else if (schemaType == "VECTOR") {
    if (val.type() == typeid(Hyprlang::VEC2)) {
      auto v = std::any_cast<Hyprlang::VEC2>(val);
      return v.x == -999999.0f && v.y == -999999.0f;
    }
  }
  return false;
}

// Convert a schema "data.default" JSON value to a display string.
// The schema may store numeric defaults as either JSON numbers or JSON strings,
// so we check the actual JSON type and convert accordingly.
static std::string schemaDefaultToString(const nlohmann::json &data,
                                         const std::string &type) {
  if (!data.contains("default"))
    return "";
  const auto &def = data["default"];

  if (type == "BOOL") {
    if (def.is_boolean()) return def.get<bool>() ? "true" : "false";
    if (def.is_number()) return def.get<int>() ? "true" : "false";
    if (def.is_string()) {
      const auto s = def.get<std::string>();
      return (s == "true" || s == "1" || s == "yes" || s == "on") ? "true" : "false";
    }
    return "false";
  }
  if (type == "INT") {
    if (def.is_number()) return std::to_string(def.get<int64_t>());
    if (def.is_string()) return def.get<std::string>();
    return def.dump();
  }
  if (type == "FLOAT") {
    if (def.is_number()) return std::to_string(def.get<float>());
    if (def.is_string()) return def.get<std::string>();
    return def.dump();
  }
  if (type == "VECTOR") {
    if (def.is_array() && def.size() == 2) {
      auto toF = [](const nlohmann::json &v) -> float {
        if (v.is_number()) return v.get<float>();
        if (v.is_string()) try { return std::stof(v.get<std::string>()); } catch (...) {}
        return 0.0f;
      };
      return std::to_string(toF(def[0])) + ", " + std::to_string(toF(def[1]));
    }
    return "0, 0";
  }
  if (def.is_string())
    return def.get<std::string>();
  return def.dump();
}

// Split "a:b:c" → ["a","b","c"]
static std::vector<std::string> splitKey(const std::string &key) {
  std::vector<std::string> parts;
  std::stringstream ss(key);
  std::string part;
  while (std::getline(ss, part, ':'))
    parts.push_back(part);
  return parts;
}

// Split "a:b.c:d" → ["a","b","c","d"]  (both ':' and '.' are separators)
// Used for Lua export so that col.active_border becomes col = { active_border = ... }
static std::vector<std::string> splitKeyFull(const std::string &key) {
  std::vector<std::string> parts;
  std::string cur;
  for (char c : key) {
    if (c == ':' || c == '.') {
      if (!cur.empty()) { parts.push_back(cur); cur.clear(); }
    } else {
      cur += c;
    }
  }
  if (!cur.empty()) parts.push_back(cur);
  return parts;
}

// Navigate/create the nested JSON object path and return a reference to the leaf parent.
// If an existing leaf node (with __v) is in the way, it is replaced with an object.
static nlohmann::json &treeNode(nlohmann::json &root,
                                 const std::vector<std::string> &parts) {
  nlohmann::json *node = &root;
  for (size_t i = 0; i + 1 < parts.size(); ++i) {
    auto &child = (*node)[parts[i]];
    if (!child.is_object() || child.contains("__v"))
      child = nlohmann::json::object();
    node = &child;
  }
  return *node;
}

// Insert a plain string value (hypr / nested-json formats).
static void insertIntoTree(nlohmann::json &root, const std::vector<std::string> &parts,
                           const std::string &value) {
  treeNode(root, parts)[parts.back()] = value;
}

// Insert a typed leaf for Lua emission: {"__v": value, "__t": type}.
static void insertIntoTreeTyped(nlohmann::json &root,
                                 const std::vector<std::string> &parts,
                                 const std::string &value,
                                 const std::string &type) {
  treeNode(root, parts)[parts.back()] = {{"__v", value}, {"__t", type}};
}

// Emit hyprland config format (nested sections)
static void emitHypr(std::ostream &out, const nlohmann::json &node,
                     const std::string &indent = "") {
  // First emit plain key = value pairs, then nested sections
  for (auto it = node.begin(); it != node.end(); ++it) {
    if (it.value().is_string())
      out << indent << it.key() << " = " << it.value().get<std::string>() << "\n";
  }
  for (auto it = node.begin(); it != node.end(); ++it) {
    if (it.value().is_object()) {
      out << indent << it.key() << " {\n";
      emitHypr(out, it.value(), indent + "  ");
      out << indent << "}\n";
    }
  }
}

// Convert a stored value string to its proper Lua literal.
static std::string toLuaLiteral(const std::string &value, const std::string &type) {
  if (type == "BOOL")
    // Config-sourced: hyprlang emits INT "0"/"1"; fallback: already "true"/"false"
    return (value == "1" || value == "true") ? "true" : "false";

  if (type == "INT" || type == "FLOAT")
    return value.empty() ? "0" : value;

  if (type == "VECTOR") {
    // "x, y" → {x, y}
    auto comma = value.find(',');
    if (comma != std::string::npos) {
      std::string x = value.substr(0, comma);
      std::string y = value.substr(comma + 1);
      while (!x.empty() && x.front() == ' ') x.erase(x.begin());
      while (!x.empty() && x.back()  == ' ') x.pop_back();
      while (!y.empty() && y.front() == ' ') y.erase(y.begin());
      while (!y.empty() && y.back()  == ' ') y.pop_back();
      return "{" + x + ", " + y + "}";
    }
    return "{0, 0}";
  }

  if (type == "GRADIENT") {
    // Format: "color1 color2 ... [Ndeg]"
    // Tokens are space-separated; rgba/0x colors have no internal spaces.
    std::vector<std::string> tokens;
    std::istringstream iss(value);
    std::string tok;
    while (iss >> tok) tokens.push_back(tok);

    if (tokens.empty()) return nlohmann::json(value).dump();

    // Check for trailing angle token (e.g. "45deg")
    int angle = -1;
    if (!tokens.empty()) {
      const std::string &last = tokens.back();
      if (last.size() >= 4 && last.substr(last.size() - 3) == "deg") {
        try {
          angle = std::stoi(last.substr(0, last.size() - 3));
          tokens.pop_back();
        } catch (...) {}
      }
    }

    if (tokens.size() == 1 && angle < 0)
      return nlohmann::json(tokens[0]).dump(); // single color — plain string

    std::string result = "{colors = {";
    for (size_t i = 0; i < tokens.size(); ++i) {
      if (i > 0) result += ", ";
      result += nlohmann::json(tokens[i]).dump();
    }
    result += "}";
    if (angle >= 0) result += ", angle = " + std::to_string(angle);
    result += "}";
    return result;
  }

  // STRING_SHORT, STRING_LONG, COLOR, CHOICE → quoted string
  return nlohmann::json(value).dump();
}

// Emit Lua table format with native types.
static void emitLua(std::ostream &out, const nlohmann::json &node,
                    const std::string &indent = "  ") {
  for (auto it = node.begin(); it != node.end(); ++it) {
    const auto &val = it.value();
    if (val.is_object() && val.contains("__v") && val.contains("__t")) {
      out << indent << it.key() << " = "
          << toLuaLiteral(val["__v"].get<std::string>(), val["__t"].get<std::string>())
          << ",\n";
    } else if (val.is_object()) {
      out << indent << it.key() << " = {\n";
      emitLua(out, val, indent + "  ");
      out << indent << "},\n";
    }
  }
}

int runDumpMode(const std::string &configFilePath,
                const std::string &schemaFilePath,
                const std::string &exportFormat, bool strictMode,
                bool useFallback, bool followSource, bool debugLogging) {
  Logger::setDebug(debugLogging);
  SourceHandler::setSuppressNoMatchErrors(true);
  SourceHandler::setStrictMode(strictMode);

  // --- Resolve paths ---
  std::string resolvedConfig = ConfigUtils::normalizePath(configFilePath);
  if (!std::filesystem::exists(resolvedConfig)) {
    Logger::error("Config file not found: {}", configFilePath);
    return 1;
  }

  std::string resolvedSchema = ConfigUtils::normalizePath(schemaFilePath);
  if (!std::filesystem::exists(resolvedSchema)) {
    Logger::error("Schema file not found: {}", schemaFilePath);
    return 1;
  }

  // --- Load schema ---
  std::ifstream schemaFile(resolvedSchema);
  if (!schemaFile.is_open()) {
    Logger::error("Failed to open schema file: {}", resolvedSchema);
    return 1;
  }
  nlohmann::json schemaJson;
  try {
    schemaFile >> schemaJson;
  } catch (const std::exception &e) {
    Logger::error("Failed to parse schema JSON: {}", e.what());
    return 1;
  }
  if (!schemaJson.contains("hyprlang_schema")) {
    Logger::error("Invalid schema: missing 'hyprlang_schema' key");
    return 1;
  }

  // --- Set up source handler context ---
  SourceHandler::setConfigDir(
      std::filesystem::path(resolvedConfig).parent_path().string());

  // --- Build hyprlang config ---
  Hyprlang::SConfigOptions opts;
  opts.allowMissingConfig = true;
  opts.pathIsStream       = false;
  auto config = std::make_unique<Hyprlang::CConfig>(resolvedConfig.c_str(), opts);

  // Register every schema key with a sentinel so we can detect post-parse
  // whether the config file actually contained a value for it.
  const auto &entries = schemaJson["hyprlang_schema"];
  for (const auto &entry : entries) {
    if (!entry.contains("value") || !entry.contains("type"))
      continue;
    const std::string key  = entry["value"].get<std::string>();
    const std::string type = entry["type"].get<std::string>();

    if (type == "INT" || type == "BOOL") {
      config->addConfigValue(key.c_str(), SENTINEL_INT);
    } else if (type == "FLOAT") {
      config->addConfigValue(key.c_str(), SENTINEL_FLOAT);
    } else if (type == "STRING_SHORT" || type == "STRING_LONG" ||
               type == "GRADIENT"     || type == "COLOR"        ||
               type == "CHOICE") {
      config->addConfigValue(key.c_str(),
                             (Hyprlang::STRING)SENTINEL_STRING.c_str());
    } else if (type == "VECTOR") {
      Hyprlang::VEC2 s{-999999.0f, -999999.0f};
      config->addConfigValue(key.c_str(), s);
    }
    // Unknown types are skipped — they will be absent from output.
  }

  config->commence();

  if (followSource)
    SourceHandler::registerHandler(config.get());

  const auto parseResult = config->parse();
  if (parseResult.error) {
    Logger::debug("Parse error: {}", parseResult.getError());
    if (strictMode) {
      Logger::error("Parse error (strict mode): {}", parseResult.getError());
      return 1;
    }
  }

  // --- Collect and emit results ---
  nlohmann::json output = nlohmann::json::array();
  bool hadError = false;

  for (const auto &entry : entries) {
    if (!entry.contains("value") || !entry.contains("type") ||
        !entry.contains("data"))
      continue;

    const std::string key  = entry["value"].get<std::string>();
    const std::string type = entry["type"].get<std::string>();
    const auto       &data = entry["data"];

    std::any rawVal = config->getConfigValue(key.c_str());
    if (!rawVal.has_value())
      continue; // unsupported / skipped type

    const bool inConfig = !isSentinel(rawVal, type);

    if (!inConfig) {
      if (strictMode) {
        Logger::error("Key '{}' not found in config (strict mode)", key);
        hadError = true;
        continue;
      }
      if (!useFallback)
        continue; // default: silently skip missing keys

      // --fallback: use schema default
      output.push_back({{"key", key},
                        {"value", schemaDefaultToString(data, type)},
                        {"type", type},
                        {"source", "default"}});
    } else {
      output.push_back({{"key", key},
                        {"value", ConfigUtils::convertValueToString(rawVal)},
                        {"type", type}});
    }
  }

  if (exportFormat == "hypr") {
    nlohmann::json tree = nlohmann::json::object();
    for (const auto &item : output) {
      insertIntoTree(tree, splitKey(item["key"].get<std::string>()),
                     item["value"].get<std::string>());
    }
    emitHypr(std::cout, tree);
  } else if (exportFormat == "lua") {
    nlohmann::json tree = nlohmann::json::object();
    for (const auto &item : output) {
      insertIntoTreeTyped(tree, splitKeyFull(item["key"].get<std::string>()),
                          item["value"].get<std::string>(),
                          item["type"].get<std::string>());
    }
    std::cout << "return {\n";
    emitLua(std::cout, tree);
    std::cout << "}\n";
  } else if (exportFormat == "nested-json") {
    nlohmann::json tree = nlohmann::json::object();
    for (const auto &item : output) {
      insertIntoTree(tree, splitKey(item["key"].get<std::string>()),
                     item["value"].get<std::string>());
    }
    std::cout << tree.dump(2) << std::endl;
  } else {
    std::cout << output.dump(2) << std::endl;
  }

  return hadError ? 1 : 0;
}

} // namespace hyprquery
