// src/core/cyre.rs - Complete Cyre implementation with full pipeline integration - FIXED

use std::sync::{ Arc, atomic::{ AtomicU64, AtomicBool, Ordering } };
use dashmap::DashMap;
use serde_json::json;

use crate::types::{
    ActionId,
    ActionPayload,
    AsyncHandler,
    CyreResponse,
    IO,
    CyreError,
    CyreResult,
    IntoActionPayload,
};
use crate::pipeline::{ PipelineCache, PipelineExecutor, PipelineResult, ExecutionPath };
use crate::channel::Channel;
use crate::utils::current_timestamp;

//=============================================================================
// COMPLETE CYRE IMPLEMENTATION
//=============================================================================

/// High-performance reactive event manager with complete pipeline system
pub struct Cyre {
    // ===== CORE STORAGE (LOCK-FREE) =====
    /// All registered channels
    channels: DashMap<ActionId, Arc<Channel>>,

    /// All registered handlers
    handlers: DashMap<ActionId, AsyncHandler>,

    /// Complete IO configurations
    configurations: DashMap<ActionId, IO>,

    // ===== PIPELINE SYSTEM =====
    /// Pipeline compilation and caching
    pipeline_cache: PipelineCache,

    // ===== PERFORMANCE COUNTERS (LOCK-FREE) =====
    total_executions: AtomicU64,
    fast_path_hits: AtomicU64,
    pipeline_hits: AtomicU64,
    protection_blocks: AtomicU64,
    authentication_blocks: AtomicU64,
    validation_blocks: AtomicU64,

    // ===== SYSTEM STATE =====
    initialized: AtomicBool,
    start_time: u64,

    // ===== CONFIGURATION =====
    debug_mode: AtomicBool,
    metrics_enabled: AtomicBool,
}

impl Cyre {
    /// Create new Cyre instance
    pub fn new() -> Self {
        Self {
            channels: DashMap::new(),
            handlers: DashMap::new(),
            configurations: DashMap::new(),
            pipeline_cache: PipelineCache::new(),
            total_executions: AtomicU64::new(0),
            fast_path_hits: AtomicU64::new(0),
            pipeline_hits: AtomicU64::new(0),
            protection_blocks: AtomicU64::new(0),
            authentication_blocks: AtomicU64::new(0),
            validation_blocks: AtomicU64::new(0),
            initialized: AtomicBool::new(false),
            start_time: current_timestamp(),
            debug_mode: AtomicBool::new(false),
            metrics_enabled: AtomicBool::new(true),
        }
    }

    //=========================================================================
    // CORE API METHODS
    //=========================================================================

    /// Register action with complete IO configuration and pipeline compilation
    pub fn action(&self, config: IO) {
        let action_id = config.id.clone();

        // 1. Compile pipeline from complete configuration
        let pipeline = self.pipeline_cache.get_or_compile(&action_id, &config);

        if self.debug_mode.load(Ordering::Relaxed) {
            println!("🔧 Compiling pipeline: {}", pipeline.description());
        }

        // 2. Create optimized channel
        let channel = Arc::new(Channel::from_config(&config));

        // 3. Store everything
        self.channels.insert(action_id.clone(), channel);
        self.configurations.insert(action_id.clone(), config);

        // 4. Log registration - FIXED: avoid temporary value borrow
        let path_desc = if pipeline.is_fast_path() {
            "fast_path: true".to_string()
        } else {
            format!("path: {}", pipeline.execution_path)
        };

        println!("✅ Action registered: {} ({})", action_id, path_desc);
    }

    /// Register handler for action
    pub fn on<F, Fut>(&self, action_id: &str, handler: F)
        where
            F: Fn(ActionPayload) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = CyreResponse> + Send + 'static
    {
        let async_handler: AsyncHandler = Arc::new(move |payload| { Box::pin(handler(payload)) });

        self.handlers.insert(action_id.to_string(), async_handler);
        println!("✅ Handler registered: {}", action_id);
    }

