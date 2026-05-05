#!/usr/bin/env python3
"""
So I just wanted to make a script that generates a JSON schema from Hyprland's ConfigValues.cpp file.
And Use its own logic to determine types, defaults, and choices, so that we can keep the schema in sync with Hyprland's actual config values without manual maintenance.

Schemas are use wit hyprquery to determine types and defaults for config values.
hyprquery alone does not have built in types,keys, or defaults.
So to parse config from a file we need look up file for keys and parse its values.
"""

import argparse
import json
import re
import urllib.request
from pathlib import Path

SRC = Path(__file__).parent / "hyprland-values.cpp"
OUT = Path(__file__).parent / "hyprland.json"
REMOTE_URL = "https://raw.githubusercontent.com/hyprwm/Hyprland/main/src/config/values/ConfigValues.cpp"

TYPE_MAP = {
    "Bool": "BOOL",
    "Int": "INT",
    "Float": "FLOAT",
    "String": "STRING_SHORT",
    "CssGap": "STRING_SHORT",
    "FontWeight": "STRING_SHORT",
    "Gradient": "GRADIENT",
    "Color": "COLOR",
    "Vec2": "VECTOR",
}

LONG_STRING_KEYS = {
    "decoration:screen_shader",
    "input:kb_file",
}


def strip_cpp_comments(text: str) -> str:
    # Remove // comments and /* */ comments while preserving strings.
    output = []
    i = 0
    state = "code"
    while i < len(text):
        c = text[i]
        if state == "code":
            if text.startswith("//", i):
                state = "line_comment"
                i += 2
                continue
            if text.startswith("/*", i):
                state = "block_comment"
                i += 2
                continue
            if c == '"' or c == "'":
                quote = c
                output.append(c)
                i += 1
                state = "string"
                continue
            output.append(c)
        elif state == "line_comment":
            if c == "\n":
                output.append(c)
                state = "code"
        elif state == "block_comment":
            if text.startswith("*/", i):
                i += 2
                state = "code"
                continue
        elif state == "string":
            output.append(c)
            if c == "\\":
                i += 1
                if i < len(text):
                    output.append(text[i])
            elif c == quote:
                state = "code"
        i += 1
    return "".join(output)


def find_ms_calls(text: str):
    pattern = re.compile(r"MS<([^>]+)>\s*\(")
    i = 0
    while True:
        m = pattern.search(text, i)
        if not m:
            break
        typ = m.group(1).strip()
        start = m.end() - 1
        depth = 0
        in_string = False
        escape = False
        string_char = None
        j = start
        while j < len(text):
            ch = text[j]
            if in_string:
                if escape:
                    escape = False
                elif ch == "\\":
                    escape = True
                elif ch == string_char:
                    in_string = False
            else:
                if ch == '"' or ch == "'":
                    in_string = True
                    string_char = ch
                elif ch == '(':
                    depth += 1
                elif ch == ')':
                    depth -= 1
                    if depth == 0:
                        break
            j += 1
        if depth != 0:
            raise ValueError(f"Unbalanced parentheses in MS call starting at {m.start()}")
        body = text[start + 1:j].strip()
        yield typ, body
        i = j + 1


def split_top_level_commas(text: str):
    parts = []
    current = []
    depth = 0
    in_string = False
    escape = False
    string_char = None
    for ch in text:
        if in_string:
            current.append(ch)
            if escape:
                escape = False
            elif ch == "\\":
                escape = True
            elif ch == string_char:
                in_string = False
        else:
            if ch == '"' or ch == "'":
                in_string = True
                string_char = ch
                current.append(ch)
            elif ch in "({[":
                depth += 1
                current.append(ch)
            elif ch in ")]}":
                depth -= 1
                current.append(ch)
            elif ch == ',' and depth == 0:
                part = ''.join(current).strip()
                if part:
                    parts.append(part)
                current = []
            else:
                current.append(ch)
    last = ''.join(current).strip()
    if last:
        parts.append(last)
    return parts


