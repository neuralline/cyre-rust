// src/schema/executor.rs
// Pipeline execution orchestration - coordinates operator flow
// File location: src/schema/executor.rs

use crate::types::{ IO, ActionPayload, CyreResponse };
use crate::context::sensor;
use crate::utils::current_timestamp;
use serde_json::{ Value as JsonValue, json };
//=============================================================================
// PIPELINE EXECUTION RESULT TYPES
//=============================================================================

#[derive(Debug)]
pub enum PipelineResult {
  Continue(ActionPayload), // Pipeline completed successfully
  Block(String), // Pipeline blocked execution
  Schedule, // Pipeline scheduled for future execution
}

impl PipelineResult {
  pub fn into_response(self) -> CyreResponse {
    match self {
      PipelineResult::Continue(payload) => { CyreResponse::success(payload, "Pipeline completed") }
      PipelineResult::Block(reason) => {
        CyreResponse {
          ok: false,
          payload: JsonValue::Null,
          message: "Pipeline blocked".to_string(),
          error: Some(reason),
          timestamp: current_timestamp(),
          metadata: None,
        }
      }
      PipelineResult::Schedule => {
        CyreResponse::success(json!({"scheduled": true}), "Action scheduled for execution")
      }
    }
  }
}

//=============================================================================
// MAIN EXECUTION API - PIPELINE ORCHESTRATION ONLY
//=============================================================================

/// Main execution function - orchestrates payload through pipeline operators
/// NOTE: This function is NEVER called for fast path actions!
/// RESPONSIBILITY: Pipeline flow control only - delegates to operators.rs for actual execution
pub async fn execute_pipeline(
  action: &IO,
  payload: ActionPayload
) -> Result<PipelineResult, String> {
  // SAFETY CHECK: This should never be called for fast path actions
  if action._has_fast_path {
    sensor::warn(
      "pipeline_executor",
      "execute_pipeline called for fast path action - this should not happen",
      true
    );
    return Ok(PipelineResult::Continue(payload));
  }

  // Create pipeline from compiled operator names
  let operators = build_operators_from_pipeline(action)?;

  // Execute pipeline: process each operator in sequence
  let mut current_payload = payload;
  for operator in operators {
    match operator.process(current_payload).await {
      crate::schema::operators::OperatorResult::Continue(next_payload) => {
        current_payload = next_payload;
      }
      crate::schema::operators::OperatorResult::Block(reason) => {
        sensor::debug("executor", &format!("Pipeline execution failed: {}", reason), false);
        return Ok(PipelineResult::Block(reason));
      }
      crate::schema::operators::OperatorResult::Schedule(next_payload, _schedule_config) => {
        // Optionally handle schedule config if needed
        return Ok(PipelineResult::Schedule);
      }
      crate::schema::operators::OperatorResult::Defer(next_payload, _duration) => {
        // Optionally handle defer if needed
        current_payload = next_payload;
      }
    }
  }
  sensor::debug("executor", "Pipeline execution completed successfully", false);
  Ok(PipelineResult::Continue(current_payload))
}

//=============================================================================
// OPERATOR BUILDING FROM PIPELINE STRINGS - DELEGATES TO OPERATORS.RS
//=============================================================================

