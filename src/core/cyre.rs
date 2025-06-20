// src/core/cyre.rs - FIXED: All import and compilation errors

use crate::types::{ ActionPayload, CyreResponse, IO };
use crate::context::state;
use crate::context::{ metrics_state, sensor };
use crate::breathing::is_breathing; // Removed unused start_breathing
use crate::config::Messages;
use crate::utils::current_timestamp;

// FIXED: Add all missing imports
use serde_json::{ json, Value as JsonValue };
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;

// CORRECTED imports - use schema module instead of pipeline
use crate::schema::{
  compile_pipeline, // Returns CompileResult
  execute_pipeline,
  PipelineResult,
};

//=============================================================================
// RESPONSE TEMPLATES (Same as your optimized version)
//=============================================================================

#[derive(Debug, Clone)]
struct ResponseTemplates {
  success_base: CyreResponse,
  error_base: CyreResponse,
}

impl ResponseTemplates {
  fn new() -> Self {
    Self {
      success_base: CyreResponse {
        ok: true,
        payload: JsonValue::Null, // FIXED: Add JsonValue::
        message: String::new(),
        error: None,
        timestamp: 0,
        metadata: None,
      },
      error_base: CyreResponse {
        ok: false,
        payload: JsonValue::Null, // FIXED: Add JsonValue::
        message: String::new(),
        error: None,
        timestamp: 0,
        metadata: None,
      },
    }
  }

  #[inline(always)]
  fn success_with_payload(&self, payload: ActionPayload, message: String) -> CyreResponse {
    CyreResponse {
      payload,
      message,
      timestamp: current_timestamp(),
      ..self.success_base.clone()
    }
  }

  #[inline(always)]
  fn error_with_message(&self, message: String, error: Option<String>) -> CyreResponse {
    CyreResponse {
      message,
      error,
      timestamp: current_timestamp(),
      ..self.error_base.clone()
    }
  }
}

//=============================================================================
// CORE CYRE STRUCTURE (Same as your optimized version)
//=============================================================================

#[derive(Debug)]
pub struct Cyre {
  is_initialized: bool,
  templates: ResponseTemplates,
  performance_mode: bool,
}

impl Cyre {
  /// Create a new Cyre instance
  pub fn new() -> Self {
    sensor::debug("system", Messages::WELCOME, true);
    Self {
      is_initialized: false,
      templates: ResponseTemplates::new(),
      performance_mode: false,
    }
  }

  /// Initialize the Cyre system
  pub async fn init(&mut self) -> Result<CyreResponse, String> {
    if self.is_initialized {
      return Ok(
        self.templates.success_with_payload(
          json!({"already_initialized": true}), // FIXED: Add json! import
          "Cyre already initialized".to_string()
        )
      );
    }

    sensor::sys("system", Messages::SYS.to_string(), true);

    self.is_initialized = true;

    Ok(
      self.templates.success_with_payload(
        json!({ // FIXED: Add json! import
                "initialized": true,
                "breathing": !self.performance_mode && is_breathing(),
                "timestamp": current_timestamp()
            }),
        Messages::CYRE_INITIALIZED_SUCCESS.to_string()
      )
    )
  }

  /// Check if Cyre is initialized
  #[inline(always)]
  pub fn is_initialized(&self) -> bool {
    self.is_initialized
  }

  /// Register a new action with CORRECTED result flow control
  pub fn action(&mut self, mut config: IO) -> Result<(), String> {
    if !self.is_initialized {
      return Err(Messages::CYRE_NOT_INITIALIZED.to_string());
    }

    let action_id = config.id.clone();

    // Check if action already exists
    if state::io::get(&action_id).is_some() {
      return Err(format!("Action '{}' already exists", action_id));
    }

    // 1. EXTRACT PAYLOAD FIRST (before compilation)
    let default_payload = config.payload.take(); // Remove from config

    // 2. COMPILE PIPELINE (modifies config._pipeline)
    let compile_result = compile_pipeline(&mut config);

    if !compile_result.ok {
      // COMPILATION FAILED - errors already logged via sensor
      return Err(
        format!("Failed to compile action '{}': {}", action_id, compile_result.errors.join("; "))
      );
    }

    // 3. STORE CONFIG (without payload) in io store
    state::io::set(action_id.clone(), config)?;

    // 4. STORE PAYLOAD (if provided) in payload store
    if let Some(payload) = default_payload {
      state::payload::set(action_id.clone(), payload)?;
    }

    Ok(())
  }
  /// Register handler (SAME as your optimized version)
  pub fn on<F>(&mut self, action_id: &str, handler: F) -> Result<(), String>
    where
      F: Fn(ActionPayload) -> Pin<Box<dyn Future<Output = CyreResponse> + Send>> + // FIXED: Add Pin import
        Send +
        Sync +
        'static
  {
    if !self.is_initialized {
      return Err(Messages::CYRE_NOT_INITIALIZED.to_string());
    }

    if state::io::get(action_id).is_none() {
      return Err(
        format!("Action '{}' not found. Register action first with cyre.action()", action_id)
      );
    }

    let handler_arc: crate::types::AsyncHandler = Arc::new(move |payload| handler(payload)); // FIXED: Add Arc import

    let subscriber = state::ISubscriber::new(action_id.to_string(), handler_arc);
    state::subscribers::set(action_id.to_string(), subscriber)?;

    Ok(())
  }

