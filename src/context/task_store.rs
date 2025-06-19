// src/context/task_store.rs
// File location: src/context/task_store.rs
// Task storage and management

//=============================================================================
// IMPORTS
//=============================================================================

use std::sync::{ Arc, RwLock, OnceLock };
use std::collections::HashMap;
use serde::{ Serialize, Deserialize };
use serde_json::{ Value as JsonValue, json };
use crate::types::{ ActionPayload, Priority };
use crate::utils::current_timestamp;

//=============================================================================
// TASK TYPES
//=============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Simple,
    Timeout,
    Interval,
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskRepeat {
    Once,
    Forever,
    Count(u32),
}

//=============================================================================
// TASK CONFIGURATION
//=============================================================================

#[derive(Debug, Clone)]
pub struct TaskConfig {
    pub id: String,
    pub task_type: TaskType,
    pub action_id: String,
    pub payload: ActionPayload,
    pub delay: Option<u64>,
    pub interval: Option<u64>,
    pub repeat: TaskRepeat,
    pub priority: TaskPriority,
    pub max_retries: u32,
}

impl TaskConfig {
    pub fn new(id: impl Into<String>, action_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            task_type: TaskType::Simple,
            action_id: action_id.into(),
            payload: json!({}),
            delay: None,
            interval: None,
            repeat: TaskRepeat::Once,
            priority: TaskPriority::Medium,
            max_retries: 0,
        }
    }

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay = Some(delay_ms);
        self.task_type = TaskType::Timeout;
        self
    }

    pub fn with_interval(mut self, interval_ms: u64) -> Self {
        self.interval = Some(interval_ms);
        self.task_type = TaskType::Interval;
        self
    }

    pub fn with_repeat(mut self, repeat: TaskRepeat) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_payload(mut self, payload: ActionPayload) -> Self {
        self.payload = payload;
        self
    }
}

//=============================================================================
// TASK INSTANCE
//=============================================================================

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub config: TaskConfig,
    pub status: TaskStatus,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub execution_count: u32,
    pub retry_count: u32,
    pub last_error: Option<String>,
    pub next_execution: Option<u64>,
}

impl Task {
    pub fn new(config: TaskConfig) -> Self {
        let now = current_timestamp();

        Self {
            id: config.id.clone(),
            config,
            status: TaskStatus::Pending,
            created_at: now,
            started_at: None,
            completed_at: None,
            execution_count: 0,
            retry_count: 0,
            last_error: None,
            next_execution: None,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, TaskStatus::Pending | TaskStatus::Running)
    }

    pub fn is_due(&self, current_time: u64) -> bool {
        match self.next_execution {
            Some(time) => current_time >= time,
            None => true,
        }
    }

    pub fn mark_started(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Some(current_timestamp());
    }

    pub fn mark_completed(&mut self) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(current_timestamp());
        self.execution_count += 1;
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.last_error = Some(error);
        self.retry_count += 1;
    }

    pub fn schedule_next(&mut self) {
        let now = current_timestamp();

        match &self.config.repeat {
            TaskRepeat::Once => {
                self.next_execution = None;
            }
            TaskRepeat::Forever => {
                if let Some(interval) = self.config.interval {
                    self.next_execution = Some(now + interval);
                }
            }
            TaskRepeat::Count(count) => {
                if self.execution_count < *count {
                    if let Some(interval) = self.config.interval {
                        self.next_execution = Some(now + interval);
                    }
                } else {
                    self.next_execution = None;
                }
            }
        }
    }
}

//=============================================================================
// TASK STORE
//=============================================================================

#[derive(Debug)]
pub struct TaskStore {
    tasks: HashMap<String, Task>,
}

impl TaskStore {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    pub fn add(&mut self, task: Task) {
        self.tasks.insert(task.id.clone(), task);
    }

    pub fn get(&self, task_id: &str) -> Option<&Task> {
        self.tasks.get(task_id)
    }

    pub fn get_mut(&mut self, task_id: &str) -> Option<&mut Task> {
        self.tasks.get_mut(task_id)
    }

