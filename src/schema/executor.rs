// src/schema/executor.rs - FIXED executor with action_id parameter
use crate::types::{ IO, ActionPayload, CyreResponse };
use crate::schema::operators::OperatorResult;
use crate::utils::current_timestamp;
use serde_json::{ Value as JsonValue, json };

/// FIXED: Main execution function that passes action_id to operators
pub async fn execute_pipeline(
  action: &IO,
  payload: ActionPayload
) -> Result<PipelineResult, String> {
  // Fast path check
  if action._has_fast_path {
    return Ok(PipelineResult::Continue(payload));
  }

  let mut current_payload = payload;
  let action_id = &action.id; // Get action ID for operators

  // FIXED: Execute operators with action_id parameter
  for operator in &action._pipeline {
    match operator.process(action_id, current_payload).await {
      OperatorResult::Continue(payload) => {
        current_payload = payload;
      }
      OperatorResult::Block(reason) => {
        return Ok(PipelineResult::Block(reason));
      }
      OperatorResult::Schedule(_, _) => {
        return Ok(PipelineResult::Schedule);
      }
      OperatorResult::Defer(payload, duration) => {
        // For debounce: wait for the specified duration then continue
        tokio::time::sleep(duration).await;
        current_payload = payload;
      }
    }
  }

  Ok(PipelineResult::Continue(current_payload))
}

#[derive(Debug)]
pub enum PipelineResult {
  Continue(ActionPayload),
  Block(String),
  Schedule,
}

impl PipelineResult {
  pub fn into_response(self) -> CyreResponse {
    match self {
      PipelineResult::Continue(payload) => CyreResponse::success(payload, "Pipeline completed"),
      PipelineResult::Block(reason) =>
        CyreResponse {
          ok: false,
          payload: JsonValue::Null,
          message: "Pipeline blocked".to_string(),
          error: Some(reason),
          timestamp: current_timestamp(),
          metadata: None,
        },
      PipelineResult::Schedule =>
        CyreResponse::success(json!({"scheduled": true}), "Action scheduled for execution"),
    }
  }
}

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

/// Check if action requires pipeline execution
pub fn requires_pipeline_execution(action: &IO) -> bool {
  !action._has_fast_path && !action._pipeline.is_empty()
}
