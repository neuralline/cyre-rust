// src/core/cyre.rs - Enhanced with shutdown, lock, reset, and signal detection
// File location: src/core/cyre.rs

use crate::types::{ ActionPayload, CyreResponse, IO };
use crate::context::state;
use crate::context::{ metrics_state, sensor };
use crate::breathing::is_breathing;
use crate::config::Messages;
use crate::utils::current_timestamp;

// Core imports
use serde_json::{ json, Value as JsonValue };
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;

// Pipeline imports
use crate::schema::{ compile_pipeline, execute_pipeline, PipelineResult };

//=============================================================================
// ENHANCED RESPONSE TEMPLATES
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
                payload: JsonValue::Null,
                message: String::new(),
                error: None,
                timestamp: 0,
                metadata: None,
            },
            error_base: CyreResponse {
                ok: false,
                payload: JsonValue::Null,
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
// ENHANCED CYRE STRUCTURE WITH LIFECYCLE MANAGEMENT VIA METRICS_STATE
//=============================================================================

#[derive(Debug)]
pub struct Cyre {
    templates: ResponseTemplates,
    performance_mode: bool,
}

impl Cyre {
    /// Create a new Cyre instance with lifecycle management
    pub fn new() -> Self {
        sensor::debug("system", Messages::WELCOME, true);
        Self {
            templates: ResponseTemplates::new(),
            performance_mode: false,
        }
    }

    /// Initialize the Cyre system with signal handlers
    pub async fn init(&mut self) -> Result<CyreResponse, String> {
        if metrics_state::is_init() {
            return Ok(
                self.templates.success_with_payload(
                    json!({"already_initialized": true}),
                    Messages::CYRE_ALREADY_INITIALIZED.to_string()
                )
            );
        }

        sensor::sys("system", Messages::SYS, true);
        sensor::debug("system", Messages::CYRE_SYSTEM_INITIALIZING, true);

        // Mark as initialized using metrics_state update function (not direct set)
        if
            let Err(e) = metrics_state::update(|state| {
                state._init = true;
            })
        {
            sensor::error(
                "init",
                &format!("Failed to mark system as initialized: {}", e),
                Some("init"),
                None
            );
            return Err("Failed to complete initialization".to_string());
        }
        // Initialize metrics state first
        if let Err(e) = metrics_state::init() {
            sensor::error("system", &format!("Metrics init failed: {}", e), Some("init"), None);
            return Err(Messages::METRICS_INIT_FAILED.to_string());
        }
        sensor::success("system", Messages::CYRE_INITIALIZED_SUCCESS, true);
        Ok(
            self.templates.success_with_payload(
                json!({
          "initialized": true,
          "timestamp": current_timestamp(),
          "signal_handlers": "not_implemented" // signals::get_signal_handler_description()
        }),
                Messages::CYRE_INITIALIZED_SUCCESS.to_string()
            )
        )
    }

    /// Lock the system - prevents new action and handler registrations (uses metrics_state)
    pub fn lock(&mut self) -> CyreResponse {
        if !metrics_state::is_init() {
            return self.templates.error_with_message(
                Messages::CYRE_NOT_INITIALIZED.to_string(),
                Some("System not initialized".to_string())
            );
        }

        if metrics_state::is_shutdown() {
            return self.templates.error_with_message(
                Messages::CANNOT_LOCK_SHUTDOWN_SYSTEM.to_string(),
                Some("Cannot lock shutdown system".to_string())
            );
        }

        // Lock via metrics_state (central system settings)
        if let Err(e) = metrics_state::lock() {
            sensor::error("system", &format!("Failed to lock system: {}", e), Some("lock"), None);
            return self.templates.error_with_message(
                Messages::SYSTEM_LOCK_FAILED.to_string(),
                Some(e)
            );
        }

        sensor::debug("system", Messages::SYSTEM_LOCKED, true);

        self.templates.success_with_payload(
            json!({
        "locked": true,
        "timestamp": current_timestamp(),
        "message": Messages::SYSTEM_LOCKED
      }),
            Messages::SYSTEM_LOCKED.to_string()
        )
    }

    /// Reset the system - clear all state but keep system running
    pub fn reset(&mut self) -> CyreResponse {
        if !metrics_state::is_init() {
            return self.templates.error_with_message(
                Messages::CYRE_NOT_INITIALIZED.to_string(),
                Some("System not initialized".to_string())
            );
        }

        sensor::info("system", Messages::SYSTEM_RESET_INITIATED, true);

        let mut reset_operations = Vec::new();
        let mut failed_operations = Vec::new();

        // Clear IO actions
        state::io::clear();

        // Clear subscribers
        state::subscribers::clear();

        // Clear timeline
        state::timeline::clear();

        // Reset metrics state but keep system running
        metrics_state::reset();
        if let Err(e) = metrics_state::init() {
            failed_operations.push(format!("metrics_reinit: {}", e));
            sensor::error(
                "reset",
                &format!("Failed to reinitialize metrics: {}", e),
                Some("reset"),
                None
            );
        } else {
            reset_operations.push("metrics");
        }

        // TODO: Clear orchestrations when available
        // TODO: Clear query cache when available
        // TODO: Clear group operations when available

        // Keep system initialized using update function
        if
            let Err(e) = metrics_state::update(|state| {
                state._init = true;
            })
        {
            sensor::error(
                "reset",
                &format!("Failed to mark system as initialized: {}", e),
                Some("reset"),
                None
            );
        }

        let response_payload =
            json!({
      "reset": true,
      "timestamp": current_timestamp(),
      "cleared_components": reset_operations,
      "failed_operations": failed_operations,
      "system_status": {
        "initialized": metrics_state::is_init(),
        "locked": metrics_state::is_locked(),
        "shutdown": metrics_state::is_shutdown()
      }
    });

        if failed_operations.is_empty() {
            sensor::success("system", Messages::SYSTEM_RESET_COMPLETED, true);
            self.templates.success_with_payload(
                response_payload,
                Messages::SYSTEM_RESET_COMPLETED.to_string()
            )
        } else {
            sensor::warn(
                "system",
                &format!("Reset completed with {} failed operations", failed_operations.len()),
                false
            );
            self.templates.success_with_payload(
                response_payload,
                Messages::SYSTEM_RESET_PARTIAL.to_string()
            )
        }
    }

    /// Shutdown the system completely - graceful cleanup and exit
    pub fn shutdown(&mut self) -> CyreResponse {
        if !metrics_state::is_init() {
            return self.templates.error_with_message(
                Messages::CYRE_NOT_INITIALIZED.to_string(),
                Some("System not initialized".to_string())
            );
        }

        sensor::critical("system", Messages::SYSTEM_SHUTDOWN_INITIATED, Some("cyre_init"), None);

        // Reset all state (same as reset but more thorough)
        state::io::clear();
        state::subscribers::clear();
        state::timeline::clear();

        let _ = metrics_state::shutdown();

        let response_payload =
            json!({
      "shutdown": true,
      "timestamp": current_timestamp(),
      "system_status": {
        "initialized": false,
        "locked": metrics_state::is_locked(),
        "shutdown": metrics_state::is_shutdown()
      }
    });

        sensor::sys("system", "System offline!", true);
        self.templates.success_with_payload(
            response_payload,
            Messages::SYSTEM_SHUTDOWN_COMPLETED.to_string()
        )
    }

    /// Get system status (uses metrics_state for central status)
    pub fn status(&self) -> CyreResponse {
        let metrics_status = metrics_state::status();

        self.templates.success_with_payload(
            json!({
        "timestamp": current_timestamp(),
        "cyre": {
          "initialized": metrics_state::is_init(),
          "performance_mode": self.performance_mode
        },
        "metrics": {
          "initialized": metrics_status._init,
          "locked": metrics_status._locked,
          "shutdown": metrics_status._shutdown,
          "hibernating": metrics_status.hibernating,
          "in_recuperation": metrics_status.in_recuperation
        },
        "system": {
          "breathing": is_breathing(),
          "signal_handlers_active": false, // signals::supports_system_signals(),
          "shutdown_monitor_active": "selfis_some(),",
          "platform_signals": "not_implemented" // signals::get_signal_handler_description()
        },
        "stores": {
          "actions": state::io::size(),
          "subscribers": state::subscribers::size(),
          "timeline": state::timeline::size()
        }
      }),
            "System status retrieved".to_string()
        )
    }

    /// Register new action (with lock check via metrics_state)
    pub fn action(&mut self, config: IO) -> Result<(), String> {
        if !metrics_state::is_init() {
            return Err(Messages::CYRE_NOT_INITIALIZED.to_string());
        }

        if metrics_state::is_locked() {
            sensor::warn(
                "action",
                &format!("Action registration blocked - system is locked: {}", config.id),
                false
            );
            return Err(Messages::REGISTRATION_BLOCKED_LOCKED.to_string());
        }

        if metrics_state::is_shutdown() {
            return Err("System is shutdown - no new registrations allowed".to_string());
        }

        // Check if action already exists
        if state::io::get(&config.id).is_some() {
            return Err(format!("Action '{}' already exists", config.id));
        }

        // Compile pipeline for this action
        let mut compiled_config = config.clone();
        let compile_result = compile_pipeline(&mut compiled_config);

        if !compile_result.ok {
            return Err(format!("Pipeline compilation failed: {:?}", compile_result.errors));
        }

        // Store compiled action
        state::io::set(config.id.clone(), compiled_config)?;

        // sensor::debug("action", &format!("Action '{}' registered successfully", config.id), false);
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
        if !metrics_state::is_init() {
            return Err(Messages::CYRE_NOT_INITIALIZED.to_string());
        }

        if metrics_state::is_locked() {
            sensor::warn(
                "handler",
                &format!("Handler registration blocked - system is locked: {}", action_id),
                false
            );
            return Err(Messages::REGISTRATION_BLOCKED_LOCKED.to_string());
        }

        if metrics_state::is_shutdown() {
            return Err("System is shutdown - no new registrations allowed".to_string());
        }

        // Check if action exists
        if state::io::get(action_id).is_none() {
            return Err(
                format!("Action '{}' not found. Register action first with cyre.action()", action_id)
            );
        }

        let handler_arc: crate::types::AsyncHandler = Arc::new(move |payload| handler(payload));
        let subscriber = state::ISubscriber::new(action_id.to_string(), handler_arc);
        state::subscribers::set(action_id.to_string(), subscriber)?;

        //sensor::debug("handler", &format!("Handler registered for action '{}'", action_id), false);
        Ok(())
    }

    /// Call action (with shutdown check via metrics_state)
    #[inline]
    pub async fn call(&self, action_id: &str, payload: ActionPayload) -> CyreResponse {
        if !metrics_state::is_init() {
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
                sensor::error(
                    "cyre",
                    &format!("Failed to update action state: {}", e),
                    Some("call"),
                    None
                );
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

    /// Get action state
    pub fn get(action_id: &str) -> Option<IO> {
        state::io::get(action_id)
    }

    /// Remove action and associated handlers
    pub fn forget(action_id: &str) -> Result<bool, String> {
        if !metrics_state::is_init() {
            return Err(Messages::CYRE_NOT_INITIALIZED.to_string());
        }

        let action_removed = state::io::forget(action_id);
        let handler_removed = state::subscribers::forget(action_id);

        let success = action_removed || handler_removed;

        if success {
            sensor::info("cyre", &format!("Action '{}' and its handler removed", action_id), false);
        }

        Ok(success)
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> JsonValue {
        let health_summary = metrics_state::get_summary();
        match serde_json::to_value(health_summary) {
            Ok(mut json_value) => {
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
    }
}