    pub fn remove(&mut self, task_id: &str) -> Option<Task> {
        self.tasks.remove(task_id)
    }

    pub fn list_active(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.is_active())
            .collect()
    }

    pub fn list_due(&self, current_time: u64) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.is_active() && task.is_due(current_time))
            .collect()
    }

    pub fn count(&self) -> usize {
        self.tasks.len()
    }

    pub fn count_active(&self) -> usize {
        self.tasks
            .values()
            .filter(|task| task.is_active())
            .count()
    }

    pub fn clear(&mut self) {
        self.tasks.clear();
    }
}

//=============================================================================
// GLOBAL TASK STORE
//=============================================================================

static TASK_STORE: OnceLock<Arc<RwLock<TaskStore>>> = OnceLock::new();

fn get_task_store() -> &'static Arc<RwLock<TaskStore>> {
    TASK_STORE.get_or_init(|| Arc::new(RwLock::new(TaskStore::new())))
}

//=============================================================================
// PUBLIC API
//=============================================================================

/// Add a task to the store
pub fn keep(task: Task) -> Result<(), String> {
    let store = get_task_store();
    match store.write() {
        Ok(mut store) => {
            store.add(task);
            Ok(())
        }
        Err(e) => Err(format!("Failed to add task: {}", e)),
    }
}

/// Remove a task from the store
pub fn forget(task_id: &str) -> Result<bool, String> {
    let store = get_task_store();
    match store.write() {
        Ok(mut store) => { Ok(store.remove(task_id).is_some()) }
        Err(e) => Err(format!("Failed to remove task: {}", e)),
    }
}

/// Activate a task (mark as pending)
pub fn activate(task_id: &str) -> Result<(), String> {
    let store = get_task_store();
    match store.write() {
        Ok(mut store) => {
            if let Some(task) = store.get_mut(task_id) {
                task.status = TaskStatus::Pending;
                Ok(())
            } else {
                Err(format!("Task '{}' not found", task_id))
            }
        }
        Err(e) => Err(format!("Failed to activate task: {}", e)),
    }
}

/// Get a task by ID
pub fn get(task_id: &str) -> Option<Task> {
    let store = get_task_store();
    store.read().ok()?.get(task_id).cloned()
}

/// List all tasks
pub fn list() -> Vec<Task> {
    let store = get_task_store();
    match store.read() {
        Ok(store) => store.tasks.values().cloned().collect(),
        Err(_) => Vec::new(),
    }
}

/// Get task store statistics
pub fn stats() -> TaskStats {
    let store = get_task_store();
    match store.read() {
        Ok(store) => {
            let total = store.count();
            let active = store.count_active();
            let pending = store.tasks
                .values()
                .filter(|t| matches!(t.status, TaskStatus::Pending))
                .count();
            let running = store.tasks
                .values()
                .filter(|t| matches!(t.status, TaskStatus::Running))
                .count();
            let completed = store.tasks
                .values()
                .filter(|t| matches!(t.status, TaskStatus::Completed))
                .count();
            let failed = store.tasks
                .values()
                .filter(|t| matches!(t.status, TaskStatus::Failed))
                .count();

            TaskStats {
                total,
                active,
                pending,
                running,
                completed,
                failed,
            }
        }
        Err(_) => TaskStats::default(),
    }
}

//=============================================================================
// CONVENIENCE BUILDERS
//=============================================================================

/// Create a timeout task
pub fn timeout(id: impl Into<String>, action_id: impl Into<String>, delay_ms: u64) -> TaskBuilder {
    TaskBuilder::new(TaskConfig::new(id, action_id).with_delay(delay_ms))
}

/// Create an interval task
pub fn interval(
    id: impl Into<String>,
    action_id: impl Into<String>,
    interval_ms: u64
) -> TaskBuilder {
    TaskBuilder::new(
        TaskConfig::new(id, action_id).with_interval(interval_ms).with_repeat(TaskRepeat::Forever)
    )
}

