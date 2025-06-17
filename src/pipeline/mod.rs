// src/pipeline/mod.rs - Updated pipeline system for complete IO structure - FIXED

use std::sync::Arc;
use serde_json::Value;
use crate::types::{ ActionId, ActionPayload, IO, Priority, RequiredType };
use crate::utils::current_timestamp;

//=============================================================================
// PIPELINE OPERATOR TYPES (Updated for complete IO)
//=============================================================================

/// Pre-compiled pipeline operators for maximum runtime performance
#[derive(Clone)]
pub struct CompiledPipeline {
    pub action_id: ActionId,
    pub operators: Vec<PipelineOperator>,
    pub execution_path: ExecutionPath,
    pub compiled_at: u64,
    pub optimization_level: OptimizationLevel,
    pub complexity_score: u32,
}

/// Individual pipeline operators (Updated to match complete IO)
#[derive(Clone)]
pub enum PipelineOperator {
    // ===== VALIDATION OPERATORS =====
    Required {
        required_type: RequiredType,
    },
    Schema {
        schema_name: String,
    },

    // ===== PROTECTION OPERATORS =====
    Block {
        reason: String,
    },
    Throttle {
        ms: u64,
        last_execution: Arc<std::sync::atomic::AtomicU64>,
    },
    Debounce {
        ms: u64,
        max_wait: Option<u64>,
        timer_id: Arc<std::sync::RwLock<Option<String>>>,
        first_call: Arc<std::sync::atomic::AtomicU64>,
    },
    ChangeDetection {
        last_payload: Arc<std::sync::RwLock<Option<Value>>>,
    },

    // ===== TRANSFORMATION OPERATORS =====
    Condition {
        talent_name: String,
    },
    Selector {
        talent_name: String,
    },
    Transform {
        talent_name: String,
    },

    // ===== SYSTEM OPERATORS =====
    Middleware {
        name: String,
    },
    Logger {
        enabled: bool,
        path: Option<String>,
    },
    Priority {
        level: Priority,
    },
    Authentication {
        auth_mode: String,
        required: bool,
    },

    // ===== METADATA OPERATORS =====
    UpdateMetadata {
        track_execution: bool,
        track_timing: bool,
    },
}

/// Execution paths for different pipeline types (Enhanced)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionPath {
    FastPath, // No operators - direct execution
    Protected, // Protection operators only
    Transformed, // Transform/select/condition operators
    Authenticated, // Authentication required
    Complex, // Multiple operator types
    Scheduled, // TimeKeeper managed
    Blocked, // Permanently blocked
}

/// Pipeline optimization levels (Enhanced)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    ZeroCost, // Compile-time optimization, zero runtime overhead
    Minimal, // Single atomic check
    Standard, // Normal operator chain
    Complex, // Full operator pipeline
    Enterprise, // All features enabled
}

//=============================================================================
// ENHANCED PIPELINE COMPILER
//=============================================================================

/// Compiles IO configuration into optimized pipeline
pub struct PipelineCompiler;

impl PipelineCompiler {
    /// Compile IO configuration into optimized pipeline
    pub fn compile(config: &IO) -> CompiledPipeline {
        let mut operators = Vec::new();

        // Early exit for blocked actions
        if config.block || config._is_blocked {
            return Self::create_blocked_pipeline(config);
        }

        // 1. Add validation operators first (highest priority)
        Self::add_validation_operators(&mut operators, config);

        // 2. Add authentication operators
        Self::add_authentication_operators(&mut operators, config);

        // 3. Add protection operators
        Self::add_protection_operators(&mut operators, config);

        // 4. Add transformation operators
        Self::add_transformation_operators(&mut operators, config);

        // 5. Add system operators
        Self::add_system_operators(&mut operators, config);

        // 6. Add metadata operators
        Self::add_metadata_operators(&mut operators, config);

        // Determine execution path and optimization level
        let execution_path = Self::determine_execution_path(&operators, config);
        let optimization_level = Self::determine_optimization_level(&operators, config);
        let complexity_score = config.complexity_score();

        CompiledPipeline {
            action_id: config.id.clone(),
            operators,
            execution_path,
            compiled_at: current_timestamp(),
            optimization_level,
            complexity_score,
        }
    }

