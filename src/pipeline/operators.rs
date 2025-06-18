// src/pipeline/operators.rs
// File location: src/pipeline/operators.rs
// Fixed pipeline operators using enum instead of trait objects

//=============================================================================
// IMPORTS
//=============================================================================

use crate::types::{ ActionPayload, IO, RequiredType };
use crate::utils::current_timestamp;
use std::sync::atomic::{ AtomicU64, Ordering };
use std::sync::Mutex;
use std::time::{ Duration, Instant };
use tokio::task::JoinHandle;

//=============================================================================
// OPERATOR RESULT ENUM
//=============================================================================

#[derive(Debug)]
pub enum OperatorResult {
    Continue(ActionPayload), // Keep going in pipeline
    Block(String), // Stop pipeline with error
    Defer(ActionPayload, Duration), // Pause pipeline, resume later
    Schedule(ActionPayload, ScheduleConfig), // Schedule future pipeline execution
}

#[derive(Debug, Clone)]
pub struct ScheduleConfig {
    pub delay: Option<u64>,
    pub interval: Option<u64>,
    pub repeat: Option<u32>,
}

//=============================================================================
// OPERATOR ENUM - Instead of trait objects!
//=============================================================================

#[derive(Debug)]
pub enum Operator {
    Block(BlockOperator),
    Throttle(ThrottleOperator),
    Debounce(DebounceOperator),
    Required(RequiredOperator),
    Schema(SchemaOperator),
    Condition(ConditionOperator),
    Selector(SelectorOperator),
    Transform(TransformOperator),
    Schedule(ScheduleOperator),
}

impl Operator {
    pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
        match self {
            Operator::Block(op) => op.process(payload).await,
            Operator::Throttle(op) => op.process(payload).await,
            Operator::Debounce(op) => op.process(payload).await,
            Operator::Required(op) => op.process(payload).await,
            Operator::Schema(op) => op.process(payload).await,
            Operator::Condition(op) => op.process(payload).await,
            Operator::Selector(op) => op.process(payload).await,
            Operator::Transform(op) => op.process(payload).await,
            Operator::Schedule(op) => op.process(payload).await,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Operator::Block(_) => "block",
            Operator::Throttle(_) => "throttle",
            Operator::Debounce(_) => "debounce",
            Operator::Required(_) => "required",
            Operator::Schema(_) => "schema",
            Operator::Condition(_) => "condition",
            Operator::Selector(_) => "selector",
            Operator::Transform(_) => "transform",
            Operator::Schedule(_) => "schedule",
        }
    }
}

//=============================================================================
// PROTECTION OPERATORS (First in pipeline)
//=============================================================================

#[derive(Debug)]
pub struct BlockOperator {
    blocked: bool,
}

impl BlockOperator {
    pub fn new() -> Self {
        Self { blocked: true }
    }

    pub async fn process(&self, _payload: ActionPayload) -> OperatorResult {
        OperatorResult::Block("Action is blocked".to_string())
    }
}

#[derive(Debug)]
pub struct ThrottleOperator {
    ms: u64,
    last_execution: AtomicU64,
}

impl ThrottleOperator {
    pub fn new(ms: u64) -> Self {
        Self {
            ms,
            last_execution: AtomicU64::new(0),
        }
    }

    pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
        let now = current_timestamp();
        let last = self.last_execution.load(Ordering::Relaxed);

        if now - last < self.ms {
            OperatorResult::Block(format!("Throttled: {}ms remaining", self.ms - (now - last)))
        } else {
            self.last_execution.store(now, Ordering::Relaxed);
            OperatorResult::Continue(payload)
        }
    }
}

#[derive(Debug)]
pub struct DebounceOperator {
    ms: u64,
    last_call: Mutex<Option<Instant>>,
    pending_timer: Mutex<Option<JoinHandle<()>>>,
}

impl DebounceOperator {
    pub fn new(ms: u64) -> Self {
        Self {
            ms,
            last_call: Mutex::new(None),
            pending_timer: Mutex::new(None),
        }
    }

    pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
        let now = Instant::now();

        // Cancel previous timer if exists
        {
            let mut timer_guard = self.pending_timer.lock().unwrap();
            if let Some(handle) = timer_guard.take() {
                handle.abort();
            }
        }

