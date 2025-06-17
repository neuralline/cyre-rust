// src/pipeline/mod.rs
// Simple pipeline compiler - compile on creation, execute on runtime

use std::sync::{ Arc, RwLock, OnceLock };
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::oneshot;
use parking_lot::RwLock as FastRwLock;

use crate::types::{ ActionId, ActionPayload, IO };
use crate::utils::current_timestamp;

//=============================================================================
// COMPILED PIPELINE CACHE
//=============================================================================

/// Pipeline cache (like a compiled code cache)
static PIPELINE_CACHE: OnceLock<Arc<RwLock<HashMap<ActionId, CompiledPipeline>>>> = OnceLock::new();

fn get_pipeline_cache() -> &'static Arc<RwLock<HashMap<ActionId, CompiledPipeline>>> {
    PIPELINE_CACHE.get_or_init(|| { Arc::new(RwLock::new(HashMap::new())) })
}

//=============================================================================
// COMPILED PIPELINE
//=============================================================================

/// Pre-compiled pipeline for ultra-fast execution
#[derive(Clone)]
pub struct CompiledPipeline {
    pub action_id: ActionId,
    pub is_zero_overhead: bool,

    // Protection checks (compiled as simple flags)
    pub has_block: bool,
    pub block_reason: String,

    pub has_throttle: bool,
    pub throttle_ms: u64,
    pub throttle_last: Arc<std::sync::atomic::AtomicU64>,

    pub has_debounce: bool,
    pub debounce_ms: u64,
    pub debounce_timer: Arc<FastRwLock<Option<tokio::task::JoinHandle<()>>>>,

    pub has_required: bool,

    pub has_scheduling: bool,

    pub compiled_at: u64,
}

impl CompiledPipeline {
    /// Compile pipeline from IO configuration
    fn from_config(action_id: &str, config: &IO) -> Self {
        let has_throttle = config.throttle.is_some();
        let has_debounce = config.debounce.is_some();
        let has_required = config.required.is_some();
        let has_block = config.block;
        let has_scheduling = config.delay.is_some() || config.interval.is_some();

        // Zero overhead = no protections, no scheduling, no processing
        let is_zero_overhead =
            !has_throttle &&
            !has_debounce &&
            !has_required &&
            !has_block &&
            !has_scheduling &&
            !config.log &&
            config.transform.is_none() &&
            config.condition.is_none();

        CompiledPipeline {
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
            has_scheduling,
            compiled_at: current_timestamp(),
        }
    }

    /// Execute compiled pipeline
    #[inline(always)]
    pub async fn execute(&self, payload: ActionPayload) -> PipelineResult {
        // ZERO OVERHEAD FAST PATH
        if self.is_zero_overhead {
            return PipelineResult::Continue(payload);
        }

        // COMPILED PROTECTION CHECKS

        // 1. Block check (fastest)
        if self.has_block {
            return PipelineResult::Block(self.block_reason.clone());
        }

        // 2. Scheduling check
        if self.has_scheduling {
            return PipelineResult::Schedule;
        }

        // 3. Throttle check
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

        // 4. Required check
        if self.has_required && payload.is_null() {
            return PipelineResult::Block("Required payload missing".to_string());
        }

        // 5. Debounce check (async)
        if self.has_debounce {
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
                Ok(_) => PipelineResult::Continue(payload),
                Err(_) => PipelineResult::Block("Debounced (cancelled)".to_string()),
            }
        } else {
            PipelineResult::Continue(payload)
        }
    }

    /// Get pipeline info
    pub fn get_info(&self) -> PipelineInfo {
        PipelineInfo {
            action_id: self.action_id.clone(),
            is_zero_overhead: self.is_zero_overhead,
            protection_count: self.count_protections(),
            compiled_at: self.compiled_at,
        }
    }

    fn count_protections(&self) -> u32 {
        let mut count = 0;
        if self.has_block {
            count += 1;
        }
        if self.has_throttle {
            count += 1;
        }
        if self.has_debounce {
            count += 1;
        }
        if self.has_required {
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

/// Compile pipeline for channel (called by channel manager)
pub fn compile_pipeline(action_id: &str, config: &IO) -> Result<(), String> {
    // Validate config for pipeline conflicts
    if config.throttle.is_some() && config.debounce.is_some() {
        return Err(
            "Cannot compile pipeline: Throttle and debounce are mutually exclusive".to_string()
        );
    }

    // Compile pipeline
    let pipeline = CompiledPipeline::from_config(action_id, config);

    // Store in cache
    let cache = get_pipeline_cache();
    let mut cache_lock = cache.write().unwrap();
    cache_lock.insert(action_id.to_string(), pipeline.clone());

    println!("🔧 Pipeline compiled: {} ({})", action_id, if pipeline.is_zero_overhead {
        "zero_overhead"
    } else {
        "protected"
    });

    Ok(())
}

/// Get compiled pipeline (called by execution engine)
pub fn get_compiled_pipeline(action_id: &str) -> Option<CompiledPipeline> {
    let cache = get_pipeline_cache();
    let cache_lock = cache.read().unwrap();
    cache_lock.get(action_id).cloned()
}

/// Remove compiled pipeline (called by channel manager on forget)
pub fn remove_compiled_pipeline(action_id: &str) {
    let cache = get_pipeline_cache();
    let mut cache_lock = cache.write().unwrap();
    if cache_lock.remove(action_id).is_some() {
        println!("🗑️ Pipeline removed: {}", action_id);
    }
}

/// Execute pipeline (called by execution engine)
pub async fn execute_pipeline(
    action_id: &str,
    payload: ActionPayload
) -> Result<PipelineResult, String> {
    let pipeline = get_compiled_pipeline(action_id).ok_or_else(||
        format!("Pipeline not found for action '{}'", action_id)
    )?;

    Ok(pipeline.execute(payload).await)
}

/// Get pipeline info
pub fn get_pipeline_info(action_id: &str) -> Option<PipelineInfo> {
    let cache = get_pipeline_cache();
    let cache_lock = cache.read().unwrap();
    cache_lock.get(action_id).map(|p| p.get_info())
}

/// List all compiled pipelines
pub fn list_compiled_pipelines() -> Vec<PipelineInfo> {
    let cache = get_pipeline_cache();
    let cache_lock = cache.read().unwrap();
    cache_lock
        .values()
        .map(|p| p.get_info())
        .collect()
}

/// Get pipeline cache stats
pub fn get_pipeline_stats() -> PipelineStats {
    let cache = get_pipeline_cache();
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

/// Clear all compiled pipelines (useful for testing)
pub fn clear_pipeline_cache() {
    let cache = get_pipeline_cache();
    let mut cache_lock = cache.write().unwrap();
    let count = cache_lock.len();
    cache_lock.clear();
    println!("🧹 Pipeline cache cleared: {} pipelines removed", count);
}

/// Recompile pipeline (useful for hot reload)
pub fn recompile_pipeline(action_id: &str, config: &IO) -> Result<(), String> {
    // Remove existing
    remove_compiled_pipeline(action_id);

    // Recompile
    compile_pipeline(action_id, config)?;

    println!("🔄 Pipeline recompiled: {}", action_id);
    Ok(())
}

/// Batch compile pipelines
pub fn batch_compile_pipelines(configs: Vec<(String, IO)>) -> Vec<Result<(), String>> {
    configs
        .into_iter()
        .map(|(action_id, config)| compile_pipeline(&action_id, &config))
        .collect()
}
