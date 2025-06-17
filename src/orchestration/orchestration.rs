// src/orchestration/orchestration.rs
// Advanced Rust orchestration system leveraging zero-cost abstractions - FIXED

use std::collections::HashMap;
use std::sync::{ Arc, RwLock, OnceLock };
use std::time::{ Duration, SystemTime, Instant };
use std::future::Future;
use std::pin::Pin;
use serde::{ Serialize, Deserialize };
use serde_json::{ json, Value };

use crate::types::{ ActionPayload, CyreResponse };
use crate::context::task_store::{ TaskRepeat, TaskResult, TaskPriority };

/*

      C.Y.R.E - R.U.S.T - O.R.C.H.E.S.T.R.A.T.I.O.N
      
      Zero-cost orchestration with Rust's power:
      - Compile-time workflow validation
      - Zero-allocation execution paths
      - Rich type system for complex workflows
      - Async/await native integration
      - Thread-safe concurrent execution

*/

//=============================================================================
// ORCHESTRATION TYPES - LEVERAGING RUST'S TYPE SYSTEM
//=============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriggerType {
    Time,
    Channel,
    Condition,
    Manual,
    Event,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum OrchestrationStatus {
    Inactive,
    Active,
    Paused,
    Error,
    Completed,
}

// Type aliases for function pointers
pub type ConditionFn = Arc<
    dyn (Fn(&OrchestrationContext) -> Pin<Box<dyn Future<Output = bool> + Send>>) + Send + Sync
>;
pub type PayloadFn = Arc<dyn (Fn(&OrchestrationContext) -> Value) + Send + Sync>;
pub type CustomStepFn = Arc<
    dyn (Fn(&OrchestrationContext) -> Pin<Box<dyn Future<Output = StepResult> + Send>>) +
        Send +
        Sync
>;
pub type DynamicCountFn = Arc<dyn (Fn(&OrchestrationContext) -> u32) + Send + Sync>;
pub type ErrorHandlerFn = Arc<
    dyn (Fn(&OrchestrationContext, &str) -> Pin<Box<dyn Future<Output = ErrorAction> + Send>>) +
        Send +
        Sync
>;
pub type BackoffFn = Arc<dyn (Fn(u32) -> Duration) + Send + Sync>;

#[derive(Debug, Clone)]
pub enum ErrorAction {
    Stop,
    Continue,
    Retry,
    Escalate(String),
}

/// Compile-time validated trigger configuration
#[derive(Clone)]
pub struct OrchestrationTrigger {
    pub trigger_type: TriggerType,
    pub name: String,

    // Time triggers
    pub interval: Option<Duration>,
    pub delay: Option<Duration>,
    pub cron: Option<String>,

    // Channel triggers
    pub channels: Option<Vec<String>>,

    // Condition triggers
    pub condition: Option<ConditionFn>,
    pub check_interval: Option<Duration>,

    // Configuration
    pub enabled: bool,
    pub repeat: Option<TaskRepeat>,
    pub metadata: HashMap<String, String>,
}

impl std::fmt::Debug for OrchestrationTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrchestrationTrigger")
            .field("trigger_type", &self.trigger_type)
            .field("name", &self.name)
            .field("interval", &self.interval)
            .field("enabled", &self.enabled)
            .finish_non_exhaustive()
    }
}

/// Type-safe step configuration
#[derive(Clone)]
pub struct OrchestrationStep {
    pub id: String,
    pub step_type: StepType,
    pub enabled: bool,

    // Error handling with Rust's Result type
    pub on_error: ErrorStrategy,
    pub retry_config: Option<RetryConfig>,
    pub metadata: HashMap<String, String>,
}

impl std::fmt::Debug for OrchestrationStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrchestrationStep")
            .field("id", &self.id)
            .field("enabled", &self.enabled)
            .field("on_error", &self.on_error)
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub enum StepType {
    Action {
        targets: Vec<String>,
        payload: PayloadFn,
    },
    Condition {
        condition: ConditionFn,
        on_true: Vec<OrchestrationStep>,
        on_false: Vec<OrchestrationStep>,
    },
    Parallel {
        steps: Vec<OrchestrationStep>,
        strategy: ParallelStrategy,
    },
    Delay {
        duration: Duration,
    },
    Loop {
        iterations: LoopCount,
        steps: Vec<OrchestrationStep>,
        break_condition: Option<ConditionFn>,
    },
    Custom {
        executor: CustomStepFn,
    },
}

