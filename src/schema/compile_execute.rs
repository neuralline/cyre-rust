// src/schema/compile_execute.rs - FIXED: Proper pipeline storage and execution

use crate::types::{ IO, RequiredType, ActionPayload };
use crate::schema::operators::{
    Operator,
    OperatorResult,
    BlockOperator,
    ThrottleOperator,
    DebounceOperator,
    RequiredOperator,
    SchemaOperator,
    TransformOperator,
    ConditionOperator,
    SelectorOperator,
};
use crate::schema::data_definitions::{
    validate_field,
    DataDefResult,
    PROTECTION_TALENTS,
    PROCESSING_TALENTS,
    SCHEDULING_TALENTS,
};
use crate::context::sensor;
use serde_json::Value as JsonValue;

//=============================================================================
// COMPILATION RESULT WITH PROPER FLOW CONTROL
//=============================================================================

#[derive(Debug)]
pub struct CompileResult {
    pub ok: bool,
    pub operators: Vec<Operator>,
    pub errors: Vec<String>,
    pub suggestions: Vec<String>,
    pub has_fast_path: bool,
    pub has_protections: bool,
    pub has_processing: bool,
    pub has_scheduling: bool,
}

impl CompileResult {
    pub fn success(
        operators: Vec<Operator>,
        has_fast_path: bool,
        has_protections: bool,
        has_processing: bool,
        has_scheduling: bool
    ) -> Self {
        Self {
            ok: true,
            operators,
            errors: Vec::new(),
            suggestions: Vec::new(),
            has_fast_path,
            has_protections,
            has_processing,
            has_scheduling,
        }
    }

    pub fn failure(errors: Vec<String>, suggestions: Vec<String>) -> Self {
        Self {
            ok: false,
            operators: Vec::new(),
            errors,
            suggestions,
            has_fast_path: false,
            has_protections: false,
            has_processing: false,
            has_scheduling: false,
        }
    }
}

//=============================================================================
// PIPELINE RESULT (Same as existing)
//=============================================================================

#[derive(Debug)]
pub enum PipelineResult {
    Continue(ActionPayload),
    Block(String),
    Schedule,
}

//=============================================================================
// CONFIG FIELD ITERATOR
//=============================================================================

pub struct ConfigFieldIterator<'a> {
    config: &'a IO,
    fields: Vec<(&'static str, JsonValue)>,
    position: usize,
}

impl<'a> ConfigFieldIterator<'a> {
    pub fn new(config: &'a IO) -> Self {
        let mut fields = Vec::new();

        // Always include ID first
        fields.push(("id", JsonValue::String(config.id.clone())));

        // Protection fields first (your requested order)
        if config.block {
            fields.push(("block", JsonValue::Bool(true)));
        }
        if let Some(throttle) = config.throttle {
            fields.push(("throttle", JsonValue::from(throttle)));
        }
        if let Some(debounce) = config.debounce {
            fields.push(("debounce", JsonValue::from(debounce)));
        }

        // User-declared order for validation/processing
        if let Some(ref required) = config.required {
            let value = match required {
                RequiredType::Basic(b) => JsonValue::Bool(*b),
                RequiredType::NonEmpty => JsonValue::String("non-empty".to_string()),
            };
            fields.push(("required", value));
        }
        if let Some(ref schema) = config.schema {
            fields.push(("schema", JsonValue::String(schema.clone())));
        }
        if config.condition.is_some() {
            fields.push(("condition", JsonValue::String("function".to_string())));
        }
        if config.selector.is_some() {
            fields.push(("selector", JsonValue::String("function".to_string())));
        }
        if config.transform.is_some() {
            fields.push(("transform", JsonValue::String("function".to_string())));
        }

        // Schedule fields last
        if let Some(delay) = config.delay {
            fields.push(("delay", JsonValue::from(delay)));
        }
        if let Some(interval) = config.interval {
            fields.push(("interval", JsonValue::from(interval)));
        }
        if let Some(ref repeat) = config.repeat {
            let value = match repeat {
                crate::types::RepeatType::Count(n) => JsonValue::from(*n),
                crate::types::RepeatType::Infinite => JsonValue::Bool(true),
            };
            fields.push(("repeat", value));
        }

        // Other fields
        if let Some(ref name) = config.name {
            fields.push(("name", JsonValue::String(name.clone())));
        }
        fields.push(("priority", JsonValue::String(format!("{:?}", config.priority))));

        Self {
            config,
            fields,
            position: 0,
        }
    }
}

impl<'a> Iterator for ConfigFieldIterator<'a> {
    type Item = (&'static str, JsonValue);

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.fields.len() {
            let item = self.fields[self.position].clone();
            self.position += 1;
            Some(item)
        } else {
            None
        }
    }
}

/// Extension trait to add .fields() method to IO
pub trait ConfigFields {
    fn fields(&self) -> ConfigFieldIterator;
}

impl ConfigFields for IO {
    fn fields(&self) -> ConfigFieldIterator {
        ConfigFieldIterator::new(self)
    }
}

//=============================================================================
// SINGLE-PASS VERIFY + COMPILE (FIXED type annotations)
//=============================================================================

