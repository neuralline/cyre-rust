// src/context/task_store.rs
// Functional task store as part of context management

use std::collections::HashMap;
use std::sync::{ Arc, RwLock, OnceLock };
use std::time::{ Duration, Instant, SystemTime };
use serde::{ Serialize, Deserialize };

use crate::types::CyreResponse;
use crate::timekeeper::{ get_timekeeper, TimerRepeat };

/*

      C.Y.R.E - C.O.N.T.E.X.T - T.A.S.K - S.T.O.R.E
      
      Task state management integrated with context:
      - Central task storage
      - Timeline integration
      - TimeKeeper coordination
      - Functional API with Rust power

*/

//=============================================================================
// TASK TYPES
//=============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskStatus {
    Inactive,
    Active,
    Paused,
    Error,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    Timeout,
    Interval,
    Complex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskRepeat {
    Once,
    Forever,
    Count(u32),
}

impl From<TaskRepeat> for TimerRepeat {
    fn from(repeat: TaskRepeat) -> Self {
        match repeat {
            TaskRepeat::Once => TimerRepeat::Once,
            TaskRepeat::Forever => TimerRepeat::Forever,
            TaskRepeat::Count(n) => TimerRepeat::Count(n),
        }
    }
}

/// Task execution callback
pub type TaskCallback = Arc<
    dyn (Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = CyreResponse> + Send>>) +
        Send +
        Sync
>;

#[derive(Clone)]
pub struct TaskConfig {
    pub id: String,
    pub task_type: TaskType,
    pub interval: Option<Duration>,
    pub delay: Option<Duration>,
    pub repeat: Option<TaskRepeat>,
    pub callback: TaskCallback,
    pub priority: TaskPriority,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone)]
pub struct Task {
    pub id: String,
    pub config: TaskConfig,
    pub status: TaskStatus,
    pub created_at: SystemTime,
    pub last_activated: Option<SystemTime>,
    pub last_deactivated: Option<SystemTime>,
    pub activation_count: u32,
    pub timeline_id: Option<String>,
    pub metrics: TaskMetrics,
}

#[derive(Debug, Clone, Default)]
pub struct TaskMetrics {
    pub total_activations: u64,
    pub total_executions: u64,
    pub total_errors: u64,
    pub average_execution_time: Duration,
    pub last_execution_time: Duration,
    pub total_active_time: Duration,
    pub success_rate: f64,
}

#[derive(Debug, Default)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub task_type: Option<TaskType>,
    pub priority: Option<TaskPriority>,
    pub active_since: Option<SystemTime>,
    pub has_errors: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskStats {
    pub total: usize,
    pub by_status: HashMap<TaskStatus, usize>,
    pub by_type: HashMap<TaskType, usize>,
    pub active_count: usize,
    pub error_count: usize,
    pub average_active_time: Duration,
    pub system_health: SystemHealth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SystemHealth {
    Healthy,
    Degraded,
    Critical,
}

#[derive(Debug)]
pub struct TaskResult<T> {
    pub success: bool,
    pub value: Option<T>,
    pub task_id: String,
    pub message: String,
    pub execution_time: Duration,
}

impl<T> TaskResult<T> {
    pub fn ok(task_id: String, value: T, message: String) -> Self {
        Self {
            success: true,
            value: Some(value),
            task_id,
            message,
            execution_time: Duration::default(),
        }
    }

    pub fn error(task_id: String, message: String) -> Self {
        Self {
            success: false,
            value: None,
            task_id,
            message,
            execution_time: Duration::default(),
        }
    }

    pub fn with_execution_time(mut self, duration: Duration) -> Self {
        self.execution_time = duration;
        self
    }
}

//=============================================================================
// GLOBAL TASK STORE
//=============================================================================

static TASK_STORE: OnceLock<Arc<RwLock<HashMap<String, Task>>>> = OnceLock::new();

fn get_task_store() -> &'static Arc<RwLock<HashMap<String, Task>>> {
    TASK_STORE.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}

//=============================================================================
// TASK BUILDER
//=============================================================================

pub struct TaskBuilder {
    id: Option<String>,
    task_type: Option<TaskType>,
    interval: Option<Duration>,
    delay: Option<Duration>,
    repeat: Option<TaskRepeat>,
    priority: TaskPriority,
    metadata: HashMap<String, String>,
}