        // Check if we're within debounce window
        {
            let mut last_guard = self.last_call.lock().unwrap();
            if let Some(last) = *last_guard {
                if now.duration_since(last) < Duration::from_millis(self.ms) {
                    *last_guard = Some(now);
                    return OperatorResult::Defer(payload, Duration::from_millis(self.ms));
                }
            }
            *last_guard = Some(now);
        }

        OperatorResult::Continue(payload)
    }
}

//=============================================================================
// VALIDATION OPERATORS
//=============================================================================

#[derive(Debug)]
pub struct RequiredOperator;

impl RequiredOperator {
    pub fn new() -> Self {
        Self
    }

    pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
        if payload.is_null() {
            OperatorResult::Block("Required payload missing".to_string())
        } else {
            OperatorResult::Continue(payload)
        }
    }
}

#[derive(Debug)]
pub struct SchemaOperator {
    schema_name: String,
}

impl SchemaOperator {
    pub fn new(schema_name: &str) -> Self {
        Self {
            schema_name: schema_name.to_string(),
        }
    }

    pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
        // TODO: Implement actual schema validation with talent system
        OperatorResult::Continue(payload)
    }
}

//=============================================================================
// PROCESSING OPERATORS
//=============================================================================

#[derive(Debug)]
pub struct ConditionOperator {
    talent_name: String,
}

impl ConditionOperator {
    pub fn new(talent_name: &str) -> Self {
        Self {
            talent_name: talent_name.to_string(),
        }
    }

    pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
        // TODO: Implement talent system integration
        OperatorResult::Continue(payload)
    }
}

#[derive(Debug)]
pub struct SelectorOperator {
    talent_name: String,
}

impl SelectorOperator {
    pub fn new(talent_name: &str) -> Self {
        Self {
            talent_name: talent_name.to_string(),
        }
    }

    pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
        // TODO: Implement talent system integration
        OperatorResult::Continue(payload)
    }
}

#[derive(Debug)]
pub struct TransformOperator {
    talent_name: String,
}

impl TransformOperator {
    pub fn new(talent_name: &str) -> Self {
        Self {
            talent_name: talent_name.to_string(),
        }
    }

    pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
        // TODO: Implement talent system integration
        OperatorResult::Continue(payload)
    }
}

//=============================================================================
// SCHEDULE OPERATOR (Last in pipeline)
//=============================================================================

#[derive(Debug)]
pub struct ScheduleOperator {
    delay: Option<u64>,
    interval: Option<u64>,
    repeat: Option<u32>,
}

impl ScheduleOperator {
    pub fn new(delay: Option<u64>, interval: Option<u64>, repeat: Option<u32>) -> Self {
        Self { delay, interval, repeat }
    }

    pub fn get_schedule_config(&self) -> ScheduleConfig {
        ScheduleConfig {
            delay: self.delay,
            interval: self.interval,
            repeat: self.repeat,
        }
    }

    pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
        // Schedule future pipeline execution
        OperatorResult::Schedule(payload, self.get_schedule_config())
    }
}

//=============================================================================
// PIPELINE TYPE - USES OPERATOR ENUM
//=============================================================================

pub struct Pipeline {
    operators: Vec<Operator>,
}

impl Pipeline {
    pub fn new(operators: Vec<Operator>) -> Self {
        Self { operators }
    }

    pub fn len(&self) -> usize {
        self.operators.len()
    }

    pub fn is_empty(&self) -> bool {
        self.operators.is_empty()
    }

    pub fn operator_names(&self) -> Vec<&'static str> {
        self.operators
            .iter()
            .map(|op| op.name())
            .collect()
    }

    pub async fn process(&self, mut payload: ActionPayload) -> Result<ActionPayload, String> {
        // FAST PATH: Empty array = zero overhead!
        if self.is_empty() {
            return Ok(payload); // 1.8M ops/sec target! ⚡
        }

        // PIPELINE PATH: Just loop over array!
        for operator in &self.operators {
            match operator.process(payload).await {
                OperatorResult::Continue(new_payload) => {
                    payload = new_payload; // Keep going
                }
                OperatorResult::Block(error) => {
                    return Err(error); // Stop pipeline
                }
                OperatorResult::Defer(deferred_payload, _delay) => {
                    // TODO: TimeKeeper handles deferred execution
                    return Ok(deferred_payload);
                }
                OperatorResult::Schedule(scheduled_payload, _schedule) => {
                    // TODO: TimeKeeper handles scheduled execution
                    return Ok(scheduled_payload);
                }
            }
        }

        Ok(payload) // All operators passed
    }
}

