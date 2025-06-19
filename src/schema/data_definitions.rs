// src/schema/data_definitions.rs
// Pure validation logic - ported from TypeScript schema/data-definitions.ts

use serde_json::Value as JsonValue;

//=============================================================================
// DATA DEFINITION RESULT (Matches TypeScript DataDefResult exactly)
//=============================================================================

#[derive(Debug, Clone)]
pub struct DataDefResult {
  pub ok: bool,
  pub data: Option<JsonValue>,
  pub error: Option<String>,
  pub blocking: Option<bool>,
  pub talent_name: Option<String>,
  pub suggestions: Option<Vec<String>>,
}

impl DataDefResult {
  pub fn success(data: JsonValue, talent_name: Option<String>) -> Self {
    Self {
      ok: true,
      data: Some(data),
      error: None,
      blocking: None,
      talent_name,
      suggestions: None,
    }
  }

  pub fn error(error: String, blocking: bool, suggestions: Vec<String>) -> Self {
    Self {
      ok: false,
      data: None,
      error: Some(error),
      blocking: Some(blocking),
      talent_name: None,
      suggestions: Some(suggestions),
    }
  }

  pub fn skip() -> Self {
    Self {
      ok: true,
      data: None,
      error: None,
      blocking: None,
      talent_name: None,
      suggestions: None,
    }
  }
}

//=============================================================================
// UTILITY FUNCTIONS (Matches TypeScript exactly)
//=============================================================================

/// Helper to describe value (matches TypeScript describeValue exactly)
pub fn describe_value(value: &JsonValue) -> String {
  match value {
    JsonValue::Null => "null".to_string(),
    JsonValue::Bool(b) => format!("boolean {}", b),
    JsonValue::Number(n) => format!("number {}", n),
    JsonValue::String(s) => format!("string \"{}\"", s),
    JsonValue::Array(arr) => format!("array with {} items", arr.len()),
    JsonValue::Object(obj) => {
      let keys: Vec<String> = obj
        .keys()
        .map(|k| k.to_string())
        .collect();
      format!("object with keys: {}", keys.join(", "))
    }
  }
}

/// Fast validation helpers (matches TypeScript)
pub fn is_string(value: &JsonValue) -> bool {
  value.is_string()
}

pub fn is_number(value: &JsonValue) -> bool {
  value.is_number()
}

pub fn is_boolean(value: &JsonValue) -> bool {
  value.is_boolean()
}

pub fn is_function(value: &JsonValue) -> bool {
  // In Rust, we can't check for functions at runtime like TypeScript
  // So we assume any non-null value could be a function reference
  !value.is_null()
}

//=============================================================================
// CORE DATA DEFINITIONS (Ported from TypeScript dataDefinitions exactly)
//=============================================================================

/// Core required fields - ID validation (matches TypeScript exactly)
pub fn validate_id(value: &JsonValue) -> DataDefResult {
  if !is_string(value) || value.as_str().unwrap_or("").trim().is_empty() {
    return DataDefResult::error(
      format!("Channel ID must be a non-empty text value, but received {}", describe_value(value)),
      true,
      vec![
        "Provide a unique text identifier for this channel".to_string(),
        "Example: \"user-validator\" or \"sensor-IUG576&$\"".to_string(),
        "Avoid spaces - use hyphens or underscores".to_string()
      ]
    );
  }

  DataDefResult::success(value.clone(), None)
}

/// Condition validation (matches TypeScript exactly)
pub fn validate_condition(value: &JsonValue) -> DataDefResult {
  if value.is_null() {
    return DataDefResult::success(JsonValue::Null, None);
  }

  if !is_function(value) {
    return DataDefResult::error(
      format!(
        "Condition must be a function that returns true or false, but received {}",
        describe_value(value)
      ),
      false,
      vec![
        "Function should return boolean: (payload) => boolean".to_string(),
        "Return true to allow execution, false to skip".to_string(),
        "Example: (payload) => payload.status === \"active\"".to_string()
      ]
    );
  }

  DataDefResult::success(value.clone(), Some("condition".to_string()))
}

/// Selector validation (matches TypeScript exactly)
pub fn validate_selector(value: &JsonValue) -> DataDefResult {
  if value.is_null() {
    return DataDefResult::success(JsonValue::Null, None);
  }

  if !is_function(value) {
    return DataDefResult::error(
      format!(
        "Selector must be a function that extracts data, but received {}",
        describe_value(value)
      ),
      false,
      vec![
        "Function should extract part of your data: (payload) => any".to_string(),
        "Return the specific data you want to use".to_string(),
        "Example: (payload) => payload.user.email".to_string()
      ]
    );
  }

  DataDefResult::success(value.clone(), Some("selector".to_string()))
}