/// Create a complex task
pub fn complex(id: impl Into<String>, action_id: impl Into<String>) -> TaskBuilder {
    TaskBuilder::new(TaskConfig::new(id, action_id))
}

//=============================================================================
// TASK BUILDER
//=============================================================================

pub struct TaskBuilder {
    config: TaskConfig,
}

impl TaskBuilder {
    pub fn new(config: TaskConfig) -> Self {
        Self { config }
    }

    pub fn with_payload(mut self, payload: ActionPayload) -> Self {
        self.config = self.config.with_payload(payload);
        self
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.config = self.config.with_priority(priority);
        self
    }

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.config = self.config.with_delay(delay_ms);
        self
    }

    pub fn with_interval(mut self, interval_ms: u64) -> Self {
        self.config = self.config.with_interval(interval_ms);
        self
    }

    pub fn with_repeat(mut self, repeat: TaskRepeat) -> Self {
        self.config = self.config.with_repeat(repeat);
        self
    }

    pub fn schedule(self) -> Result<String, String> {
        let task = Task::new(self.config);
        let task_id = task.id.clone();
        keep(task)?;
        Ok(task_id)
    }
}

//=============================================================================
// FILTER AND RESULT TYPES
//=============================================================================

#[derive(Debug, Clone)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub task_type: Option<TaskType>,
}

impl TaskFilter {
    pub fn new() -> Self {
        Self {
            status: None,
            priority: None,
            task_type: None,
        }
    }

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn with_type(mut self, task_type: TaskType) -> Self {
        self.task_type = Some(task_type);
        self
    }
}

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub success: bool,
    pub task_id: String,
    pub execution_time: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TaskStats {
    pub total: usize,
    pub active: usize,
    pub pending: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
}

//=============================================================================
// SYSTEM HEALTH
//=============================================================================

#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_tasks: usize,
    pub overloaded: bool,
}

impl SystemHealth {
    pub fn check() -> Self {
        let stats = stats();
        Self {
            cpu_usage: 0.0, // Would integrate with system monitoring
            memory_usage: 0.0, // Would integrate with system monitoring
            active_tasks: stats.active,
            overloaded: stats.active > 100, // Simple threshold
        }
    }
}

//=============================================================================
// TASK METRICS
//=============================================================================

#[derive(Debug, Clone)]
pub struct TaskMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time: f64,
    pub last_execution: Option<u64>,
}

impl Default for TaskMetrics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_execution_time: 0.0,
            last_execution: None,
        }
    }
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_task_creation() {
        let config = TaskConfig::new("test-task", "test-action")
            .with_delay(1000)
            .with_priority(TaskPriority::High);

        let task = Task::new(config);

        assert_eq!(task.id, "test-task");
        assert_eq!(task.config.action_id, "test-action");
        assert!(task.is_active());
    }

    #[test]
    fn test_task_store_operations() {
        let mut store = TaskStore::new();

        let config = TaskConfig::new("test", "action");
        let task = Task::new(config);

        store.add(task);
        assert_eq!(store.count(), 1);

        let retrieved = store.get("test");
        assert!(retrieved.is_some());

        let removed = store.remove("test");
        assert!(removed.is_some());
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_global_api() {
        let config = TaskConfig::new("global-test", "action");
        let task = Task::new(config);

        let result = keep(task);
        assert!(result.is_ok());

        let retrieved = get("global-test");
        assert!(retrieved.is_some());

        let removed = forget("global-test");
        assert!(removed.is_ok());
        assert!(removed.unwrap());
    }

    #[test]
    fn test_task_builder() {
        let result = timeout("builder-test", "action", 5000)
            .with_payload(json!({"test": true}))
            .with_priority(TaskPriority::High)
            .schedule();

        assert!(result.is_ok());

        let task = get("builder-test");
        assert!(task.is_some());

        let task = task.unwrap();
        assert_eq!(task.config.delay, Some(5000));
        assert!(matches!(task.config.priority, TaskPriority::High));
    }
}