impl TaskBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            task_type: None,
            interval: None,
            delay: None,
            repeat: None,
            priority: TaskPriority::Normal,
            metadata: HashMap::new(),
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn timeout_task(mut self, delay: Duration) -> Self {
        self.task_type = Some(TaskType::Timeout);
        self.delay = Some(delay);
        self.repeat = Some(TaskRepeat::Once);
        self
    }

    pub fn interval_task(mut self, interval: Duration, repeat: Option<TaskRepeat>) -> Self {
        self.task_type = Some(TaskType::Interval);
        self.interval = Some(interval);
        self.repeat = repeat.or(Some(TaskRepeat::Forever));
        self
    }

    pub fn complex_task(
        mut self,
        delay: Option<Duration>,
        interval: Duration,
        repeat: Option<TaskRepeat>
    ) -> Self {
        self.task_type = Some(TaskType::Complex);
        self.delay = delay;
        self.interval = Some(interval);
        self.repeat = repeat.or(Some(TaskRepeat::Forever));
        self
    }

    pub fn priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn build<F, Fut>(self, callback: F) -> Result<TaskConfig, String>
        where
            F: Fn() -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = CyreResponse> + Send + 'static
    {
        let id = self.id.ok_or("Task ID is required")?;
        let task_type = self.task_type.ok_or("Task type is required")?;

        // Validate configuration
        match task_type {
            TaskType::Timeout => {
                if self.delay.is_none() {
                    return Err("Timeout tasks require a delay".to_string());
                }
            }
            TaskType::Interval => {
                if self.interval.is_none() {
                    return Err("Interval tasks require an interval".to_string());
                }
            }
            TaskType::Complex => {
                if self.interval.is_none() {
                    return Err("Complex tasks require an interval".to_string());
                }
            }
        }

        // Wrap callback
        let callback_wrapper: TaskCallback = Arc::new(move || {
            let fut = callback();
            Box::pin(fut)
        });

        Ok(TaskConfig {
            id,
            task_type,
            interval: self.interval,
            delay: self.delay,
            repeat: self.repeat,
            callback: callback_wrapper,
            priority: self.priority,
            metadata: self.metadata,
        })
    }
}

//=============================================================================
// CORE TASK STORE FUNCTIONS
//=============================================================================

/// Create and store a task (inactive by default)
pub fn keep(config: TaskConfig) -> TaskResult<String> {
    let start = Instant::now();
    let task_id = config.id.clone();

    // Check if task already exists
    {
        let store = get_task_store().read().unwrap();
        if store.contains_key(&task_id) {
            return TaskResult::error(task_id, "Task already exists".to_string());
        }
    }

    // Create task
    let task = Task {
        id: task_id.clone(),
        config,
        status: TaskStatus::Inactive,
        created_at: SystemTime::now(),
        last_activated: None,
        last_deactivated: None,
        activation_count: 0,
        timeline_id: None,
        metrics: TaskMetrics::default(),
    };

    // Store task
    {
        let mut store = get_task_store().write().unwrap();
        store.insert(task_id.clone(), task);
    }

    log_task_event(&task_id, "task-created", &format!("Task {} created", task_id));

    TaskResult::ok(
        task_id.clone(),
        task_id,
        "Task created successfully".to_string()
    ).with_execution_time(start.elapsed())
}

