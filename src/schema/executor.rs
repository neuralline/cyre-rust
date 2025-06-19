// src/schema/executor.rs
// Pipeline execution orchestration - coordinates operator flow
// File location: src/schema/executor.rs

use crate::schema::Operator;
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
  let operators = build_operators_from_pipeline(action);

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
fn build_operators_from_pipeline(action: &IO) -> Vec<Operator> {
  action._pipeline
    .iter()
    .filter_map(|name| Operator::from_name(name, action))
    .collect()
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
