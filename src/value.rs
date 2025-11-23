//! Configuration value conversion and utility functions.
//!
//! This module provides helper functions for working with hyprlang ConfigValue:
//! - String conversion for all value types
//! - Type name extraction
//! - Hashing for dynamic variable key generation

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher}
};

/// Hash a string to create a unique identifier.
///
/// Used for generating unique keys for dynamic variable lookups.
/// Uses the default hasher for consistent results within a single run.
///
/// # Arguments
///
/// * `s` - String to hash
///
/// # Returns
///
/// A 64-bit hash value.
pub fn hash_string(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Convert a ConfigValue to its string representation.
///
/// Handles all hyprlang value types:
/// - `Int` - Decimal string
/// - `Float` - Decimal string with fractional part
/// - `String` - The string value itself
/// - `Vec2` - Format: `x, y`
/// - `Color` - Format: `rgba(r, g, b, a)`
/// - `Custom` - Returns `"custom"`
///
/// # Arguments
///
/// * `value` - The ConfigValue to convert
///
/// # Returns
///
/// String representation of the value.
pub fn config_value_to_string(value: &hyprlang::ConfigValue) -> String {
    match value {
        hyprlang::ConfigValue::Int(i) => i.to_string(),
        hyprlang::ConfigValue::Float(f) => f.to_string(),
        hyprlang::ConfigValue::String(s) => s.clone(),
        hyprlang::ConfigValue::Vec2(v) => format!("{}, {}", v.x, v.y),
        hyprlang::ConfigValue::Color(c) => format!("rgba({}, {}, {}, {})", c.r, c.g, c.b, c.a),
        hyprlang::ConfigValue::Custom {
            ..
        } => "custom".to_string()
    }
}

/// Get the type name for a ConfigValue.
///
/// Returns a static string representing the value's type,
/// used for type filtering and output formatting.
///
/// # Arguments
///
/// * `value` - The ConfigValue to inspect
///
/// # Returns
///
/// One of: `"INT"`, `"FLOAT"`, `"STRING"`, `"VEC2"`, `"COLOR"`, `"CUSTOM"`
pub fn config_value_type_name(value: &hyprlang::ConfigValue) -> &'static str {
    match value {
        hyprlang::ConfigValue::Int(_) => "INT",
        hyprlang::ConfigValue::Float(_) => "FLOAT",
        hyprlang::ConfigValue::String(_) => "STRING",
        hyprlang::ConfigValue::Vec2(_) => "VEC2",
        hyprlang::ConfigValue::Color(_) => "COLOR",
        hyprlang::ConfigValue::Custom {
            ..
        } => "CUSTOM"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_string_deterministic() {
        let hash1 = hash_string("$GTK_THEME");
        let hash2 = hash_string("$GTK_THEME");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_string_different_vars() {
        let hash1 = hash_string("$GTK_THEME");
        let hash2 = hash_string("$ICON_THEME");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_string_cursor_theme() {
        let hash = hash_string("$CURSOR_THEME");
        assert!(hash > 0);
    }

    #[test]
    fn test_config_value_to_string_border_size() {
        let value = hyprlang::ConfigValue::Int(2);
        assert_eq!(config_value_to_string(&value), "2");
    }

    #[test]
    fn test_config_value_to_string_cursor_size() {
        let value = hyprlang::ConfigValue::Int(20);
        assert_eq!(config_value_to_string(&value), "20");
    }

    #[test]
    fn test_config_value_to_string_gaps() {
        let value = hyprlang::ConfigValue::Int(8);
        assert_eq!(config_value_to_string(&value), "8");
    }

    #[test]
    fn test_config_value_to_string_float_opacity() {
        let value = hyprlang::ConfigValue::Float(0.95);
        assert_eq!(config_value_to_string(&value), "0.95");
    }

    #[test]
    fn test_config_value_to_string_gtk_theme() {
        let value = hyprlang::ConfigValue::String("Gruvbox-Retro".to_string());
        assert_eq!(config_value_to_string(&value), "Gruvbox-Retro");
    }

    #[test]
    fn test_config_value_to_string_icon_theme() {
        let value = hyprlang::ConfigValue::String("Gruvbox-Plus-Dark".to_string());
        assert_eq!(config_value_to_string(&value), "Gruvbox-Plus-Dark");
    }

    #[test]
    fn test_config_value_to_string_color_scheme() {
        let value = hyprlang::ConfigValue::String("prefer-dark".to_string());
        assert_eq!(config_value_to_string(&value), "prefer-dark");
    }

    #[test]
    fn test_config_value_to_string_vec2() {
        let value = hyprlang::ConfigValue::Vec2(hyprlang::Vec2 {
            x: 1920.0,
            y: 1080.0
        });
        assert_eq!(config_value_to_string(&value), "1920, 1080");
    }

    #[test]
    fn test_config_value_to_string_color_active_border() {
        let value = hyprlang::ConfigValue::Color(hyprlang::Color {
            r: 144,
            g: 206,
            b: 170,
            a: 255
        });
        assert_eq!(config_value_to_string(&value), "rgba(144, 206, 170, 255)");
    }

    #[test]
    fn test_config_value_to_string_color_inactive_border() {
        let value = hyprlang::ConfigValue::Color(hyprlang::Color {
            r: 30,
            g: 139,
            b: 80,
            a: 217
        });
        assert_eq!(config_value_to_string(&value), "rgba(30, 139, 80, 217)");
    }

    #[test]
    fn test_config_value_type_name_int() {
        assert_eq!(
            config_value_type_name(&hyprlang::ConfigValue::Int(2)),
            "INT"
        );
    }

    #[test]
    fn test_config_value_type_name_float() {
        assert_eq!(
            config_value_type_name(&hyprlang::ConfigValue::Float(0.5)),
            "FLOAT"
        );
    }

    #[test]
    fn test_config_value_type_name_string() {
        assert_eq!(
            config_value_type_name(&hyprlang::ConfigValue::String("Gruvbox-Retro".to_string())),
            "STRING"
        );
    }

    #[test]
    fn test_config_value_type_name_vec2() {
        assert_eq!(
            config_value_type_name(&hyprlang::ConfigValue::Vec2(hyprlang::Vec2 {
                x: 0.0,
                y: 0.0
            })),
            "VEC2"
        );
    }

    #[test]
    fn test_config_value_type_name_color() {
        assert_eq!(
            config_value_type_name(&hyprlang::ConfigValue::Color(hyprlang::Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0
            })),
            "COLOR"
        );
    }
}