    /// Call action with complete pipeline execution
    pub async fn call<P>(&self, action_id: &str, payload: P) -> CyreResponse
        where P: IntoActionPayload
    {
        let payload = payload.into_payload();
        let execution_start = current_timestamp();

        // Increment total executions
        self.total_executions.fetch_add(1, Ordering::Relaxed);

        // 1. Get configuration and validate action exists
        let config = match self.configurations.get(action_id) {
            Some(config) => config.clone(),
            None => {
                return CyreResponse::error(
                    format!("Action '{}' not found", action_id),
                    "Action not registered"
                );
            }
        };

        // 2. Get or compile pipeline
        let pipeline = self.pipeline_cache.get_or_compile(action_id, &config);

        // 3. Execute pipeline based on execution path
        let pipeline_result = match pipeline.execution_path {
            ExecutionPath::FastPath => {
                // Zero-cost fast path - skip pipeline entirely
                self.fast_path_hits.fetch_add(1, Ordering::Relaxed);
                PipelineResult::Continue(payload)
            }

            ExecutionPath::Blocked => {
                // Permanently blocked - don't even try
                PipelineResult::Block("Action is blocked".to_string())
            }

            _ => {
                // Execute compiled pipeline
                self.pipeline_hits.fetch_add(1, Ordering::Relaxed);
                PipelineExecutor::execute(&pipeline, payload).await
            }
        };

        // 4. Handle pipeline result and execute handler
        let response = match pipeline_result {
            PipelineResult::Continue(processed_payload) => {
                // Pipeline passed - execute handler
                self.execute_handler(action_id, processed_payload).await
            }

            PipelineResult::Block(reason) => {
                // Pipeline blocked execution - categorize the block type
                self.categorize_and_count_block(&reason);

                CyreResponse::error(
                    reason.clone(),
                    format!("Call to '{}' was blocked", action_id)
                ).with_metadata(
                    json!({
                    "action_id": action_id,
                    "block_reason": reason,
                    "execution_path": format!("{}", pipeline.execution_path),
                    "timestamp": current_timestamp()
                })
                )
            }

            PipelineResult::Error(error) => {
                // Pipeline error
                CyreResponse::error(
                    error.clone(),
                    format!("Pipeline error for '{}'", action_id)
                ).with_metadata(
                    json!({
                    "action_id": action_id,
                    "pipeline_error": error,
                    "execution_path": format!("{}", pipeline.execution_path)
                })
                )
            }
        };

        // 5. Update execution timing
        let execution_time = current_timestamp() - execution_start;

        // 6. Add execution metadata to response if enabled
        if self.metrics_enabled.load(Ordering::Relaxed) {
            let mut response_with_metrics = response;
            if response_with_metrics.metadata.is_none() {
                response_with_metrics.metadata = Some(json!({}));
            }

            if let Some(ref mut metadata) = response_with_metrics.metadata {
                if let Some(obj) = metadata.as_object_mut() {
                    obj.insert("execution_time_ms".to_string(), json!(execution_time));
                    obj.insert(
                        "execution_path".to_string(),
                        json!(format!("{}", pipeline.execution_path))
                    );
                    obj.insert("pipeline_operators".to_string(), json!(pipeline.operators.len()));
                }
            }

            return response_with_metrics;
        }

        response
    }

    /// Execute handler after successful pipeline processing
    async fn execute_handler(&self, action_id: &str, payload: ActionPayload) -> CyreResponse {
        match self.handlers.get(action_id) {
            Some(handler) => {
                // Execute the actual handler
                handler(payload).await
            }
            None => {
                CyreResponse::error(
                    format!("Handler for '{}' not found", action_id),
                    "Handler not registered"
                )
            }
        }
    }

