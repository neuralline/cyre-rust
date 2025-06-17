// src/core/cyre.rs
// Debug version - explicitly show which functions we're calling

use crate::types::{ ActionPayload, CyreResponse, IO };
use crate::context::state; // Import the module, not specific functions
use crate::context::sensor;
use crate::utils::current_timestamp;
use std::sync::Arc;
use serde_json::json;
use crate::context::state::{ io, PayloadStateOps };

#[derive(Debug)]
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

    /// Create new channel with compiled pipeline operators
    pub fn action(&mut self, mut config: IO) -> Result<(), String> {
        let action_id = config.id.clone();
        sensor::debug("cyre", &format!("Creating channel: {}", action_id), false);

        // Validate configuration
        if action_id.trim().is_empty() {
            sensor::error("cyre", "Channel ID cannot be empty", Some("Cyre::action"), None);
            return Err("Channel ID cannot be empty".to_string());
        }

        // Check if channel already exists
        if state::io::get(&action_id).is_some() {
            sensor::error(
                "cyre",
                &format!("Channel '{}' already exists", action_id),
                Some("Cyre::action"),
                None
            );
            return Err(format!("Channel '{}' already exists", action_id));
        }

        // COMPILE PIPELINE AND POPULATE _pipeline FIELD
        match crate::pipeline::compile_pipeline(&action_id, &mut config) {
            Ok(_) => {
                sensor::debug(
                    "cyre",
                    &format!(
                        "Compiled {} operators for '{}': {:?}",
                        config._pipeline.len(),
                        action_id,
                        config._pipeline
                    ),
                    false
                );
            }
            Err(e) => {
                sensor::error(
                    "cyre",
                    &format!("Pipeline compilation failed for '{}': {}", action_id, e),
                    Some("Cyre::action"),
                    None
                );
                return Err(format!("Pipeline compilation failed: {}", e));
            }
        }

        // Store in centralized IO store with compiled pipeline
        match state::io::set(config) {
            Ok(_) => {
                sensor::success(
                    "cyre",
                    &format!("Channel '{}' created successfully", action_id),
                    false
                );
                Ok(())
            }
            Err(e) => {
                sensor::error(
                    "cyre",
                    &format!("Failed to create channel '{}': {}", action_id, e),
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
        // Check if channel exists in centralized IO store
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

        // Store in centralized subscriber store
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

    /// Execute channel with compiled pipeline
    pub async fn call(&self, action_id: &str, payload: ActionPayload) -> CyreResponse {
        let start_time = std::time::Instant::now();

        // Get IO config with compiled pipeline
        let config = match io::get(action_id) {
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

        // FAST PATH - Zero overhead execution for channels with no operators
        let processed_payload = if config._has_fast_path && config._pipeline.is_empty() {
            // Skip pipeline entirely for maximum performance
            payload
        } else {
            // PIPELINE EXECUTION - Execute compiled operators
            match crate::pipeline::execute_pipeline(action_id, payload).await {
                Ok(crate::pipeline::PipelineResult::Continue(processed)) => processed,
                Ok(crate::pipeline::PipelineResult::Block(reason)) => {
                    // Pipeline blocked (throttle, debounce, etc.) - this is normal protection
                    return CyreResponse {
                        ok: true, // Not an error, just protection working
                        payload: json!(null),
                        message: format!("Pipeline protection: {}", reason),
                        error: None,
                        timestamp: current_timestamp(),
                        metadata: Some(
                            json!({
                            "pipeline_blocked": true,
                            "reason": reason,
                            "execution_time_ms": start_time.elapsed().as_millis()
                        })
                        ),
                    };
                }
                Ok(crate::pipeline::PipelineResult::Schedule) => {
                    // Scheduling requested
                    return CyreResponse {
                        ok: true,
                        payload: json!(null),
                        message: "Action scheduled for later execution".to_string(),
                        error: None,
                        timestamp: current_timestamp(),
                        metadata: Some(
                            json!({
                            "scheduled": true,
                            "execution_time_ms": start_time.elapsed().as_millis()
                        })
                        ),
                    };
                }
                Err(e) => {
                    // Actual pipeline error
                    sensor::error(
                        "cyre",
                        &format!("Pipeline execution failed for '{}': {}", action_id, e),
                        Some("Cyre::call"),
                        None
                    );
                    return CyreResponse {
                        ok: false,
                        payload: json!(null),
                        message: format!("Pipeline error: {}", e),
                        error: Some("pipeline_error".to_string()),
                        timestamp: current_timestamp(),
                        metadata: None,
                    };
                }
            }
        };

        // Get handler from centralized subscriber store
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

        // Execute handler with processed payload
        let mut result = (subscriber.handler)(processed_payload).await;

        // Add execution metadata
        let execution_time = start_time.elapsed();
        result.metadata = Some(
            json!({
            "execution_time_ms": execution_time.as_millis(),
            "pipeline_operators": config._pipeline.len(),
            "fast_path": config._has_fast_path && config._pipeline.is_empty()
        })
        );

        if result.ok {
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
        // Remove compiled pipeline first
        crate::pipeline::remove_compiled_pipeline(action_id);

        // Remove from all stores
        state::subscribers::forget(action_id);
        PayloadStateOps::forget(&action_id.to_string());
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
        let pipeline_stats = crate::pipeline::get_pipeline_stats();

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
            "pipeline": {
                "total_pipelines": pipeline_stats.total_pipelines,
                "zero_overhead_count": pipeline_stats.zero_overhead_count,
                "protected_count": pipeline_stats.protected_count,
                "optimization_ratio": pipeline_stats.optimization_ratio()
            },
            "active_channels": io_count,
            "scheduled_actions": metrics.active_formations,
            "payload_store": {
                "size": state::stores::Stores::payload_size()
            }
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
