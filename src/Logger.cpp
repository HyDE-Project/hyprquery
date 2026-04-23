#include "Logger.hpp"

#include <chrono>
#include <iomanip>
#include <iostream>
#include <sstream>

namespace hyprquery::Logger {

static bool s_debugEnabled = false;

void setDebug(bool enabled) { s_debugEnabled = enabled; }

bool isDebug() { return s_debugEnabled; }

static std::string levelPrefix(Level level) {
  switch (level) {
  case Level::Debug:
    return "[debug] ";
  case Level::Info:
    return "[info] ";
  case Level::Warn:
    return "[warn] ";
  case Level::Error:
    return "[error] ";
  }
  return "";
}

static std::string currentTimestamp() {
  auto now = std::chrono::system_clock::now();
  auto time = std::chrono::system_clock::to_time_t(now);
  auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
                now.time_since_epoch()) %
            1000;

  std::ostringstream oss;
  oss << std::put_time(std::localtime(&time), "%Y-%m-%d %H:%M:%S");
  oss << '.' << std::setfill('0') << std::setw(3) << ms.count();
  return oss.str();
}

void log(Level level, const std::string &message) {
  std::ostream &out = (level == Level::Error)   ? std::cerr
                      : (level == Level::Warn) ? std::cerr
                                               : std::clog;

  out << currentTimestamp() << ' ' << levelPrefix(level) << message << '\n';
}

} // namespace hyprquery::Logger