//=============================================================================
// PIPELINE EXECUTION RESULT
//=============================================================================

#[derive(Debug)]
pub enum PipelineResult {
    Continue(ActionPayload),
    Block(String),
    Schedule,
}

//=============================================================================
// PIPELINE COMPILER - Creates operator array (ENUM VARIANT!)
//=============================================================================

/// Compile IO configuration into array of operators
pub fn compile_pipeline(config: &mut IO) -> Result<(), String> {
    let mut operators: Vec<Operator> = Vec::new();

    // 1. PROTECTION TALENTS (First - Block Early)
    if config.block {
        operators.push(Operator::Block(BlockOperator::new()));
    }

    if let Some(throttle_ms) = config.throttle {
        operators.push(Operator::Throttle(ThrottleOperator::new(throttle_ms)));
    }

    if let Some(debounce_ms) = config.debounce {
        operators.push(Operator::Debounce(DebounceOperator::new(debounce_ms)));
    }

    // 2. VALIDATION TALENTS
    // Fix: Handle RequiredType properly
    if let Some(required) = &config.required {
        match required {
            RequiredType::Basic(true) | RequiredType::NonEmpty => {
                operators.push(Operator::Required(RequiredOperator::new()));
            }
            RequiredType::Basic(false) => {
                // Not required - skip
            }
        }
    }

    if let Some(schema) = &config.schema {
        operators.push(Operator::Schema(SchemaOperator::new(schema)));
    }

    // 3. PROCESSING TALENTS
    if let Some(condition) = &config.condition {
        operators.push(Operator::Condition(ConditionOperator::new(condition)));
    }

    if let Some(selector) = &config.selector {
        operators.push(Operator::Selector(SelectorOperator::new(selector)));
    }

    if let Some(transform) = &config.transform {
        operators.push(Operator::Transform(TransformOperator::new(transform)));
    }

    // 4. SCHEDULE TALENTS (Last - Routes to TimeKeeper)
    if config.delay.is_some() || config.interval.is_some() || config.repeat.is_some() {
        // Fix: Handle RepeatType properly
        let repeat_count = match &config.repeat {
            Some(crate::types::RepeatType::Count(count)) => Some(*count),
            Some(crate::types::RepeatType::Infinite) => Some(0), // 0 = infinite
            None => None,
        };

        operators.push(
            Operator::Schedule(ScheduleOperator::new(config.delay, config.interval, repeat_count))
        );
    }

    // Store compiled pipeline in the action
    config._pipeline = operators
        .iter()
        .map(|op| op.name().to_string())
        .collect();
    config._has_fast_path = operators.is_empty();

    Ok(())
}

/// Execute compiled pipeline - using action's compiled pipeline
pub async fn execute_pipeline(
    action: &mut IO,
    payload: ActionPayload
) -> Result<PipelineResult, String> {
    // Check if this action has fast path
    if action._has_fast_path {
        return Ok(PipelineResult::Continue(payload)); // Fast path! ⚡
    }

    // For now, we need to recompile from config
    // In a more optimized version, we'd store the compiled operators
    let mut operators: Vec<Operator> = Vec::new();

    // Rebuild operators from config (simplified for now)
    if action.block {
        operators.push(Operator::Block(BlockOperator::new()));
    }

    if let Some(throttle_ms) = action.throttle {
        operators.push(Operator::Throttle(ThrottleOperator::new(throttle_ms)));
    }

    if let Some(debounce_ms) = action.debounce {
        operators.push(Operator::Debounce(DebounceOperator::new(debounce_ms)));
    }

    if let Some(required) = &action.required {
        match required {
            RequiredType::Basic(true) | RequiredType::NonEmpty => {
                operators.push(Operator::Required(RequiredOperator::new()));
            }
            RequiredType::Basic(false) => {
                // Not required - skip
            }
        }
    }

    // Execute pipeline
    let pipeline = Pipeline::new(operators);
    match pipeline.process(payload).await {
        Ok(result_payload) => Ok(PipelineResult::Continue(result_payload)),
        Err(error) => Ok(PipelineResult::Block(error)),
    }
}

//=============================================================================
// CONVENIENCE FUNCTIONS
//=============================================================================

/// Check if configuration will result in fast path
pub fn is_fast_path(config: &IO) -> bool {
    !config.block &&
        config.throttle.is_none() &&
        config.debounce.is_none() &&
        !matches!(
            config.required,
            Some(RequiredType::Basic(true)) | Some(RequiredType::NonEmpty)
        ) &&
        config.schema.is_none() &&
        config.condition.is_none() &&
        config.selector.is_none() &&
        config.transform.is_none() &&
        config.delay.is_none() &&
        config.interval.is_none() &&
        config.repeat.is_none()
}