  /// Call method (SAME as your optimized version)
  #[inline]
  pub async fn call(&self, action_id: &str, payload: ActionPayload) -> CyreResponse {
    if !self.is_initialized {
      return self.templates.error_with_message(
        Messages::CYRE_NOT_INITIALIZED.to_string(),
        Some("System not initialized".to_string())
      );
    }

    // Get mutable action from central state
    let mut action = match state::io::get(action_id) {
      Some(action) => action,
      None => {
        return self.templates.error_with_message(
          format!("Action '{}' not found", action_id),
          Some("Action not found".to_string())
        );
      }
    };

    // Execute pipeline
    let processed_payload = if action._has_fast_path {
      payload
    } else {
      match execute_pipeline(&mut action, payload).await {
        Ok(PipelineResult::Continue(processed)) => processed,
        Ok(PipelineResult::Block(reason)) => {
          return CyreResponse {
            ok: false,
            payload: JsonValue::Null,
            message: reason,
            error: Some("blocked".to_string()),
            timestamp: current_timestamp(),
            metadata: None,
          };
        }
        Ok(PipelineResult::Schedule) => {
          return self.templates.success_with_payload(
            json!({"scheduled": true}),
            "Action scheduled for execution".to_string()
          );
        }
        Err(error) => {
          return self.templates.error_with_message(error.clone(), Some(error));
        }
      }
    };

    // Get handler
    let handler = match state::subscribers::get(action_id) {
      Some(subscriber) => subscriber,
      None => {
        return self.templates.error_with_message(
          format!("No handler registered for action '{}'", action_id),
          Some("Handler not found".to_string())
        );
      }
    };

    // EXECUTE HANDLER - Simple, no panic catching
    let response = (handler.handler)(processed_payload).await;
    let now = current_timestamp();

    // UPDATE IO STATE AFTER EXECUTION
    if response.ok {
      // Success: Update _last_exec_time and other fields
      let updated_action = IO {
        _last_exec_time: Some(now),
        timestamp: Some(now),
        _execution_count: action._execution_count + 1,
        ..action
      };

      if let Err(e) = state::io::set(action_id.to_string(), updated_action) {
        sensor::error("cyre", &format!("Failed to update action state: {}", e), Some("call"), None);
      }
    } else {
      // Error: DON'T update _last_exec_time (preserves throttle behavior)
      let updated_action = IO {
        timestamp: Some(now),
        _execution_count: action._execution_count + 1,
        ..action
      };

      if let Err(e) = state::io::set(action_id.to_string(), updated_action) {
        sensor::error(
          "cyre",
          &format!("Failed to update action state after error: {}", e),
          Some("call"),
          None
        );
      }
    }

    // CHECK FOR INTRALINK - Simple detection
    if let Some(id_value) = response.payload.get("id") {
      if let Some(intra_id) = id_value.as_str() {
        if !intra_id.is_empty() {
          // Get intralink payload or use empty object
          let intra_payload = response.payload
            .get("payload")
            .cloned()
            .unwrap_or(json!({}));

          // Simple call - no complications
          return Box::pin(self.call(intra_id, intra_payload)).await;
        }
      }
    }

    // Return original response if no intralink
    response
  }

  /// Get action configuration
  pub fn get(&self, action_id: &str) -> Option<IO> {
    if !self.is_initialized {
      return None;
    }
    state::io::get(action_id)
  }

  /// Remove an action and its handler
  pub fn forget(&mut self, action_id: &str) -> Result<bool, String> {
    if !self.is_initialized {
      return Err(Messages::CYRE_NOT_INITIALIZED.to_string());
    }

    let action_removed = state::io::forget(action_id);
    let handler_removed = state::subscribers::forget(action_id);

    let success = action_removed || handler_removed;

    if success && !self.performance_mode {
      sensor::info("cyre", &format!("Action '{}' and its handler removed", action_id), false);
    }

    Ok(success)
  }

  /// Get system status
  pub fn status(&self) -> CyreResponse {
    let metrics = if self.is_initialized {
      json!({
                "initialized": true,
                "breathing": !self.performance_mode && is_breathing(),
                "performance_mode": self.performance_mode,
                "stores": {
                    "actions": state::io::size(),
                    "handlers": state::subscribers::size(),
                    "timeline": state::timeline::size()
                },
                "timestamp": current_timestamp()
            })
    } else {
      json!({
                "initialized": false,
                "error": "System not initialized"
            })
    };

    self.templates.success_with_payload(metrics, "System status".to_string())
  }

  /// Get performance metrics
  pub fn get_performance_metrics(&self) -> JsonValue {
    // FIXED: Add JsonValue import
    if !self.performance_mode {
      let health_summary = metrics_state::get_summary();
      match serde_json::to_value(health_summary) {
        Ok(mut json_value) => {
          // Add additional metrics
          if let Some(obj) = json_value.as_object_mut() {
            obj.insert("active_channels".to_string(), json!(state::io::size()));
            obj.insert(
              "executions".to_string(),
              json!({
                                "total_executions": 0,
                                "fast_path_hits": 0,
                                "fast_path_ratio": 0.0
                            })
            );
          }
          json_value
        }
        Err(_) =>
          json!({
                    "error": "Failed to serialize health summary",
                    "performance_mode": false
                }),
      }
    } else {
      json!({
                "performance_mode": true,
                "metrics_disabled": "Enable normal mode for detailed metrics"
            })
    }
  }
}
