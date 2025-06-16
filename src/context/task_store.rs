// src/context/task_store.rs
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::types::task::{Task, TaskExecution, TaskMetrics, TaskType};

#[derive(Debug)]
pub struct TaskEntry {
    pub task: Box<dyn Task>,
    pub active: bool,
    pub created_at: u64,
    pub last_execution: Option<TaskExecution>,
    pub execution_count: u64,
    pub timer_ids: Vec<String>,
    pub metrics: TaskMetrics,
}

impl TaskEntry {
    pub fn new(task: Box<dyn Task>) -> Self {
        Self {
            task,
            active: false,
            created_at: crate::utils::current_timestamp(),
            last_execution: None,
            execution_count: 0,
            timer_ids: Vec::new(),
            metrics: TaskMetrics {
                total_executions: 0,
                successful_executions: 0,
                failed_executions: 0,
                skipped_executions: 0,
                average_duration: 0.0,
                last_execution_time: None,
                longest_execution: 0,
                shortest_execution: u64::MAX,
            },
        }
    }
}

#[derive(Debug)]
pub struct TaskStore {
    tasks: HashMap<String, TaskEntry>,
}

impl TaskStore {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }
    
    pub fn add(&mut self, task_id: String, task: Box<dyn Task>) {
        let entry = TaskEntry::new(task);
        self.tasks.insert(task_id, entry);
    }
    
    pub fn get(&self, task_id: &str) -> Option<&TaskEntry> {
        self.tasks.get(task_id)
    }
    
    pub fn get_mut(&mut self, task_id: &str) -> Option<&mut TaskEntry> {
        self.tasks.get_mut(task_id)
    }
    
    pub fn remove(&mut self, task_id: &str) -> Option<TaskEntry> {
        self.tasks.remove(task_id)
    }
    
    pub fn get_all(&self) -> Vec<&TaskEntry> {
        self.tasks.values().collect()
    }
    
    pub fn get_active(&self) -> Vec<&TaskEntry> {
        self.tasks.values().filter(|entry| entry.active).collect()
    }
    
    pub fn get_by_type(&self, task_type: TaskType) -> Vec<&TaskEntry> {
        self.tasks.values()
            .filter(|entry| entry.task.task_type() == task_type)
            .collect()
    }
}

// src/timeline/mod.rs - Updated for task integration
use std::collections::{HashMap, BinaryHeap};
use crate::types::task::Task;

#[derive(Debug)]
pub struct TimelineEntry {
    pub task_id: String,
    pub task: Box<dyn Task>,
    pub next_execution: u64,
    pub added_at: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ScheduledExecution {
    pub task_id: String,
    pub execution_time: u64,
}

impl Ord for ScheduledExecution {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse ordering for min-heap (earliest execution first)
        other.execution_time.cmp(&self.execution_time)
    }
}

impl PartialOrd for ScheduledExecution {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct Timeline {
    active_tasks: HashMap<String, TimelineEntry>,
    execution_queue: BinaryHeap<ScheduledExecution>,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            active_tasks: HashMap::new(),
            execution_queue: BinaryHeap::new(),
        }
    }
    
    pub fn add_task(&mut self, task_id: &str, task: Box<dyn Task>, next_execution: u64) {
        let entry = TimelineEntry {
            task_id: task_id.to_string(),
            task,
            next_execution,
            added_at: crate::utils::current_timestamp(),
        };
        
        self.active_tasks.insert(task_id.to_string(), entry);
        self.execution_queue.push(ScheduledExecution {
            task_id: task_id.to_string(),
            execution_time: next_execution,
        });
    }
    
    pub fn remove_task(&mut self, task_id: &str) -> Option<TimelineEntry> {
        // Note: Can't efficiently remove from BinaryHeap, but will be filtered out during execution
        self.active_tasks.remove(task_id)
    }
    
    pub fn get_next_execution(&self, task_id: &str) -> Option<u64> {
        self.active_tasks.get(task_id).map(|entry| entry.next_execution)
    }
    
    pub fn get_ready_tasks(&mut self, current_time: u64) -> Vec<String> {
        let mut ready_tasks = Vec::new();
        
        while let Some(scheduled) = self.execution_queue.peek() {
            if scheduled.execution_time <= current_time {
                let scheduled = self.execution_queue.pop().unwrap();
                
                // Check if task is still active (not removed)
                if self.active_tasks.contains_key(&scheduled.task_id) {
                    ready_tasks.push(scheduled.task_id);
                }
            } else {
                break;
            }
        }
        
        ready_tasks
    }
}