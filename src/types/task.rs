// src/types/task.rs
use serde::{Serialize, Deserialize};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use crate::types::{ActionPayload, Priority};
use crate::timekeeper::TimerRepeat;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Simple,
    Orchestration,
    SystemMonitor,
    Maintenance,
    DataCollection,
}

#[derive(Debug, Clone)]
pub struct TaskContext {
    pub task_id: String,
    pub trigger_time: u64,
    pub system_metrics: SystemMetrics,
    pub cyre_handle: CyreHandle,
}

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_load: f64,
    pub memory_usage_mb: u64,
    pub free_memory_mb: u64,
    pub disk_space_gb: u64,
    pub call_rate: f64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub enum TaskResult {
    Success { message: String, data: JsonValue },
    Error { error: String },
    Skipped { reason: String },
}

#[derive(Debug, Clone)]
pub struct TaskExecution {
    pub timestamp: u64,
    pub duration: u64,
    pub result: TaskResult,
}

#[derive(Debug, Clone)]
pub struct TaskMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub skipped_executions: u64,
    pub average_duration: f64,
    pub last_execution_time: Option<u64>,
    pub longest_execution: u64,
    pub shortest_execution: u64,
}

#[derive(Debug, Clone)]
pub struct TimerConfig {
    pub interval: u64,
    pub delay: Option<u64>,
    pub repeat: TimerRepeat,
    pub priority: Priority,
}

#[derive(Debug, Clone)]
pub enum TriggerCondition {
    Interval { interval_ms: u64, delay_ms: Option<u64> },
    Schedule { cron_expression: String },
    SystemLoad { threshold: f64 },
    MemoryAvailable { min_mb: u64 },
    DiskSpace { min_gb: u64 },
    ChannelSuccess { channel: String },
    ChannelFailure { channel: String },
    FileExists { path: String },
    External { trigger_name: String },
}

// Core Task trait
#[async_trait::async_trait]
pub trait Task: Send + Sync + std::fmt::Debug {
    fn id(&self) -> &str;
    fn task_type(&self) -> TaskType;
    
    // Trigger evaluation
    async fn should_trigger(&self, context: &TaskContext) -> bool;
    
    // Main execution
    async fn execute(&mut self, context: &TaskContext) -> TaskResult;
    
    // TimeKeeper integration
    fn create_timers(&self) -> Vec<TimerConfig>;
    
    // Lifecycle
    fn clone_for_timeline(&self) -> Box<dyn Task>;
    async fn on_activate(&mut self) -> Result<(), String> { Ok(()) }
    async fn on_deactivate(&mut self) -> Result<(), String> { Ok(()) }
    
    // Next execution time for timeline
    fn next_execution_time(&self, current_time: u64) -> Option<u64>;
}