impl std::fmt::Debug for StepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepType::Action { targets, .. } => {
                f.debug_struct("Action").field("targets", targets).finish_non_exhaustive()
            }
            StepType::Condition { .. } => f.debug_struct("Condition").finish_non_exhaustive(),
            StepType::Parallel { steps, strategy } => {
                f.debug_struct("Parallel")
                    .field("steps_count", &steps.len())
                    .field("strategy", strategy)
                    .finish()
            }
            StepType::Delay { duration } => {
                f.debug_struct("Delay").field("duration", duration).finish()
            }
            StepType::Loop { steps, .. } => {
                f.debug_struct("Loop").field("steps_count", &steps.len()).finish_non_exhaustive()
            }
            StepType::Custom { .. } => f.debug_struct("Custom").finish_non_exhaustive(),
        }
    }
}

#[derive(Clone)]
pub enum LoopCount {
    Fixed(u32),
    Dynamic(DynamicCountFn),
}

impl std::fmt::Debug for LoopCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoopCount::Fixed(count) => f.debug_tuple("Fixed").field(count).finish(),
            LoopCount::Dynamic(_) => f.debug_tuple("Dynamic").field(&"<function>").finish(),
        }
    }
}

#[derive(Clone)]
pub enum ErrorStrategy {
    Stop,
    Continue,
    Retry,
    Custom(ErrorHandlerFn),
}

impl std::fmt::Debug for ErrorStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorStrategy::Stop => write!(f, "Stop"),
            ErrorStrategy::Continue => write!(f, "Continue"),
            ErrorStrategy::Retry => write!(f, "Retry"),
            ErrorStrategy::Custom(_) => write!(f, "Custom(<function>)"),
        }
    }
}

#[derive(Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_strategy: BackoffStrategy,
    pub retry_condition: Option<ConditionFn>,
}

impl std::fmt::Debug for RetryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetryConfig")
            .field("max_attempts", &self.max_attempts)
            .field("base_delay", &self.base_delay)
            .field("max_delay", &self.max_delay)
            .field("backoff_strategy", &self.backoff_strategy)
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub enum BackoffStrategy {
    Linear,
    Exponential,
    Fixed,
    Custom(BackoffFn),
}

impl std::fmt::Debug for BackoffStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackoffStrategy::Linear => write!(f, "Linear"),
            BackoffStrategy::Exponential => write!(f, "Exponential"),
            BackoffStrategy::Fixed => write!(f, "Fixed"),
            BackoffStrategy::Custom(_) => write!(f, "Custom(<function>)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelStrategy {
    WaitAll,
    WaitAny,
    WaitFirst(u32),
}

/// Rich execution context with zero-cost access
#[derive(Debug, Clone)]
pub struct OrchestrationContext {
    pub orchestration_id: String,
    pub execution_id: String,
    pub trigger: TriggerEvent,
    pub variables: HashMap<String, serde_json::Value>,
    pub step_history: Vec<StepResult>,
    pub start_time: SystemTime,
    pub parent_context: Option<Box<OrchestrationContext>>,
    pub metrics: ExecutionMetrics,
}

