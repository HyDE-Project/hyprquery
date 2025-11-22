/// Input query with optional type and regex hints
#[derive(Debug, Clone)]
pub struct QueryInput {
    /// The query key to look up
    pub query:               String,
    /// Expected type hint (INT, FLOAT, STRING, etc.)
    pub expected_type:       Option<String>,
    /// Expected value regex pattern
    pub expected_regex:      Option<String>,
    /// Whether this query is for a dynamic variable ($var)
    pub is_dynamic_variable: bool
}

/// Result of a configuration query
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// The original query key
    pub key:        Box<str>,
    /// The resolved value as string
    pub value:      Box<str>,
    /// The type of the value
    pub value_type: &'static str
}

/// Parse raw query strings into QueryInput structs
///
/// Query format: `query[expectedType][expectedRegex]`
///
/// # Arguments
///
/// * `raw_queries` - Vector of raw query strings
///
/// # Returns
///
/// Vector of parsed QueryInput structs
///
/// # Examples
///
/// ```
/// use hyprquery::query::parse_query_inputs;
///
/// let queries = parse_query_inputs(&["general:border_size".to_string()]);
/// assert_eq!(queries[0].query, "general:border_size");
/// ```
pub fn parse_query_inputs(raw_queries: &[String]) -> Vec<QueryInput> {
    raw_queries
        .iter()
        .map(|raw| {
            let first_bracket = raw.find('[');

            if first_bracket.is_none() {
                return QueryInput {
                    is_dynamic_variable: raw.starts_with('$'),
                    query:               raw.clone(),
                    expected_type:       None,
                    expected_regex:      None
                };
            }

            let first_bracket = first_bracket.unwrap_or(raw.len());
            let query = raw[..first_bracket].to_string();
            let is_dynamic_variable = query.starts_with('$');

            let second_bracket = raw[first_bracket..].find(']').map(|i| i + first_bracket);

            let expected_type = second_bracket.and_then(|end| {
                let type_str = raw[first_bracket + 1..end].to_string();
                if type_str.is_empty() {
                    None
                } else {
                    Some(type_str)
                }
            });

            let expected_regex = if let Some(second_end) = second_bracket {
                let remaining = &raw[second_end + 1..];
                if let Some(third_start) = remaining.find('[') {
                    if let Some(third_end) = remaining[third_start..].find(']') {
                        let regex_str =
                            remaining[third_start + 1..third_start + third_end].to_string();
                        if regex_str.is_empty() {
                            None
                        } else {
                            Some(regex_str)
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            QueryInput {
                query,
                expected_type,
                expected_regex,
                is_dynamic_variable
            }
        })
        .collect()
}

/// Normalize type string to uppercase for comparison
pub fn normalize_type(type_str: &str) -> String {
    type_str.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let queries = parse_query_inputs(&["general:border_size".to_string()]);
        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query, "general:border_size");
        assert!(queries[0].expected_type.is_none());
        assert!(queries[0].expected_regex.is_none());
        assert!(!queries[0].is_dynamic_variable);
    }

    #[test]
    fn test_parse_query_with_type() {
        let queries = parse_query_inputs(&["general:border_size[INT]".to_string()]);
        assert_eq!(queries[0].query, "general:border_size");
        assert_eq!(queries[0].expected_type.as_deref(), Some("INT"));
        assert!(queries[0].expected_regex.is_none());
    }

    #[test]
    fn test_parse_query_with_type_and_regex() {
        let queries = parse_query_inputs(&["general:border_size[INT][^\\d+$]".to_string()]);
        assert_eq!(queries[0].query, "general:border_size");
        assert_eq!(queries[0].expected_type.as_deref(), Some("INT"));
        assert_eq!(queries[0].expected_regex.as_deref(), Some("^\\d+$"));
    }

    #[test]
    fn test_parse_dynamic_variable() {
        let queries = parse_query_inputs(&["$terminal".to_string()]);
        assert!(queries[0].is_dynamic_variable);
        assert_eq!(queries[0].query, "$terminal");
    }

    #[test]
    fn test_parse_empty_type_bracket() {
        let queries = parse_query_inputs(&["key[][regex]".to_string()]);
        assert_eq!(queries[0].query, "key");
        assert!(queries[0].expected_type.is_none());
        assert_eq!(queries[0].expected_regex.as_deref(), Some("regex"));
    }

    #[test]
    fn test_parse_multiple_queries() {
        let queries = parse_query_inputs(&[
            "key1".to_string(),
            "key2[STRING]".to_string(),
            "$var".to_string()
        ]);
        assert_eq!(queries.len(), 3);
        assert_eq!(queries[0].query, "key1");
        assert_eq!(queries[1].expected_type.as_deref(), Some("STRING"));
        assert!(queries[2].is_dynamic_variable);
    }

    #[test]
    fn test_normalize_type() {
        assert_eq!(normalize_type("int"), "INT");
        assert_eq!(normalize_type("STRING"), "STRING");
        assert_eq!(normalize_type("Float"), "FLOAT");
    }
}