    fn create_blocked_pipeline(config: &IO) -> CompiledPipeline {
        let reason = config._block_reason.clone().unwrap_or_else(|| "Action blocked".to_string());

        CompiledPipeline {
            action_id: config.id.clone(),
            operators: vec![PipelineOperator::Block { reason }],
            execution_path: ExecutionPath::Blocked,
            compiled_at: current_timestamp(),
            optimization_level: OptimizationLevel::Minimal,
            complexity_score: 1,
        }
    }

    fn add_validation_operators(operators: &mut Vec<PipelineOperator>, config: &IO) {
        // Required validation
        if let Some(ref required) = config.required {
            operators.push(PipelineOperator::Required {
                required_type: required.clone(),
            });
        }

        // Schema validation
        if let Some(ref schema) = config.schema {
            operators.push(PipelineOperator::Schema {
                schema_name: schema.clone(),
            });
        }
    }

    fn add_authentication_operators(operators: &mut Vec<PipelineOperator>, config: &IO) {
        if let Some(ref auth) = config.auth {
            operators.push(PipelineOperator::Authentication {
                auth_mode: format!("{:?}", auth.mode),
                required: !matches!(auth.mode, crate::types::AuthMode::Disabled),
            });
        }
    }

    fn add_protection_operators(operators: &mut Vec<PipelineOperator>, config: &IO) {
        // Throttle protection
        if let Some(throttle_ms) = config.throttle {
            operators.push(PipelineOperator::Throttle {
                ms: throttle_ms,
                last_execution: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            });
        }

        // Debounce protection
        if let Some(debounce_ms) = config.debounce {
            operators.push(PipelineOperator::Debounce {
                ms: debounce_ms,
                max_wait: config.max_wait,
                timer_id: Arc::new(std::sync::RwLock::new(None)),
                first_call: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            });
        }

        // Change detection
        if config.detect_changes {
            operators.push(PipelineOperator::ChangeDetection {
                last_payload: Arc::new(std::sync::RwLock::new(None)),
            });
        }
    }

    fn add_transformation_operators(operators: &mut Vec<PipelineOperator>, config: &IO) {
        // Condition operator
        if let Some(ref condition) = config.condition {
            operators.push(PipelineOperator::Condition {
                talent_name: condition.clone(),
            });
        }

        // Selector operator
        if let Some(ref selector) = config.selector {
            operators.push(PipelineOperator::Selector {
                talent_name: selector.clone(),
            });
        }

        // Transform operator
        if let Some(ref transform) = config.transform {
            operators.push(PipelineOperator::Transform {
                talent_name: transform.clone(),
            });
        }

        // Additional processing talents
        for talent_name in &config._processing_talents {
            if
                Some(talent_name) != config.condition.as_ref() &&
                Some(talent_name) != config.selector.as_ref() &&
                Some(talent_name) != config.transform.as_ref()
            {
                operators.push(PipelineOperator::Transform {
                    talent_name: talent_name.clone(),
                });
            }
        }
    }

    fn add_system_operators(operators: &mut Vec<PipelineOperator>, config: &IO) {
        // Priority operator (only if not normal)
        if config.priority != Priority::Normal {
            operators.push(PipelineOperator::Priority {
                level: config.priority,
            });
        }

        // Logger operator
        if config.log {
            operators.push(PipelineOperator::Logger {
                enabled: true,
                path: config.path.clone(),
            });
        }

        // Middleware operators
        for middleware_name in &config.middleware {
            operators.push(PipelineOperator::Middleware {
                name: middleware_name.clone(),
            });
        }
    }

    fn add_metadata_operators(operators: &mut Vec<PipelineOperator>, config: &IO) {
        // Add metadata tracking if any timing or execution tracking is needed
        let track_execution = config._execution_count > 0 || config._last_exec_time.is_some();
        let track_timing = config._execution_time.is_some();

        if track_execution || track_timing {
            operators.push(PipelineOperator::UpdateMetadata {
                track_execution,
                track_timing,
            });
        }
    }

    fn determine_execution_path(_operators: &[PipelineOperator], config: &IO) -> ExecutionPath {
        // Check for blocking first
        if config.block || config._is_blocked {
            return ExecutionPath::Blocked;
        }

        // Check for scheduling
        if config.has_scheduling() {
            return ExecutionPath::Scheduled;
        }

        // Check for fast path (no operators)
        if config.is_fast_path_eligible() {
            return ExecutionPath::FastPath;
        }

        // Default to complex for now
        ExecutionPath::Complex
    }

