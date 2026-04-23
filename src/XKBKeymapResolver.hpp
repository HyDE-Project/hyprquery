#pragma once
#include <xkbcommon/xkbcommon.h>
#include <string>
#include <memory>
#include <unordered_map>
#include <cstdint>

namespace hyprquery {

class XKBKeymapResolver {
public:
    XKBKeymapResolver();
    ~XKBKeymapResolver();
    // Returns a modmask for a comma/plus/space-separated modifier string (layout-aware)
    uint32_t resolveModMask(const std::string& mods) const;
    // Returns a keycode for a key string (layout-aware)
    uint32_t resolveKeycode(const std::string& key) const;
    // Returns a Hyprland-style modmask for a comma/plus/space-separated modifier string
    uint32_t resolveHyprlandModMask(const std::string& mods) const;
    // Returns true if resolver is valid
    bool valid() const { return m_keymap != nullptr && m_state != nullptr; }
private:
    struct xkb_context* m_context = nullptr;
    struct xkb_keymap* m_keymap = nullptr;
    struct xkb_state* m_state = nullptr;
    mutable std::unordered_map<std::string, uint32_t> m_modCache;
    mutable std::unordered_map<std::string, uint32_t> m_keycodeCache;
    void loadKeymapFromEnv();
};

} // namespace hyprquery
