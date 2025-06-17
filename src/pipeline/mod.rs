// src/pipeline/mod.rs - RENAMED CompiledPipeline to PipelineExecutor

use std::sync::{ Arc, RwLock, OnceLock };
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::oneshot;
use parking_lot::RwLock as FastRwLock;

use crate::types::{ ActionId, ActionPayload, IO };
use crate::utils::current_timestamp;

//=============================================================================
// PIPELINE EXECUTOR CACHE
//=============================================================================

/// Pipeline executor cache (runtime execution engines)
static EXECUTOR_CACHE: OnceLock<Arc<RwLock<HashMap<ActionId, PipelineExecutor>>>> = OnceLock::new();

fn get_executor_cache() -> &'static Arc<RwLock<HashMap<ActionId, PipelineExecutor>>> {
    EXECUTOR_CACHE.get_or_init(|| { Arc::new(RwLock::new(HashMap::new())) })
}

//=============================================================================
// PIPELINE EXECUTOR (RUNTIME DECOMPILER)
//=============================================================================

/// Runtime pipeline executor - decompiles and executes operators
#[derive(Clone)]
pub struct PipelineExecutor {
    pub action_id: ActionId,
    pub is_zero_overhead: bool,

    // Protection operators (compiled for speed)
    pub has_block: bool,
    pub block_reason: String,

    pub has_throttle: bool,
    pub throttle_ms: u64,
    pub throttle_last: Arc<std::sync::atomic::AtomicU64>,

    pub has_debounce: bool,
    pub debounce_ms: u64,
    pub debounce_timer: Arc<FastRwLock<Option<tokio::task::JoinHandle<()>>>>,

    pub has_required: bool,
    pub has_change_detection: bool,
    pub has_scheduling: bool,

    // Processing operators
    pub has_condition: bool,
    pub condition_talent: Option<String>,

    pub has_selector: bool,
    pub selector_talent: Option<String>,

    pub has_transform: bool,
    pub transform_talent: Option<String>,

    pub has_schema: bool,
    pub schema_name: Option<String>,

    pub compiled_at: u64,
}

impl PipelineExecutor {
    /// Create pipeline executor from IO configuration
    fn from_config(action_id: &str, config: &IO) -> Self {
        let has_throttle = config.throttle.is_some();
        let has_debounce = config.debounce.is_some();
        let has_required = config.required.is_some();
        let has_block = config.block;
        let has_change_detection = config.detect_changes;
        let has_scheduling = config.delay.is_some() || config.interval.is_some();
        let has_condition = config.condition.is_some();
        let has_selector = config.selector.is_some();
        let has_transform = config.transform.is_some();
        let has_schema = config.schema.is_some();

        // Zero overhead = no operators at all
        let is_zero_overhead =
            !has_throttle &&
            !has_debounce &&
            !has_required &&
            !has_block &&
            !has_change_detection &&
            !has_scheduling &&
            !has_condition &&
            !has_selector &&
            !has_transform &&
            !has_schema &&
            !config.log;

        PipelineExecutor {
            action_id: action_id.to_string(),
            is_zero_overhead,
            has_block,
            block_reason: if has_block {
                "Channel blocked".to_string()
            } else {
                String::new()
            },
            has_throttle,
            throttle_ms: config.throttle.unwrap_or(0),
            throttle_last: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            has_debounce,
            debounce_ms: config.debounce.unwrap_or(0),
            debounce_timer: Arc::new(FastRwLock::new(None)),
            has_required,
            has_change_detection,
            has_scheduling,
            has_condition,
            condition_talent: config.condition.clone(),
            has_selector,
            selector_talent: config.selector.clone(),
            has_transform,
            transform_talent: config.transform.clone(),
            has_schema,
            schema_name: config.schema.clone(),
            compiled_at: current_timestamp(),
        }
    }

