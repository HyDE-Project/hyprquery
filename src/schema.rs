use std::{fs::File, io::BufReader, path::Path};

use hyprlang::{Config, ConfigValue, Vec2};
use serde::Deserialize;
use serde_json::Value;

use crate::error::AppError;

/// Schema option data with default value
#[derive(Debug, Deserialize)]
pub struct SchemaData {
    /// Default value for the option
    pub default: Option<Value>
}

/// Single schema option definition
#[derive(Debug, Deserialize)]
pub struct SchemaOption {
    /// Configuration key path
    pub value:       String,
    /// Type of the value (INT, FLOAT, STRING, etc.)
    #[serde(rename = "type")]
    pub option_type: String,
    /// Data with default
    pub data:        SchemaData
}

/// Root schema structure
#[derive(Debug, Deserialize)]
pub struct Schema {
    /// List of schema options
    pub hyprlang_schema: Vec<SchemaOption>
}

/// Load schema from JSON file and register config values
///
/// # Arguments
///
/// * `config` - Configuration instance to add values to
/// * `schema_path` - Path to the schema JSON file
///
/// # Returns
///
/// Result indicating success or error
///
/// # Errors
///
/// Returns error if file cannot be read or parsed
pub fn load_schema(config: &mut Config, schema_path: &Path) -> Result<(), AppError> {
    let file = File::open(schema_path)
        .map_err(|_| AppError::schema_not_found(&schema_path.display().to_string()))?;

    let reader = BufReader::new(file);
    let schema: Schema = serde_json::from_reader(reader)?;

    for option in schema.hyprlang_schema {
        if let Some(default) = option.data.default {
            let config_value = match option.option_type.as_str() {
                "INT" => default.as_i64().map(ConfigValue::Int),
                "BOOL" => default
                    .as_bool()
                    .map(|v| ConfigValue::Int(if v { 1 } else { 0 })),
                "FLOAT" => default.as_f64().map(ConfigValue::Float),
                "STRING_SHORT" | "STRING_LONG" | "GRADIENT" | "COLOR" => {
                    default.as_str().map(|s| ConfigValue::String(s.to_string()))
                }
                "VECTOR" => {
                    if let Some(arr) = default.as_array() {
                        if arr.len() == 2 {
                            match (arr[0].as_f64(), arr[1].as_f64()) {
                                (Some(x), Some(y)) => Some(ConfigValue::Vec2(Vec2 {
                                    x,
                                    y
                                })),
                                _ => None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None
            };

            if let Some(value) = config_value {
                config.set(&option.value, value);
            }
        }
    }

    Ok(())
}

/// Get all schema keys from JSON file
///
/// # Arguments
///
/// * `schema_path` - Path to the schema JSON file
///
/// # Returns
///
/// Vector of all configuration keys in the schema
///
/// # Errors
///
/// Returns error if file cannot be read or parsed
pub fn get_schema_keys(schema_path: &Path) -> Result<Vec<String>, AppError> {
    let file = File::open(schema_path)
        .map_err(|_| AppError::schema_not_found(&schema_path.display().to_string()))?;

    let reader = BufReader::new(file);
    let schema: Schema = serde_json::from_reader(reader)?;

    let keys = schema
        .hyprlang_schema
        .into_iter()
        .map(|option| option.value)
        .collect();

    Ok(keys)
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use super::*;

    fn create_test_schema() -> (std::path::PathBuf, String) {
        let temp_dir = std::env::temp_dir();
        let schema_path = temp_dir.join("test_schema.json");
        let schema_content = r#"{
            "hyprlang_schema": [
                {
                    "value": "general:border_size",
                    "type": "INT",
                    "data": { "default": 2 }
                },
                {
                    "value": "decoration:rounding",
                    "type": "FLOAT",
                    "data": { "default": 8.0 }
                },
                {
                    "value": "general:layout",
                    "type": "STRING_SHORT",
                    "data": { "default": "dwindle" }
                }
            ]
        }"#;
        (schema_path, schema_content.to_string())
    }

    #[test]
    fn test_load_schema() {
        let (schema_path, content) = create_test_schema();
        let mut file = fs::File::create(&schema_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let mut config = Config::default();
        let result = load_schema(&mut config, &schema_path);
        assert!(result.is_ok());

        let _ = fs::remove_file(schema_path);
    }

    #[test]
    fn test_get_schema_keys() {
        let temp_dir = std::env::temp_dir();
        let schema_path = temp_dir.join("test_get_keys_schema.json");
        let schema_content = r#"{
            "hyprlang_schema": [
                {
                    "value": "general:border_size",
                    "type": "INT",
                    "data": { "default": 2 }
                },
                {
                    "value": "decoration:rounding",
                    "type": "FLOAT",
                    "data": { "default": 8.0 }
                },
                {
                    "value": "general:layout",
                    "type": "STRING_SHORT",
                    "data": { "default": "dwindle" }
                }
            ]
        }"#;

        let mut file = fs::File::create(&schema_path).unwrap();
        file.write_all(schema_content.as_bytes()).unwrap();

        let result = get_schema_keys(&schema_path);
        assert!(result.is_ok());
        let keys = result.unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"general:border_size".to_string()));
        assert!(keys.contains(&"decoration:rounding".to_string()));
        assert!(keys.contains(&"general:layout".to_string()));

        let _ = fs::remove_file(schema_path);
    }

    #[test]
    fn test_schema_not_found() {
        let result = get_schema_keys(Path::new("/nonexistent/schema.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_parse_vector() {
        let temp_dir = std::env::temp_dir();
        let schema_path = temp_dir.join("test_vector_schema.json");
        let schema_content = r#"{
            "hyprlang_schema": [
                {
                    "value": "general:gaps",
                    "type": "VECTOR",
                    "data": { "default": [5.0, 10.0] }
                }
            ]
        }"#;

        let mut file = fs::File::create(&schema_path).unwrap();
        file.write_all(schema_content.as_bytes()).unwrap();

        let mut config = Config::default();
        let result = load_schema(&mut config, &schema_path);
        assert!(result.is_ok());

        let _ = fs::remove_file(schema_path);
    }

    #[test]
    fn test_schema_parse_bool() {
        let temp_dir = std::env::temp_dir();
        let schema_path = temp_dir.join("test_bool_schema.json");
        let schema_content = r#"{
            "hyprlang_schema": [
                {
                    "value": "decoration:blur:enabled",
                    "type": "BOOL",
                    "data": { "default": true }
                }
            ]
        }"#;

        let mut file = fs::File::create(&schema_path).unwrap();
        file.write_all(schema_content.as_bytes()).unwrap();

        let mut config = Config::default();
        let result = load_schema(&mut config, &schema_path);
        assert!(result.is_ok());

        let _ = fs::remove_file(schema_path);
    }
}