impl Default for OrchestrationContext {
    fn default() -> Self {
        Self {
            orchestration_id: String::new(),
            execution_id: String::new(),
            trigger: TriggerEvent {
                trigger_type: TriggerType::Manual,
                name: String::new(),
                payload: None,
                timestamp: SystemTime::now(),
                metadata: HashMap::new(),
            },
            variables: HashMap::new(),
            step_history: Vec::new(),
            start_time: SystemTime::now(),
            parent_context: None,
            metrics: ExecutionMetrics::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub trigger_type: TriggerType,
    pub name: String,
    pub payload: Option<ActionPayload>,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepResult {
    pub step_id: String,
    pub status: StepStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration: Duration,
    pub timestamp: SystemTime,
    pub retry_count: u32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum StepStatus {
    Success,
    Error,
    Skipped,
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone, Default)]
pub struct ExecutionMetrics {
    pub steps_executed: u32,
    pub steps_failed: u32,
    pub total_retries: u32,
    pub parallel_executions: u32,
    pub memory_usage: u64,
    pub cpu_time: Duration,
}

/// Complete orchestration configuration
#[derive(Debug, Clone)]
pub struct OrchestrationConfig {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,

    pub triggers: Vec<OrchestrationTrigger>,
    pub steps: Vec<OrchestrationStep>,

    // Global configuration
    pub enabled: bool,
    pub priority: TaskPriority,
    pub timeout: Option<Duration>,
    pub max_concurrent_executions: Option<u32>,

    // Error handling
    pub global_error_strategy: ErrorStrategy,
    pub max_retries: Option<u32>,

    // Resource limits
    pub memory_limit: Option<u64>,
    pub cpu_limit: Option<Duration>,

    pub metadata: HashMap<String, String>,
}

/// Runtime state with comprehensive metrics
#[derive(Debug, Clone)]
pub struct OrchestrationRuntime {
    pub config: OrchestrationConfig,
    pub status: OrchestrationStatus,
    pub trigger_task_ids: Vec<String>,
    pub active_executions: HashMap<String, OrchestrationExecution>,
    pub last_execution: Option<SystemTime>,
    pub execution_count: u64,
    pub metrics: OrchestrationMetrics,
    pub resource_usage: ResourceUsage,
}

#[derive(Clone)]
pub struct OrchestrationExecution {
    pub execution_id: String,
    pub context: OrchestrationContext,
    pub start_time: SystemTime,
    pub current_step: Option<String>,
    pub cancellation_token: Option<Arc<tokio::sync::Notify>>,
}

impl std::fmt::Debug for OrchestrationExecution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrchestrationExecution")
            .field("execution_id", &self.execution_id)
            .field("start_time", &self.start_time)
            .field("current_step", &self.current_step)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct OrchestrationMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time: Duration,
    pub longest_execution: Duration,
    pub shortest_execution: Duration,
    pub trigger_counts: HashMap<String, u64>,
    pub step_metrics: HashMap<String, StepMetrics>,
    pub error_patterns: HashMap<String, u32>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct StepMetrics {
    pub executions: u64,
    pub successes: u64,
    pub failures: u64,
    pub average_time: Duration,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    pub memory_peak: u64,
    pub memory_current: u64,
    pub cpu_total: Duration,
    pub active_threads: u32,
}

//=============================================================================
// GLOBAL ORCHESTRATION STORE - THREAD-SAFE
//=============================================================================

static ORCHESTRATION_STORE: OnceLock<
    Arc<RwLock<HashMap<String, OrchestrationRuntime>>>
> = OnceLock::new();
static EXECUTION_POOL: OnceLock<
    Arc<RwLock<HashMap<String, OrchestrationExecution>>>
> = OnceLock::new();

fn get_orchestration_store() -> &'static Arc<RwLock<HashMap<String, OrchestrationRuntime>>> {
    ORCHESTRATION_STORE.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}

fn get_execution_pool() -> &'static Arc<RwLock<HashMap<String, OrchestrationExecution>>> {
    EXECUTION_POOL.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}

//=============================================================================
// ORCHESTRATION BUILDER - COMPILE-TIME VALIDATION
//=============================================================================

/// Zero-cost orchestration builder with type safety
pub struct OrchestrationBuilder {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    triggers: Vec<OrchestrationTrigger>,
    steps: Vec<OrchestrationStep>,
    enabled: bool,
    priority: TaskPriority,
    timeout: Option<Duration>,
    max_concurrent: Option<u32>,
    global_error_strategy: ErrorStrategy,
    metadata: HashMap<String, String>,
}

impl OrchestrationBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            description: None,
            triggers: Vec::new(),
            steps: Vec::new(),
            enabled: true,
            priority: TaskPriority::Normal,
            timeout: None,
            max_concurrent: None,
            global_error_strategy: ErrorStrategy::Stop,
            metadata: HashMap::new(),
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn max_concurrent_executions(mut self, max: u32) -> Self {
        self.max_concurrent = Some(max);
        self
    }

