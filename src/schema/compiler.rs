// src/schema/compiler.rs
// Pipeline compilation system with dynamic field discovery
// File location: src/schema/compiler.rs

use crate::types::IO;
use crate::schema::data_definitions::validate_field;
use crate::context::sensor;
use serde_json::{ Value as JsonValue };
use std::collections::HashSet;

//=============================================================================
// COMPILATION RESULT TYPES
//=============================================================================

#[derive(Debug)]
pub struct CompileResult {
  pub ok: bool,
  pub pipeline: Vec<String>, // String operator names only
  pub errors: Vec<String>,
  pub suggestions: Vec<String>,
  pub has_fast_path: bool,
  pub has_protections: bool,
  pub has_scheduling: bool,
}

impl CompileResult {
  pub fn success(
    pipeline: Vec<String>,
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

//=============================================================================
// DYNAMIC FIELD DISCOVERY FOR IO STRUCT
//=============================================================================

impl IO {
  /// Extract all fields that have values - NO hardcoded field enumeration
  pub fn get_set_fields(&self) -> Vec<(String, JsonValue)> {
    // Use serde serialization for true dynamic field discovery
    let json_value = match serde_json::to_value(self) {
      Ok(value) => value,
      Err(_) => {
        return Vec::new();
      }
    };

    if let JsonValue::Object(map) = json_value {
      map
        .into_iter()
        .filter(|(k, v)| {
          // Only include fields with actual values
          !v.is_null() &&
            // Skip internal metadata fields
            !k.starts_with('_') &&
            // Skip empty collections
            !(v.is_array() && v.as_array().unwrap().is_empty()) &&
            !(v.is_object() && v.as_object().unwrap().is_empty()) &&
            // Skip default string values
            !(v.is_string() && v.as_str().unwrap().is_empty()) &&
            // Skip additional_properties HashMap (not a pipeline field)
            k != "additional_properties"
        })
        .collect()
    } else {
      Vec::new()
    }
  }
}

//=============================================================================
// MAIN COMPILATION API - TRULY DYNAMIC FIELD PROCESSING
//=============================================================================

/// Main compilation function - processes IO config into string pipeline
/// NO hardcoded field enumeration - uses dynamic field discovery
pub fn compile_pipeline(config: &mut IO) -> CompileResult {
  let compile_result = truly_dynamic_compile(config);

  if !compile_result.ok {
    // Log compilation errors via sensor
    for error in &compile_result.errors {
      sensor::error("pipeline_compiler", error, Some("compile_pipeline"), None);
    }

    // Log suggestions as warnings
    for suggestion in &compile_result.suggestions {
      sensor::warn("pipeline_compiler", &format!("Suggestion: {}", suggestion), false);
    }

    return compile_result;
  }

  // SUCCESS: Update config with compilation metadata
  config._pipeline = compile_result.pipeline.clone();
  config._has_fast_path = compile_result.has_fast_path;
  config._has_protections = compile_result.has_protections;
  config._has_scheduling = compile_result.has_scheduling;

  compile_result
}

//=============================================================================
// TRULY DYNAMIC COMPILATION WITH POST-PROCESSING ORDERING
//=============================================================================

/// Dynamic compilation - no hardcoded field knowledge
/// Uses serde serialization for true field discovery
fn truly_dynamic_compile(config: &IO) -> CompileResult {
  let mut pipeline_names = Vec::new();
  let mut errors = Vec::new();
  let mut suggestions = Vec::new();

  // STEP 1: TRULY DYNAMIC field discovery and validation
  // NO hardcoded if-statements checking specific fields!
  for (field_name, field_value) in config.get_set_fields() {
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

      // Blocking error = stop compilation immediately
      //   if result.blocking.unwrap_or(false) {
      //     return CompileResult::failure(errors, suggestions);
      //   }
    } else {
      // Verify + Push operator name in same loop iteration
      if let Some(talent_name) = result.talent_name {
        pipeline_names.push(talent_name);
      }
    }
  }

  // If any non-blocking errors occurred, return failure
  if !errors.is_empty() {
    return CompileResult::failure(errors, suggestions);
  }

  // STEP 2: POST-PROCESSING ORDERING - Enforce system architecture
  let ordered_pipeline = enforce_pipeline_ordering(pipeline_names);

  // STEP 3: Calculate flags
  let has_fast_path = ordered_pipeline.is_empty();
  let has_protections = has_protection_operators(&ordered_pipeline);
  let has_scheduling = has_schedule_operator(&ordered_pipeline);

  CompileResult::success(ordered_pipeline, has_fast_path, has_protections, has_scheduling)
}

//=============================================================================
// POST-PROCESSING PIPELINE ORDERING
//=============================================================================

/// Enforce proper pipeline ordering after dynamic compilation
/// Protection → Validation → Processing → Scheduling
fn enforce_pipeline_ordering(mut pipeline_names: Vec<String>) -> Vec<String> {
  let mut ordered_pipeline = Vec::new();
  let mut remaining_names = Vec::new();

  // STEP 1: Move protection operators to FRONT (in specific order)
  let protection_order = ["block", "throttle", "debounce"];
  for protection_name in &protection_order {
    if let Some(pos) = pipeline_names.iter().position(|name| name == protection_name) {
      ordered_pipeline.push(pipeline_names.remove(pos));
    }
  }

  // STEP 2: Add validation operators next
  let validation_order = ["required", "schema"];
  for validation_name in &validation_order {
    if let Some(pos) = pipeline_names.iter().position(|name| name == validation_name) {
      ordered_pipeline.push(pipeline_names.remove(pos));
    }
  }

  // STEP 3: Keep user order for processing operators (middle section)
  let scheduling_fields = ["repeat", "interval", "delay"];
  for name in pipeline_names {
    if !scheduling_fields.contains(&name.as_str()) {
      remaining_names.push(name); // Preserve user order
    }
  }
  ordered_pipeline.extend(remaining_names);

  // STEP 4: Replace scheduling fields with single 'schedule' operator at END
  let original_names: HashSet<String> = ordered_pipeline.iter().cloned().collect();
  let has_scheduling = scheduling_fields.iter().any(|field| original_names.contains(*field));

  if has_scheduling {
    ordered_pipeline.push("schedule".to_string()); // Single schedule operator
  }

  ordered_pipeline
}

//=============================================================================
// OPERATOR NAME EXTRACTION FROM FIELD - DYNAMIC MAPPING
//=============================================================================

/// Get operator name from field validation result
/// This is the ONLY place that maps field names to operator names
///
///
///
fn field_to_operator_name(field_name: &str) -> &str {
  match field_name {
    // The ONLY case where mapping differs from field name
    "delay" | "interval" | "repeat" => "schedule",

    // Everything else: field name IS the operator name
    _ => field_name, // Just return the field name as-is!
  }
}

//=============================================================================
// FLAG CALCULATION HELPERS
//=============================================================================

fn has_protection_operators(pipeline: &[String]) -> bool {
  pipeline.iter().any(|op| matches!(op.as_str(), "block" | "throttle" | "debounce"))
}

fn has_schedule_operator(pipeline: &[String]) -> bool {
  pipeline.iter().any(|op| op == "schedule")
}

//=============================================================================
// PUBLIC API FOR CONFIGURATION FIELDS
//=============================================================================
