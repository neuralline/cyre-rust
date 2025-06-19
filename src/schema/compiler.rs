// src/schema/compiler.rs - DYNAMIC FIELD DISCOVERY IN COMPILER

use crate::types::IO;
use crate::schema::data_definitions::validate_field;
use crate::schema::operators::*;
use crate::context::sensor;
use serde_json::Value as JsonValue;

/// Dynamic field discovery - done by compiler, not IO struct
fn get_dynamic_fields(config: &IO) -> Vec<(String, JsonValue)> {
  let mut fields = Vec::new();

  // Use serde serialization for true dynamic field discovery
  if let Ok(json_value) = serde_json::to_value(config) {
    if let JsonValue::Object(map) = json_value {
      for (key, value) in map {
        if should_include_field(&key, &value) {
          fields.push((key, value));
        }
      }
    }
  }

  fields
}

/// Determine if field should be included in compilation
fn should_include_field(key: &str, value: &JsonValue) -> bool {
  // Skip internal fields (starting with _)
  if key.starts_with('_') {
    return false;
  }

  // Only include fields with meaningful values
  match value {
    JsonValue::Null => false,
    JsonValue::Bool(b) => *b, // Include true, skip false
    JsonValue::String(s) => !s.is_empty(),
    JsonValue::Number(_) => true,
    JsonValue::Array(arr) => !arr.is_empty(),
    JsonValue::Object(obj) => !obj.is_empty(),
  }
}

/// Dynamic compilation - NO hardcoded field enumeration
fn dynamic_compile(config: &IO) -> CompileResult {
  let mut operators = Vec::new();
  let mut errors = Vec::new();
  let mut suggestions = Vec::new();

  // STEP 1: Dynamic field discovery and operator creation
  for (field_name, field_value) in get_dynamic_fields(config) {
    let result = validate_field(&field_name, &field_value);

    if !result.ok {
      if let Some(error) = result.error {
        errors.push(format!("Field '{}': {}", field_name, error));
      }
      if let Some(field_suggestions) = result.suggestions {
        for suggestion in field_suggestions {
          suggestions.push(format!("Field '{}': {}", field_name, suggestion));
        }
      }
      if result.blocking.unwrap_or(false) {
        return CompileResult::failure(errors, suggestions);
      }
    } else {
      // CREATE OPERATOR INSTANCE directly during compilation
      if let Some(operator) = create_operator_from_field(&field_name, &field_value, config) {
        operators.push(operator);
      }
    }
  }

  if !errors.is_empty() {
    return CompileResult::failure(errors, suggestions);
  }

  // STEP 2: Enforce pipeline ordering
  let ordered_operators = enforce_operator_ordering(operators);

  // STEP 3: Calculate flags
  let has_fast_path = ordered_operators.is_empty();
  let has_protections = has_protection_operators(&ordered_operators);
  let has_scheduling = has_schedule_operator(&ordered_operators);

  CompileResult::success(ordered_operators, has_fast_path, has_protections, has_scheduling)
}

/// Create operator instance from field configuration
fn create_operator_from_field(field: &str, value: &JsonValue, config: &IO) -> Option<Operator> {
  match field {
    "block" => {
      if value.as_bool() == Some(true) { Some(Operator::Block(BlockOperator::new())) } else { None }
    }
    "throttle" => {
      if let Some(ms) = value.as_u64() {
        if ms > 0 { Some(Operator::Throttle(ThrottleOperator::new(ms))) } else { None }
      } else {
        None
      }
    }
    "debounce" => {
      if let Some(ms) = value.as_u64() {
        if ms > 0 { Some(Operator::Debounce(DebounceOperator::new(ms))) } else { None }
      } else {
        None
      }
    }
    "required" => {
      if value.as_bool() == Some(true) || value.as_str() == Some("non-empty") {
        Some(Operator::Required(RequiredOperator::new()))
      } else {
        None
      }
    }
    "schema" => {
      if let Some(schema_str) = value.as_str() {
        Some(Operator::Schema(SchemaOperator::new(schema_str)))
      } else {
        None
      }
    }
    "condition" => {
      if let Some(condition_str) = value.as_str() {
        Some(Operator::Condition(ConditionOperator::new(condition_str)))
      } else {
        None
      }
    }
    "selector" => {
      if let Some(selector_str) = value.as_str() {
        Some(Operator::Selector(SelectorOperator::new(selector_str)))
      } else {
        None
      }
    }
    "transform" => {
      if let Some(transform_str) = value.as_str() {
        Some(Operator::Transform(TransformOperator::new(transform_str)))
      } else {
        None
      }
    }
    "detect_changes" => {
      if value.as_bool() == Some(true) {
        Some(Operator::DetectChanges(DetectChangesOperator::new()))
      } else {
        None
      }
    }
    // Scheduling fields create single schedule operator
    "delay" | "interval" | "repeat" => {
      Some(
        Operator::Schedule(
          ScheduleOperator::new(
            config.delay,
            config.interval,
            config.repeat.as_ref().and_then(|v| v.as_u64().map(|n| n as u32))
          )
        )
      )
    }
    _ => None,
  }
}

