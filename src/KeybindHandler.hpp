#pragma once

#include <hyprlang.hpp>
#include <memory>
#include <optional>
#include <string>
#include <vector>

namespace hyprquery {

struct BindEntry {

  int modmask = 0;
  int keycode = 0;
  std::string submap;
  std::string flags;

  bool locked = false;
  bool release = false;
  bool click = false;
  bool drag = false;
  bool longPress = false;
  bool repeat = false;
  bool non_consuming = false;
  bool mouse = false;
  bool transparent = false;
  bool ignore_mods = false;
  bool separate = false;
  bool has_description = false;
  bool bypass = false;
  bool submap_universal = false;
  bool per_device = false;

  bool catch_all = false;
  std::string modifiers;
  std::string key;
  std::vector<std::string> devices;
  bool devices_exclude = false;
  std::string description;
  std::string dispatcher;
  std::string arg;
  std::vector<std::string> extra_args;
  std::string raw;
};

class KeybindCollector {
public:
  std::vector<BindEntry> activeBinds;
  static KeybindCollector *instance;
  static std::unique_ptr<class XKBKeymapResolver> resolver;

  static Hyprlang::CParseResult handleBind(const char *command,
                                           const char *value);
  static Hyprlang::CParseResult handleUnbind(const char *command,
                                             const char *value);
  static std::optional<BindEntry> parseBindLine(std::string_view line);

private:
  static std::string_view trimView(std::string_view sv);
  static std::pair<std::string_view, std::string_view>
  splitAtEquals(std::string_view line);
  static std::string extractFlags(std::string_view command);
  static std::vector<std::string>
  splitByCommaRespectingQuotes(std::string_view right);
  static void trimStrings(std::vector<std::string> &parts);

  static void setFlagBooleans(BindEntry &entry, const std::string &flags);
  static void resolveKeycodes(BindEntry &entry,
                              const std::vector<std::string> &parts);
  static void parseDevices(BindEntry &entry,
                           const std::vector<std::string> &parts, size_t &idx);
  static void parseDescription(BindEntry &entry,
                               const std::vector<std::string> &parts,
                               size_t &idx);
  static void parseDispatcherAndArgs(BindEntry &entry,
                                     const std::vector<std::string> &parts,
                                     size_t idx);
};

class KeybindHandler {
public:
  static std::vector<BindEntry>
  parseKeybinds(const std::string &configFilePath, bool strictMode = false,
                bool followSource = false,
                Hyprlang::CConfig *externalConfig = nullptr);
};

} // namespace hyprquery