/// Remove task completely
pub fn forget(task_id: &str) -> TaskResult<bool> {
    let start = Instant::now();

    // Deactivate first if active
    if let Some(task) = get(task_id) {
        if task.status == TaskStatus::Active {
            let _ = tokio::runtime::Handle::current().block_on(activate(task_id, false));
        }
    }

    // Remove from store - FIX: Bind the result to control drop order in Rust 2024
    let removed = {
        let mut store = get_task_store().write().unwrap();
        let result = store.remove(task_id); // Explicit binding
        result.is_some()
    };

    (
        if removed {
            log_task_event(task_id, "task-removed", &format!("Task {} removed", task_id));
            TaskResult::ok(task_id.to_string(), true, "Task removed successfully".to_string())
        } else {
            TaskResult::error(task_id.to_string(), "Task not found".to_string())
        }
    ).with_execution_time(start.elapsed())
}
/// Activate/deactivate task with timeline integration
pub async fn activate(task_id: &str, active: bool) -> TaskResult<String> {
    let start = Instant::now();

    let mut task = {
        let store = get_task_store().read().unwrap();
        match store.get(task_id) {
            Some(task) => task.clone(),
            None => {
                return TaskResult::error(task_id.to_string(), "Task not found".to_string());
            }
        }
    };

    let current_time = SystemTime::now();

    (
        if active && task.status != TaskStatus::Active {
            // ACTIVATE: Move to timeline + TimeKeeper
            let timekeeper = get_timekeeper().await;

            // Create enhanced callback with metrics tracking
            let task_id_clone = task_id.to_string();
            let callback = task.config.callback.clone();

            let enhanced_callback = move || {
                let task_id_inner = task_id_clone.clone();
                let callback_inner = callback.clone();

                Box::pin(async move {
                    let _execution_start = Instant::now();

                    let result = callback_inner().await;

                    // Update metrics (simplified for now)
                    log_task_event(&task_id_inner, "task-executed", "Task executed successfully");

                    result
                }) as std::pin::Pin<Box<dyn std::future::Future<Output = CyreResponse> + Send>>
            };

            // Schedule with TimeKeeper based on task type
            let formation_result = match task.config.task_type {
                TaskType::Timeout => {
                    // Use timekeeper timeout functionality
                    timekeeper.schedule_timeout(
                        task_id,
                        Box::new(enhanced_callback),
                        task.config.delay.unwrap()
                    ).await
                }
                TaskType::Interval => {
                    // Use timekeeper interval functionality
                    timekeeper.schedule_interval(
                        task_id,
                        Box::new(enhanced_callback),
                        task.config.interval.unwrap(),
                        task.config.repeat.unwrap_or(TaskRepeat::Forever).into()
                    ).await
                }
                TaskType::Complex => {
                    // Use timekeeper complex scheduling
                    timekeeper.schedule_complex(
                        task_id,
                        Box::new(enhanced_callback),
                        task.config.delay,
                        task.config.interval.unwrap(),
                        task.config.repeat.unwrap_or(TaskRepeat::Forever).into()
                    ).await
                }
            };

            match formation_result {
                Ok(formation_id) => {
                    // Update task state
                    task.status = TaskStatus::Active;
                    task.last_activated = Some(current_time);
                    task.activation_count += 1;
                    task.metrics.total_activations += 1;
                    task.timeline_id = Some(formation_id);

                    // Store updated task
                    {
                        let mut store = get_task_store().write().unwrap();
                        store.insert(task_id.to_string(), task);
                    }

                    log_task_event(
                        task_id,
                        "task-activated",
                        &format!("Task {} activated", task_id)
                    );
                    TaskResult::ok(
                        task_id.to_string(),
                        "activated".to_string(),
                        "Task activated successfully".to_string()
                    )
                }
                Err(e) => {
                    TaskResult::error(task_id.to_string(), format!("TimeKeeper failed: {}", e))
                }
            }
        } else if !active && task.status == TaskStatus::Active {
            // DEACTIVATE: Remove from timeline + TimeKeeper
            let timekeeper = get_timekeeper().await;
            if let Some(timeline_id) = &task.timeline_id {
                let _ = timekeeper.cancel_formation(timeline_id).await;
            }

            // Update task state
            let active_time = current_time
                .duration_since(task.last_activated.unwrap_or(current_time))
                .unwrap_or(Duration::default());

            task.status = TaskStatus::Inactive;
            task.last_deactivated = Some(current_time);
            task.metrics.total_active_time += active_time;
            task.timeline_id = None;

            // Store updated task
            {
                let mut store = get_task_store().write().unwrap();
                store.insert(task_id.to_string(), task);
            }

            log_task_event(task_id, "task-deactivated", &format!("Task {} deactivated", task_id));
            TaskResult::ok(
                task_id.to_string(),
                "deactivated".to_string(),
                "Task deactivated successfully".to_string()
            )
        } else {
            TaskResult::error(task_id.to_string(), format!("Task is already {:?}", task.status))
        }
    ).with_execution_time(start.elapsed())
}

