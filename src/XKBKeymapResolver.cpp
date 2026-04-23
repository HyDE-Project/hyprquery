#include "XKBKeymapResolver.hpp"
#include <algorithm>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <string>
#include <unordered_map>
#include <vector>

namespace hyprquery {

XKBKeymapResolver::XKBKeymapResolver() {
  m_context = xkb_context_new(XKB_CONTEXT_NO_FLAGS);
  loadKeymapFromEnv();
}

XKBKeymapResolver::~XKBKeymapResolver() {
  if (m_state)
    xkb_state_unref(m_state);
  if (m_keymap)
    xkb_keymap_unref(m_keymap);
  if (m_context)
    xkb_context_unref(m_context);
}

void XKBKeymapResolver::loadKeymapFromEnv() {
  if (!m_context)
    return;
  const char *rules = getenv("XKB_DEFAULT_RULES");
  const char *model = getenv("XKB_DEFAULT_MODEL");
  const char *layout = getenv("XKB_DEFAULT_LAYOUT");
  const char *variant = getenv("XKB_DEFAULT_VARIANT");
  const char *options = getenv("XKB_DEFAULT_OPTIONS");
  struct xkb_rule_names names = {.rules = rules ? rules : "evdev",
                                 .model = model ? model : "pc105",
                                 .layout = layout ? layout : "us",
                                 .variant = variant ? variant : "",
                                 .options = options ? options : ""};
  m_keymap =
      xkb_keymap_new_from_names(m_context, &names, XKB_KEYMAP_COMPILE_NO_FLAGS);
  if (m_keymap)
    m_state = xkb_state_new(m_keymap);
}

uint32_t XKBKeymapResolver::resolveModMask(const std::string &mods) const {
  if (!m_keymap)
    return 0;
  std::string norm_mods = mods;
  std::replace(norm_mods.begin(), norm_mods.end(), '+', ' ');
  std::replace(norm_mods.begin(), norm_mods.end(), ',', ' ');
  std::istringstream iss(norm_mods);
  std::string token;
  uint32_t mask = 0;
  while (iss >> token) {
    xkb_mod_index_t mod_idx = xkb_keymap_mod_get_index(m_keymap, token.c_str());
    if (mod_idx != XKB_MOD_INVALID) {
      uint32_t mod_mask = (1u << mod_idx);
      mask |= mod_mask;
    }
  }
  return mask;
}

uint32_t XKBKeymapResolver::resolveKeycode(const std::string &key) const {
  if (!m_keymap)
    return 0;

  auto it = m_keycodeCache.find(key);
  if (it != m_keycodeCache.end())
    return it->second;

  xkb_keysym_t sym =
      xkb_keysym_from_name(key.c_str(), XKB_KEYSYM_CASE_INSENSITIVE);
  if (sym == XKB_KEY_NoSymbol) {
    m_keycodeCache[key] = 0;
    return 0;
  }

  xkb_keycode_t min = xkb_keymap_min_keycode(m_keymap);
  xkb_keycode_t max = xkb_keymap_max_keycode(m_keymap);
  for (xkb_keycode_t kc = min; kc <= max; ++kc) {
    const xkb_keysym_t *syms = nullptr;
    int n = xkb_keymap_key_get_syms_by_level(m_keymap, kc, 0, 0, &syms);
    for (int i = 0; i < n; ++i) {
      if (syms[i] == sym) {
        m_keycodeCache[key] = kc;
        return kc;
      }
    }
  }

  m_keycodeCache[key] = 0;
  return 0;
}

static const std::unordered_map<std::string, uint32_t> HYPRLAND_MODS = {
    {"SHIFT", 1 << 0},   {"CONTROL", 1 << 2}, {"ALT", 1 << 3},
    {"SUPER", 1 << 6},   {"CAPS", 1 << 1},    {"ALTGR", 1 << 4},
    {"NUMLOCK", 1 << 5},

};

uint32_t
XKBKeymapResolver::resolveHyprlandModMask(const std::string &mods) const {
  std::string norm_mods = mods;
  std::replace(norm_mods.begin(), norm_mods.end(), '+', ' ');
  std::replace(norm_mods.begin(), norm_mods.end(), ',', ' ');
  std::istringstream iss(norm_mods);
  std::string token;
  uint32_t mask = 0;
  while (iss >> token) {
    std::transform(token.begin(), token.end(), token.begin(), ::toupper);
    auto it = HYPRLAND_MODS.find(token);
    if (it != HYPRLAND_MODS.end()) {
      mask |= it->second;
    }
  }
  return mask;
}

} // namespace hyprquery