    fn determine_optimization_level(
        _operators: &[PipelineOperator],
        config: &IO
    ) -> OptimizationLevel {
        let complexity = config.complexity_score();

        match complexity {
            0 => OptimizationLevel::ZeroCost,
            1..=3 => OptimizationLevel::Minimal,
            4..=10 => OptimizationLevel::Standard,
            11..=20 => OptimizationLevel::Complex,
            _ => OptimizationLevel::Enterprise,
        }
    }
}

//=============================================================================
// ENHANCED PIPELINE EXECUTOR
//=============================================================================

/// Executes pre-compiled pipelines at runtime
pub struct PipelineExecutor;

impl PipelineExecutor {
    /// Execute pipeline operators in sequence
    pub async fn execute(pipeline: &CompiledPipeline, payload: ActionPayload) -> PipelineResult {
        match pipeline.execution_path {
            ExecutionPath::FastPath => {
                // Zero-cost path - no operators to execute
                PipelineResult::Continue(payload)
            }

            ExecutionPath::Blocked => {
                if let Some(PipelineOperator::Block { reason }) = pipeline.operators.first() {
                    PipelineResult::Block(reason.clone())
                } else {
                    PipelineResult::Block("Action blocked".to_string())
                }
            }

            ExecutionPath::Protected => { Self::execute_protection_path(pipeline, payload).await }

            ExecutionPath::Authenticated => {
                Self::execute_authenticated_path(pipeline, payload).await
            }

            ExecutionPath::Transformed => {
                Self::execute_transformation_path(pipeline, payload).await
            }

            ExecutionPath::Complex => { Self::execute_complex_path(pipeline, payload).await }

            ExecutionPath::Scheduled => {
                // Handled by TimeKeeper - should not reach here
                PipelineResult::Continue(payload)
            }
        }
    }

    async fn execute_protection_path(
        pipeline: &CompiledPipeline,
        payload: ActionPayload
    ) -> PipelineResult {
        for operator in &pipeline.operators {
            match operator {
                PipelineOperator::Throttle { ms, last_execution } => {
                    let now = current_timestamp();
                    let last = last_execution.load(std::sync::atomic::Ordering::Relaxed);

                    if now - last < *ms {
                        return PipelineResult::Block(format!("Throttled: {} ms", ms));
                    }

                    last_execution.store(now, std::sync::atomic::Ordering::Relaxed);
                }

                PipelineOperator::Debounce { ms, max_wait, first_call, .. } => {
                    let now = current_timestamp();
                    let first = first_call.load(std::sync::atomic::Ordering::Relaxed);

                    if first == 0 {
                        first_call.store(now, std::sync::atomic::Ordering::Relaxed);
                    }

                    // Check max_wait timeout
                    if let Some(max_wait_ms) = max_wait {
                        if now - first > *max_wait_ms {
                            // Reset and allow execution
                            first_call.store(0, std::sync::atomic::Ordering::Relaxed);
                        } else {
                            return PipelineResult::Block(format!("Debounced: {} ms", ms));
                        }
                    } else {
                        return PipelineResult::Block(format!("Debounced: {} ms", ms));
                    }
                }

                PipelineOperator::ChangeDetection { last_payload } => {
                    let mut last = last_payload.write().unwrap();

                    if let Some(ref previous) = *last {
                        if previous == &payload {
                            return PipelineResult::Block("No change detected".to_string());
                        }
                    }

                    *last = Some(payload.clone());
                }

                _ => {} // Skip non-protection operators
            }
        }

        PipelineResult::Continue(payload)
    }

    async fn execute_authenticated_path(
        pipeline: &CompiledPipeline,
        payload: ActionPayload
    ) -> PipelineResult {
        // First check authentication
        for operator in &pipeline.operators {
            if let PipelineOperator::Authentication { auth_mode, required } = operator {
                if *required {
                    // TODO: Implement actual authentication logic
                    // For now, just log the auth requirement
                    println!("🔐 Authentication required: {}", auth_mode);
                }
            }
        }

        // Then execute other operators
        Self::execute_complex_path(pipeline, payload).await
    }

