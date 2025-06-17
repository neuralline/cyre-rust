// src/core/cyre.rs
// Debug version - explicitly show which functions we're calling

use crate::types::{ ActionPayload, CyreResponse, IO };
use crate::context::state; // Import the module, not specific functions
use crate::context::sensor;
use crate::utils::current_timestamp;
use std::sync::Arc;
use serde_json::json;

pub struct Cyre;

impl Cyre {
    pub fn new() -> Self {
        sensor::info("system", "Creating new Cyre instance", true);
        Self
    }

    /// Initialize timekeeper integration (placeholder)
    pub async fn init_timekeeper(&self) -> Result<(), String> {
        sensor::info("cyre", "Initializing timekeeper integration", true);
        // TimeKeeper initialization would go here
        Ok(())
    }

    pub fn action(&mut self, config: IO) -> Result<(), String> {
        let action_id = config.id.clone();
        sensor::debug("cyre", &format!("Creating action: {}", action_id), false);

        // Explicitly use the wrapper function
        match state::io::set(config) {
            Ok(_) => {
                sensor::success(
                    "cyre",
                    &format!("Action '{}' created successfully", action_id),
                    false
                );
                Ok(())
            }
            Err(e) => {
                sensor::error(
                    "cyre",
                    &format!("Failed to create action '{}': {}", action_id, e),
                    Some("Cyre::action"),
                    None
                );
                Err(e)
            }
        }
    }

    pub fn on<F>(&mut self, action_id: &str, handler: F) -> Result<(), String>
        where
            F: Fn(
                ActionPayload
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = CyreResponse> + Send>> +
                Send +
                Sync +
                'static
    {
        // Explicitly use the wrapper function
        if state::io::get(action_id).is_none() {
            let error =
                format!("Action '{}' does not exist. Create it first with .action()", action_id);
            sensor::error("cyre", &error, Some("Cyre::on"), None);
            return Err(error);
        }

        let async_handler: crate::types::AsyncHandler = Arc::new(handler);

        let subscriber = state::ISubscriber {
            id: action_id.to_string(),
            handler: async_handler,
            created_at: current_timestamp(),
        };

        // Explicitly use the wrapper function
        match state::subscribers::add(subscriber) {
            Ok(_) => {
                sensor::success("cyre", &format!("Handler registered for '{}'", action_id), false);
                Ok(())
            }
            Err(e) => {
                sensor::error(
                    "cyre",
                    &format!("Failed to register handler for '{}': {}", action_id, e),
                    Some("Cyre::on"),
                    None
                );
                Err(e)
            }
        }
    }

    pub async fn call(&self, action_id: &str, payload: ActionPayload) -> CyreResponse {
        let start_time = std::time::Instant::now();
        //sensor::debug("cyre", &format!("Calling action: {}", action_id), false);

        // Explicitly use the wrapper function
        let config = match state::io::get(action_id) {
            Some(config) => config,
            None => {
                let error = format!("Action '{}' not found", action_id);
                sensor::error("cyre", &error, Some("Cyre::call"), None);
                return CyreResponse {
                    ok: false,
                    payload: json!(null),
                    message: error,
                    error: Some("action_not_found".to_string()),
                    timestamp: current_timestamp(),
                    metadata: None,
                };
            }
        };

        // Explicitly use the wrapper function
        let subscriber = match state::subscribers::get(action_id) {
            Some(subscriber) => subscriber,
            None => {
                let error = format!("No handler registered for action '{}'", action_id);
                sensor::error("cyre", &error, Some("Cyre::call"), None);
                return CyreResponse {
                    ok: false,
                    payload: json!(null),
                    message: error,
                    error: Some("handler_not_found".to_string()),
                    timestamp: current_timestamp(),
                    metadata: None,
                };
            }
        };

        // Execute handler
        //sensor::debug("cyre", &format!("Executing handler for '{}'", action_id), false);
        let result = (subscriber.handler)(payload).await;
        let execution_time = start_time.elapsed();

        // Update metrics
        state::MetricsOps::update(state::MetricsUpdate {
            total_executions: Some(state::MetricsOps::get().total_executions + 1),
            fast_path_hits: if config.is_fast_path_eligible() {
                Some(state::MetricsOps::get().fast_path_hits + 1)
            } else {
                None
            },
            ..Default::default()
        });

        if result.ok {
            // sensor::success(
            //     "cyre",
            //     &format!("Action '{}' completed in {:.2}ms", action_id, execution_time.as_millis()),
            //     false
            // );
        } else {
            sensor::error(
                "cyre",
                &format!("Action '{}' failed: {}", action_id, result.message),
                Some("Cyre::call"),
                None
            );
        }

        result
    }

    pub fn get(&self, action_id: &str) -> Option<IO> {
        state::io::get(action_id)
    }

    pub fn forget(&mut self, action_id: &str) -> bool {
        state::subscribers::forget(action_id);
        let removed = state::io::forget(action_id);

        if removed {
            sensor::info("cyre", &format!("Action '{}' forgotten", action_id), false);
        } else {
            sensor::warn(
                "cyre",
                &format!("Attempted to forget non-existent action '{}'", action_id),
                false
            );
        }

        removed
    }

    pub fn get_performance_metrics(&self) -> serde_json::Value {
        let metrics = state::MetricsOps::get();
        let io_count = state::stores::Stores::io_size();
        let handler_count = state::stores::Stores::subscribers_size();

        json!({
            "system": {
                "total_actions": io_count,
                "total_handlers": handler_count,
                "uptime": current_timestamp()
            },
            "executions": {
                "total_executions": metrics.total_executions,
                "fast_path_hits": metrics.fast_path_hits,
                "fast_path_ratio": if metrics.total_executions > 0 {
                    (metrics.fast_path_hits as f64 / metrics.total_executions as f64) * 100.0
                } else {
                    0.0
                },
                "pipeline_hits": metrics.total_executions - metrics.fast_path_hits
            },
            "protection": {
                "total_blocks": metrics.protection_blocks
            },
            "active_channels": io_count,
            "scheduled_actions": metrics.active_formations
        })
    }

    // Static methods
    pub fn action_exists(action_id: &str) -> bool {
        state::io::get(action_id).is_some()
    }

    pub fn handler_exists(action_id: &str) -> bool {
        state::subscribers::get(action_id).is_some()
    }
}