/// Transform validation (matches TypeScript exactly)
pub fn validate_transform(value: &JsonValue) -> DataDefResult {
  if value.is_null() {
    return DataDefResult::success(JsonValue::Null, None);
  }

  if !is_function(value) {
    return DataDefResult::error(
      format!(
        "Transform must be a function that modifies data, but received {}",
        describe_value(value)
      ),
      false,
      vec![
        "Function should return modified data: (payload) => any".to_string(),
        "Transform and return your data as needed".to_string(),
        "Example: (payload) => ({ ...payload, processed: true })".to_string()
      ]
    );
  }

  DataDefResult::success(value.clone(), Some("transform".to_string()))
}

/// DetectChanges validation (matches TypeScript exactly)
pub fn validate_detect_changes(value: &JsonValue) -> DataDefResult {
  if value.is_null() {
    return DataDefResult::success(JsonValue::Null, None);
  }

  if !is_boolean(value) {
    return DataDefResult::error(
      format!("DetectChanges must be true or false, but received {}", describe_value(value)),
      false,
      vec![
        "Use true to only execute when data changes from previous call".to_string(),
        "Use false to execute every time regardless of changes".to_string(),
        "This helps prevent unnecessary processing of duplicate data".to_string()
      ]
    );
  }

  DataDefResult::success(value.clone(), Some("detect_changes".to_string()))
}

/// Required validation (matches TypeScript exactly)
pub fn validate_required(value: &JsonValue) -> DataDefResult {
  if value.is_null() {
    return DataDefResult::success(JsonValue::Null, None);
  }

  if !is_boolean(value) && value.as_str() != Some("non-empty") {
    return DataDefResult::error(
      format!(
        "Required must be true, false, or \"non-empty\", but received {}",
        describe_value(value)
      ),
      false,
      vec![
        "Use true to require any value (including empty strings and zero)".to_string(),
        "Use \"non-empty\" to require non-empty values (excludes \"\", [], {})".to_string(),
        "Use false to make the field optional".to_string()
      ]
    );
  }

  DataDefResult::success(value.clone(), Some("required".to_string()))
}

/// Block validation (matches TypeScript exactly)
pub fn validate_block(value: &JsonValue) -> DataDefResult {
  if value.is_null() {
    return DataDefResult::success(JsonValue::Null, None);
  }

  if !is_boolean(value) {
    return DataDefResult::error(
      format!("Block must be true or false, but received {}", describe_value(value)),
      false,
      vec![
        "Use true to block all execution".to_string(),
        "Use false to allow execution".to_string()
      ]
    );
  }

  DataDefResult::success(value.clone(), Some("block".to_string()))
}

/// Throttle validation (matches TypeScript exactly)
pub fn validate_throttle(value: &JsonValue) -> DataDefResult {
  if value.is_null() {
    return DataDefResult::success(JsonValue::Null, None);
  }

  if !is_number(value) || value.as_u64().unwrap_or(0) == 0 {
    return DataDefResult::error(
      format!(
        "Throttle must be a positive number (milliseconds), but received {}",
        describe_value(value)
      ),
      false,
      vec![
        "Specify time in milliseconds to limit execution frequency".to_string(),
        "Example: 1000 for maximum once per second".to_string(),
        "Use 0 to disable throttling".to_string()
      ]
    );
  }

  DataDefResult::success(value.clone(), Some("throttle".to_string()))
}

/// Debounce validation (matches TypeScript exactly)
pub fn validate_debounce(value: &JsonValue) -> DataDefResult {
  if value.is_null() {
    return DataDefResult::success(JsonValue::Null, None);
  }

  if !is_number(value) || value.as_u64().unwrap_or(0) == 0 {
    return DataDefResult::error(
      format!(
        "Debounce must be a positive number (milliseconds), but received {}",
        describe_value(value)
      ),
      false,
      vec![
        "Specify delay in milliseconds to wait for rapid calls to settle".to_string(),
        "Example: 300 to wait 300ms after last call before executing".to_string(),
        "Use 0 to disable debouncing".to_string()
      ]
    );
  }

  DataDefResult::success(value.clone(), Some("debounce".to_string()))
}

/// Schema validation (matches TypeScript exactly)
pub fn validate_schema(value: &JsonValue) -> DataDefResult {
  if value.is_null() {
    return DataDefResult::success(JsonValue::Null, None);
  }

  if !is_function(value) {
    return DataDefResult::error(
      format!("Schema must be a validation function, but received {}", describe_value(value)),
      false,
      vec![
        "Use cyre-schema builders: schema.object({ name: schema.string() })".to_string(),
        "Or provide custom function: (data) => ({ ok: true, data })".to_string(),
        "Function should return { ok: boolean, data?: any, errors?: string[] }".to_string()
      ]
    );
  }

  DataDefResult::success(value.clone(), Some("schema".to_string()))
}