    /// Execute pipeline operators in proper order
    #[inline(always)]
    pub async fn execute(&self, payload: ActionPayload) -> PipelineResult {
        // ZERO OVERHEAD FAST PATH
        if self.is_zero_overhead {
            return PipelineResult::Continue(payload);
        }

        // PIPELINE EXECUTION IN ORDER

        // 1. Block check (immediate rejection)
        if self.has_block {
            return PipelineResult::Block(self.block_reason.clone());
        }

        // 2. Required payload validation
        if self.has_required && payload.is_null() {
            return PipelineResult::Block("Required payload missing".to_string());
        }

        // 3. Schema validation
        let payload_after_schema = if self.has_schema {
            match self.execute_schema_validation(&payload).await {
                Ok(p) => p,
                Err(e) => {
                    return PipelineResult::Block(format!("Schema validation failed: {}", e));
                }
            }
        } else {
            payload
        };

        // 4. Throttle protection
        if self.has_throttle {
            let now = current_timestamp();
            let last = self.throttle_last.load(std::sync::atomic::Ordering::Relaxed);
            if now - last < self.throttle_ms {
                return PipelineResult::Block(
                    format!("Throttled: {}ms remaining", self.throttle_ms - (now - last))
                );
            }
            self.throttle_last.store(now, std::sync::atomic::Ordering::Relaxed);
        }

        // 5. Change detection
        let payload_after_change_detection = if self.has_change_detection {
            match self.execute_change_detection(&payload_after_schema).await {
                Ok(p) => p,
                Err(e) => {
                    return PipelineResult::Block(e);
                }
            }
        } else {
            payload_after_schema
        };

        // 6. Condition check
        let payload_after_condition = if self.has_condition {
            match self.execute_condition(&payload_after_change_detection).await {
                Ok(p) => p,
                Err(e) => {
                    return PipelineResult::Block(format!("Condition failed: {}", e));
                }
            }
        } else {
            payload_after_change_detection
        };

        // 7. Selector transformation
        let payload_after_selector = if self.has_selector {
            match self.execute_selector(&payload_after_condition).await {
                Ok(p) => p,
                Err(e) => {
                    return PipelineResult::Block(format!("Selector failed: {}", e));
                }
            }
        } else {
            payload_after_condition
        };

        // 8. Transform processing
        let payload_after_transform = if self.has_transform {
            match self.execute_transform(&payload_after_selector).await {
                Ok(p) => p,
                Err(e) => {
                    return PipelineResult::Block(format!("Transform failed: {}", e));
                }
            }
        } else {
            payload_after_selector
        };

        // 9. Debounce protection (async - most expensive, do last)
        let final_payload = if self.has_debounce {
            // Cancel existing timer
            if let Some(handle) = self.debounce_timer.write().take() {
                handle.abort();
            }

            let (tx, rx) = oneshot::channel();
            let debounce_ms = self.debounce_ms;

            let handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(debounce_ms)).await;
                let _ = tx.send(());
            });

            *self.debounce_timer.write() = Some(handle);

            match rx.await {
                Ok(_) => payload_after_transform,
                Err(_) => {
                    return PipelineResult::Block("Debounced (cancelled)".to_string());
                }
            }
        } else {
            payload_after_transform
        };

        // 10. Scheduling check
        if self.has_scheduling {
            return PipelineResult::Schedule;
        }

        PipelineResult::Continue(final_payload)
    }

    /// Execute schema validation operator
    async fn execute_schema_validation(
        &self,
        payload: &ActionPayload
    ) -> Result<ActionPayload, String> {
        if let Some(ref schema_name) = self.schema_name {
            // TODO: Implement actual schema validation with talent system
            // For now, just validate that payload is not null for non-empty schemas
            if schema_name == "non-empty" && payload.is_null() {
                return Err("Schema requires non-empty payload".to_string());
            }
        }
        Ok(payload.clone())
    }

    /// Execute change detection operator
    async fn execute_change_detection(
        &self,
        payload: &ActionPayload
    ) -> Result<ActionPayload, String> {
        // Use existing PayloadStateOps for change detection
        let last_payload = crate::context::state::PayloadStateOps::get(&self.action_id);

        if let Some(last) = last_payload {
            if last == *payload {
                return Err("Payload unchanged, skipping execution".to_string());
            }
        }

        // Store current payload for next comparison
        let _ = crate::context::state::PayloadStateOps::set(
            self.action_id.clone(),
            payload.clone()
        );
        Ok(payload.clone())
    }

    /// Execute condition operator
    async fn execute_condition(&self, payload: &ActionPayload) -> Result<ActionPayload, String> {
        if let Some(ref condition_talent) = self.condition_talent {
            // TODO: Implement talent system integration
            // For now, simple condition checks
            match condition_talent.as_str() {
                "always_true" => Ok(payload.clone()),
                "never_true" => Err("Condition always false".to_string()),
                "has_data" => {
                    if payload.is_null() {
                        Err("Condition requires data".to_string())
                    } else {
                        Ok(payload.clone())
                    }
                }
                _ => {
                    // Unknown condition - log and pass through
                    crate::context::sensor::warn(
                        "pipeline",
                        &format!(
                            "Unknown condition talent '{}' for '{}'",
                            condition_talent,
                            self.action_id
                        ),
                        false
                    );
                    Ok(payload.clone())
                }
            }
        } else {
            Ok(payload.clone())
        }
    }

    /// Execute selector operator
    async fn execute_selector(&self, payload: &ActionPayload) -> Result<ActionPayload, String> {
        if let Some(ref selector_talent) = self.selector_talent {
            // TODO: Implement talent system integration
            // For now, simple selector operations
            match selector_talent.as_str() {
                "identity" => Ok(payload.clone()),
                "data" => {
                    if let Some(data) = payload.get("data") {
                        Ok(data.clone())
                    } else {
                        Ok(payload.clone())
                    }
                }
                "value" => {
                    if let Some(value) = payload.get("value") {
                        Ok(value.clone())
                    } else {
                        Ok(payload.clone())
                    }
                }
                _ => {
                    // Unknown selector - log and pass through
                    crate::context::sensor::warn(
                        "pipeline",
                        &format!(
                            "Unknown selector talent '{}' for '{}'",
                            selector_talent,
                            self.action_id
                        ),
                        false
                    );
                    Ok(payload.clone())
                }
            }
        } else {
            Ok(payload.clone())
        }
    }

    /// Execute transform operator
    async fn execute_transform(&self, payload: &ActionPayload) -> Result<ActionPayload, String> {
        if let Some(ref transform_talent) = self.transform_talent {
            // TODO: Implement talent system integration
            // For now, simple transform operations
            match transform_talent.as_str() {
                "identity" => Ok(payload.clone()),
                "uppercase" => {
                    if let Some(text) = payload.as_str() {
                        Ok(serde_json::json!(text.to_uppercase()))
                    } else {
                        Ok(payload.clone())
                    }
                }
                "add_timestamp" => {
                    let mut result = payload.clone();
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("timestamp".to_string(), serde_json::json!(current_timestamp()));
                    }
                    Ok(result)
                }
                _ => {
                    // Unknown transform - log and pass through
                    crate::context::sensor::warn(
                        "pipeline",
                        &format!(
                            "Unknown transform talent '{}' for '{}'",
                            transform_talent,
                            self.action_id
                        ),
                        false
                    );
                    Ok(payload.clone())
                }
            }
        } else {
            Ok(payload.clone())
        }
    }

    /// Get executor info
    pub fn get_info(&self) -> PipelineInfo {
        PipelineInfo {
            action_id: self.action_id.clone(),
            is_zero_overhead: self.is_zero_overhead,
            protection_count: self.count_operators(),
            compiled_at: self.compiled_at,
        }
    }

    fn count_operators(&self) -> u32 {
        let mut count = 0;
        if self.has_block {
            count += 1;
        }
        if self.has_required {
            count += 1;
        }
        if self.has_schema {
            count += 1;
        }
        if self.has_throttle {
            count += 1;
        }
        if self.has_debounce {
            count += 1;
        }
        if self.has_change_detection {
            count += 1;
        }
        if self.has_condition {
            count += 1;
        }
        if self.has_selector {
            count += 1;
        }
        if self.has_transform {
            count += 1;
        }
        if self.has_scheduling {
            count += 1;
        }
        count
    }
}

