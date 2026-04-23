#include "ExportJson.hpp"
#include "KeybindHandler.hpp"
#include <iostream>
#include <nlohmann/json.hpp>
#include <vector>

namespace hyprquery {

void exportBindsJson(const std::vector<BindEntry> &binds) {
  nlohmann::json jsonArr = nlohmann::json::array();
  for (const auto &bind : binds) {
    nlohmann::json obj;
    obj["modmask"] = bind.modmask;
    obj["keycode"] = bind.keycode;
    obj["submap"] = bind.submap;
    obj["flags"] = bind.flags;
    obj["locked"] = bind.locked;
    obj["release"] = bind.release;
    obj["click"] = bind.click;
    obj["drag"] = bind.drag;
    obj["longPress"] = bind.longPress;
    obj["repeat"] = bind.repeat;
    obj["non_consuming"] = bind.non_consuming;
    obj["mouse"] = bind.mouse;
    obj["transparent"] = bind.transparent;
    obj["ignore_mods"] = bind.ignore_mods;
    obj["separate"] = bind.separate;
    obj["has_description"] = bind.has_description;
    obj["bypass"] = bind.bypass;
    obj["submap_universal"] = bind.submap_universal;
    obj["per_device"] = bind.per_device;
    obj["catch_all"] = bind.catch_all;
    obj["modifiers"] = bind.modifiers;
    obj["key"] = bind.key;
    if (!bind.devices.empty()) {
      obj["devices"] = bind.devices;
      if (bind.devices_exclude)
        obj["devices_exclude"] = true;
    }
    obj["description"] = bind.description;
    obj["dispatcher"] = bind.dispatcher;
    if (!bind.arg.empty())
      obj["arg"] = bind.arg;
    if (!bind.extra_args.empty())
      obj["extra_args"] = bind.extra_args;
    obj["raw"] = bind.raw;
    jsonArr.push_back(obj);
  }
  std::cout << jsonArr.dump(2) << std::endl;
}

void exportJson(const std::vector<QueryResult> &results) {
  nlohmann::json jsonArr = nlohmann::json::array();
  for (const auto &result : results) {
    jsonArr.push_back({{"key", result.key},
                       {"val", result.value},
                       {"type", result.type},
                       {"flags", result.flags}});
  }
  std::cout << jsonArr.dump(2) << std::endl;
}

} // namespace hyprquery
