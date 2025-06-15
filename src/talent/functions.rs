// src/talent/functions.rs
// Common talent functions and utilities - FIXED type annotations

use crate::types::{ActionPayload, ValidationResult};
use serde_json::{json, Value};

//=============================================================================
// SCHEMA VALIDATION FUNCTIONS
//=============================================================================

/// Validate that payload contains required fields
pub fn require_fields<'a>(fields: &'a [&'a str]) -> impl Fn(ActionPayload) -> ValidationResult + 'a {
    move |payload| {
        let mut errors = Vec::new();
        let warnings = Vec::new();
        
        for field in fields {
            if payload.get(field).is_none() {
                errors.push(format!("Required field '{}' is missing", field));
            }
        }
        
        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

/// Validate that numeric fields are within range
pub fn validate_numeric_range(field: &str, min: f64, max: f64) -> impl Fn(ActionPayload) -> ValidationResult + '_ {
    let field = field.to_string();
    move |payload| {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        if let Some(value) = payload.get(&field) {
            if let Some(num) = value.as_f64() {
                if num < min || num > max {
                    errors.push(format!("Field '{}' value {} is outside range [{}, {}]", field, num, min, max));
                }
            } else {
                errors.push(format!("Field '{}' is not a valid number", field));
            }
        } else {
            warnings.push(format!("Field '{}' not found for range validation", field));
        }
        
        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

/// Validate string length
pub fn validate_string_length(field: &str, min_len: usize, max_len: usize) -> impl Fn(ActionPayload) -> ValidationResult + '_ {
    let field = field.to_string();
    move |payload| {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        if let Some(value) = payload.get(&field) {
            if let Some(string) = value.as_str() {
                let len = string.len();
                if len < min_len || len > max_len {
                    errors.push(format!("Field '{}' length {} is outside range [{}, {}]", field, len, min_len, max_len));
                }
            } else {
                errors.push(format!("Field '{}' is not a string", field));
            }
        } else {
            warnings.push(format!("Field '{}' not found for length validation", field));
        }
        
        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

/// Validate email format (simple check)
pub fn validate_email(field: &str) -> impl Fn(ActionPayload) -> ValidationResult + '_ {
    let field = field.to_string();
    move |payload| {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        if let Some(value) = payload.get(&field) {
            if let Some(email) = value.as_str() {
                if !email.contains('@') || !email.contains('.') || email.len() < 5 {
                    errors.push(format!("Field '{}' is not a valid email address", field));
                }
            } else {
                errors.push(format!("Field '{}' is not a string", field));
            }
        } else {
            warnings.push(format!("Field '{}' not found for email validation", field));
        }
        
        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

//=============================================================================
// CONDITION FUNCTIONS
//=============================================================================

/// Check if field equals a specific value
pub fn field_equals(field: &str, expected: Value) -> impl Fn(ActionPayload) -> bool + '_ {
    let field = field.to_string();
    move |payload| {
        payload.get(&field) == Some(&expected)
    }
}

/// Check if field exists
pub fn field_exists(field: &str) -> impl Fn(ActionPayload) -> bool + '_ {
    let field = field.to_string();
    move |payload| {
        payload.get(&field).is_some()
    }
}

/// Check if numeric field is greater than value
pub fn field_greater_than(field: &str, threshold: f64) -> impl Fn(ActionPayload) -> bool + '_ {
    let field = field.to_string();
    move |payload| {
        payload.get(&field)
            .and_then(|v| v.as_f64())
            .map(|n| n > threshold)
            .unwrap_or(false)
    }
}

/// Check if string field matches pattern (simple contains check)
pub fn field_contains<'a>(field: &'a str, pattern: &'a str) -> impl Fn(ActionPayload) -> bool + 'a {
    let field = field.to_string();
    let pattern = pattern.to_string();
    move |payload| {
        payload.get(&field)
            .and_then(|v| v.as_str())
            .map(|s| s.contains(&pattern))
            .unwrap_or(false)
    }
}

/// Check if array field has minimum length
pub fn array_min_length(field: &str, min_len: usize) -> impl Fn(ActionPayload) -> bool + '_ {
    let field = field.to_string();
    move |payload| {
        payload.get(&field)
            .and_then(|v| v.as_array())
            .map(|arr| arr.len() >= min_len)
            .unwrap_or(false)
    }
}

//=============================================================================
// TRANSFORM FUNCTIONS
//=============================================================================

/// Add a field with a specific value
pub fn add_field(field_name: &str, value: Value) -> impl Fn(ActionPayload) -> ActionPayload + '_ {
    let field_name = field_name.to_string();
    move |mut payload| {
        if let Some(obj) = payload.as_object_mut() {
            obj.insert(field_name.clone(), value.clone());
        }
        payload
    }
}

/// Remove a field
pub fn remove_field(field_name: &str) -> impl Fn(ActionPayload) -> ActionPayload + '_ {
    let field_name = field_name.to_string();
    move |mut payload| {
        if let Some(obj) = payload.as_object_mut() {
            obj.remove(&field_name);
        }
        payload
    }
}

