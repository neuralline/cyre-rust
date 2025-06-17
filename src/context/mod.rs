// src/context/mod.rs
// Context module with state management, task store, and sensor

pub mod state;
pub mod task_store;
pub mod sensor;

// Re-export enhanced state system (with wrapper functions)
pub use state::{
    // State operations (organized like TypeScript) - use wrapper functions
    io,
    subscribers,
    timeline,
    stores,
    // State management types
    StateKey,
    MetricsState,
    MetricsUpdate,
    StateActionMetrics,
    ISubscriber,
    BranchStore,
    // Operations
    MetricsOps,
    PayloadStateOps,
    // Legacy compatibility
    get_timeline,
    Timeline,
    TimelineStore,
};

// Re-export task store items
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

// Re-export sensor items
pub use sensor::{
    // Functions with prefixed names to avoid conflicts
    log as sensor_log,
    success as sensor_success,
    error as sensor_error,
    warn as sensor_warn,
    info as sensor_info,
    debug as sensor_debug,
    critical as sensor_critical,
    sys as sensor_sys,
    // Types
    LogLevel,
    Sensor,
    SENSOR,
};
