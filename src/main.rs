mod error;
mod export;
mod path;
mod query;
mod schema;

use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process
};

use clap::Parser;
use error::AppError;
use export::{export_env, export_json, export_plain};
use hyprlang::{Config, ConfigValue};
use path::{normalize_path, resolve_glob};
use query::{QueryResult, normalize_type, parse_query_inputs};
use regex::Regex;

/// A command-line utility for querying configuration values from Hyprland
/// configuration files
#[derive(Parser, Debug)]
#[command(name = "hyprquery")]
#[command(version)]
#[command(about = "A configuration parser for hypr* config files")]
struct Args {
    /// Query to execute (format: query[expectedType][expectedRegex])
    #[arg(short = 'Q', long = "query", required = true, num_args = 1..)]
    queries: Vec<String>,

    /// Configuration file path
    #[arg(required = true)]
    config_file: String,

    /// Schema file path
    #[arg(long)]
    schema: Option<String>,

    /// Allow missing values
    #[arg(long)]
    allow_missing: bool,

    /// Get default keys from schema
    #[arg(long)]
    get_defaults: bool,

    /// Enable strict mode validation
    #[arg(long)]
    strict: bool,

    /// Export format: json or env
    #[arg(long)]
    export: Option<String>,

    /// Follow source directives in config files
    #[arg(short = 's', long)]
    source: bool,

    /// Enable debug logging
    #[arg(long)]
    debug: bool,

    /// Delimiter for plain output
    #[arg(short = 'D', long, default_value = "\n")]
    delimiter: String
}

fn hash_string(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

fn config_value_to_string(value: &ConfigValue) -> String {
    match value {
        ConfigValue::Int(i) => i.to_string(),
        ConfigValue::Float(f) => f.to_string(),
        ConfigValue::String(s) => s.clone(),
        ConfigValue::Vec2(v) => format!("{}, {}", v.x, v.y),
        ConfigValue::Color(c) => format!("rgba({}, {}, {}, {})", c.r, c.g, c.b, c.a),
        ConfigValue::Custom {
            ..
        } => "custom".to_string()
    }
}

fn config_value_type_name(value: &ConfigValue) -> &'static str {
    match value {
        ConfigValue::Int(_) => "INT",
        ConfigValue::Float(_) => "FLOAT",
        ConfigValue::String(_) => "STRING",
        ConfigValue::Vec2(_) => "VEC2",
        ConfigValue::Color(_) => "COLOR",
        ConfigValue::Custom {
            ..
        } => "CUSTOM"
    }
}

fn run() -> Result<i32, AppError> {
    let args = Args::parse();

    let config_path = normalize_path(&args.config_file)?;
    if !config_path.exists() {
        return Err(AppError::config_not_found(
            &config_path.display().to_string()
        ));
    }

    let config_dir = config_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let queries = parse_query_inputs(&args.queries);

    let has_dynamic = queries.iter().any(|q| q.is_dynamic_variable);

    let mut config = Config::new();

    if has_dynamic {
        let mut content = fs::read_to_string(&config_path)?;

        for query in &queries {
            if query.is_dynamic_variable {
                let dyn_key = format!("Dynamic_{}", hash_string(&query.query));
                let dyn_line = format!("\n{}={}\n", dyn_key, query.query);
                content.push_str(&dyn_line);

                if args.debug {
                    eprintln!("[debug] Injecting line: {}", dyn_line.trim());
                }
            }
        }

        let parse_result = config.parse(&content);
        if let Err(e) = parse_result {
            if args.debug {
                eprintln!("[debug] Parse error: {}", e);
            }
            if args.strict {
                return Err(AppError::config_parse(&e.to_string()));
            }
        }
    } else {
        let parse_result = config.parse_file(&config_path);
        if let Err(e) = parse_result {
            if args.debug {
                eprintln!("[debug] Parse error: {}", e);
            }
            if args.strict {
                return Err(AppError::config_parse(&e.to_string()));
            }
        }
    }

    if let Some(ref schema_path_str) = args.schema {
        let schema_path = normalize_path(schema_path_str)?;
        if !schema_path.exists() {
            return Err(AppError::schema_not_found(
                &schema_path.display().to_string()
            ));
        }
        schema::load_schema(&mut config, &schema_path)?;
    }

    if args.source {
        let sources = find_sources(&config_path, &config_dir)?;
        for source_path in sources {
            if source_path.exists() && source_path.is_file() {
                let _ = config.parse_file(&source_path);
            }
        }
    }

    let mut results = Vec::with_capacity(queries.len());

    for query in &queries {
        let lookup_key = if query.is_dynamic_variable {
            format!("Dynamic_{}", hash_string(&query.query))
        } else {
            query.query.clone()
        };

        if args.debug {
            eprintln!("[debug] Looking up key: {}", lookup_key);
        }

        let (value_str, type_str) = match config.get(&lookup_key) {
            Ok(value) => {
                let v = config_value_to_string(value);
                let t = config_value_type_name(value);

                if query.is_dynamic_variable && v == query.query {
                    (String::new(), "NULL")
                } else {
                    (v, t)
                }
            }
            Err(_) => {
                if query.is_dynamic_variable {
                    match config.get_variable(&query.query[1..]) {
                        Some(var_value) => (var_value.to_string(), "STRING"),
                        None => (String::new(), "NULL")
                    }
                } else {
                    (String::new(), "NULL")
                }
            }
        };

        let (final_value, final_type) = apply_filters(
            value_str,
            type_str,
            &query.expected_type,
            &query.expected_regex
        )?;

        results.push(QueryResult {
            key:        query.query.clone(),
            value:      final_value,
            value_type: final_type.to_string()
        });
    }

    let null_count = results.iter().filter(|r| r.value_type == "NULL").count();

    match args.export.as_deref() {
        Some("json") => export_json(&results),
        Some("env") => export_env(&results, &queries),
        _ => export_plain(&results, &args.delimiter)
    }

    if null_count > 0 { Ok(1) } else { Ok(0) }
}

fn apply_filters<'a>(
    value: String,
    type_str: &'a str,
    expected_type: &Option<String>,
    expected_regex: &Option<String>
) -> Result<(String, &'a str), AppError> {
    if let Some(exp_type) = expected_type
        && normalize_type(type_str) != normalize_type(exp_type)
    {
        return Ok((String::new(), "NULL"));
    }

    if let Some(pattern) = expected_regex {
        let rx = Regex::new(pattern)?;
        if !rx.is_match(&value) {
            return Ok((String::new(), "NULL"));
        }
    }

    Ok((value, type_str))
}

fn find_sources(config_path: &Path, base_dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    let content = fs::read_to_string(config_path)?;
    let mut sources = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("source")
            && let Some(eq_pos) = trimmed.find('=')
        {
            let path_part = trimmed[eq_pos + 1..].trim();
            let paths = resolve_glob(path_part, base_dir)?;
            sources.extend(paths);
        }
    }

    Ok(sources)
}

fn main() {
    match run() {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