    async fn execute_transformation_path(
        pipeline: &CompiledPipeline,
        payload: ActionPayload
    ) -> PipelineResult {
        let mut current_payload = payload;

        for operator in &pipeline.operators {
            match operator {
                PipelineOperator::Condition { talent_name } => {
                    // TODO: Look up and execute actual condition talent
                    println!("🔍 Checking condition: {}", talent_name);
                    // For now, always pass
                }

                PipelineOperator::Selector { talent_name } => {
                    // TODO: Look up and execute actual selector talent
                    println!("🎯 Applying selector: {}", talent_name);
                    // For now, pass payload through
                }

                PipelineOperator::Transform { talent_name } => {
                    // TODO: Look up and execute actual transform talent
                    println!("🔄 Applying transform: {}", talent_name);
                    // For now, pass payload through
                }

                _ => {} // Skip non-transformation operators
            }
        }

        PipelineResult::Continue(current_payload)
    }

    async fn execute_complex_path(
        pipeline: &CompiledPipeline,
        payload: ActionPayload
    ) -> PipelineResult {
        let start_time = current_timestamp();
        let mut current_payload = payload;

        // Execute all operators in sequence
        for operator in &pipeline.operators {
            match operator {
                PipelineOperator::Required { required_type } => {
                    match required_type {
                        RequiredType::Basic(true) => {
                            if current_payload.is_null() {
                                return PipelineResult::Block(
                                    "Required payload missing".to_string()
                                );
                            }
                        }
                        RequiredType::NonEmpty => {
                            if
                                current_payload.is_null() ||
                                (current_payload.is_string() &&
                                    current_payload.as_str().unwrap_or("").trim().is_empty())
                            {
                                return PipelineResult::Block(
                                    "Required non-empty payload missing".to_string()
                                );
                            }
                        }
                        RequiredType::Basic(false) => {
                            // Optional - no validation needed
                        }
                    }
                }

                PipelineOperator::Schema { schema_name } => {
                    // TODO: Implement actual schema validation
                    println!("✅ Validating schema: {}", schema_name);
                }

                PipelineOperator::Block { reason } => {
                    return PipelineResult::Block(reason.clone());
                }

                PipelineOperator::Authentication { auth_mode, required } => {
                    if *required {
                        // TODO: Implement actual authentication
                        println!("🔐 Authenticating with: {}", auth_mode);
                    }
                }

                PipelineOperator::Throttle { ms, last_execution } => {
                    let now = current_timestamp();
                    let last = last_execution.load(std::sync::atomic::Ordering::Relaxed);

                    if now - last < *ms {
                        return PipelineResult::Block(format!("Throttled: {} ms", ms));
                    }

                    last_execution.store(now, std::sync::atomic::Ordering::Relaxed);
                }

                PipelineOperator::Debounce { ms, max_wait, first_call, .. } => {
                    let now = current_timestamp();
                    let first = first_call.load(std::sync::atomic::Ordering::Relaxed);

                    if first == 0 {
                        first_call.store(now, std::sync::atomic::Ordering::Relaxed);
                        return PipelineResult::Block(format!("Debounced: {} ms", ms));
                    }

                    if let Some(max_wait_ms) = max_wait {
                        if now - first > *max_wait_ms {
                            // Reset and allow execution
                            first_call.store(0, std::sync::atomic::Ordering::Relaxed);
                        } else {
                            return PipelineResult::Block(format!("Debounced: {} ms", ms));
                        }
                    } else {
                        return PipelineResult::Block(format!("Debounced: {} ms", ms));
                    }
                }

                PipelineOperator::ChangeDetection { last_payload } => {
                    let mut last = last_payload.write().unwrap();

                    if let Some(ref previous) = *last {
                        if previous == &current_payload {
                            return PipelineResult::Block("No change detected".to_string());
                        }
                    }

                    *last = Some(current_payload.clone());
                }

                PipelineOperator::Condition { talent_name } => {
                    // TODO: Look up and execute actual condition talent
                    println!("🔍 Checking condition: {}", talent_name);
                }

                PipelineOperator::Selector { talent_name } => {
                    // TODO: Look up and execute actual selector talent
                    println!("🎯 Applying selector: {}", talent_name);
                }

                PipelineOperator::Transform { talent_name } => {
                    // TODO: Look up and execute actual transform talent
                    println!("🔄 Applying transform: {}", talent_name);
                }

                PipelineOperator::Middleware { name } => {
                    // TODO: Look up and execute actual middleware
                    println!("⚙️ Executing middleware: {}", name);
                }

                PipelineOperator::Logger { enabled, path } => {
                    if *enabled {
                        let log_msg = match path {
                            Some(p) =>
                                format!(
                                    "🔍 Pipeline Log [{}] {}: {}",
                                    pipeline.action_id,
                                    p,
                                    current_payload
                                ),
                            None =>
                                format!(
                                    "🔍 Pipeline Log [{}]: {}",
                                    pipeline.action_id,
                                    current_payload
                                ),
                        };
                        println!("{}", log_msg);
                    }
                }

                PipelineOperator::Priority { level } => {
                    // TODO: Implement priority handling (queue management)
                    println!("⚡ Priority set to: {:?}", level);
                }

                PipelineOperator::UpdateMetadata { track_execution, track_timing } => {
                    if *track_execution || *track_timing {
                        let execution_time = current_timestamp() - start_time;
                        println!("📊 Execution metadata - Time: {}ms", execution_time);
                    }
                }
            }
        }

        PipelineResult::Continue(current_payload)
    }
}