//=============================================================================
// PIPELINE RESULT
//=============================================================================

#[derive(Debug)]
pub enum PipelineResult {
    Continue(ActionPayload),
    Block(String),
    Schedule,
}

//=============================================================================
// PIPELINE INFO
//=============================================================================

#[derive(Debug, Clone)]
pub struct PipelineInfo {
    pub action_id: ActionId,
    pub is_zero_overhead: bool,
    pub protection_count: u32,
    pub compiled_at: u64,
}

//=============================================================================
// PIPELINE COMPILER FUNCTIONS
//=============================================================================

/// Compile pipeline for channel - UPDATED to use PipelineExecutor
pub fn compile_pipeline(action_id: &str, config: &mut IO) -> Result<(), String> {
    // Validate config for pipeline conflicts
    if config.throttle.is_some() && config.debounce.is_some() {
        return Err(
            "Cannot compile pipeline: Throttle and debounce are mutually exclusive".to_string()
        );
    }

    // Build _pipeline operators in execution order
    let mut pipeline_operators = Vec::new();

    // 1. Validation operators first
    if config.required.is_some() {
        pipeline_operators.push("required".to_string());
    }

    if let Some(ref schema) = config.schema {
        pipeline_operators.push(format!("schema:{}", schema));
    }

    // 2. Protection operators
    if config.block {
        pipeline_operators.push("block".to_string());
    } else {
        // Only add other operators if not blocked
        if let Some(throttle_ms) = config.throttle {
            pipeline_operators.push(format!("throttle:{}", throttle_ms));
        }

        if let Some(debounce_ms) = config.debounce {
            if let Some(max_wait_ms) = config.max_wait {
                pipeline_operators.push(format!("debounce:{}:{}", debounce_ms, max_wait_ms));
            } else {
                pipeline_operators.push(format!("debounce:{}", debounce_ms));
            }
        }

        if config.detect_changes {
            pipeline_operators.push("detect_changes".to_string());
        }

        // 3. Processing operators
        if let Some(ref condition) = config.condition {
            pipeline_operators.push(format!("condition:{}", condition));
        }

        if let Some(ref selector) = config.selector {
            pipeline_operators.push(format!("selector:{}", selector));
        }

        if let Some(ref transform) = config.transform {
            pipeline_operators.push(format!("transform:{}", transform));
        }

        // 4. Scheduling operators
        if let Some(delay_ms) = config.delay {
            pipeline_operators.push(format!("delay:{}", delay_ms));
        }

        if let Some(interval_ms) = config.interval {
            pipeline_operators.push(format!("interval:{}", interval_ms));
        }
    }

    // Store compiled operators in IO config _pipeline field
    config._pipeline = pipeline_operators;

    // Update optimization flags based on compiled pipeline
    config._has_fast_path = config._pipeline.is_empty();
    config._has_protections =
        config.throttle.is_some() ||
        config.debounce.is_some() ||
        config.block ||
        config.detect_changes ||
        config.required.is_some();
    config._has_processing =
        config.condition.is_some() ||
        config.selector.is_some() ||
        config.transform.is_some() ||
        config.schema.is_some();
    config._has_scheduling = config.delay.is_some() || config.interval.is_some();

    // Create pipeline executor for runtime
    let executor = PipelineExecutor::from_config(action_id, config);

    // Store in executor cache
    let cache = get_executor_cache();
    match cache.write() {
        Ok(mut cache_lock) => {
            cache_lock.insert(action_id.to_string(), executor.clone());
            println!("🔧 Pipeline executor created: {} ({})", action_id, if
                executor.is_zero_overhead
            {
                "zero_overhead"
            } else {
                "protected"
            });
            Ok(())
        }
        Err(e) => { Err(format!("Failed to store pipeline executor: {}", e)) }
    }
}