def unquote_string(token: str) -> str:
    token = token.strip()
    if token.startswith('"') and token.endswith('"'):
        inner = token[1:-1]
        return bytes(inner, "utf-8").decode("unicode_escape")
    if token == "STRVAL_EMPTY":
        return ""
    if token in {"[[EMPTY]]", "[[Empty]]", "[[Auto]]"}:
        return ""
    return token


def parse_number(token: str):
    token = token.strip()
    if token == "std::numeric_limits<int>::max()":
        return 2147483647
    if token == "std::numeric_limits<float>::max()":
        return 3.4028234663852886e+38
    if token.startswith("0x") or token.startswith("-0x"):
        return int(token, 16)
    if token.lower() in {"true", "false"}:
        return token.lower() == "true"
    try:
        if "." in token or "e" in token.lower():
            return float(token)
        return int(token)
    except ValueError:
        return token


def parse_color(token: str):
    token = token.strip()
    if token.startswith("CHyprColor{") or token.startswith("Color{"):
        inner = token[token.index("{") + 1 : token.rindex("}")].strip()
        if inner.startswith("0x") or inner.startswith("0X"):
            return inner.lower()
        return inner
    if token.startswith("0x") or token.startswith("0X"):
        return token.lower()
    if token == "-1":
        return None
    return token


def parse_vec2(token: str):
    token = token.strip()
    if token.startswith("Config::VEC2{") or token.startswith("VEC2{") or token.startswith("Config::VEC2<"):
        inner = token[token.index("{") + 1 : token.rindex("}")].strip()
        values = [v.strip() for v in inner.split(",") if v.strip()]
        if len(values) == 2:
            return [parse_number(values[0]), parse_number(values[1])]
    return None


def parse_option_map(token: str):
    if "OptionMap" not in token:
        return None
    # greedy capture of braces inside OptionMap{{...}}
    m = re.search(r"OptionMap\s*\{\s*\{(.*)\}\s*\}", token, re.DOTALL)
    if not m:
        return None
    body = m.group(1)
    entries = []
    depth = 0
    current = []
    in_string = False
    escape = False
    string_char = None
    for ch in body:
        if in_string:
            current.append(ch)
            if escape:
                escape = False
            elif ch == "\\":
                escape = True
            elif ch == string_char:
                in_string = False
        else:
            if ch == '"' or ch == "'":
                in_string = True
                string_char = ch
                current.append(ch)
            elif ch == '{':
                depth += 1
                current.append(ch)
            elif ch == '}':
                depth -= 1
                current.append(ch)
            elif ch == ',' and depth == 0:
                part = ''.join(current).strip()
                if part:
                    entries.append(part)
                current = []
            else:
                current.append(ch)
    last = ''.join(current).strip()
    if last:
        entries.append(last)
    choices = []
    for entry in entries:
        m2 = re.search(r'"([^"]+)"\s*,\s*([^\s}]+)', entry)
        if m2:
            choices.append(m2.group(1))
    return choices or None


def parse_options(token: str):
    opts = {}
    if ".min" in token:
        m = re.search(r"\.min\s*=\s*([^,}\n]+)", token)
        if m:
            value = parse_number(m.group(1).strip())
            opts["min"] = value
    if ".max" in token:
        m = re.search(r"\.max\s*=\s*([^,}\n]+)", token)
        if m:
            value = parse_number(m.group(1).strip())
            opts["max"] = value
    choices = parse_option_map(token)
    if choices:
        opts["choices"] = choices
    return opts