/// Single pass: verify field + push operator simultaneously
fn single_pass_compile(config: &IO) -> CompileResult {
    let mut operators: Vec<Operator> = Vec::new();
    let mut all_errors: Vec<String> = Vec::new();
    let mut all_suggestions: Vec<String> = Vec::new();
    let mut processing_pipeline: Vec<String> = Vec::new();

    // Track talent categories
    let mut has_protections = false;
    let mut has_processing = false;
    let mut has_scheduling = false;

    // Single pass verify + compile
    for (field, value) in config.fields() {
        let result = validate_field(field, &value);

        if !result.ok {
            // Collect all errors and suggestions
            if let Some(error) = result.error {
                all_errors.push(format!("Field '{}': {}", field, error));
            }
            if let Some(suggestions) = result.suggestions {
                for suggestion in suggestions {
                    all_suggestions.push(format!("Field '{}': {}", field, suggestion));
                }
            }

            // If blocking error, return immediately
            if result.blocking.unwrap_or(false) {
                return CompileResult::failure(all_errors, all_suggestions);
            }
        } else {
            // Build pipeline as we verify
            if let Some(operator) = create_operator_immediately(field, &value, &result) {
                operators.push(operator);
            }

            // Track talent categories (FIXED type annotation)
            if let Some(ref talent_name) = result.talent_name {
                if PROTECTION_TALENTS.contains(&talent_name.as_str()) {
                    has_protections = true;
                }
                if PROCESSING_TALENTS.contains(&talent_name.as_str()) {
                    has_processing = true;
                    processing_pipeline.push(talent_name.clone());
                }
                if SCHEDULING_TALENTS.contains(&talent_name.as_str()) {
                    has_scheduling = true;
                }
            }
        }
    }

    // If any non-blocking errors occurred, return failure
    if !all_errors.is_empty() {
        return CompileResult::failure(all_errors, all_suggestions);
    }

    // Determine fast path
    let has_fast_path = !has_protections && !has_processing && !has_scheduling;

    CompileResult::success(
        operators,
        has_fast_path,
        has_protections,
        has_processing,
        has_scheduling
    )
}

/// Create operator immediately during verification (FIXED types)
fn create_operator_immediately(
    field: &str,
    value: &JsonValue,
    _result: &DataDefResult
) -> Option<Operator> {
    match field {
        "block" => {
            if value.as_bool() == Some(true) {
                Some(Operator::Block(BlockOperator::new()))
            } else {
                None
            }
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
            if value.as_bool() == Some(true) {
                Some(Operator::Required(RequiredOperator::new()))
            } else if value.as_str() == Some("non-empty") {
                // FIXED: Remove call to non-existent new_non_empty method
                Some(Operator::Required(RequiredOperator::new()))
            } else {
                None
            }
        }
        "schema" => {
            // FIXED: Convert JsonValue to &str
            if let Some(schema_str) = value.as_str() {
                Some(Operator::Schema(SchemaOperator::new(schema_str)))
            } else {
                None
            }
        }
        "condition" => {
            // FIXED: Convert JsonValue to &str
            if let Some(condition_str) = value.as_str() {
                Some(Operator::Condition(ConditionOperator::new(condition_str)))
            } else {
                None
            }
        }
        "selector" => {
            // FIXED: Convert JsonValue to &str
            if let Some(selector_str) = value.as_str() {
                Some(Operator::Selector(SelectorOperator::new(selector_str)))
            } else {
                None
            }
        }
        "transform" => {
            // FIXED: Convert JsonValue to &str
            if let Some(transform_str) = value.as_str() {
                Some(Operator::Transform(TransformOperator::new(transform_str)))
            } else {
                None
            }
        }
        // Scheduling handled by TimeKeeper
        "delay" | "interval" | "repeat" => None,
        // Other fields don't create operators
        _ => None,
    }
}

//=============================================================================
// MAIN API FUNCTION (FIXED sensor calls)
//=============================================================================

/// Main compilation function with proper result flow control
pub fn compile_pipeline(config: &mut IO) -> CompileResult {
    let compile_result = single_pass_compile(config);

    if !compile_result.ok {
        // Log all errors via sensor (FIXED function signature)
        for error in &compile_result.errors {
            sensor::error(
                "pipeline_compiler",
                error,
                Some("compile_pipeline"),
                None // ← FIXED: Added missing metadata parameter
            );
        }

        // Log suggestions as warnings (FIXED function call - sensor::warn only takes 3 params)
        for suggestion in &compile_result.suggestions {
            sensor::warn(
                "pipeline_compiler",
                &format!("Suggestion: {}", suggestion),
                false // ← FIXED: Use boolean instead of Some("location")
            );
        }

        return compile_result;
    }

    // SUCCESS: Update config with compilation metadata
    config._has_fast_path = compile_result.has_fast_path;
    config._has_protections = compile_result.has_protections;
    config._has_processing = compile_result.has_processing;
    config._has_scheduling = compile_result.has_scheduling;

    // Store compiled operators as pipeline metadata
    config._pipeline = compile_result.operators
        .iter()
        .map(|op| {
            match op {
                Operator::Block(_) => "block".to_string(),
                Operator::Throttle(_) => "throttle".to_string(),
                Operator::Debounce(_) => "debounce".to_string(),
                Operator::Required(_) => "required".to_string(),
                Operator::Schema(_) => "schema".to_string(),
                Operator::Transform(_) => "transform".to_string(),
                Operator::Condition(_) => "condition".to_string(),
                Operator::Selector(_) => "selector".to_string(),
                Operator::Schedule(_) => "schedule".to_string(),
            }
        })
        .collect();

    compile_result
}

