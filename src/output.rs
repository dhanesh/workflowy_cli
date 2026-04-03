// Satisfies: T2 (compact JSON), T7 (--fields), O3 (stdout for data), RT-3

use crate::error::CliError;
use serde::Serialize;
use serde_json::{Map, Value};

/// Print a serializable value as compact JSON to stdout.
/// If fields is Some, only include those keys in the output.
pub fn print_json<T: Serialize>(value: &T, fields: Option<&str>) -> Result<(), CliError> {
    let json = serde_json::to_value(value)?;
    let filtered = match fields {
        Some(f) => {
            let field_list: Vec<&str> = f.split(',').map(|s| s.trim()).collect();
            filter_fields(json, &field_list)
        }
        None => json,
    };
    let output = serde_json::to_string(&filtered)?;
    println!("{}", output);
    Ok(())
}

/// Recursively filter JSON to only include specified field names.
/// Works on objects (filter keys) and arrays (filter each element).
fn filter_fields(value: Value, fields: &[&str]) -> Value {
    match value {
        Value::Object(map) => {
            let filtered: Map<String, Value> = map
                .into_iter()
                .filter(|(k, _)| fields.iter().any(|f| *f == k))
                .collect();
            Value::Object(filtered)
        }
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(|v| filter_fields(v, fields)).collect())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Validates: T7 — --fields flag filters output to specified keys
    #[test]
    fn filter_fields_selects_specified_keys() {
        let input = json!({"id": "x", "name": "Y", "priority": 5, "note": "N"});
        let result = filter_fields(input, &["id", "name"]);
        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("name"));
        assert!(!obj.contains_key("priority"));
        assert!(!obj.contains_key("note"));
    }

    // Validates: T7 — field filtering works on arrays (each element filtered)
    #[test]
    fn filter_fields_applies_to_array_elements() {
        let input = json!([
            {"id": "1", "name": "A", "priority": 1},
            {"id": "2", "name": "B", "priority": 2}
        ]);
        let result = filter_fields(input, &["id", "priority"]);
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        for item in arr {
            let obj = item.as_object().unwrap();
            assert_eq!(obj.len(), 2);
            assert!(obj.contains_key("id"));
            assert!(obj.contains_key("priority"));
            assert!(!obj.contains_key("name"));
        }
    }

    // Validates: T6 — no fields = passthrough (no structural modification)
    #[test]
    fn no_filter_preserves_full_structure() {
        let input = json!({"id": "x", "name": "Y", "priority": 5});
        // When fields is None, print_json passes the value through unchanged.
        // We test filter_fields with all keys to simulate.
        let result = filter_fields(input.clone(), &["id", "name", "priority"]);
        assert_eq!(input, result);
    }

    // Validates: T7 — empty fields list returns empty objects
    #[test]
    fn filter_fields_with_empty_list_returns_empty() {
        let input = json!({"id": "x", "name": "Y"});
        let result = filter_fields(input, &[]);
        assert_eq!(result.as_object().unwrap().len(), 0);
    }

    // Validates: T7 — filter ignores nonexistent field names gracefully
    #[test]
    fn filter_fields_ignores_nonexistent_keys() {
        let input = json!({"id": "x", "name": "Y"});
        let result = filter_fields(input, &["nonexistent", "also_missing"]);
        assert_eq!(result.as_object().unwrap().len(), 0);
    }
}