/// Build operator instances from compiled pipeline names
/// RESPONSIBILITY: Only operator instantiation - execution logic is in operators.rs
fn build_operators_from_pipeline(
  action: &IO
) -> Result<Vec<crate::schema::operators::Operator>, String> {
  use crate::schema::operators::*;
  use crate::types::RequiredType;

  let mut operators = Vec::new();

  for operator_name in &action._pipeline {
    let operator = match operator_name.as_str() {
      "block" => { Operator::Block(BlockOperator::new()) }
      "throttle" => {
        if let Some(ms) = action.throttle {
          Operator::Throttle(ThrottleOperator::new(ms))
        } else {
          return Err("Throttle operator without throttle configuration".to_string());
        }
      }
      "debounce" => {
        if let Some(ms) = action.debounce {
          Operator::Debounce(DebounceOperator::new(ms))
        } else {
          return Err("Debounce operator without debounce configuration".to_string());
        }
      }
      "required" => {
        // Convert RequiredType to boolean for RequiredOperator
        match &action.required {
          Some(RequiredType::Basic(true)) | Some(RequiredType::NonEmpty) => {
            Operator::Required(RequiredOperator::new())
          }
          _ => {
            return Err("Required operator without valid required configuration".to_string());
          }
        }
      }
      "schema" => {
        if let Some(ref schema_name) = action.schema {
          Operator::Schema(SchemaOperator::new(schema_name))
        } else {
          return Err("Schema operator without schema configuration".to_string());
        }
      }
      "condition" => {
        if let Some(ref condition_name) = action.condition {
          Operator::Condition(ConditionOperator::new(condition_name))
        } else {
          return Err("Condition operator without condition configuration".to_string());
        }
      }
      "selector" => {
        if let Some(ref selector_name) = action.selector {
          Operator::Selector(SelectorOperator::new(selector_name))
        } else {
          return Err("Selector operator without selector configuration".to_string());
        }
      }
      "transform" => {
        if let Some(ref transform_name) = action.transform {
          Operator::Transform(TransformOperator::new(transform_name))
        } else {
          return Err("Transform operator without transform configuration".to_string());
        }
      }
      "detect_changes" => {
        if action.detect_changes {
          Operator::DetectChanges(DetectChangesOperator::new())
        } else {
          continue; // Skip if not enabled
        }
      }
      "schedule" => {
        // Extract scheduling configuration for ScheduleOperator
        let delay = action.delay;
        let interval = action.interval;
        let repeat = match &action.repeat {
          Some(val) => {
            if val.is_number() {
              val.as_u64().map(|n| n as u32)
            } else if val.is_boolean() {
              if val.as_bool().unwrap() {
                Some(0u32) // true = infinite
              } else {
                Some(1u32) // false = once
              }
            } else {
              None
            }
          }
          None => None,
        };

        Operator::Schedule(ScheduleOperator::new(delay, interval, repeat))
      }
      _ => {
        return Err(format!("Unknown operator in pipeline: {}", operator_name));
      }
    };

    operators.push(operator);
  }

  Ok(operators)
}

//=============================================================================
// CONVENIENCE FUNCTIONS FOR EXTERNAL INTEGRATION
//=============================================================================

/// Execute pipeline and convert result to CyreResponse
pub async fn execute_pipeline_to_response(action: &IO, payload: ActionPayload) -> CyreResponse {
  match execute_pipeline(action, payload).await {
    Ok(result) => result.into_response(),
    Err(error) =>
      CyreResponse {
        ok: false,
        payload: JsonValue::Null,
        message: "Pipeline execution error".to_string(),
        error: Some(error),
        timestamp: current_timestamp(),
        metadata: None,
      },
  }
}

/// Check if action requires pipeline execution (not fast path)
pub fn requires_pipeline_execution(action: &IO) -> bool {
  !action._has_fast_path && !action._pipeline.is_empty()
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use crate::types::Priority;
  use serde_json::json;

  #[tokio::test]
  async fn test_fast_path_warning() {
    let mut config = IO::new("fast-test");
    config._has_fast_path = true; // Force fast path

    let payload = json!({"test": "data"});
    let result = execute_pipeline(&config, payload.clone()).await;

    // Should succeed but log warning
    assert!(result.is_ok());
    match result.unwrap() {
      PipelineResult::Continue(returned_payload) => {
        assert_eq!(returned_payload, payload);
      }
      _ => panic!("Expected Continue result"),
    }
  }

  #[tokio::test]
  async fn test_empty_pipeline() {
    let config = IO::new("empty-test");
    // Empty pipeline should create empty operator list

    let payload = json!({"test": "data"});
    let result = execute_pipeline(&config, payload.clone()).await;

    assert!(result.is_ok());
    match result.unwrap() {
      PipelineResult::Continue(returned_payload) => {
        assert_eq!(returned_payload, payload);
      }
      _ => panic!("Expected Continue result"),
    }
  }

  #[test]
  fn test_requires_pipeline_execution() {
    let mut fast_path_config = IO::new("fast");
    fast_path_config._has_fast_path = true;
    assert!(!requires_pipeline_execution(&fast_path_config));

    let mut pipeline_config = IO::new("pipeline");
    pipeline_config._has_fast_path = false;
    pipeline_config._pipeline = vec!["required".to_string()];
    assert!(requires_pipeline_execution(&pipeline_config));

    let empty_config = IO::new("empty");
    assert!(!requires_pipeline_execution(&empty_config));
  }

  #[test]
  fn test_build_operators_from_pipeline() {
    let mut config = IO::new("test").with_required(true).with_throttle(1000);

    // Manually set pipeline as if compiled
    config._pipeline = vec!["throttle".to_string(), "required".to_string()];

    let operators = build_operators_from_pipeline(&config);
    assert!(operators.is_ok());

    let ops = operators.unwrap();
    assert_eq!(ops.len(), 2);

    // Verify operator types by their names
    assert_eq!(ops[0].name(), "throttle");
    assert_eq!(ops[1].name(), "required");
  }
}