/// Delay validation (matches TypeScript exactly)
pub fn validate_delay(value: &JsonValue) -> DataDefResult {
  if value.is_null() {
    return DataDefResult::success(JsonValue::Null, None);
  }

  if !is_number(value) || value.as_u64().unwrap_or(0) == 0 {
    return DataDefResult::error(
      format!(
        "Delay must be a positive number (milliseconds), but received {}",
        describe_value(value)
      ),
      false,
      vec![
        "Specify initial delay in milliseconds before first execution".to_string(),
        "Example: 1000 to wait 1 second before executing".to_string(),
        "Use 0 for immediate execution".to_string()
      ]
    );
  }

  DataDefResult::success(value.clone(), Some("schedule".to_string()))
}

/// Interval validation (matches TypeScript exactly)
pub fn validate_interval(value: &JsonValue) -> DataDefResult {
  if value.is_null() {
    return DataDefResult::success(JsonValue::Null, None);
  }

  if !is_number(value) || value.as_u64().unwrap_or(0) == 0 {
    return DataDefResult::error(
      format!(
        "Interval must be a positive number (milliseconds), but received {}",
        describe_value(value)
      ),
      false,
      vec![
        "Specify time in milliseconds between repeated executions".to_string(),
        "Example: 5000 to execute every 5 seconds".to_string(),
        "Must be greater than 0 for repeated execution".to_string()
      ]
    );
  }

  DataDefResult::success(value.clone(), Some("schedule".to_string()))
}

/// Repeat validation (matches TypeScript exactly)
pub fn validate_repeat(value: &JsonValue) -> DataDefResult {
  if value.is_null() {
    return DataDefResult::success(JsonValue::Null, None);
  }

  if !is_number(value) && !is_boolean(value) {
    return DataDefResult::error(
      format!("Repeat must be a number, true, or false, but received {}", describe_value(value)),
      false,
      vec![
        "Use a number to specify exact repetitions (e.g., 5)".to_string(),
        "Use true for infinite repetitions".to_string(),
        "Use false or omit to execute only once".to_string()
      ]
    );
  }

  if let Some(num) = value.as_u64() {
    if num == 0 {
      return DataDefResult::error(
        format!("Repeat count must be a positive whole number, but received {}", num),
        false,
        vec![
          "Use positive integers: 1, 2, 3, etc.".to_string(),
          "Use true for infinite repetitions".to_string(),
          "Decimals are not allowed for repeat counts".to_string()
        ]
      );
    }
  }

  DataDefResult::success(value.clone(), Some("schedule".to_string()))
}

/// Pass-through validation for additional fields
pub fn validate_pass_through(value: &JsonValue) -> DataDefResult {
  DataDefResult::success(value.clone(), None)
}

//=============================================================================
// TALENT CATEGORIES (Matches TypeScript exactly)
//=============================================================================

/// Protection talents for optimization flags
pub const PROTECTION_TALENTS: &[&str] = &["block", "throttle", "debounce"];

/// Processing talents
pub const PROCESSING_TALENTS: &[&str] = &[
  "schema",
  "condition",
  "selector",
  "transform",
  "detect_changes",
  "required",
];

/// Scheduling talents
pub const SCHEDULING_TALENTS: &[&str] = &["interval", "delay", "repeat"];

//=============================================================================
// VALIDATION DISPATCHER (O(1) Match Statement)
//=============================================================================

/// Single validation dispatcher (replaces HashMap lookup)
pub fn validate_field(field_name: &str, value: &JsonValue) -> DataDefResult {
  match field_name {
    // Core required fields
    "id" => validate_id(value),

    // Protection talents (processed first)
    "block" => validate_block(value),
    "throttle" => validate_throttle(value),
    "debounce" => validate_debounce(value),

    // Validation talents
    "required" => validate_required(value),
    "schema" => validate_schema(value),

    // Processing talents (preserve user order)
    "condition" => validate_condition(value),
    "selector" => validate_selector(value),
    "transform" => validate_transform(value),
    "detect_changes" => validate_detect_changes(value),

    // Scheduling talents (processed last)
    "delay" => validate_delay(value),
    "interval" => validate_interval(value),
    "repeat" => validate_repeat(value),

    // Pass-through fields (matches TypeScript exactly)
    | "payload"
    | "type"
    | "log"
    | "priority"
    | "maxWait"
    | "_hasFastPath"
    | "_hasProtections"
    | "_hasProcessing"
    | "_hasScheduling"
    | "_processingPipeline"
    | "_isBlocked"
    | "_blockReason"
    | "timestamp"
    | "time_of_creation"
    | "name" => {
      validate_pass_through(value)
    }

    // Unknown field
    _ =>
      DataDefResult::error(
        format!("Unknown field message from data-definitions: {}", field_name),
        false,
        vec![
          "Check field name spelling".to_string(),
          "Refer to documentation for valid fields".to_string()
        ]
      ),
  }
}
