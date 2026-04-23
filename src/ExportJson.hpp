#pragma once
#include "ConfigUtils.hpp"
#include "KeybindHandler.hpp"
#include <vector>

namespace hyprquery {
void exportBindsJson(const std::vector<BindEntry> &binds);
void exportJson(const std::vector<QueryResult> &results);
}