//=============================================================================
// EXECUTOR FUNCTIONS (FIXED to rebuild operators from config)
//=============================================================================

/// Pipeline execution function - FIXED to rebuild operators from stored config
pub async fn execute_pipeline(
    action: &mut IO,
    payload: ActionPayload
) -> Result<PipelineResult, String> {
    // Fast path check first
    if action._has_fast_path {
        return Ok(PipelineResult::Continue(payload));
    }

    // Rebuild operators from the stored pipeline names
    let mut operators: Vec<Operator> = Vec::new();

    for operator_name in &action._pipeline {
        match operator_name.as_str() {
            "block" => {
                operators.push(Operator::Block(BlockOperator::new()));
            }
            "throttle" => {
                if let Some(throttle_ms) = action.throttle {
                    operators.push(Operator::Throttle(ThrottleOperator::new(throttle_ms)));
                }
            }
            "debounce" => {
                if let Some(debounce_ms) = action.debounce {
                    operators.push(Operator::Debounce(DebounceOperator::new(debounce_ms)));
                }
            }
            "required" => {
                operators.push(Operator::Required(RequiredOperator::new()));
            }
            "schema" => {
                if let Some(ref schema) = action.schema {
                    operators.push(Operator::Schema(SchemaOperator::new(schema)));
                }
            }
            "condition" => {
                if let Some(ref condition) = action.condition {
                    operators.push(Operator::Condition(ConditionOperator::new(condition)));
                }
            }
            "selector" => {
                if let Some(ref selector) = action.selector {
                    operators.push(Operator::Selector(SelectorOperator::new(selector)));
                }
            }
            "transform" => {
                if let Some(ref transform) = action.transform {
                    operators.push(Operator::Transform(TransformOperator::new(transform)));
                }
            }
            "schedule" => {
                // Handle scheduling operators if needed
                // For now, skip as scheduling is handled by TimeKeeper
            }
            _ => {
                // Unknown operator name - log warning but continue
                sensor::warn(
                    "pipeline_executor",
                    &format!("Unknown operator name: {}", operator_name),
                    false
                );
            }
        }
    }

    // Execute operators in sequence
    let mut current_payload = payload;

    for operator in operators {
        match operator.process(current_payload).await {
            OperatorResult::Continue(new_payload) => {
                current_payload = new_payload;
            }
            OperatorResult::Block(error) => {
                return Ok(PipelineResult::Block(error));
            }
            OperatorResult::Defer(_deferred_payload, _delay) => {
                return Ok(PipelineResult::Schedule);
            }
            OperatorResult::Schedule(_scheduled_payload, _schedule) => {
                return Ok(PipelineResult::Schedule);
            }
        }
    }

    Ok(PipelineResult::Continue(current_payload))
}

/// Fast path detection (same API as existing)
pub fn is_fast_path(config: &IO) -> bool {
    !config.block &&
        config.throttle.is_none() &&
        config.debounce.is_none() &&
        !matches!(
            config.required,
            Some(RequiredType::Basic(true)) | Some(RequiredType::NonEmpty)
        ) &&
        config.schema.is_none() &&
        config.condition.is_none() &&
        config.selector.is_none() &&
        config.transform.is_none() &&
        config.delay.is_none() &&
        config.interval.is_none() &&
        config.repeat.is_none()
}

/// Get estimated pipeline length
pub fn estimate_operator_count(config: &IO) -> usize {
    let compile_result = single_pass_compile(config);
    compile_result.operators.len()
}

/// Get estimated performance based on operator count
pub fn estimated_performance(config: &IO) -> u32 {
    let operator_count = estimate_operator_count(config);

    match operator_count {
        0 => 1_800_000, // Fast path
        1..=2 => 1_200_000, // Lightweight
        3..=5 => 800_000, // Standard
        _ => 400_000, // Complex
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
    fn test_successful_compilation() {
        let mut config = IO::new("test-success").with_throttle(1000).with_required(true);

        let result = compile_pipeline(&mut config);

        assert!(result.ok);
        assert_eq!(result.operators.len(), 2);
        assert!(result.errors.is_empty());
        assert!(config._has_protections);
        assert!(config._has_processing);
        assert!(!config._has_fast_path);
    }

    #[test]
    fn test_fast_path_success() {
        let mut config = IO::new("fast-test");

        let result = compile_pipeline(&mut config);

        assert!(result.ok);
        assert!(result.has_fast_path);
        assert_eq!(result.operators.len(), 0);
        assert!(config._has_fast_path);
    }
}