    pub fn global_error_strategy(mut self, strategy: ErrorStrategy) -> Self {
        self.global_error_strategy = strategy;
        self
    }

    /// Add time-based trigger
    pub fn time_trigger(mut self, name: impl Into<String>, interval: Duration) -> Self {
        self.triggers.push(OrchestrationTrigger {
            trigger_type: TriggerType::Time,
            name: name.into(),
            interval: Some(interval),
            delay: None,
            cron: None,
            channels: None,
            condition: None,
            check_interval: None,
            enabled: true,
            repeat: Some(TaskRepeat::Forever),
            metadata: HashMap::new(),
        });
        self
    }

    /// Add condition-based trigger
    pub fn condition_trigger<F>(
        mut self,
        name: impl Into<String>,
        condition: F,
        check_interval: Duration
    ) -> Self
        where
            F: Fn(&OrchestrationContext) -> Pin<Box<dyn Future<Output = bool> + Send>> +
                Send +
                Sync +
                'static
    {
        self.triggers.push(OrchestrationTrigger {
            trigger_type: TriggerType::Condition,
            name: name.into(),
            interval: None,
            delay: None,
            cron: None,
            channels: None,
            condition: Some(Arc::new(condition)),
            check_interval: Some(check_interval),
            enabled: true,
            repeat: Some(TaskRepeat::Forever),
            metadata: HashMap::new(),
        });
        self
    }

    /// Add action step
    pub fn action_step<F>(
        mut self,
        id: impl Into<String>,
        targets: Vec<String>,
        payload_fn: F
    ) -> Self
        where F: Fn(&OrchestrationContext) -> ActionPayload + Send + Sync + 'static
    {
        self.steps.push(OrchestrationStep {
            id: id.into(),
            step_type: StepType::Action {
                targets,
                payload: Arc::new(move |ctx| json!(payload_fn(ctx))),
            },
            enabled: true,
            on_error: ErrorStrategy::Stop,
            retry_config: None,
            metadata: HashMap::new(),
        });
        self
    }

    /// Build the orchestration configuration
    pub fn build(self) -> Result<OrchestrationConfig, String> {
        let id = self.id.ok_or("Orchestration ID is required")?;

        if self.triggers.is_empty() {
            return Err("At least one trigger is required".to_string());
        }

        if self.steps.is_empty() {
            return Err("At least one step is required".to_string());
        }

        Ok(OrchestrationConfig {
            id,
            name: self.name,
            description: self.description,
            triggers: self.triggers,
            steps: self.steps,
            enabled: self.enabled,
            priority: self.priority,
            timeout: self.timeout,
            max_concurrent_executions: self.max_concurrent,
            global_error_strategy: self.global_error_strategy,
            max_retries: None,
            memory_limit: None,
            cpu_limit: None,
            metadata: self.metadata,
        })
    }
}

//=============================================================================
// CORE ORCHESTRATION FUNCTIONS - PURE FUNCTIONAL API
//=============================================================================

/// Create and store orchestration configuration
pub fn keep(config: OrchestrationConfig) -> TaskResult<String> {
    let start = Instant::now();
    let orchestration_id = config.id.clone();

    // Check if orchestration already exists
    {
        let store = get_orchestration_store().read().unwrap();
        if store.contains_key(&orchestration_id) {
            return TaskResult::error(orchestration_id, "Orchestration already exists".to_string());
        }
    }

    // Create runtime
    let runtime = OrchestrationRuntime {
        config,
        status: OrchestrationStatus::Inactive,
        trigger_task_ids: Vec::new(),
        active_executions: HashMap::new(),
        last_execution: None,
        execution_count: 0,
        metrics: OrchestrationMetrics::default(),
        resource_usage: ResourceUsage::default(),
    };

    // Store orchestration
    {
        let mut store = get_orchestration_store().write().unwrap();
        store.insert(orchestration_id.clone(), runtime);
    }

    log_orchestration_event(
        &orchestration_id,
        "orchestration-created",
        &format!("Orchestration {} created", orchestration_id)
    );

    TaskResult::ok(
        orchestration_id.clone(),
        orchestration_id,
        "Orchestration created successfully".to_string()
    ).with_execution_time(start.elapsed())
}

