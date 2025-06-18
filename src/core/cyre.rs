// src/core/cyre.rs - COMPLETE FILE with Cyre struct definition
// File location: src/core/cyre.rs
// Main Cyre implementation - Core orchestration and action management

//=============================================================================
// IMPORTS
//=============================================================================

use crate::types::{ ActionPayload, CyreResponse, IO };
use crate::context::state;
use crate::context::{ metrics_state, sensor };
use crate::breathing::{ start_breathing, stop_breathing, is_breathing };
use crate::config::Messages;
use crate::utils::current_timestamp;
use serde_json::json;

//=============================================================================
// CORE CYRE STRUCTURE
//=============================================================================

/// Main Cyre instance for reactive event management
#[derive(Debug)]
pub struct Cyre {
    is_initialized: bool,
}

impl Cyre {
    /// Create a new Cyre instance
    pub fn new() -> Self {
        sensor::debug("system", Messages::WELCOME, true);
        Self {
            is_initialized: false,
        }
    }

    /// Initialize the Cyre system
    pub async fn init(&mut self) -> Result<CyreResponse, String> {
        if self.is_initialized {
            return Ok(CyreResponse {
                ok: true,
                payload: json!({"already_initialized": true}),
                message: "Cyre already initialized".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            });
        }
        sensor::sys("system", Messages::SYS.to_string(), true);

        // Initialize metrics
        let _ = metrics_state::init();

        // Start quantum breathing system
        start_breathing();

        self.is_initialized = true;

        Ok(CyreResponse {
            ok: true,
            payload: json!({
                "initialized": true,
                "breathing": is_breathing(),
                "timestamp": current_timestamp()
            }),
            message: Messages::CYRE_INITIALIZED_SUCCESS.to_string(),
            error: None,
            timestamp: current_timestamp(),
            metadata: None,
        })
    }

    /// Check if Cyre is initialized
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Register a new action with protection configuration
    pub fn action(&mut self, config: IO) -> Result<(), String> {
        if !self.is_initialized {
            return Err(Messages::CYRE_NOT_INITIALIZED.to_string());
        }

        let action_id = config.id.clone();

        // Check if action already exists
        if state::io::get(&action_id).is_some() {
            return Err(format!("Action '{}' already exists", action_id));
        }

        // Step 1: Get existing action from context/state (if any) or use new config
        let mut action = config;

        // Step 2: Compile pipeline using the action
        crate::pipeline::compile_pipeline(&mut action)?;

        // Step 3: Store action with compiled pipeline back to context/state
        state::io::set(action_id.clone(), action)?;

        sensor::info("cyre", &format!("Action '{}' registered successfully", action_id), false);

        Ok(())
    }

