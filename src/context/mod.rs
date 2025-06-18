// src/context/mod.rs
// Context module with state management, task store, sensor, and metrics

pub mod state;
pub mod task_store;
pub mod sensor;
pub mod metrics_state;

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
    // Legacy compatibility
    get_timeline,
    Timeline,
    TimelineStore,
    TimelineEntry,
    get_state_status,
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

// Re-export metrics state items - THE BRAIN OF THE SYSTEM
pub use metrics_state::{
    // Core metrics operations - PROPER NAMING
    get as metrics_get,
    set as metrics_set,
    update as metrics_update,
    reset as metrics_reset,
    // Lifecycle operations - PROPER NAMING
    init as metrics_init,
    lock as metrics_lock,
    unlock as metrics_unlock,
    shutdown as metrics_shutdown,
    // State checks - PROPER NAMING
    is_init as metrics_is_init,
    is_locked as metrics_is_locked,
    is_shutdown as metrics_is_shutdown,
    // Main update functions with proper naming
    breath as metrics_breath, // metrics_state::breath()
    system as metrics_system, // metrics_state::system()
    response as metrics_response, // metrics_state::response() - renamed from call
    store as metrics_store, // metrics_state::store()
    // Status and information functions
    status as metrics_status,
    // Lifecycle management
    enter_recuperation,
    exit_recuperation,
    enter_hibernation,
    exit_hibernation,
    // System information queries - with better names
    get_summary as metrics_summary, // Renamed from get_health_summary
    get_breathing_info,
    get_performance_summary,
    get_store_summary,
    needs_breathing,
    // Convenience functions
    update_system_metrics,
    update_breathing,
    update_store_metrics,
    record_call,
    record_channel_operation,
    record_protection_trigger,
    get_current_load,
    can_handle_load,
    // Aliases for consistency
    initialize as metrics_initialize,
    is_initialized as metrics_is_initialized,
    // Types
    QuantumState,
    SystemMetrics,
    BreathingState,
    PerformanceMetrics,
    SystemStress,
    StoreMetrics,
    HealthSummary,
    BreathingInfo,
    PerformanceSummary,
    StoreSummary,
    BreathingUpdate,
    SystemUpdate,
    StoreUpdate,
};