/// Enforce operator ordering (Protection → Validation → Processing → Scheduling)
fn enforce_operator_ordering(operators: Vec<Operator>) -> Vec<Operator> {
  let mut ordered = Vec::new();
  let mut remaining = Vec::new();

  // STEP 1: Protection operators first
  for op in operators {
    match op {
      Operator::Block(_) | Operator::Throttle(_) | Operator::Debounce(_) => {
        ordered.push(op);
      }
      _ => remaining.push(op),
    }
  }

  // STEP 2: Validation operators next
  let mut processing = Vec::new();
  for op in remaining {
    match op {
      Operator::Required(_) | Operator::Schema(_) => {
        ordered.push(op);
      }
      _ => processing.push(op),
    }
  }

  // STEP 3: Processing operators (preserve user order)
  let mut scheduling = Vec::new();
  for op in processing {
    match op {
      Operator::Schedule(_) => scheduling.push(op),
      _ => ordered.push(op),
    }
  }

  // STEP 4: Scheduling operators last
  ordered.extend(scheduling);

  ordered
}

fn has_protection_operators(operators: &[Operator]) -> bool {
  operators
    .iter()
    .any(|op| matches!(op, Operator::Block(_) | Operator::Throttle(_) | Operator::Debounce(_)))
}

fn has_schedule_operator(operators: &[Operator]) -> bool {
  operators.iter().any(|op| matches!(op, Operator::Schedule(_)))
}

#[derive(Debug)]
pub struct CompileResult {
  pub ok: bool,
  pub pipeline: Vec<Operator>,
  pub errors: Vec<String>,
  pub suggestions: Vec<String>,
  pub has_fast_path: bool,
  pub has_protections: bool,
  pub has_scheduling: bool,
}

impl CompileResult {
  pub fn success(
    pipeline: Vec<Operator>,
    has_fast_path: bool,
    has_protections: bool,
    has_scheduling: bool
  ) -> Self {
    Self {
      ok: true,
      pipeline,
      errors: Vec::new(),
      suggestions: Vec::new(),
      has_fast_path,
      has_protections,
      has_scheduling,
    }
  }

  pub fn failure(errors: Vec<String>, suggestions: Vec<String>) -> Self {
    Self {
      ok: false,
      pipeline: Vec::new(),
      errors,
      suggestions,
      has_fast_path: false,
      has_protections: false,
      has_scheduling: false,
    }
  }
}

/// Main compilation function
pub fn compile_pipeline(config: &mut IO) -> CompileResult {
  let compile_result = dynamic_compile(config);

  if !compile_result.ok {
    for error in &compile_result.errors {
      sensor::error("pipeline_compiler", error, Some("compile_pipeline"), None);
    }
    return compile_result;
  }

  // SUCCESS: Update config with compiled operators
  config._pipeline = compile_result.pipeline.clone();
  config._has_fast_path = compile_result.has_fast_path;
  config._has_protections = compile_result.has_protections;
  config._has_scheduling = compile_result.has_scheduling;

  compile_result
}
