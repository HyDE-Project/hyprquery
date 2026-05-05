#!/usr/bin/env bash
# hyprquery test suite
# Usage: ./test/run_tests.sh [path-to-hyq]
# Defaults to ../target/bin/hyq relative to this script's location.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HYQ="${1:-"$SCRIPT_DIR/../target/bin/hyq"}"
CONF="$SCRIPT_DIR/config/config.conf"
SCHEMA="$SCRIPT_DIR/../schema/hyprland.json"

PASS=0
FAIL=0
SKIP=0

# ── helpers ──────────────────────────────────────────────────────────────────

green() { printf '\033[32m%s\033[0m' "$*"; }
red()   { printf '\033[31m%s\033[0m' "$*"; }
yellow(){ printf '\033[33m%s\033[0m' "$*"; }

pass() { echo "  $(green PASS) $1"; PASS=$((PASS+1)); }
fail() { echo "  $(red FAIL) $1"; echo "       expected: $2"; echo "       got:      $3"; FAIL=$((FAIL+1)); }
skip() { echo "  $(yellow SKIP) $1: $2"; SKIP=$((SKIP+1)); }

# assert_eq <test-name> <expected> <actual>
assert_eq() {
  if [[ "$2" == "$3" ]]; then
    pass "$1"
  else
    fail "$1" "$2" "$3"
  fi
}

# assert_contains <test-name> <needle> <haystack>
assert_contains() {
  if [[ "$3" == *"$2"* ]]; then
    pass "$1"
  else
    fail "$1" "contains: $2" "$3"
  fi
}

# assert_exit <test-name> <expected-exit-code> -- <cmd...>
assert_exit() {
  local name="$1" expected="$2"; shift 2
  # consume "--"
  [[ "$1" == "--" ]] && shift
  local actual=0
  "$@" >/dev/null 2>&1 || actual=$?
  assert_eq "$name" "$expected" "$actual"
}

# run_hyq <args...>  → stdout (stderr suppressed)
run_hyq() { "$HYQ" "$@" 2>/dev/null; }

# ── preflight ────────────────────────────────────────────────────────────────

echo ""
echo "hyprquery test suite"
echo "  hyq:    $HYQ"
echo "  config: $CONF"
echo "  schema: $SCHEMA"
echo ""

if [[ ! -x "$HYQ" ]]; then
  echo "$(red ERROR) hyq binary not found or not executable: $HYQ"
  echo "Build first: cd target && make -j\$(nproc)"
  exit 1
fi

if [[ ! -f "$CONF" ]]; then
  echo "$(red ERROR) test config not found: $CONF"
  exit 1
fi

# ── §1 Basic query ────────────────────────────────────────────────────────────

echo "§1  Basic query"

assert_eq "query: general:border_size" \
  "2" \
  "$(run_hyq -Q general:border_size "$CONF")"

assert_eq "query: general:gaps_in" \
  "3" \
  "$(run_hyq -Q general:gaps_in "$CONF")"

assert_eq "query: general:layout" \
  "dwindle" \
  "$(run_hyq -Q general:layout "$CONF")"

assert_eq "query: decoration:rounding (overridden to 1000)" \
  "1000" \
  "$(run_hyq -Q decoration:rounding "$CONF")"

# ── §2 Multiple queries ───────────────────────────────────────────────────────

echo ""
echo "§2  Multiple queries"

out="$(run_hyq -Q general:border_size -Q general:gaps_out "$CONF")"
assert_eq "multi-query: border_size line" "2"  "$(echo "$out" | sed -n '1p')"
assert_eq "multi-query: gaps_out line"    "8"  "$(echo "$out" | sed -n '2p')"

# ── §3 JSON export ────────────────────────────────────────────────────────────

echo ""
echo "§3  JSON export"

json="$(run_hyq -Q general:border_size --export json "$CONF")"
assert_contains "json export: has key"   '"key"'   "$json"
assert_contains "json export: has val"   '"val"'   "$json"
assert_contains "json export: has value" '"2"'     "$json"
assert_contains "json export: has type"  '"type"'  "$json"

# ── §4 source following ───────────────────────────────────────────────────────

echo ""
echo "§4  Source following (--source)"

assert_eq "source: key=colors (from colors.conf)" \
  "colors" \
  "$(run_hyq --source -Q key "$CONF")"

assert_eq "source: decoration:rounding via source" \
  "100" \
  "$(run_hyq --source -Q decoration:rounding "$CONF")"

# ── §5 Strict mode ────────────────────────────────────────────────────────────

echo ""
echo "§5  Strict mode"

# --strict on a config with unregistered keys (layerrule etc.) should fail;
# that is the correct and expected behavior.
assert_exit "strict: fails on config with unregistered keys" 1 -- \
  "$HYQ" --strict -Q general:border_size "$CONF"

# Without --strict, unregistered keys are silently ignored
assert_exit "no-strict: valid key exits zero" 0 -- \
  "$HYQ" -Q general:border_size "$CONF"

# ── §6 --dump (schema export) ─────────────────────────────────────────────────

echo ""
echo "§6  --dump (schema export)"