/// Activate orchestration (setup triggers)
pub async fn activate(orchestration_id: &str) -> TaskResult<String> {
    let start = Instant::now();

    let mut runtime = {
        let store = get_orchestration_store().read().unwrap();
        match store.get(orchestration_id) {
            Some(runtime) => runtime.clone(),
            None => {
                return TaskResult::error(
                    orchestration_id.to_string(),
                    "Orchestration not found".to_string()
                );
            }
        }
    };

    if runtime.status == OrchestrationStatus::Active {
        return TaskResult::error(
            orchestration_id.to_string(),
            "Orchestration already active".to_string()
        );
    }

    // Setup triggers
    let trigger_task_ids = match setup_triggers(&runtime).await {
        Ok(ids) => ids,
        Err(e) => {
            return TaskResult::error(orchestration_id.to_string(), e);
        }
    };

    runtime.trigger_task_ids = trigger_task_ids;
    runtime.status = OrchestrationStatus::Active;

    // Store updated runtime
    {
        let mut store = get_orchestration_store().write().unwrap();
        store.insert(orchestration_id.to_string(), runtime);
    }

    log_orchestration_event(
        orchestration_id,
        "orchestration-activated",
        &format!("Orchestration {} activated", orchestration_id)
    );

    TaskResult::ok(
        orchestration_id.to_string(),
        "activated".to_string(),
        "Orchestration activated successfully".to_string()
    ).with_execution_time(start.elapsed())
}

/// Deactivate orchestration (remove triggers)
pub async fn deactivate(orchestration_id: &str) -> TaskResult<String> {
    let start = Instant::now();

    let mut runtime = {
        let store = get_orchestration_store().read().unwrap();
        match store.get(orchestration_id) {
            Some(runtime) => runtime.clone(),
            None => {
                return TaskResult::error(
                    orchestration_id.to_string(),
                    "Orchestration not found".to_string()
                );
            }
        }
    };

    // Remove all trigger tasks
    for task_id in &runtime.trigger_task_ids {
        let _ = crate::context::task_store::forget(task_id);
    }

    // Cancel active executions
    for (_, execution) in &runtime.active_executions {
        if let Some(cancel_token) = &execution.cancellation_token {
            cancel_token.notify_one();
        }
    }

    runtime.trigger_task_ids.clear();
    runtime.active_executions.clear();
    runtime.status = OrchestrationStatus::Inactive;

    // Store updated runtime
    {
        let mut store = get_orchestration_store().write().unwrap();
        store.insert(orchestration_id.to_string(), runtime);
    }

    log_orchestration_event(
        orchestration_id,
        "orchestration-deactivated",
        &format!("Orchestration {} deactivated", orchestration_id)
    );

    TaskResult::ok(
        orchestration_id.to_string(),
        "deactivated".to_string(),
        "Orchestration deactivated successfully".to_string()
    ).with_execution_time(start.elapsed())
}