/// Get task by ID
pub fn get(task_id: &str) -> Option<Task> {
    let store = get_task_store().read().unwrap();
    store.get(task_id).cloned()
}

/// List tasks with filtering
pub fn list(filter: Option<TaskFilter>) -> Vec<Task> {
    let store = get_task_store().read().unwrap();
    let mut tasks: Vec<Task> = store.values().cloned().collect();

    if let Some(f) = filter {
        if let Some(status) = f.status {
            tasks.retain(|task| task.status == status);
        }
        if let Some(task_type) = f.task_type {
            tasks.retain(|task| task.config.task_type == task_type);
        }
        if let Some(priority) = f.priority {
            tasks.retain(|task| task.config.priority == priority);
        }
        if let Some(since) = f.active_since {
            tasks.retain(|task| {
                task.last_activated.map_or(false, |activated| activated >= since)
            });
        }
        if let Some(has_errors) = f.has_errors {
            tasks.retain(|task| (task.metrics.total_errors > 0) == has_errors);
        }
    }

    tasks
}

/// Get comprehensive system statistics
pub fn stats() -> TaskStats {
    let store = get_task_store().read().unwrap();
    let tasks: Vec<&Task> = store.values().collect();
    let total = tasks.len();

    // Count by status
    let mut by_status = HashMap::new();
    let mut by_type = HashMap::new();

    for task in &tasks {
        *by_status.entry(task.status).or_insert(0) += 1;
        *by_type.entry(task.config.task_type).or_insert(0) += 1;
    }

    let active_count = by_status.get(&TaskStatus::Active).copied().unwrap_or(0);
    let error_count = by_status.get(&TaskStatus::Error).copied().unwrap_or(0);

    // Calculate average active time
    let total_active_time: Duration = tasks
        .iter()
        .map(|task| task.metrics.total_active_time)
        .sum();

    let average_active_time = if total > 0 {
        total_active_time / (total as u32)
    } else {
        Duration::default()
    };

    // Determine system health
    let error_rate = if total > 0 { (error_count as f64) / (total as f64) } else { 0.0 };
    let system_health = if error_rate > 0.3 {
        SystemHealth::Critical
    } else if error_rate > 0.1 {
        SystemHealth::Degraded
    } else {
        SystemHealth::Healthy
    };

    TaskStats {
        total,
        by_status,
        by_type,
        active_count,
        error_count,
        average_active_time,
        system_health,
    }
}

//=============================================================================
// CONVENIENCE BUILDERS
//=============================================================================

/// Create timeout task
pub fn timeout<F, Fut>(
    task_id: impl Into<String>,
    delay: Duration,
    callback: F
)
    -> TaskResult<String>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = CyreResponse> + Send + 'static
{
    let config = TaskBuilder::new().id(task_id).timeout_task(delay).build(callback);

    match config {
        Ok(cfg) => keep(cfg),
        Err(e) => TaskResult::error("".to_string(), e),
    }
}

/// Create interval task
pub fn interval<F, Fut>(
    task_id: impl Into<String>,
    interval: Duration,
    repeat: Option<TaskRepeat>,
    callback: F
)
    -> TaskResult<String>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = CyreResponse> + Send + 'static
{
    let config = TaskBuilder::new().id(task_id).interval_task(interval, repeat).build(callback);

    match config {
        Ok(cfg) => keep(cfg),
        Err(e) => TaskResult::error("".to_string(), e),
    }
}

/// Create complex task
pub fn complex<F, Fut>(
    task_id: impl Into<String>,
    delay: Option<Duration>,
    interval: Duration,
    repeat: Option<TaskRepeat>,
    callback: F
)
    -> TaskResult<String>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = CyreResponse> + Send + 'static
{
    let config = TaskBuilder::new()
        .id(task_id)
        .complex_task(delay, interval, repeat)
        .build(callback);

    match config {
        Ok(cfg) => keep(cfg),
        Err(e) => TaskResult::error("".to_string(), e),
    }
}

//=============================================================================
// HELPER FUNCTIONS
//=============================================================================

fn log_task_event(task_id: &str, event_type: &str, message: &str) {
    println!("[TASK] {} - {}: {}", event_type.to_uppercase(), task_id, message);
}
