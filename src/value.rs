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
        let hash1 = hash_string("test");
        let hash2 = hash_string("test");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_string_different() {
        let hash1 = hash_string("test1");
        let hash2 = hash_string("test2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_config_value_to_string_int() {
        let value = hyprlang::ConfigValue::Int(42);
        assert_eq!(config_value_to_string(&value), "42");
    }

    #[test]
    fn test_config_value_to_string_float() {
        let value = hyprlang::ConfigValue::Float(3.14);
        assert_eq!(config_value_to_string(&value), "3.14");
    }

    #[test]
    fn test_config_value_to_string_string() {
        let value = hyprlang::ConfigValue::String("hello".to_string());
        assert_eq!(config_value_to_string(&value), "hello");
    }

    #[test]
    fn test_config_value_type_name() {
        assert_eq!(
            config_value_type_name(&hyprlang::ConfigValue::Int(1)),
            "INT"
        );
        assert_eq!(
            config_value_type_name(&hyprlang::ConfigValue::Float(1.0)),
            "FLOAT"
        );
        assert_eq!(
            config_value_type_name(&hyprlang::ConfigValue::String("s".to_string())),
            "STRING"
        );
    }
}
