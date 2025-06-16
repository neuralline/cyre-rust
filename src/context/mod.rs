// src/context/mod.rs
// Context module with state management and task store

pub mod state;
pub mod task_store;

// Re-export key items for easy access
pub use state::{ get_timeline, Timeline, TimelineStore };
pub use task_store::{
    // Core functions
    keep,
    forget,
    activate,
    get,
    list,
    stats,
    // Convenience builders
    timeout,
    interval,
    complex,
    // Types
    TaskBuilder,
    TaskFilter,
    TaskResult,
    TaskStats,
    TaskStatus,
    TaskType,
    TaskPriority,
    TaskRepeat,
    Task,
    TaskConfig,
    TaskMetrics,
    SystemHealth,
};
