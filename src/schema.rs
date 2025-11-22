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
