#pragma once

#include <format>
#include <iostream>
#include <string>

namespace hyprquery::Logger {

enum class Level { Debug, Info, Warn, Error };

void setDebug(bool enabled);
bool isDebug();

void log(Level level, const std::string &message);

// One-line logging: debug() is a no-op when debug is off
// Use std::format for type-safe formatting

template <typename... Args>
void debug(std::format_string<Args...> fmt, Args &&...args) {
  if (!isDebug())
    return;
  log(Level::Debug, std::format(fmt, std::forward<Args>(args)...));
}

template <typename... Args>
void info(std::format_string<Args...> fmt, Args &&...args) {
  log(Level::Info, std::format(fmt, std::forward<Args>(args)...));
}

template <typename... Args>
void warn(std::format_string<Args...> fmt, Args &&...args) {
  log(Level::Warn, std::format(fmt, std::forward<Args>(args)...));
}

template <typename... Args>
void error(std::format_string<Args...> fmt, Args &&...args) {
  log(Level::Error, std::format(fmt, std::forward<Args>(args)...));
}

} // namespace hyprquery::Logger