/// Rename a field
pub fn rename_field<'a>(old_name: &'a str, new_name: &'a str) -> impl Fn(ActionPayload) -> ActionPayload + 'a {
    let old_name = old_name.to_string();
    let new_name = new_name.to_string();
    move |mut payload| {
        if let Some(obj) = payload.as_object_mut() {
            if let Some(value) = obj.remove(&old_name) {
                obj.insert(new_name.clone(), value);
            }
        }
        payload
    }
}

/// Transform string to uppercase
pub fn uppercase_field(field_name: &str) -> impl Fn(ActionPayload) -> ActionPayload + '_ {
    let field_name = field_name.to_string();
    move |mut payload| {
        if let Some(obj) = payload.as_object_mut() {
            if let Some(value) = obj.get(&field_name).and_then(|v| v.as_str()) {
                obj.insert(field_name.clone(), json!(value.to_uppercase()));
            }
        }
        payload
    }
}

/// Transform string to lowercase
pub fn lowercase_field(field_name: &str) -> impl Fn(ActionPayload) -> ActionPayload + '_ {
    let field_name = field_name.to_string();
    move |mut payload| {
        if let Some(obj) = payload.as_object_mut() {
            if let Some(value) = obj.get(&field_name).and_then(|v| v.as_str()) {
                obj.insert(field_name.clone(), json!(value.to_lowercase()));
            }
        }
        payload
    }
}

/// Add timestamp field
pub fn add_timestamp(field_name: &str) -> impl Fn(ActionPayload) -> ActionPayload + '_ {
    let field_name = field_name.to_string();
    move |mut payload| {
        if let Some(obj) = payload.as_object_mut() {
            obj.insert(field_name.clone(), json!(crate::utils::current_timestamp()));
        }
        payload
    }
}

/// Increment numeric field
pub fn increment_field(field_name: &str, amount: f64) -> impl Fn(ActionPayload) -> ActionPayload + '_ {
    let field_name = field_name.to_string();
    move |mut payload| {
        if let Some(obj) = payload.as_object_mut() {
            if let Some(current) = obj.get(&field_name).and_then(|v| v.as_f64()) {
                obj.insert(field_name.clone(), json!(current + amount));
            }
        }
        payload
    }
}

//=============================================================================
// SELECTOR FUNCTIONS
//=============================================================================

/// Select only specific fields
pub fn select_fields<'a>(fields: &'a [&'a str]) -> impl Fn(ActionPayload) -> ActionPayload + 'a {
    let fields: Vec<String> = fields.iter().map(|s| s.to_string()).collect();
    move |payload| {
        let mut result = json!({});
        if let Some(result_obj) = result.as_object_mut() {
            for field in &fields {
                if let Some(value) = payload.get(field) {
                    result_obj.insert(field.clone(), value.clone());
                }
            }
        }
        result
    }
}

/// Select nested field - FIXED with explicit type annotation
pub fn select_nested(path: &str) -> impl Fn(ActionPayload) -> ActionPayload + '_ {
    let path = path.to_string();
    move |payload: ActionPayload| -> ActionPayload {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &payload;
        
        for part in parts {
            if let Some(next) = current.get(part) {
                current = next;
            } else {
                return json!(null);
            }
        }
        
        current.clone()
    }
}

/// Select array elements by index range
pub fn select_array_range(field: &str, start: usize, end: Option<usize>) -> impl Fn(ActionPayload) -> ActionPayload + '_ {
    let field = field.to_string();
    move |payload| {
        if let Some(array) = payload.get(&field).and_then(|v| v.as_array()) {
            let end_idx = end.unwrap_or(array.len());
            let selected: Vec<Value> = array.iter()
                .skip(start)
                .take(end_idx.saturating_sub(start))
                .cloned()
                .collect();
            json!(selected)
        } else {
            json!([])
        }
    }
}