    /// Categorize block reasons and update appropriate counters
    fn categorize_and_count_block(&self, reason: &str) {
        if
            reason.contains("Throttled") ||
            reason.contains("Debounced") ||
            reason.contains("change")
        {
            self.protection_blocks.fetch_add(1, Ordering::Relaxed);
        } else if reason.contains("Authentication") || reason.contains("auth") {
            self.authentication_blocks.fetch_add(1, Ordering::Relaxed);
        } else if
            reason.contains("Required") ||
            reason.contains("validation") ||
            reason.contains("schema")
        {
            self.validation_blocks.fetch_add(1, Ordering::Relaxed);
        }
    }

    //=========================================================================
    // CONVENIENCE METHODS - FIXED: Remove generic parameters
    //=========================================================================

    /// Register action and handler in one call
    pub fn register<F, Fut>(&self, config: IO, handler: F)
        where
            F: Fn(ActionPayload) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = CyreResponse> + Send + 'static
    {
        let action_id = config.id.clone();
        self.action(config);
        self.on(&action_id, handler);
    }

    /// Quick registration for simple actions
    pub fn simple<F, Fut>(&self, action_id: &str, handler: F)
        where
            F: Fn(ActionPayload) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = CyreResponse> + Send + 'static
    {
        self.register(IO::new(action_id), handler);
    }

    /// Register API endpoint with standard protection
    pub fn api<F, Fut>(&self, action_id: &str, throttle_ms: u64, handler: F)
        where
            F: Fn(ActionPayload) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = CyreResponse> + Send + 'static
    {
        self.register(IO::api(action_id, throttle_ms), handler);
    }

    /// Register search endpoint with debouncing
    pub fn search<F, Fut>(&self, action_id: &str, debounce_ms: u64, handler: F)
        where
            F: Fn(ActionPayload) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = CyreResponse> + Send + 'static
    {
        self.register(IO::search(action_id, debounce_ms), handler);
    }

    //=========================================================================
    // SYSTEM MANAGEMENT
    //=========================================================================

    /// Initialize TimeKeeper integration (placeholder for future implementation)
    pub async fn init_timekeeper(&self) -> CyreResult<()> {
        // For now, just return Ok - actual TimeKeeper integration can be added later
        println!("✅ TimeKeeper integration placeholder");
        Ok(())
    }