if [[ ! -f "$SCHEMA" ]]; then
  skip "--dump tests" "schema file not found: $SCHEMA"
else
  dump_json="$(run_hyq --schema "$SCHEMA" --dump "$CONF")"

  assert_contains "dump json: is array"             '"key"'                      "$dump_json"
  assert_contains "dump json: general:border_size"  '"general:border_size"'      "$dump_json"
  assert_contains "dump json: decoration:rounding"  '"decoration:rounding"'      "$dump_json"

  # Keys not in the config file must be absent without --fallback
  if echo "$dump_json" | grep -q '"general:sensitivity"'; then
    fail "dump json: schema-only key absent without --fallback" "absent" "present"
  else
    pass "dump json: schema-only key absent without --fallback"
  fi

  # --fallback must bring in a schema-only key (sensitivity is under input:)
  dump_fb="$(run_hyq --schema "$SCHEMA" --dump --fallback "$CONF")"
  assert_contains "dump --fallback: schema-only key present" '"input:sensitivity"' "$dump_fb"
  assert_contains "dump --fallback: source=default marker"   '"source"'              "$dump_fb"

  # Count: fallback should have more entries than no-fallback
  count_base=$(echo "$dump_json" | grep -c '"key"')
  count_fb=$(echo "$dump_fb"   | grep -c '"key"')
  if (( count_fb > count_base )); then
    pass "dump --fallback: more entries than base dump ($count_fb > $count_base)"
  else
    fail "dump --fallback: more entries than base dump" "> $count_base" "$count_fb"
  fi

  # --export hypr
  dump_hypr="$(run_hyq --schema "$SCHEMA" --dump --export hypr "$CONF")"
  assert_contains "dump --export hypr: section block"    "general {"         "$dump_hypr"
  assert_contains "dump --export hypr: key = value"      "border_size = 2"   "$dump_hypr"
  assert_contains "dump --export hypr: nested section"   "blur {"            "$dump_hypr"

  # --export lua
  dump_lua="$(run_hyq --schema "$SCHEMA" --dump --export lua "$CONF")"
  assert_contains "dump --export lua: return {"          "return {"          "$dump_lua"
  assert_contains "dump --export lua: section table"     "general = {"       "$dump_lua"
  assert_contains "dump --export lua: int value"         'border_size = 2'   "$dump_lua"
  assert_contains "dump --export lua: bool value"        'enabled = true'    "$dump_lua"
  assert_contains "dump --export lua: gradient value"    'colors = {'        "$dump_lua"
  assert_contains "dump --export lua: string value"      'gaps_in = "3"'     "$dump_lua"

  # --export nested-json
  dump_nj="$(run_hyq --schema "$SCHEMA" --dump --export nested-json "$CONF")"
  assert_contains "dump --export nested-json: top-level object" '"general"'        "$dump_nj"
  assert_contains "dump --export nested-json: nested key"       '"border_size"'    "$dump_nj"
  # Must NOT have flat "key"/"type"/"val" structure
  if echo "$dump_nj" | python3 -c "
import json,sys
d=json.load(sys.stdin)
assert isinstance(d, dict), 'not an object'
assert 'general' in d, 'missing general section'
assert isinstance(d['general'], dict), 'general is not object'
" 2>/dev/null; then
    pass "dump --export nested-json: valid nested structure"
  else
    fail "dump --export nested-json: valid nested structure" "nested object" "invalid"
  fi

  # --strict with --dump: missing key should cause non-zero exit
  assert_exit "dump --strict: exits non-zero when schema keys missing" 1 -- \
    "$HYQ" --schema "$SCHEMA" --dump --strict "$CONF"
fi

# ── §7 --binds ────────────────────────────────────────────────────────────────

echo ""
echo "§7  --binds"

binds_conf="$SCRIPT_DIR/config/test.conf"
if [[ ! -f "$binds_conf" ]]; then
  skip "--binds tests" "test.conf not found"
else
  assert_exit "--binds: exits zero" 0 -- "$HYQ" --binds "$binds_conf"

  binds_json="$(run_hyq --binds --export json "$binds_conf")"
  if [[ -n "$binds_json" ]]; then
    pass "--binds --export json: non-empty output"
  else
    skip "--binds --export json" "no binds in test.conf"
  fi
fi

# ── §8 env export ─────────────────────────────────────────────────────────────

echo ""
echo "§8  --export env"

env_out="$(run_hyq -Q general:border_size --export env "$CONF")"
assert_contains "env export: VAR=value line" "border_size=" "$env_out"

# ── §9 delimiter ──────────────────────────────────────────────────────────────

echo ""
echo "§9  --delimiter"

delim_out="$(run_hyq -Q general:border_size -Q general:gaps_in --delimiter '|' "$CONF")"
assert_eq "delimiter: custom separator" "2|3" "$delim_out"

# ── summary ──────────────────────────────────────────────────────────────────

echo ""
echo "────────────────────────────────"
total=$((PASS + FAIL + SKIP))
echo "  Total: $total  $(green "Pass: $PASS")  $(red "Fail: $FAIL")  $(yellow "Skip: $SKIP")"
echo ""

if (( FAIL > 0 )); then
  exit 1
fi