def normalize_default(value, typ, key):
    if isinstance(value, str):
        if value in {"STRVAL_EMPTY", "[[EMPTY]]", "[[Empty]]", "[[Auto]]"}:
            return ""
        return value
    if typ == "STRING_SHORT" and value is not None:
        return str(value)
    if typ == "STRING_LONG" and value is not None:
        return str(value)
    if typ == "GRADIENT" or typ == "COLOR":
        if value is None:
            return None
        if isinstance(value, int):
            return f"0x{value:08x}"
        if isinstance(value, str):
            if value.startswith("0x") or value.startswith("0X"):
                return value.lower()
            return value
    return value


def schema_type_for(key: str, ctype: str, opts: dict):
    if ctype == "Int" and opts.get("choices"):
        return "CHOICE"
    schema = TYPE_MAP.get(ctype, "STRING_SHORT")
    if schema == "STRING_SHORT" and key in LONG_STRING_KEYS:
        return "STRING_LONG"
    if schema == "STRING_SHORT" and "shader" in key:
        return "STRING_LONG"
    if schema == "STRING_SHORT" and key.endswith(":kb_file"):
        return "STRING_LONG"
    return schema


def build_entry(ctype: str, args: list[str]):
    if len(args) < 2:
        raise ValueError(f"Too few arguments for MS<{ctype}>: {args}")
    key = unquote_string(args[0])
    desc = unquote_string(args[1])
    default = None
    data = {}
    if len(args) >= 3:
        raw_default = args[2].strip()
        if raw_default:
            if ctype == "Bool":
                default = raw_default == "true"
            elif ctype == "Int":
                default = parse_number(raw_default)
            elif ctype == "Float":
                default = parse_number(raw_default)
            elif ctype == "String":
                default = unquote_string(raw_default)
            elif ctype == "CssGap":
                val = parse_number(raw_default)
                default = str(val) if val is not None else ""
            elif ctype == "Gradient":
                default = parse_color(raw_default)
            elif ctype == "Color":
                default = parse_color(raw_default)
            elif ctype == "Vec2":
                default = parse_vec2(raw_default)
            elif ctype == "FontWeight":
                default = None
            else:
                default = unquote_string(raw_default)
    opts = {}
    if len(args) >= 4:
        opts = parse_options(args[3])
    if default is not None:
        if ctype == "Color" and default is None:
            pass
        elif ctype == "Vec2" and default is not None:
            data["default"] = default
        else:
            data["default"] = normalize_default(default, schema_type_for(key, ctype, opts), key)
            if data["default"] == "" and ctype == "String" and raw_default in {"[[EMPTY]]", "[[Empty]]", "STRVAL_EMPTY", "[[Auto]]"}:
                # keep as empty string
                pass
    if "min" in opts:
        data["min"] = opts["min"]
    if "max" in opts:
        data["max"] = opts["max"]
    if "choices" in opts:
        data["choices"] = opts["choices"]
    entry = {"value": key, "description": desc, "type": schema_type_for(key, ctype, opts), "data": data}
    return entry


def fetch_remote_source(url: str) -> str:
    with urllib.request.urlopen(url, timeout=20) as response:
        return response.read().decode("utf-8")


def parse_args():
    parser = argparse.ArgumentParser(description="Generate hyprland.json schema from ConfigValues.cpp")
    parser.add_argument("--remote", action="store_true", help="Fetch source from upstream Hyprland GitHub instead of local file")
    parser.add_argument("--url", default=REMOTE_URL, help="Remote URL to fetch source from")
    return parser.parse_args()


def main():
    args = parse_args()
    if args.remote:
        print(f"Fetching upstream source from {args.url}")
        text = fetch_remote_source(args.url)
    else:
        text = SRC.read_text()
    text = strip_cpp_comments(text)
    entries = []
    for ctype, body in find_ms_calls(text):
        args = split_top_level_commas(body)
        if not args:
            continue
        entry = build_entry(ctype, args)
        entries.append(entry)
    output = {"hyprlang_schema": entries}
    OUT.write_text(json.dumps(output, indent=2) + "\n")
    print(f"Generated {len(entries)} schema entries into {OUT}")


if __name__ == "__main__":
    main()