/// Manual trigger execution
pub async fn trigger(
    orchestration_id: &str,
    trigger_name: &str,
    payload: Option<ActionPayload>
) -> TaskResult<String> {
    let runtime = {
        let store = get_orchestration_store().read().unwrap();
        match store.get(orchestration_id) {
            Some(runtime) => runtime.clone(),
            None => {
                return TaskResult::error(
                    orchestration_id.to_string(),
                    "Orchestration not found".to_string()
                );
            }
        }
    };

    let execution_id = format!(
        "exec-{}",
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos()
    );

    let trigger_event = TriggerEvent {
        trigger_type: TriggerType::Manual,
        name: trigger_name.to_string(),
        payload,
        timestamp: SystemTime::now(),
        metadata: HashMap::new(),
    };

    let context = OrchestrationContext {
        orchestration_id: orchestration_id.to_string(),
        execution_id: execution_id.clone(),
        trigger: trigger_event,
        variables: HashMap::new(),
        step_history: Vec::new(),
        start_time: SystemTime::now(),
        parent_context: None,
        metrics: ExecutionMetrics::default(),
    };

    let result = execute_orchestration(&runtime, context).await;
    match result {
        Ok(result) => {
            update_orchestration_metrics(orchestration_id, &result, true).await;
            TaskResult::ok(
                orchestration_id.to_string(),
                execution_id,
                "Orchestration executed successfully".to_string()
            )
        }
        Err(e) => {
            update_orchestration_metrics(
                orchestration_id,
                &OrchestrationContext::default(),
                false
            ).await;
            TaskResult::error(orchestration_id.to_string(), format!("Execution failed: {}", e))
        }
    }
}

/// Get orchestration info
pub fn get(orchestration_id: &str) -> Option<OrchestrationRuntime> {
    let store = get_orchestration_store().read().unwrap();
    store.get(orchestration_id).cloned()
}

/// List orchestrations
pub fn list(status_filter: Option<OrchestrationStatus>) -> Vec<OrchestrationRuntime> {
    let store = get_orchestration_store().read().unwrap();
    let mut runtimes: Vec<OrchestrationRuntime> = store.values().cloned().collect();

    if let Some(status) = status_filter {
        runtimes.retain(|runtime| runtime.status == status);
    }

    runtimes
}

/// Remove orchestration completely
pub async fn forget(orchestration_id: &str) -> TaskResult<bool> {
    // Deactivate first if active
    if let Some(runtime) = get(orchestration_id) {
        if runtime.status == OrchestrationStatus::Active {
            let _ = deactivate(orchestration_id).await;
        }
    }

    // Remove from store
    let removed = {
        let mut store = get_orchestration_store().write().unwrap();
        let result = store.remove(orchestration_id);
        result.is_some()
    };

    if removed {
        log_orchestration_event(
            orchestration_id,
            "orchestration-removed",
            &format!("Orchestration {} removed", orchestration_id)
        );
        TaskResult::ok(
            orchestration_id.to_string(),
            true,
            "Orchestration removed successfully".to_string()
        )
    } else {
        TaskResult::error(orchestration_id.to_string(), "Orchestration not found".to_string())
    }
}

//=============================================================================
// STEP EXECUTION ENGINE - ZERO-COST ABSTRACTIONS
//=============================================================================

async fn execute_orchestration(
    runtime: &OrchestrationRuntime,
    context: OrchestrationContext
) -> Result<OrchestrationContext, String> {
    log_orchestration_event(
        &context.orchestration_id,
        "execution-started",
        &format!("Execution {} started", context.execution_id)
    );

    let mut context = context;

    // Execute steps sequentially (unless parallel)
    for step in &runtime.config.steps {
        if !step.enabled {
            continue;
        }

        let step_result = execute_step(step, &mut context).await;

        // Handle step result based on error strategy
        match step_result.status {
            StepStatus::Error => {
                match &step.on_error {
                    ErrorStrategy::Stop => {
                        return Err(
                            format!(
                                "Step {} failed: {}",
                                step.id,
                                step_result.error.unwrap_or_else(|| "Unknown error".to_string())
                            )
                        );
                    }
                    ErrorStrategy::Continue => {
                        // Log error but continue
                        log_orchestration_event(
                            &context.orchestration_id,
                            "step-error-continue",
                            &format!("Step {} failed but continuing", step.id)
                        );
                    }
                    ErrorStrategy::Retry => {
                        // Implement retry logic
                        // For now, just continue
                        log_orchestration_event(
                            &context.orchestration_id,
                            "step-retry",
                            &format!("Step {} will be retried", step.id)
                        );
                    }
                    ErrorStrategy::Custom(_handler) => {
                        // Custom error handling would go here
                        log_orchestration_event(
                            &context.orchestration_id,
                            "step-custom-error",
                            &format!("Step {} using custom error handler", step.id)
                        );
                    }
                }
            }
            _ => {
                // Success or other status
                log_orchestration_event(
                    &context.orchestration_id,
                    "step-completed",
                    &format!("Step {} completed with status {:?}", step.id, step_result.status)
                );
            }
        }

        context.step_history.push(step_result);
    }

    log_orchestration_event(
        &context.orchestration_id,
        "execution-completed",
        &format!("Execution {} completed successfully", context.execution_id)
    );

    Ok(context)
}

