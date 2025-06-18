// src/pipeline/mod.rs
// File location: src/pipeline/mod.rs
// Pipeline system for enhanced Cyre - FULL RESTORATION

//=============================================================================
// IMPORTS
//=============================================================================

use std::sync::{ Arc, RwLock, OnceLock, Mutex };
use std::collections::HashMap;
use crate::types::{ IO, ActionPayload, ActionId };
use crate::utils::current_timestamp;

//=============================================================================
// PIPELINE EXECUTOR CACHE
//=============================================================================

/// Pipeline executor cache (runtime execution engines)
static EXECUTOR_CACHE: OnceLock<Arc<RwLock<HashMap<ActionId, PipelineExecutor>>>> = OnceLock::new();

fn get_executor_cache() -> &'static Arc<RwLock<HashMap<ActionId, PipelineExecutor>>> {
    EXECUTOR_CACHE.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}

//=============================================================================
// PIPELINE EXECUTOR (RUNTIME EXECUTION ENGINE)
//=============================================================================

/// Runtime pipeline executor - executes compiled operator chains
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
    pub debounce_timer: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,

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
    pub fn from_config(action_id: &str, config: &IO) -> Self {
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
            !has_schema;

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
            debounce_timer: Arc::new(Mutex::new(None)),
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

        // 5. Debounce protection (async)
        if self.has_debounce {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let debounce_ms = self.debounce_ms;
            let payload_clone = payload_after_schema.clone();

            // Cancel previous debounce timer
            {
                let mut timer_guard = self.debounce_timer.lock().unwrap();
                if let Some(handle) = timer_guard.take() {
                    handle.abort();
                }

                // Start new debounce timer
                let new_handle = tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(debounce_ms)).await;
                    let _ = tx.send(payload_clone);
                });

                *timer_guard = Some(new_handle);
            }

            // Wait for debounce to complete
            match rx.await {
                Ok(debounced_payload) => {
                    // Continue with debounced payload
                    return self.continue_pipeline_after_debounce(debounced_payload).await;
                }
                Err(_) => {
                    // Debounce was cancelled by newer request
                    return PipelineResult::Block("Debounced - newer request received".to_string());
                }
            }
        }

        // 6. Change detection
        let payload_after_change = if self.has_change_detection {
            match self.execute_change_detection(&payload_after_schema).await {
                Ok(p) => p,
                Err(e) => {
                    return PipelineResult::Block(format!("Change detection: {}", e));
                }
            }
        } else {
            payload_after_schema
        };

        // 7. Condition operator
        let payload_after_condition = if self.has_condition {
            match self.execute_condition(&payload_after_change).await {
                Ok(p) => p,
                Err(e) => {
                    return PipelineResult::Block(format!("Condition failed: {}", e));
                }
            }
        } else {
            payload_after_change
        };

        // 8. Selector operator
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

        // 9. Transform operator
        let final_payload = if self.has_transform {
            match self.execute_transform(&payload_after_selector).await {
                Ok(p) => p,
                Err(e) => {
                    return PipelineResult::Block(format!("Transform failed: {}", e));
                }
            }
        } else {
            payload_after_selector
        };

        // 10. Scheduling check
        if self.has_scheduling {
            return PipelineResult::Schedule;
        }

        PipelineResult::Continue(final_payload)
    }

    /// Continue pipeline execution after debounce
    async fn continue_pipeline_after_debounce(&self, payload: ActionPayload) -> PipelineResult {
        // Continue with remaining operators after debounce

        // Change detection
        let payload_after_change = if self.has_change_detection {
            match self.execute_change_detection(&payload).await {
                Ok(p) => p,
                Err(e) => {
                    return PipelineResult::Block(format!("Change detection: {}", e));
                }
            }
        } else {
            payload
        };

        // Continue with condition, selector, transform, scheduling...
        // (Same logic as above but starting from change detection)

        let payload_after_condition = if self.has_condition {
            match self.execute_condition(&payload_after_change).await {
                Ok(p) => p,
                Err(e) => {
                    return PipelineResult::Block(format!("Condition failed: {}", e));
                }
            }
        } else {
            payload_after_change
        };

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

        let final_payload = if self.has_transform {
            match self.execute_transform(&payload_after_selector).await {
                Ok(p) => p,
                Err(e) => {
                    return PipelineResult::Block(format!("Transform failed: {}", e));
                }
            }
        } else {
            payload_after_selector
        };

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
            // Basic schema validation - can be enhanced with talent system
            match schema_name.as_str() {
                "non-empty" => {
                    if payload.is_null() {
                        return Err("Schema requires non-empty payload".to_string());
                    }
                }
                "object" => {
                    if !payload.is_object() {
                        return Err("Schema requires object payload".to_string());
                    }
                }
                "array" => {
                    if !payload.is_array() {
                        return Err("Schema requires array payload".to_string());
                    }
                }
                _ => {
                    // Unknown schema - log warning but pass through
                    crate::context::sensor::warn(
                        "pipeline",
                        &format!("Unknown schema '{}' for '{}'", schema_name, self.action_id),
                        false
                    );
                }
            }
        }
        Ok(payload.clone())
    }

    /// Execute change detection operator
    async fn execute_change_detection(
        &self,
        payload: &ActionPayload
    ) -> Result<ActionPayload, String> {
        // For MVP, simple change detection without PayloadStateOps
        // In full implementation, this would use PayloadStateOps to compare with last payload

        // Simple implementation - always consider changed for now
        // TODO: Implement proper payload state tracking
        Ok(payload.clone())
    }

    /// Execute condition operator
    async fn execute_condition(&self, payload: &ActionPayload) -> Result<ActionPayload, String> {
        if let Some(ref condition_talent) = self.condition_talent {
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
                "is_object" => {
                    if payload.is_object() {
                        Ok(payload.clone())
                    } else {
                        Err("Condition requires object".to_string())
                    }
                }
                _ => {
                    crate::context::sensor::warn(
                        "pipeline",
                        &format!(
                            "Unknown condition '{}' for '{}'",
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
            match selector_talent.as_str() {
                "data" => {
                    if let Some(data) = payload.get("data") {
                        Ok(data.clone())
                    } else {
                        Err("Selector 'data' field not found".to_string())
                    }
                }
                "user" => {
                    if let Some(user) = payload.get("user") {
                        Ok(user.clone())
                    } else {
                        Err("Selector 'user' field not found".to_string())
                    }
                }
                _ => {
                    crate::context::sensor::warn(
                        "pipeline",
                        &format!("Unknown selector '{}' for '{}'", selector_talent, self.action_id),
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
            match transform_talent.as_str() {
                "uppercase" => {
                    if let Some(text) = payload.as_str() {
                        Ok(serde_json::Value::String(text.to_uppercase()))
                    } else {
                        Ok(payload.clone())
                    }
                }
                "add_timestamp" => {
                    let mut result = payload.clone();
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert(
                            "transformed_at".to_string(),
                            serde_json::Value::Number(current_timestamp().into())
                        );
                    }
                    Ok(result)
                }
                "normalize" => {
                    // Simple normalization
                    if let Some(obj) = payload.as_object() {
                        let mut normalized = serde_json::Map::new();
                        for (key, value) in obj {
                            normalized.insert(key.to_lowercase(), value.clone());
                        }
                        Ok(serde_json::Value::Object(normalized))
                    } else {
                        Ok(payload.clone())
                    }
                }
                _ => {
                    crate::context::sensor::warn(
                        "pipeline",
                        &format!(
                            "Unknown transform '{}' for '{}'",
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
// PIPELINE STATS
//=============================================================================

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
// PIPELINE COMPILER FUNCTIONS
//=============================================================================

/// Compile pipeline for action
pub fn compile_pipeline(action_id: &str, config: &mut IO) -> Result<(), String> {
    // Validate config for pipeline conflicts
    if config.throttle.is_some() && config.debounce.is_some() {
        return Err(
            "Cannot compile pipeline: Throttle and debounce are mutually exclusive".to_string()
        );
    }

    // Build pipeline operators in execution order
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
            cache_lock.insert(action_id.to_string(), executor);
            Ok(())
        }
        Err(_) => Err("Failed to store pipeline executor".to_string()),
    }
}

/// Get compiled pipeline
pub fn get_compiled_pipeline(action_id: &str) -> Option<PipelineExecutor> {
    let cache = get_executor_cache();
    cache.read().ok()?.get(action_id).cloned()
}

/// Remove compiled pipeline
pub fn remove_compiled_pipeline(action_id: &str) {
    let cache = get_executor_cache();
    if let Ok(mut cache_lock) = cache.write() {
        cache_lock.remove(action_id);
    }
}

/// Execute pipeline
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
    let cache_lock = cache.read().ok()?;
    cache_lock.get(action_id).map(|p| p.get_info())
}

/// List all pipeline executors
pub fn list_compiled_pipelines() -> Vec<PipelineInfo> {
    let cache = get_executor_cache();
    if let Ok(cache_lock) = cache.read() {
        cache_lock
            .values()
            .map(|p| p.get_info())
            .collect()
    } else {
        Vec::new()
    }
}

/// Get pipeline stats
pub fn get_pipeline_stats() -> PipelineStats {
    let cache = get_executor_cache();
    if let Ok(cache_lock) = cache.read() {
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
    } else {
        PipelineStats {
            total_pipelines: 0,
            zero_overhead_count: 0,
            protected_count: 0,
            cache_size: 0,
        }
    }
}

/// Clear pipeline cache
pub fn clear_pipeline_cache() {
    let cache = get_executor_cache();
    if let Ok(mut cache_lock) = cache.write() {
        cache_lock.clear();
    }
}

/// Recompile pipeline
pub fn recompile_pipeline(action_id: &str, config: &mut IO) -> Result<(), String> {
    remove_compiled_pipeline(action_id);
    compile_pipeline(action_id, config)
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::IO;
    use serde_json::json;

    #[test]
    fn test_zero_overhead_pipeline() {
        clear_pipeline_cache();

        let mut config = IO::new("test-zero-overhead");
        // No protection, processing, or scheduling

        let result = compile_pipeline("test-zero-overhead", &mut config);
        assert!(result.is_ok());

        let executor = get_compiled_pipeline("test-zero-overhead");
        assert!(executor.is_some());

        let executor = executor.unwrap();
        assert!(executor.is_zero_overhead);
        assert!(!executor.has_throttle);
        assert!(!executor.has_debounce);
        assert!(!executor.has_condition);
    }

    #[test]
    fn test_protected_pipeline() {
        clear_pipeline_cache();

        let mut config = IO::new("test-protected");
        config.throttle = Some(1000);
        config.required = Some(crate::types::RequiredType::Basic(true));

        let result = compile_pipeline("test-protected", &mut config);
        assert!(result.is_ok());

        let executor = get_compiled_pipeline("test-protected");
        assert!(executor.is_some());

        let executor = executor.unwrap();
        assert!(!executor.is_zero_overhead); // Has operators
        assert!(executor.has_throttle);
        assert!(executor.has_required);
        assert_eq!(executor.throttle_ms, 1000);
    }

    #[tokio::test]
    async fn test_pipeline_execution() {
        clear_pipeline_cache();

        // Test zero overhead
        let mut config1 = IO::new("test-fast");
        let _ = compile_pipeline("test-fast", &mut config1);

        let payload = json!({"test": "data"});
        let result = execute_pipeline("test-fast", payload.clone()).await;

        assert!(result.is_ok());
        match result.unwrap() {
            PipelineResult::Continue(returned_payload) => {
                assert_eq!(returned_payload, payload);
            }
            _ => panic!("Expected Continue result for zero overhead"),
        }

        // Test blocked pipeline
        let mut config2 = IO::new("test-blocked");
        config2.block = true;
        let _ = compile_pipeline("test-blocked", &mut config2);

        let result = execute_pipeline("test-blocked", payload).await;
        assert!(result.is_ok());
        match result.unwrap() {
            PipelineResult::Block(_) => (), // Expected
            _ => panic!("Expected Block result for blocked pipeline"),
        }
    }

    #[tokio::test]
    async fn test_throttle_operator() {
        clear_pipeline_cache();

        let mut config = IO::new("test-throttle");
        config.throttle = Some(100); // 100ms throttle
        let _ = compile_pipeline("test-throttle", &mut config);

        let payload = json!({"test": true});

        // First call should succeed
        let result1 = execute_pipeline("test-throttle", payload.clone()).await;
        assert!(result1.is_ok());
        match result1.unwrap() {
            PipelineResult::Continue(_) => (), // Expected
            _ => panic!("First call should succeed"),
        }

        // Second call should be throttled
        let result2 = execute_pipeline("test-throttle", payload.clone()).await;
        assert!(result2.is_ok());
        match result2.unwrap() {
            PipelineResult::Block(reason) => {
                assert!(reason.contains("Throttled"));
            }
            _ => panic!("Second call should be throttled"),
        }
    }

    #[test]
    fn test_pipeline_stats() {
        clear_pipeline_cache();

        // Add zero overhead pipeline
        let mut config1 = IO::new("test-stats-1");
        let _ = compile_pipeline("test-stats-1", &mut config1);

        // Add protected pipeline
        let mut config2 = IO::new("test-stats-2");
        config2.throttle = Some(1000);
        let _ = compile_pipeline("test-stats-2", &mut config2);

        let stats = get_pipeline_stats();
        assert_eq!(stats.total_pipelines, 2);
        assert_eq!(stats.zero_overhead_count, 1);
        assert_eq!(stats.protected_count, 1);
        assert!(stats.optimization_ratio() > 0.0);
    }

    #[tokio::test]
    async fn test_condition_operator() {
        clear_pipeline_cache();

        let mut config = IO::new("test-condition");
        config.condition = Some("has_data".to_string());
        let _ = compile_pipeline("test-condition", &mut config);

        // Test with data - should pass
        let payload_with_data = json!({"some": "data"});
        let result1 = execute_pipeline("test-condition", payload_with_data).await;
        assert!(result1.is_ok());
        match result1.unwrap() {
            PipelineResult::Continue(_) => (), // Expected
            _ => panic!("Should pass condition with data"),
        }

        // Test with null - should block
        let payload_null = serde_json::Value::Null;
        let result2 = execute_pipeline("test-condition", payload_null).await;
        assert!(result2.is_ok());
        match result2.unwrap() {
            PipelineResult::Block(reason) => {
                assert!(reason.contains("Condition"));
            }
            _ => panic!("Should block condition with null"),
        }
    }

    #[tokio::test]
    async fn test_transform_operator() {
        clear_pipeline_cache();

        let mut config = IO::new("test-transform");
        config.transform = Some("add_timestamp".to_string());
        let _ = compile_pipeline("test-transform", &mut config);

        let payload = json!({"original": "data"});
        let result = execute_pipeline("test-transform", payload).await;

        assert!(result.is_ok());
        match result.unwrap() {
            PipelineResult::Continue(transformed_payload) => {
                assert!(transformed_payload.get("original").is_some());
                assert!(transformed_payload.get("transformed_at").is_some());
            }
            _ => panic!("Transform should succeed"),
        }
    }
}