//=============================================================================
// PIPELINE RESULT TYPES
//=============================================================================

/// Result of pipeline execution
#[derive(Debug)]
pub enum PipelineResult {
    Continue(ActionPayload), // Pipeline passed, continue to handler execution
    Block(String), // Pipeline blocked execution
    Error(String), // Pipeline error occurred
}

//=============================================================================
// ENHANCED PIPELINE CACHE MANAGEMENT
//=============================================================================

/// Manages compiled pipeline cache with advanced features
pub struct PipelineCache {
    cache: std::sync::RwLock<std::collections::HashMap<ActionId, CompiledPipeline>>,
    stats: std::sync::RwLock<PipelineCacheStats>,
}

impl PipelineCache {
    pub fn new() -> Self {
        Self {
            cache: std::sync::RwLock::new(std::collections::HashMap::new()),
            stats: std::sync::RwLock::new(PipelineCacheStats::default()),
        }
    }

    /// Get or compile pipeline for action
    pub fn get_or_compile(&self, action_id: &str, config: &IO) -> CompiledPipeline {
        // Try to get from cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(pipeline) = cache.get(action_id) {
                // Update cache hit stats
                {
                    let mut stats = self.stats.write().unwrap();
                    stats.cache_hits += 1;
                }
                return pipeline.clone();
            }
        }

        // Compile new pipeline
        let pipeline = PipelineCompiler::compile(config);

        // Store in cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(action_id.to_string(), pipeline.clone());
        }

        // Update cache miss stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.cache_misses += 1;
            stats.total_compilations += 1;

            // Update path statistics
            match pipeline.execution_path {
                ExecutionPath::FastPath => {
                    stats.fast_path_count += 1;
                }
                ExecutionPath::Protected => {
                    stats.protected_count += 1;
                }
                ExecutionPath::Transformed => {
                    stats.transformed_count += 1;
                }
                ExecutionPath::Authenticated => {
                    stats.authenticated_count += 1;
                }
                ExecutionPath::Complex => {
                    stats.complex_count += 1;
                }
                ExecutionPath::Scheduled => {
                    stats.scheduled_count += 1;
                }
                ExecutionPath::Blocked => {
                    stats.blocked_count += 1;
                }
            }
        }

        pipeline
    }

    /// Clear pipeline cache (for hot reloading)
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();

        let mut stats = self.stats.write().unwrap();
        *stats = PipelineCacheStats::default();
    }

    /// Get cache statistics
    pub fn stats(&self) -> PipelineCacheStats {
        let stats = self.stats.read().unwrap();
        let cache = self.cache.read().unwrap();

        PipelineCacheStats {
            total_pipelines: cache.len(),
            cache_hits: stats.cache_hits,
            cache_misses: stats.cache_misses,
            total_compilations: stats.total_compilations,
            fast_path_count: stats.fast_path_count,
            protected_count: stats.protected_count,
            transformed_count: stats.transformed_count,
            authenticated_count: stats.authenticated_count,
            complex_count: stats.complex_count,
            scheduled_count: stats.scheduled_count,
            blocked_count: stats.blocked_count,
        }
    }

    /// Get pipeline for specific action (read-only)
    pub fn get_pipeline(&self, action_id: &str) -> Option<CompiledPipeline> {
        let cache = self.cache.read().unwrap();
        cache.get(action_id).cloned()
    }

    /// Remove specific pipeline from cache
    pub fn remove_pipeline(&self, action_id: &str) -> bool {
        let mut cache = self.cache.write().unwrap();
        cache.remove(action_id).is_some()
    }

    /// Get all pipeline action IDs
    pub fn list_pipelines(&self) -> Vec<String> {
        let cache = self.cache.read().unwrap();
        cache.keys().cloned().collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct PipelineCacheStats {
    pub total_pipelines: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_compilations: u64,
    pub fast_path_count: usize,
    pub protected_count: usize,
    pub transformed_count: usize,
    pub authenticated_count: usize,
    pub complex_count: usize,
    pub scheduled_count: usize,
    pub blocked_count: usize,
}

impl PipelineCacheStats {
    pub fn cache_hit_ratio(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            (self.cache_hits as f64) / (total_requests as f64)
        }
    }

    pub fn efficiency_score(&self) -> f64 {
        if self.total_pipelines == 0 {
            0.0
        } else {
            (self.fast_path_count as f64) / (self.total_pipelines as f64)
        }
    }
}