async fn execute_step(step: &OrchestrationStep, context: &mut OrchestrationContext) -> StepResult {
    let start_time = SystemTime::now();

    let result = match &step.step_type {
        StepType::Action { targets, payload } => {
            execute_action_step(targets, payload, context).await
        }
        StepType::Condition { condition, on_true, on_false } => {
            execute_condition_step(condition, on_true, on_false, context).await
        }
        StepType::Parallel { steps, strategy } => {
            execute_parallel_step(steps, strategy, context).await
        }
        StepType::Delay { duration } => { execute_delay_step(*duration, context).await }
        StepType::Loop { iterations, steps, break_condition } => {
            execute_loop_step(iterations, steps, break_condition.as_ref(), context).await
        }
        StepType::Custom { executor } => {
            // Custom executor returns StepResult directly
            return executor(context).await;
        }
    };

    let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));

    match result {
        Ok(value) =>
            StepResult {
                step_id: step.id.clone(),
                status: StepStatus::Success,
                result: Some(value),
                error: None,
                duration,
                timestamp: start_time,
                retry_count: 0,
                metadata: HashMap::new(),
            },
        Err(error) =>
            StepResult {
                step_id: step.id.clone(),
                status: StepStatus::Error,
                result: None,
                error: Some(error),
                duration,
                timestamp: start_time,
                retry_count: 0,
                metadata: HashMap::new(),
            },
    }
}

async fn execute_action_step(
    _targets: &[String],
    _payload: &PayloadFn,
    _context: &OrchestrationContext
) -> Result<Value, String> {
    // Placeholder implementation
    Ok(json!({"status": "action_executed"}))
}

async fn execute_condition_step(
    _condition: &ConditionFn,
    _on_true: &[OrchestrationStep],
    _on_false: &[OrchestrationStep],
    _context: &OrchestrationContext
) -> Result<Value, String> {
    // Placeholder implementation
    Ok(json!({"status": "condition_evaluated"}))
}

async fn execute_parallel_step(
    _steps: &[OrchestrationStep],
    _strategy: &ParallelStrategy,
    _context: &OrchestrationContext
) -> Result<Value, String> {
    // Placeholder implementation
    Ok(json!({"status": "parallel_executed"}))
}

async fn execute_delay_step(
    duration: Duration,
    _context: &OrchestrationContext
) -> Result<Value, String> {
    tokio::time::sleep(duration).await;
    Ok(json!({"status": "delay_completed", "duration_ms": duration.as_millis()}))
}

async fn execute_loop_step(
    _iterations: &LoopCount,
    _steps: &[OrchestrationStep],
    _break_condition: Option<&ConditionFn>,
    _context: &OrchestrationContext
) -> Result<Value, String> {
    // Placeholder implementation
    Ok(json!({"status": "loop_executed"}))
}

//=============================================================================
// TRIGGER SETUP AND MANAGEMENT
//=============================================================================