/// Get pipeline executor (called by execution engine)
pub fn get_compiled_pipeline(action_id: &str) -> Option<PipelineExecutor> {
    let cache = get_executor_cache();
    let cache_lock = cache.read().unwrap();
    cache_lock.get(action_id).cloned()
}

/// Remove pipeline executor (called by channel manager on forget)
pub fn remove_compiled_pipeline(action_id: &str) {
    let cache = get_executor_cache();
    let mut cache_lock = cache.write().unwrap();
    if cache_lock.remove(action_id).is_some() {
        println!("🗑️ Pipeline executor removed: {}", action_id);
    }
}

/// Execute pipeline (called by execution engine)
pub async fn execute_pipeline(
    action_id: &str,
    payload: ActionPayload
) -> Result<PipelineResult, String> {
    let executor = get_compiled_pipeline(action_id).ok_or_else(||
        format!("Pipeline executor not found for action '{}'", action_id)
    )?;

    Ok(executor.execute(payload).await)
}

/// Get pipeline info
pub fn get_pipeline_info(action_id: &str) -> Option<PipelineInfo> {
    let cache = get_executor_cache();
    let cache_lock = cache.read().unwrap();
    cache_lock.get(action_id).map(|p| p.get_info())
}

/// List all pipeline executors
pub fn list_compiled_pipelines() -> Vec<PipelineInfo> {
    let cache = get_executor_cache();
    let cache_lock = cache.read().unwrap();
    cache_lock
        .values()
        .map(|p| p.get_info())
        .collect()
}

/// Get pipeline cache stats
pub fn get_pipeline_stats() -> PipelineStats {
    let cache = get_executor_cache();
    let cache_lock = cache.read().unwrap();

    let total_pipelines = cache_lock.len();
    let zero_overhead_count = cache_lock
        .values()
        .filter(|p| p.is_zero_overhead)
        .count();

    PipelineStats {
        total_pipelines,
        zero_overhead_count,
        protected_count: total_pipelines - zero_overhead_count,
        cache_size: total_pipelines,
    }
}

#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub total_pipelines: usize,
    pub zero_overhead_count: usize,
    pub protected_count: usize,
    pub cache_size: usize,
}

impl PipelineStats {
    pub fn optimization_ratio(&self) -> f64 {
        if self.total_pipelines == 0 {
            0.0
        } else {
            ((self.zero_overhead_count as f64) / (self.total_pipelines as f64)) * 100.0
        }
    }
}

//=============================================================================
// PIPELINE CACHE MANAGEMENT
//=============================================================================

/// Clear all pipeline executors (useful for testing)
pub fn clear_pipeline_cache() {
    let cache = get_executor_cache();
    let mut cache_lock = cache.write().unwrap();
    let count = cache_lock.len();
    cache_lock.clear();
    println!("🧹 Pipeline executor cache cleared: {} executors removed", count);
}

/// Recompile pipeline executor (useful for hot reload)
pub fn recompile_pipeline(action_id: &str, config: &mut IO) -> Result<(), String> {
    // Remove existing
    remove_compiled_pipeline(action_id);

    // Recompile
    compile_pipeline(action_id, config)?;

    println!("🔄 Pipeline executor recompiled: {}", action_id);
    Ok(())
}