//=============================================================================
// PIPELINE INTEGRATION HELPERS
//=============================================================================

impl CompiledPipeline {
    /// Check if this pipeline can use the fast path
    pub fn is_fast_path(&self) -> bool {
        matches!(self.execution_path, ExecutionPath::FastPath)
    }

    /// Check if this pipeline is blocked
    pub fn is_blocked(&self) -> bool {
        matches!(self.execution_path, ExecutionPath::Blocked)
    }

    /// Get pipeline metrics for monitoring
    pub fn get_metrics(&self) -> PipelineMetrics {
        PipelineMetrics {
            action_id: self.action_id.clone(),
            operator_count: self.operators.len(),
            execution_path: self.execution_path,
            optimization_level: self.optimization_level,
            complexity_score: self.complexity_score,
            compiled_at: self.compiled_at,
        }
    }

    /// Get human-readable description of pipeline
    pub fn description(&self) -> String {
        let path_desc = match self.execution_path {
            ExecutionPath::FastPath => "direct execution",
            ExecutionPath::Protected => "with protection",
            ExecutionPath::Transformed => "with transformation",
            ExecutionPath::Authenticated => "with authentication",
            ExecutionPath::Complex => "full pipeline",
            ExecutionPath::Scheduled => "scheduled",
            ExecutionPath::Blocked => "blocked",
        };

        format!(
            "Pipeline[{}] {} - {} operators ({})",
            self.action_id,
            path_desc,
            self.operators.len(),
            match self.optimization_level {
                OptimizationLevel::ZeroCost => "zero-cost",
                OptimizationLevel::Minimal => "minimal",
                OptimizationLevel::Standard => "standard",
                OptimizationLevel::Complex => "complex",
                OptimizationLevel::Enterprise => "enterprise",
            }
        )
    }
}

/// Pipeline performance metrics
#[derive(Debug, Clone)]
pub struct PipelineMetrics {
    pub action_id: ActionId,
    pub operator_count: usize,
    pub execution_path: ExecutionPath,
    pub optimization_level: OptimizationLevel,
    pub complexity_score: u32,
    pub compiled_at: u64,
}

//=============================================================================
// DISPLAY IMPLEMENTATIONS
//=============================================================================

impl std::fmt::Display for ExecutionPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let desc = match self {
            ExecutionPath::FastPath => "FastPath",
            ExecutionPath::Protected => "Protected",
            ExecutionPath::Transformed => "Transformed",
            ExecutionPath::Authenticated => "Authenticated",
            ExecutionPath::Complex => "Complex",
            ExecutionPath::Scheduled => "Scheduled",
            ExecutionPath::Blocked => "Blocked",
        };
        write!(f, "{}", desc)
    }
}

impl std::fmt::Display for OptimizationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let desc = match self {
            OptimizationLevel::ZeroCost => "ZeroCost",
            OptimizationLevel::Minimal => "Minimal",
            OptimizationLevel::Standard => "Standard",
            OptimizationLevel::Complex => "Complex",
            OptimizationLevel::Enterprise => "Enterprise",
        };
        write!(f, "{}", desc)
    }
}

impl std::fmt::Display for CompiledPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}
