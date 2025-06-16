// src/task/simple_task.rs
use async_trait::async_trait;
use serde_json::json;
use crate::types::task::{Task, TaskType, TaskContext, TaskResult, TriggerCondition, TimerConfig};
use crate::types::{Priority, ActionPayload};
use crate::timekeeper::TimerRepeat;

#[derive(Debug, Clone)]
pub struct BreathingConfig {
    pub adapt_to_stress: bool,
    pub stress_multiplier: f64,
    pub pause_threshold: f64,
    pub resume_threshold: f64,
}

impl Default for BreathingConfig {
    fn default() -> Self {
        Self {
            adapt_to_stress: true,
            stress_multiplier: 1.5,
            pause_threshold: 0.8,
            resume_threshold: 0.3,
        }
    }
}

#[derive(Debug)]
pub struct SimpleTask {
    pub id: String,
    pub triggers: Vec<TriggerCondition>,
    pub target_channels: Vec<String>,
    pub breathing_config: Option<BreathingConfig>,
}

impl SimpleTask {
    pub fn new(id: String, triggers: Vec<TriggerCondition>, target_channels: Vec<String>) -> Self {
        Self {
            id,
            triggers,
            target_channels,
            breathing_config: None,
        }
    }
    
    pub fn with_breathing(mut self, config: BreathingConfig) -> Self {
        self.breathing_config = Some(config);
        self
    }
    
    async fn evaluate_trigger_conditions(&self, context: &TaskContext) -> bool {
        for trigger in &self.triggers {
            match trigger {
                TriggerCondition::SystemLoad { threshold } => {
                    if context.system_metrics.cpu_load > *threshold {
                        return false;
                    }
                }
                TriggerCondition::MemoryAvailable { min_mb } => {
                    if context.system_metrics.free_memory_mb < *min_mb {
                        return false;
                    }
                }
                TriggerCondition::DiskSpace { min_gb } => {
                    if context.system_metrics.disk_space_gb < *min_gb {
                        return false;
                    }
                }
                TriggerCondition::FileExists { path } => {
                    if !std::path::Path::new(path).exists() {
                        return false;
                    }
                }
                // Add other condition evaluations
                _ => continue,
            }
        }
        true
    }
}

#[async_trait]
impl Task for SimpleTask {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn task_type(&self) -> TaskType {
        TaskType::Simple
    }
    
    async fn should_trigger(&self, context: &TaskContext) -> bool {
        // Check breathing system first
        if let Some(ref breathing) = self.breathing_config {
            if breathing.adapt_to_stress {
                let stress_level = context.system_metrics.cpu_load / 100.0;
                if stress_level > breathing.pause_threshold {
                    return false; // System too stressed
                }
            }
        }
        
        // Evaluate trigger conditions
        self.evaluate_trigger_conditions(context).await
    }
    
    async fn execute(&mut self, context: &TaskContext) -> TaskResult {
        if self.target_channels.is_empty() {
            return TaskResult::Error {
                error: "No target channels configured".to_string(),
            };
        }
        
        let mut results = Vec::new();
        let mut errors = Vec::new();
        
        for channel in &self.target_channels {
            match context.cyre_handle.call(channel, json!({})).await {
                Ok(response) => {
                    results.push(json!({
                        "channel": channel,
                        "result": response,
                        "status": "success"
                    }));
                }
                Err(error) => {
                    errors.push(json!({
                        "channel": channel,
                        "error": error,
                        "status": "failed"
                    }));
                }
            }
        }
        
        if errors.is_empty() {
            TaskResult::Success {
                message: format!("Triggered {} channels successfully", results.len()),
                data: json!({
                    "channels_triggered": results.len(),
                    "results": results
                }),
            }
        } else {
            TaskResult::Error {
                error: format!("Failed to trigger {} channels: {:?}", errors.len(), errors),
            }
        }
    }
    
    fn create_timers(&self) -> Vec<TimerConfig> {
        let mut timers = Vec::new();
        
        for trigger in &self.triggers {
            match trigger {
                TriggerCondition::Interval { interval_ms, delay_ms } => {
                    timers.push(TimerConfig {
                        interval: *interval_ms,
                        delay: *delay_ms,
                        repeat: TimerRepeat::Forever,
                        priority: Priority::Normal,
                    });
                }
                TriggerCondition::Schedule { cron_expression: _ } => {
                    // Convert cron to next execution time
                    // For now, simplified to daily
                    timers.push(TimerConfig {
                        interval: 24 * 60 * 60 * 1000, // Daily
                        delay: None,
                        repeat: TimerRepeat::Forever,
                        priority: Priority::Normal,
                    });
                }
                _ => {
                    // For condition-based triggers, check every 30 seconds
                    timers.push(TimerConfig {
                        interval: 30000,
                        delay: None,
                        repeat: TimerRepeat::Forever,
                        priority: Priority::Low,
                    });
                }
            }
        }
        
        timers
    }
    
    fn clone_for_timeline(&self) -> Box<dyn Task> {
        Box::new(self.clone())
    }
    
    fn next_execution_time(&self, current_time: u64) -> Option<u64> {
        // Return the earliest next execution time from all triggers
        let mut next_time = None;
        
        for trigger in &self.triggers {
            let trigger_next = match trigger {
                TriggerCondition::Interval { interval_ms, delay_ms } => {
                    current_time + delay_ms.unwrap_or(*interval_ms)
                }
                TriggerCondition::Schedule { cron_expression: _ } => {
                    // Simplified: next day at same time
                    current_time + (24 * 60 * 60 * 1000)
                }
                _ => current_time + 30000, // Check conditions every 30s
            };
            
            next_time = Some(next_time.map_or(trigger_next, |t| t.min(trigger_next)));
        }
        
        next_time
    }
}