    /// Enable/disable debug mode
    pub fn set_debug_mode(&self, enabled: bool) {
        self.debug_mode.store(enabled, Ordering::Relaxed);
        println!("🐛 Debug mode: {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Enable/disable metrics collection
    pub fn set_metrics_enabled(&self, enabled: bool) {
        self.metrics_enabled.store(enabled, Ordering::Relaxed);
        println!("📊 Metrics collection: {}", if enabled { "enabled" } else { "disabled" });
    }

    //=========================================================================
    // METRICS AND MONITORING
    //=========================================================================

    /// Get comprehensive performance metrics
    pub fn get_performance_metrics(&self) -> serde_json::Value {
        let total = self.total_executions.load(Ordering::Relaxed);
        let fast_path = self.fast_path_hits.load(Ordering::Relaxed);
        let pipeline = self.pipeline_hits.load(Ordering::Relaxed);
        let protection_blocks = self.protection_blocks.load(Ordering::Relaxed);
        let auth_blocks = self.authentication_blocks.load(Ordering::Relaxed);
        let validation_blocks = self.validation_blocks.load(Ordering::Relaxed);

        let fast_path_ratio = if total > 0 {
            ((fast_path as f64) / (total as f64)) * 100.0
        } else {
            0.0
        };

        let pipeline_stats = self.pipeline_cache.stats();

        json!({
            "system": {
                "uptime_ms": current_timestamp() - self.start_time,
                "debug_mode": self.debug_mode.load(Ordering::Relaxed),
                "metrics_enabled": self.metrics_enabled.load(Ordering::Relaxed),
                "total_actions": self.configurations.len(),
                "total_handlers": self.handlers.len(),
            },
            "executions": {
                "total_executions": total,
                "fast_path_hits": fast_path,
                "pipeline_hits": pipeline,
                "fast_path_ratio": fast_path_ratio,
            },
            "blocks": {
                "total_blocks": protection_blocks + auth_blocks + validation_blocks,
                "protection_blocks": protection_blocks,
                "authentication_blocks": auth_blocks,
                "validation_blocks": validation_blocks,
            },
            "pipeline_cache": {
                "total_pipelines": pipeline_stats.total_pipelines,
                "cache_hits": pipeline_stats.cache_hits,
                "cache_misses": pipeline_stats.cache_misses,
                "cache_hit_ratio": pipeline_stats.cache_hit_ratio(),
                "efficiency_score": pipeline_stats.efficiency_score(),
                "fast_path_count": pipeline_stats.fast_path_count,
                "protected_count": pipeline_stats.protected_count,
                "transformed_count": pipeline_stats.transformed_count,
                "authenticated_count": pipeline_stats.authenticated_count,
                "complex_count": pipeline_stats.complex_count,
                "scheduled_count": pipeline_stats.scheduled_count,
                "blocked_count": pipeline_stats.blocked_count,
            },
            "performance": {
                "average_execution_efficiency": fast_path_ratio,
                "pipeline_optimization_ratio": if pipeline_stats.total_pipelines > 0 {
                    pipeline_stats.fast_path_count as f64 / pipeline_stats.total_pipelines as f64 * 100.0
                } else { 0.0 },
            },
            // Add missing active_channels field
            "active_channels": self.channels.len(),
        })
    }

    /// Get detailed information about a specific action
    pub fn get_action_info(&self, action_id: &str) -> Option<serde_json::Value> {
        let config = self.configurations.get(action_id)?;
        let pipeline = self.pipeline_cache.get_pipeline(action_id)?;
        let metrics = pipeline.get_metrics();

        Some(
            json!({
            "action_id": action_id,
            "registration": {
                "name": config.name,
                "type": config.channel_type,
                "path": config.path,
                "group": config.group,
                "tags": config.tags,
                "description": config.description,
                "version": config.version,
            },
            "configuration": {
                "required": config.required,
                "throttle": config.throttle,
                "debounce": config.debounce,
                "max_wait": config.max_wait,
                "detect_changes": config.detect_changes,
                "block": config.block,
                "log": config.log,
                "priority": format!("{:?}", config.priority),
                "middleware": config.middleware,
                "talents": {
                    "condition": config.condition,
                    "selector": config.selector,
                    "transform": config.transform,
                    "all_talents": config.get_all_talents(),
                },
                "auth": config.auth.as_ref().map(|auth| json!({
                    "mode": format!("{:?}", auth.mode),
                    "required": config.requires_auth(),
                })),
            },
            "pipeline": {
                "execution_path": format!("{}", metrics.execution_path),
                "optimization_level": format!("{}", metrics.optimization_level),
                "operator_count": metrics.operator_count,
                "complexity_score": metrics.complexity_score,
                "is_fast_path": pipeline.is_fast_path(),
                "is_blocked": pipeline.is_blocked(),
                "compiled_at": metrics.compiled_at,
                "description": pipeline.description(),
            },
            "runtime_stats": {
                "execution_count": config._execution_count,
                "last_execution_time": config._last_exec_time,
                "last_execution_duration": config._execution_time,
                "has_handler": self.handlers.contains_key(action_id),
            },
            "optimization_flags": {
                "_has_fast_path": config._has_fast_path,
                "_has_protections": config._has_protections,
                "_has_processing": config._has_processing,
                "_has_scheduling": config._has_scheduling,
                "_has_change_detection": config._has_change_detection,
                "_is_blocked": config._is_blocked,
                "_is_scheduled": config._is_scheduled,
            },
            "summary": config.summary(),
        })
        )
    }

    /// List all registered actions with basic info
    pub fn list_actions(&self) -> Vec<serde_json::Value> {
        self.configurations
            .iter()
            .map(|entry| {
                let (action_id, config) = (entry.key(), entry.value());
                let pipeline = self.pipeline_cache.get_pipeline(action_id);

                json!({
                    "id": action_id,
                    "name": config.name,
                    "path": config.path,
                    "group": config.group,
                    "execution_path": pipeline.as_ref().map(|p| format!("{}", p.execution_path)),
                    "complexity_score": config.complexity_score(),
                    "execution_count": config._execution_count,
                    "has_handler": self.handlers.contains_key(action_id),
                })
            })
            .collect()
    }

    /// Get actions by group
    pub fn get_actions_by_group(&self, group: &str) -> Vec<String> {
        self.configurations
            .iter()
            .filter_map(|entry| {
                let (action_id, config) = (entry.key(), entry.value());
                if config.group.as_deref() == Some(group) {
                    Some(action_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get actions by path prefix
    pub fn get_actions_by_path(&self, path_prefix: &str) -> Vec<String> {
        self.configurations
            .iter()
            .filter_map(|entry| {
                let (action_id, config) = (entry.key(), entry.value());
                if config.path.as_deref().map_or(false, |p| p.starts_with(path_prefix)) {
                    Some(action_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    //=========================================================================
    // PIPELINE MANAGEMENT
    //=========================================================================

    /// Clear pipeline cache (useful for hot reloading during development)
    pub fn clear_pipeline_cache(&self) {
        self.pipeline_cache.clear();
        println!("🔄 Pipeline cache cleared");
    }

    /// Recompile pipeline for specific action
    pub fn recompile_pipeline(&self, action_id: &str) -> CyreResult<()> {
        if let Some(config) = self.configurations.get(action_id) {
            // Remove from cache to force recompilation
            self.pipeline_cache.remove_pipeline(action_id);

            // Trigger recompilation
            let pipeline = self.pipeline_cache.get_or_compile(action_id, &config);

            println!("🔄 Pipeline recompiled for '{}': {}", action_id, pipeline.description());
            Ok(())
        } else {
            Err(CyreError::ActionNotFound { action_id: action_id.to_string() })
        }
    }

    /// Update action configuration and recompile pipeline
    pub fn update_action(&self, action_id: &str, new_config: IO) -> CyreResult<()> {
        if !self.configurations.contains_key(action_id) {
            return Err(CyreError::ActionNotFound { action_id: action_id.to_string() });
        }

        // Ensure the ID matches
        if new_config.id != action_id {
            return Err(CyreError::ConfigurationError {
                field: "id".to_string(),
                reason: "ID cannot be changed when updating configuration".to_string(),
            });
        }

        // Update configuration
        self.configurations.insert(action_id.to_string(), new_config.clone());

        // Force pipeline recompilation
        self.recompile_pipeline(action_id)?;

        println!("✅ Action '{}' configuration updated", action_id);
        Ok(())
    }

    //=========================================================================
    // DEBUGGING AND DIAGNOSTICS
    //=========================================================================

    /// Print detailed system status
    pub fn print_system_status(&self) {
        let metrics = self.get_performance_metrics();

        println!("\n🔍 CYRE SYSTEM STATUS");
        println!("========================");

        // System info
        let system = &metrics["system"];
        println!("🖥️  System:");
        println!("   Uptime: {}ms", system["uptime_ms"]);
        println!("   Actions: {}", system["total_actions"]);
        println!("   Handlers: {}", system["total_handlers"]);
        println!("   Debug Mode: {}", system["debug_mode"]);

        // Execution stats
        let executions = &metrics["executions"];
        println!("\n⚡ Executions:");
        println!("   Total: {}", executions["total_executions"]);
        println!(
            "   Fast Path: {} ({:.1}%)",
            executions["fast_path_hits"],
            executions["fast_path_ratio"]
        );
        println!("   Pipeline: {}", executions["pipeline_hits"]);

        // Block stats
        let blocks = &metrics["blocks"];
        println!("\n🛡️  Blocks:");
        println!("   Total: {}", blocks["total_blocks"]);
        println!("   Protection: {}", blocks["protection_blocks"]);
        println!("   Authentication: {}", blocks["authentication_blocks"]);
        println!("   Validation: {}", blocks["validation_blocks"]);

        // Pipeline cache stats
        let cache = &metrics["pipeline_cache"];
        println!("\n🏗️  Pipeline Cache:");
        println!("   Total Pipelines: {}", cache["total_pipelines"]);
        println!(
            "   Cache Hit Ratio: {:.1}%",
            cache["cache_hit_ratio"].as_f64().unwrap_or(0.0) * 100.0
        );
        println!("   Fast Path: {}", cache["fast_path_count"]);
        println!("   Protected: {}", cache["protected_count"]);
        println!("   Complex: {}", cache["complex_count"]);

        // Performance summary
        let performance = &metrics["performance"];
        println!("\n📈 Performance:");
        println!("   Execution Efficiency: {:.1}%", performance["average_execution_efficiency"]);
        println!("   Optimization Ratio: {:.1}%", performance["pipeline_optimization_ratio"]);

        println!();
    }

    /// Health check for the entire system
    pub fn health_check(&self) -> serde_json::Value {
        let metrics = self.get_performance_metrics();
        let total_actions = metrics["system"]["total_actions"].as_u64().unwrap_or(0);
        let total_handlers = metrics["system"]["total_handlers"].as_u64().unwrap_or(0);
        let cache_hit_ratio = metrics["pipeline_cache"]["cache_hit_ratio"].as_f64().unwrap_or(0.0);

        let status = if total_actions == total_handlers && cache_hit_ratio > 0.8 {
            "healthy"
        } else if total_actions > 0 && total_handlers > 0 {
            "degraded"
        } else {
            "unhealthy"
        };

        json!({
            "status": status,
            "timestamp": current_timestamp(),
            "checks": {
                "actions_registered": total_actions > 0,
                "handlers_registered": total_handlers > 0,
                "actions_handlers_match": total_actions == total_handlers,
                "cache_performance_good": cache_hit_ratio > 0.8,
                "system_responsive": true, // Always true if we can run this check
            },
            "metrics_summary": {
                "total_actions": total_actions,
                "total_handlers": total_handlers,
                "cache_hit_ratio": cache_hit_ratio,
                "uptime_ms": metrics["system"]["uptime_ms"],
            }
        })
    }

    //=========================================================================
    // UTILITY METHODS
    //=========================================================================

    /// Check if action exists
    pub fn has_action(&self, action_id: &str) -> bool {
        self.configurations.contains_key(action_id)
    }

    /// Check if handler is registered
    pub fn has_handler(&self, action_id: &str) -> bool {
        self.handlers.contains_key(action_id)
    }

    /// Remove action and its handler
    pub fn remove_action(&self, action_id: &str) -> CyreResult<()> {
        let had_config = self.configurations.remove(action_id).is_some();
        let had_handler = self.handlers.remove(action_id).is_some();
        let had_channel = self.channels.remove(action_id).is_some();

        if had_config || had_handler || had_channel {
            // Also remove from pipeline cache
            self.pipeline_cache.remove_pipeline(action_id);
            println!("🗑️  Removed action: {}", action_id);
            Ok(())
        } else {
            Err(CyreError::ActionNotFound { action_id: action_id.to_string() })
        }
    }

    /// Get system uptime in milliseconds
    pub fn uptime_ms(&self) -> u64 {
        current_timestamp() - self.start_time
    }

    /// Get total number of registered actions
    pub fn action_count(&self) -> usize {
        self.configurations.len()
    }

    /// Get total number of registered handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
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
// DISPLAY IMPLEMENTATION
//=============================================================================

impl std::fmt::Display for Cyre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cyre[actions:{}, handlers:{}, uptime:{}ms]",
            self.action_count(),
            self.handler_count(),
            self.uptime_ms()
        )
    }
}