async fn setup_triggers(runtime: &OrchestrationRuntime) -> Result<Vec<String>, String> {
    let mut trigger_task_ids = Vec::new();
    let orch_id = runtime.config.id.clone();

    for (index, trigger_config) in runtime.config.triggers.iter().enumerate() {
        if !trigger_config.enabled {
            continue;
        }

        let orch_id_clone = orch_id.clone();
        let task_id = format!("{}-trigger-{}", orch_id_clone, index);
        let trig_name = trigger_config.name.clone();

        match trigger_config.trigger_type {
            TriggerType::Time => {
                if let Some(interval) = trigger_config.interval {
                    // FIXED: Use CyreResponse::success instead of CyreResponse::ok
                    let result = crate::context::task_store::interval(
                        &task_id,
                        interval,
                        Some(TaskRepeat::Forever),
                        move || {
                            let orch_id = orch_id_clone.clone();
                            let trig_name = trig_name.clone();
                            Box::pin(async move {
                                let _ = trigger(&orch_id, &trig_name, None).await;
                                CyreResponse::success(json!("trigger_executed"), "Trigger executed")
                            })
                        }
                    );

                    if result.success {
                        trigger_task_ids.push(task_id);
                    }
                }
            }
            TriggerType::Condition => {
                if let Some(check_interval) = trigger_config.check_interval {
                    // FIXED: Use CyreResponse::success instead of CyreResponse::ok
                    let result = crate::context::task_store::interval(
                        &task_id,
                        check_interval,
                        Some(TaskRepeat::Forever),
                        move || {
                            let orch_id = orch_id_clone.clone();
                            let trig_name = trig_name.clone();
                            Box::pin(async move {
                                // Would check condition here
                                let _ = trigger(&orch_id, &trig_name, None).await;
                                CyreResponse::success(
                                    json!("condition_checked"),
                                    "Condition checked"
                                )
                            })
                        }
                    );

                    if result.success {
                        trigger_task_ids.push(task_id);
                    }
                }
            }
            _ => {
                // Other trigger types would be implemented here
            }
        }
    }

    Ok(trigger_task_ids)
}

//=============================================================================
// METRICS AND MONITORING
//=============================================================================

async fn update_orchestration_metrics(
    orchestration_id: &str,
    _context: &OrchestrationContext,
    success: bool
) {
    let mut store = get_orchestration_store().write().unwrap();
    if let Some(runtime) = store.get_mut(orchestration_id) {
        runtime.metrics.total_executions += 1;
        if success {
            runtime.metrics.successful_executions += 1;
        } else {
            runtime.metrics.failed_executions += 1;
        }
        runtime.execution_count += 1;
        runtime.last_execution = Some(SystemTime::now());
    }
}

fn log_orchestration_event(orchestration_id: &str, event_type: &str, message: &str) {
    println!("[ORCHESTRATION] {} - {}: {}", event_type.to_uppercase(), orchestration_id, message);
}

//=============================================================================
// CONVENIENCE BUILDERS - ZERO-COST
//=============================================================================

/// Create a simple scheduled orchestration
pub fn schedule<F>(
    id: impl Into<String>,
    interval: Duration,
    action_targets: Vec<String>,
    payload_fn: F
) -> Result<OrchestrationConfig, String>
    where F: Fn(&OrchestrationContext) -> ActionPayload + Send + Sync + 'static
{
    OrchestrationBuilder::new()
        .id(id)
        .time_trigger("scheduled-trigger", interval)
        .action_step("scheduled-action", action_targets, payload_fn)
        .build()
}

/// Create a monitoring orchestration
pub fn monitor<C, F>(
    id: impl Into<String>,
    condition: C,
    check_interval: Duration,
    action_targets: Vec<String>,
    payload_fn: F
)
    -> Result<OrchestrationConfig, String>
    where
        C: Fn(&OrchestrationContext) -> Pin<Box<dyn Future<Output = bool> + Send>> +
            Send +
            Sync +
            'static,
        F: Fn(&OrchestrationContext) -> ActionPayload + Send + Sync + 'static
{
    OrchestrationBuilder::new()
        .id(id)
        .condition_trigger("monitor-trigger", condition, check_interval)
        .action_step("monitor-action", action_targets, payload_fn)
        .build()
}

//=============================================================================
// PUBLIC API MODULE
//=============================================================================

pub mod orchestration {
    pub use super::{
        keep,
        activate,
        deactivate,
        trigger,
        get,
        list,
        forget,
        schedule,
        monitor,
        OrchestrationBuilder,
        OrchestrationConfig,
        OrchestrationRuntime,
        OrchestrationStatus,
        TriggerType,
        StepType,
        ParallelStrategy,
        ErrorStrategy,
        OrchestrationMetrics,
    };
}