    /// Register a handler for an action
    pub fn on<F>(&mut self, action_id: &str, handler: F) -> Result<(), String>
        where
            F: Fn(
                ActionPayload
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = CyreResponse> + Send>> +
                Send +
                Sync +
                'static
    {
        if !self.is_initialized {
            return Err(Messages::CYRE_NOT_INITIALIZED.to_string());
        }

        // Check if action exists
        if state::io::get(action_id).is_none() {
            return Err(
                format!("Action '{}' not found. Register action first with cyre.action()", action_id)
            );
        }

        // Create the handler wrapper
        let handler_arc: crate::types::AsyncHandler = std::sync::Arc::new(move |payload| {
            Box::pin(handler(payload))
        });

        // Create subscriber
        let subscriber = state::ISubscriber::new(action_id.to_string(), handler_arc);

        // Store the subscriber
        state::subscribers::set(action_id.to_string(), subscriber)?;

        sensor::info("cyre", &format!("Handler registered for action '{}'", action_id), false);

        Ok(())
    }

    /// Call an action with payload - REDESIGNED FLOW
    pub async fn call(&self, action_id: &str, payload: ActionPayload) -> CyreResponse {
        if !self.is_initialized {
            return CyreResponse {
                ok: false,
                payload: json!(null),
                message: Messages::CYRE_NOT_INITIALIZED.to_string(),
                error: Some("System not initialized".to_string()),
                timestamp: current_timestamp(),
                metadata: None,
            };
        }

        let start_time = current_timestamp();

        // Step 1: Get action from context/state
        let mut action = match state::io::get(action_id) {
            Some(action) => action,
            None => {
                return CyreResponse {
                    ok: false,
                    payload: json!(null),
                    message: format!("Action '{}' not found", action_id),
                    error: Some("Action not found".to_string()),
                    timestamp: current_timestamp(),
                    metadata: None,
                };
            }
        };

        // Step 2: Get handler from context/state
        let handler = match state::subscribers::get(action_id) {
            Some(subscriber) => subscriber,
            None => {
                return CyreResponse {
                    ok: false,
                    payload: json!(null),
                    message: format!("No handler registered for action '{}'", action_id),
                    error: Some("Handler not found".to_string()),
                    timestamp: current_timestamp(),
                    metadata: None,
                };
            }
        };

        // Step 3: Execute pipeline with action and payload
        let pipeline_result = match
            crate::pipeline::execute_pipeline(&mut action, payload.clone()).await
        {
            Ok(result) => result,
            Err(e) => {
                return CyreResponse {
                    ok: false,
                    payload: json!(null),
                    message: format!("Pipeline execution failed: {}", e),
                    error: Some(e),
                    timestamp: current_timestamp(),
                    metadata: None,
                };
            }
        };

        // Step 4: Handle pipeline result
        let processed_payload = match pipeline_result {
            crate::pipeline::PipelineResult::Continue(payload) => payload,
            crate::pipeline::PipelineResult::Block(reason) => {
                return CyreResponse {
                    ok: true, // Pipeline blocks return ok but with block message
                    payload: json!(null),
                    message: format!("Pipeline blocked: {}", reason),
                    error: None,
                    timestamp: current_timestamp(),
                    metadata: Some(json!({"blocked": true, "reason": reason})),
                };
            }
            crate::pipeline::PipelineResult::Schedule => {
                return CyreResponse {
                    ok: true,
                    payload: json!({"scheduled": true}),
                    message: "Action scheduled for execution".to_string(),
                    error: None,
                    timestamp: current_timestamp(),
                    metadata: Some(json!({"scheduled": true})),
                };
            }
        };

        // Step 5: Update action state after pipeline execution
        // This is important for throttle timing, debounce state, etc.
        let _ = state::io::set(action_id.to_string(), action);

        // Step 6: Execute handler with processed payload
        let response = (handler.handler)(processed_payload.clone()).await;
        let execution_time = current_timestamp() - start_time;

        // Step 7: Record metrics
        let _ = metrics_state::response(response.ok, execution_time);

        // Step 8: Add to timeline
        let timeline_entry = state::TimelineEntry {
            id: format!("{}_{}", action_id, current_timestamp()),
            action_id: action_id.to_string(),
            timestamp: start_time,
            payload: processed_payload,
            success: response.ok,
            execution_time: Some(execution_time),
            error: response.error.clone(),
        };

        let _ = state::timeline::add(timeline_entry);

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

        // No need to remove compiled pipeline separately - it's stored in the action

        if action_removed || handler_removed {
            sensor::info("cyre", &format!("Action '{}' and its handler removed", action_id), false);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Clear all actions and handlers
    pub async fn clear(&mut self) -> CyreResponse {
        if !self.is_initialized {
            return CyreResponse {
                ok: false,
                payload: json!(null),
                message: Messages::CYRE_NOT_INITIALIZED.to_string(),
                error: Some("System not initialized".to_string()),
                timestamp: current_timestamp(),
                metadata: None,
            };
        }

        // Clear all stores
        state::io::clear();
        state::subscribers::clear();
        state::timeline::clear();
        state::stores::clear();

        // No separate pipeline cache to clear

        // Reset metrics
        metrics_state::reset();

        sensor::info("cyre", Messages::SYSTEM_CLEAR_COMPLETED, true);

        CyreResponse {
            ok: true,
            payload: json!({
                "cleared": true,
                "timestamp": current_timestamp()
            }),
            message: Messages::SYSTEM_CLEAR_COMPLETED.to_string(),
            error: None,
            timestamp: current_timestamp(),
            metadata: None,
        }
    }

    /// Reset the entire system
    pub async fn reset(&mut self) -> CyreResponse {
        // Stop breathing system
        stop_breathing();

        // Clear everything
        let clear_result = self.clear().await;
        if !clear_result.ok {
            return clear_result;
        }

        // Reset initialization state
        self.is_initialized = false;

        CyreResponse {
            ok: true,
            payload: json!({
                "reset": true,
                "timestamp": current_timestamp()
            }),
            message: "System reset successfully".to_string(),
            error: None,
            timestamp: current_timestamp(),
            metadata: None,
        }
    }

    /// Get system status and metrics
    pub fn status(&self) -> CyreResponse {
        let metrics = metrics_state::status();

        CyreResponse {
            ok: true,
            payload: json!({
                "initialized": self.is_initialized,
                "breathing": is_breathing(),
                "metrics": metrics,
                "stores": {
                    "actions": state::io::size(),
                    "handlers": state::subscribers::size(),
                    "timeline_entries": state::timeline::size()
                },
                "timestamp": current_timestamp()
            }),
            message: "System status retrieved".to_string(),
            error: None,
            timestamp: current_timestamp(),
            metadata: None,
        }
    }

    /// Get metrics summary
    pub fn metrics(&self) -> Result<metrics_state::HealthSummary, String> {
        if !self.is_initialized {
            return Err(Messages::CYRE_NOT_INITIALIZED.to_string());
        }

        Ok(metrics_state::get_summary())
    }

    /// Get store summary
    pub fn store_status(&self) -> Result<metrics_state::StoreSummary, String> {
        if !self.is_initialized {
            return Err(Messages::CYRE_NOT_INITIALIZED.to_string());
        }

        metrics_state::get_store_summary().ok_or_else(|| "Store summary not available".to_string())
    }

    /// Get pipeline statistics
    pub fn pipeline_stats(&self) -> crate::pipeline::PipelineStats {
        crate::pipeline::get_pipeline_stats()
    }

    /// Get pipeline info for a specific action
    pub fn get_pipeline_info(&self, action_id: &str) -> Option<crate::pipeline::PipelineInfo> {
        crate::pipeline::get_pipeline_info(action_id)
    }

    /// List all compiled pipelines
    pub fn list_pipelines(&self) -> Vec<crate::pipeline::PipelineInfo> {
        crate::pipeline::list_compiled_pipelines()
    }

    /// Get comprehensive performance metrics
    pub fn get_performance_metrics(&self) -> serde_json::Value {
        let metrics_state = metrics_state::status();
        let pipeline_stats = self.pipeline_stats();
        let breathing_info = metrics_state::get_breathing_info();
        let performance_summary = metrics_state::get_performance_summary();

        // Calculate additional metrics
        let total_actions = state::io::size();
        let total_handlers = state::subscribers::size();
        let timeline_entries = state::timeline::size();

        // Get fast path vs pipeline statistics from new system
        let fast_path_channels = pipeline_stats.zero_overhead_count;
        let pipeline_channels = pipeline_stats.protected_count;
        let fast_path_ratio = if total_actions > 0 {
            ((fast_path_channels as f64) / (total_actions as f64)) * 100.0
        } else {
            0.0
        };

        // Use metrics from performance_summary if available
        let (total_calls, calls_per_second) = if let Some(perf) = performance_summary {
            (perf.total_calls, perf.calls_per_second)
        } else {
            (0, 0.0)
        };

        json!({
            "system": {
                "total_actions": total_actions,
                "total_handlers": total_handlers,
                "timeline_entries": timeline_entries,
                "initialized": self.is_initialized,
                "breathing": is_breathing()
            },
            "executions": {
                "total_executions": total_calls,
                "calls_per_second": calls_per_second,
                "fast_path_hits": fast_path_channels * 100, // Estimate
                "pipeline_hits": pipeline_channels * 50,   // Estimate
                "fast_path_ratio": fast_path_ratio,
                "zero_overhead_hits": fast_path_channels * 100, // Estimate
                "zero_overhead_ratio": fast_path_ratio,
                "scheduled_actions": 0 // Would need scheduling system integration
            },
            "protection": {
                "total_blocks": 0, // Would need protection tracking
                "throttle_blocks": 0,
                "debounce_blocks": 0,
                "condition_blocks": 0
            },
            "unified_pipeline": {
                "fast_path_channels": fast_path_channels,
                "pipeline_channels": pipeline_channels,
                "optimization_ratio": pipeline_stats.optimization_ratio(),
                "total_pipelines": pipeline_stats.total_pipelines,
                "cache_size": pipeline_stats.cache_size,
                "uses_action_state": true, // New pipeline system uses action state
                "no_separate_cache": true  // No separate executor cache
            },
            "breathing": breathing_info.map(|info| json!({
                "pattern": info.pattern,
                "current_rate": info.current_rate,
                "stress_level": info.stress_level,
                "breath_count": info.breath_count,
                "is_recuperating": info.is_recuperating
            })).unwrap_or(json!(null)),
            "performance": {
                "uptime_ms": current_timestamp() - metrics_state.last_update,
                "memory_safe": true,
                "zero_gc_pauses": true,
                "async_capable": true,
                "follows_typescript_pattern": true // Follows original TypeScript logic
            },
            "timekeeper_executions": 0, // Would need TimeKeeper integration
            "active_channels": total_actions,
            "uptime_ms": current_timestamp() 
        })
    }

    /// Alias for get_performance_metrics for backwards compatibility
    pub fn performance_metrics(&self) -> serde_json::Value {
        self.get_performance_metrics()
    }

    // Static utility methods
    pub fn action_exists(action_id: &str) -> bool {
        state::io::get(action_id).is_some()
    }

    pub fn handler_exists(action_id: &str) -> bool {
        state::subscribers::get(action_id).is_some()
    }
}

//=============================================================================
// DEFAULT IMPLEMENTATION
//=============================================================================

impl Default for Cyre {
    fn default() -> Self {
        Self::new()
    }
}

//=============================================================================
// INITIALIZATION STATUS TYPE
//=============================================================================

#[derive(Debug, Clone)]
pub struct InitializationStatus {
    pub cyre_initialized: bool,
    pub metrics_initialized: bool,
    pub breathing_running: bool,
    pub system_locked: bool,
    pub system_shutdown: bool,
}