/// Get estimated pipeline length
pub fn estimate_operator_count(config: &IO) -> usize {
    let mut count = 0;

    // Protection operators
    if config.block {
        count += 1;
    }
    if config.throttle.is_some() {
        count += 1;
    }
    if config.debounce.is_some() {
        count += 1;
    }

    // Validation operators
    if matches!(config.required, Some(RequiredType::Basic(true)) | Some(RequiredType::NonEmpty)) {
        count += 1;
    }
    if config.schema.is_some() {
        count += 1;
    }

    // Processing operators
    if config.condition.is_some() {
        count += 1;
    }
    if config.selector.is_some() {
        count += 1;
    }
    if config.transform.is_some() {
        count += 1;
    }

    // Schedule operator
    if config.delay.is_some() || config.interval.is_some() || config.repeat.is_some() {
        count += 1;
    }

    count
}

/// Get estimated performance based on operator count
pub fn estimated_performance(config: &IO) -> u32 {
    let operator_count = estimate_operator_count(config);

    match operator_count {
        0 => 1_800_000, // Fast path
        1..=2 => 1_200_000, // Lightweight
        3..=5 => 800_000, // Standard
        _ => 400_000, // Complex
    }
}

//=============================================================================
// PIPELINE STATS
//=============================================================================

pub struct PipelineStats {
    pub total_pipelines: usize,
    pub zero_overhead_count: usize,
    pub protected_count: usize,
    pub cache_size: usize,
}

impl PipelineStats {
    pub fn optimization_ratio(&self) -> f64 {
        if self.total_pipelines == 0 {
            return 0.0;
        }
        ((self.zero_overhead_count as f64) / (self.total_pipelines as f64)) * 100.0
    }
}

pub fn get_pipeline_stats() -> PipelineStats {
    // For now, return mock stats
    // In real implementation, would track actual pipeline statistics
    PipelineStats {
        total_pipelines: 0,
        zero_overhead_count: 0,
        protected_count: 0,
        cache_size: 0,
    }
}

pub struct PipelineInfo {
    pub action_id: String,
    pub operator_count: usize,
    pub is_fast_path: bool,
    pub operator_names: Vec<String>,
}

pub fn get_pipeline_info(action_id: &str) -> Option<PipelineInfo> {
    // For now, return None
    // In real implementation, would look up action and return its pipeline info
    None
}

pub fn list_compiled_pipelines() -> Vec<PipelineInfo> {
    // For now, return empty
    // In real implementation, would list all compiled pipelines
    Vec::new()
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_fast_path_compilation() {
        let config = IO::new("fast");
        assert!(is_fast_path(&config));

        let operators: Vec<Operator> = Vec::new();
        let pipeline = Pipeline::new(operators);
        assert_eq!(pipeline.len(), 0);
        assert!(pipeline.is_empty());

        // Fast path should process payload without overhead
        let payload = json!({"test": "fast"});
        let result = pipeline.process(payload.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), payload);
    }

    #[tokio::test]
    async fn test_pipeline_compilation() {
        let config = IO::new("pipeline").with_throttle(100).with_required(true);

        assert!(!is_fast_path(&config));
        assert_eq!(estimate_operator_count(&config), 2);

        // Test that we can build operators
        let mut operators = Vec::new();
        operators.push(Operator::Throttle(ThrottleOperator::new(100)));
        operators.push(Operator::Required(RequiredOperator::new()));

        let pipeline = Pipeline::new(operators);
        assert_eq!(pipeline.len(), 2);

        let names = pipeline.operator_names();
        // Verify correct ordering
        assert_eq!(names[0], "throttle"); // Protection first
        assert_eq!(names[1], "required"); // Validation second
    }

    #[tokio::test]
    async fn test_operator_execution() {
        let mut operators = Vec::new();
        operators.push(Operator::Required(RequiredOperator::new()));
        let pipeline = Pipeline::new(operators);

        // Test with valid payload
        let valid_payload = json!({"data": "test"});
        let result = pipeline.process(valid_payload.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), valid_payload);

        // Test with null payload (should be blocked)
        let null_payload = json!(null);
        let result = pipeline.process(null_payload).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Required payload missing"));
    }
}
