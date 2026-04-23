
#include "KeybindHandler.hpp"
#include "Logger.hpp"
#include "SourceHandler.hpp"
#include "XKBKeymapResolver.hpp"
#include <algorithm>
#include <fstream>
#include <hyprlang.hpp>
#include <regex>
#include <sstream>
#include <string>
#include <vector>

namespace hyprquery {

static const std::vector<std::pair<char, std::string>> FLAG_DEFS = {
    {'l', "locked"},        {'r', "release"},
    {'c', "click"},         {'g', "drag"},
    {'o', "longPress"},     {'e', "repeat"},
    {'n', "non_consuming"}, {'m', "mouse"},
    {'t', "transparent"},   {'i', "ignore_mods"},
    {'s', "separate"},      {'d', "has_description"},
    {'p', "bypass"},        {'u', "submap_universal"},
    {'k', "per_device"}};

Hyprlang::CParseResult KeybindCollector::handleBind(const char *command,
                                                    const char *value) {
  Logger::debug("[KeybindCollector] handleBind: {} = {}", command, value);
  if (instance) {
    std::string line = std::string(command) + " = " + std::string(value);
    auto entry = parseBindLine(line);
    if (entry.has_value()) {
      instance->activeBinds.push_back(entry.value());
    }
  }
  return Hyprlang::CParseResult{};
}

Hyprlang::CParseResult KeybindCollector::handleUnbind(const char *command,
                                                      const char *value) {
  Logger::debug("[KeybindCollector] handleUnbind: {} = {}", command, value);
  if (instance) {
    std::string line = std::string(command) + " = " + std::string(value);
    auto entry = parseBindLine(line);
    if (entry.has_value()) {
      const BindEntry &key = entry.value();
      auto &binds = instance->activeBinds;
      bool wildcard = key.flags.empty();
      binds.erase(std::remove_if(binds.begin(), binds.end(),
                                 [&key, wildcard](const BindEntry &bind) {
                                   if (bind.modmask != key.modmask ||
                                       bind.submap != key.submap)
                                     return false;

                                   if (key.keycode != 0 && bind.keycode != 0) {
                                     if (bind.keycode != key.keycode)
                                       return false;
                                   } else {
                                     if (bind.key != key.key)
                                       return false;
                                   }
                                   return wildcard || bind.flags == key.flags;
                                 }),
                  binds.end());
    }
  }
  return Hyprlang::CParseResult{};
}

KeybindCollector *KeybindCollector::instance = nullptr;
std::unique_ptr<XKBKeymapResolver> KeybindCollector::resolver = nullptr;

std::string_view KeybindCollector::trimView(std::string_view sv) {
  size_t start = sv.find_first_not_of(" \t");
  if (start == std::string_view::npos)
    return {};
  size_t end = sv.find_last_not_of(" \t");
  return sv.substr(start, end - start + 1);
}

std::pair<std::string_view, std::string_view>
KeybindCollector::splitAtEquals(std::string_view line) {
  size_t eqPos = line.find('=');
  if (eqPos == std::string_view::npos)
    return {{}, {}};

  std::string_view left = line.substr(0, eqPos);
  std::string_view right = line.substr(eqPos + 1);
  return {trimView(left), trimView(right)};
}

std::string KeybindCollector::extractFlags(std::string_view command) {
  std::string flags;
  if (command.size() > 4 && command.substr(0, 4) == "bind") {
    flags = std::string(command.substr(4));
  } else if (command.size() > 6 && command.substr(0, 6) == "unbind") {
    flags = std::string(command.substr(6));
  }
  std::sort(flags.begin(), flags.end());
  return flags;
}

std::vector<std::string>
KeybindCollector::splitByCommaRespectingQuotes(std::string_view right) {
  std::vector<std::string> parts;
  std::string current;
  bool inQuotes = false;

  for (char c : right) {
    if (c == '"')
      inQuotes = !inQuotes;
    if (c == ',' && !inQuotes) {
      parts.push_back(std::move(current));
      current.clear();
    } else {
      current += c;
    }
  }
  if (!current.empty())
    parts.push_back(std::move(current));

  return parts;
}

void KeybindCollector::trimStrings(std::vector<std::string> &parts) {
  for (auto &p : parts) {
    std::string_view trimmed = trimView(p);
    p = std::string(trimmed);
  }
}

struct FlagMapping {
  char flag;
  bool BindEntry::*member;
};

static const std::vector<FlagMapping> FLAG_MAP = {
    {'l', &BindEntry::locked},        {'r', &BindEntry::release},
    {'c', &BindEntry::click},         {'g', &BindEntry::drag},
    {'o', &BindEntry::longPress},     {'e', &BindEntry::repeat},
    {'n', &BindEntry::non_consuming}, {'m', &BindEntry::mouse},
    {'t', &BindEntry::transparent},   {'i', &BindEntry::ignore_mods},
    {'s', &BindEntry::separate},      {'d', &BindEntry::has_description},
    {'p', &BindEntry::bypass},        {'u', &BindEntry::submap_universal},
    {'k', &BindEntry::per_device},
};

void KeybindCollector::setFlagBooleans(BindEntry &entry,
                                       const std::string &flags) {
  for (const auto &map : FLAG_MAP) {
    entry.*map.member = flags.find(map.flag) != std::string::npos;
  }
}

void KeybindCollector::resolveKeycodes(BindEntry &entry,
                                       const std::vector<std::string> &parts) {
  int modmask = 0;
  int keycode = 0;

  std::string keyStr = parts.size() > 1 ? parts[1] : "";

  if (resolver && resolver->valid()) {
    if (!parts.empty()) {
      modmask = resolver->resolveHyprlandModMask(parts[0]);
    }
    keycode = resolver->resolveKeycode(keyStr);
    entry.catch_all = (keyStr == "catchall");
  } else {
    entry.catch_all = false;
  }

  entry.modmask = modmask;
  entry.keycode = keycode;
  entry.submap = "";
  entry.modifiers = !parts.empty() ? parts[0] : "";

  if (keyStr.size() > 5 && keyStr.substr(0, 5) == "code:" && keycode != 0) {
    entry.key = "";
  } else {
    entry.key = keyStr;
  }
}

void KeybindCollector::parseDevices(BindEntry &entry,
                                    const std::vector<std::string> &parts,
                                    size_t &idx) {
  if (idx >= parts.size()) {
    idx++;
    return;
  }

  const std::string &devicesField = parts[idx];
  std::vector<std::string> devices;
  bool exclude = false;

  size_t start = 0;
  while (start < devicesField.size()) {
    while (start < devicesField.size() &&
           (devicesField[start] == ' ' || devicesField[start] == '\t'))
      start++;
    if (start >= devicesField.size())
      break;

    size_t end = start;
    while (end < devicesField.size() && devicesField[end] != ' ' &&
           devicesField[end] != '\t')
      end++;

    std::string dev = devicesField.substr(start, end - start);
    if (!dev.empty()) {
      if (dev[0] == '!') {
        exclude = true;
        dev = dev.substr(1);
      }
      if (!dev.empty())
        devices.push_back(std::move(dev));
    }
    start = end;
  }

  if (!devices.empty()) {
    entry.devices = std::move(devices);
    entry.devices_exclude = exclude;
  }
  idx++;
}

void KeybindCollector::parseDescription(BindEntry &entry,
                                        const std::vector<std::string> &parts,
                                        size_t &idx) {
  entry.description = parts.size() > idx ? parts[idx] : "";
  idx++;
}

void KeybindCollector::parseDispatcherAndArgs(
    BindEntry &entry, const std::vector<std::string> &parts, size_t idx) {
  entry.dispatcher = parts.size() > idx ? parts[idx] : "";
  idx++;

  if (parts.size() > idx) {
    entry.arg = parts[idx];
    idx++;
  }

  for (size_t i = idx; i < parts.size(); ++i)
    entry.extra_args.push_back(parts[i]);
}

std::optional<BindEntry>
KeybindCollector::parseBindLine(std::string_view line) {

  line = trimView(line);
  if (line.empty())
    return std::nullopt;

  auto [left, right] = splitAtEquals(line);
  if (left.empty() || right.empty())
    return std::nullopt;

  std::string_view command = left.substr(0, left.find(' '));
  std::string flags = extractFlags(command);

  std::vector<std::string> parts = splitByCommaRespectingQuotes(right);
  trimStrings(parts);

  BindEntry entry;
  entry.raw = std::string(line);
  entry.flags = flags;

  setFlagBooleans(entry, flags);
  resolveKeycodes(entry, parts);

  size_t idx = 2;
  if (flags.find('k') != std::string::npos)
    parseDevices(entry, parts, idx);

  if (flags.find('d') != std::string::npos)
    parseDescription(entry, parts, idx);

  parseDispatcherAndArgs(entry, parts, idx);

  return entry;
}

static Hyprlang::CConfig *
createConfigPtr(const std::string &configFilePath,
                Hyprlang::SConfigOptions &options,
                Hyprlang::CConfig *externalConfig,
                std::unique_ptr<Hyprlang::CConfig> &owned) {
  if (externalConfig)
    return externalConfig;
  owned = std::make_unique<Hyprlang::CConfig>(configFilePath.c_str(), options);
  return owned.get();
}

static bool runConfigParse(Hyprlang::CConfig *config, bool strictMode) {
  config->commence();
  auto parseResult = config->parse();

  if (parseResult.error && strictMode) {
    Logger::error("[KeybindHandler] ERROR: {}", parseResult.getError());
    exit(1);
  }
  return !parseResult.error;
}

std::vector<BindEntry>
KeybindHandler::parseKeybinds(const std::string &configFilePath,
                              bool strictMode, bool followSource,
                              Hyprlang::CConfig *externalConfig) {
  Hyprlang::SConfigOptions options;
  options.allowMissingConfig = true;
  options.pathIsStream = false;

  hyprquery::KeybindCollector collector;
  hyprquery::KeybindCollector::instance = &collector;
  hyprquery::KeybindCollector::resolver =
      std::make_unique<hyprquery::XKBKeymapResolver>();

  std::ifstream infile(configFilePath);
  Logger::debug("[KeybindHandler] Starting parseKeybinds for: {}",
                configFilePath);
  if (!infile.good()) {
    Logger::debug(
        "[KeybindHandler] ERROR: Config file does not exist or is not "
        "readable: {}",
        configFilePath);
    return {};
  }

  try {
    std::unique_ptr<Hyprlang::CConfig> ownedConfig;
    Hyprlang::CConfig *configPtr =
        createConfigPtr(configFilePath, options, externalConfig, ownedConfig);

    Logger::debug("[DEBUG] Calling config.commence()...");

    runConfigParse(configPtr, strictMode);

    Logger::debug(
        "[KeybindHandler] config.commence() finished. Active binds: {}",
        collector.activeBinds.size());

    Logger::debug("[KeybindHandler] Finished output loop in parseKeybinds.");

    return collector.activeBinds;
  } catch (const std::exception &ex) {
    Logger::error("[KeybindHandler] Exception: {}", ex.what());
  } catch (const char *msg) {
    Logger::error("[KeybindHandler] Exception: {}", msg);
  } catch (...) {
    Logger::error(
        "[KeybindHandler] Unknown exception occurred in parseKeybinds.");
  }
  return {};
}

} // namespace hyprquery