/// Filter array elements by condition
pub fn filter_array<F>(field: &str, condition: F) -> impl Fn(ActionPayload) -> ActionPayload + '_
where
    F: Fn(&Value) -> bool + 'static,
{
    let field = field.to_string();
    move |payload| {
        if let Some(array) = payload.get(&field).and_then(|v| v.as_array()) {
            let filtered: Vec<Value> = array.iter()
                .filter(|v| condition(v))
                .cloned()
                .collect();
            json!(filtered)
        } else {
            json!([])
        }
    }
}

//=============================================================================
// UTILITY FUNCTIONS
//=============================================================================

/// Create a composite validation function
pub fn composite_validation(validators: Vec<Box<dyn Fn(ActionPayload) -> ValidationResult>>) -> impl Fn(ActionPayload) -> ValidationResult {
    move |payload| {
        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();
        
        for validator in &validators {
            let result = validator(payload.clone());
            all_errors.extend(result.errors);
            all_warnings.extend(result.warnings);
        }
        
        ValidationResult {
            valid: all_errors.is_empty(),
            errors: all_errors,
            warnings: all_warnings,
        }
    }
}

/// Create a composite condition (AND logic)
pub fn all_conditions(conditions: Vec<Box<dyn Fn(ActionPayload) -> bool>>) -> impl Fn(ActionPayload) -> bool {
    move |payload| {
        conditions.iter().all(|condition| condition(payload.clone()))
    }
}

/// Create a composite condition (OR logic)
pub fn any_condition(conditions: Vec<Box<dyn Fn(ActionPayload) -> bool>>) -> impl Fn(ActionPayload) -> bool {
    move |payload| {
        conditions.iter().any(|condition| condition(payload.clone()))
    }
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_require_fields() {
        let validator = require_fields(&["name", "email"]);
        
        // Valid payload
        let valid_payload = json!({"name": "John", "email": "john@example.com"});
        let result = validator(valid_payload);
        assert!(result.valid);
        
        // Invalid payload
        let invalid_payload = json!({"name": "John"});
        let result = validator(invalid_payload);
        assert!(!result.valid);
        assert!(result.errors[0].contains("email"));
    }

    #[test]
    fn test_validate_numeric_range() {
        let validator = validate_numeric_range("age", 0.0, 120.0);
        
        // Valid
        let result = validator(json!({"age": 25}));
        assert!(result.valid);
        
        // Invalid - too high
        let result = validator(json!({"age": 150}));
        assert!(!result.valid);
        
        // Invalid - not a number
        let result = validator(json!({"age": "twenty"}));
        assert!(!result.valid);
    }

    #[test]
    fn test_field_equals() {
        let condition = field_equals("status", json!("active"));
        
        assert!(condition(json!({"status": "active"})));
        assert!(!condition(json!({"status": "inactive"})));
        assert!(!condition(json!({"other": "active"})));
    }

    #[test]
    fn test_add_field() {
        let transform = add_field("timestamp", json!(1234567890));
        
        let result = transform(json!({"data": "test"}));
        assert_eq!(result.get("timestamp").unwrap(), &json!(1234567890));
        assert_eq!(result.get("data").unwrap(), &json!("test"));
    }

    #[test]
    fn test_select_fields() {
        let selector = select_fields(&["name", "email"]);
        
        let input = json!({
            "name": "John",
            "email": "john@example.com",
            "password": "secret",
            "age": 30
        });
        
        let result = selector(input);
        assert_eq!(result.get("name").unwrap(), &json!("John"));
        assert_eq!(result.get("email").unwrap(), &json!("john@example.com"));
        assert!(result.get("password").is_none());
        assert!(result.get("age").is_none());
    }

    #[test]
    fn test_uppercase_field() {
        let transform = uppercase_field("name");
        
        let result = transform(json!({"name": "john doe"}));
        assert_eq!(result.get("name").unwrap(), &json!("JOHN DOE"));
    }

    #[test]
    fn test_select_nested() {
        let selector = select_nested("user.profile.name");
        
        let input = json!({
            "user": {
                "profile": {
                    "name": "John Doe",
                    "age": 30
                },
                "id": 123
            }
        });
        
        let result = selector(input);
        assert_eq!(result, json!("John Doe"));
    }

    #[test]
    fn test_array_min_length() {
        let condition = array_min_length("items", 3);
        
        assert!(condition(json!({"items": [1, 2, 3, 4]})));
        assert!(!condition(json!({"items": [1, 2]})));
        assert!(!condition(json!({"items": "not an array"})));
    }
}