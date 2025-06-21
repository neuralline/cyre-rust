// src/context/mod.rs
// Context module with state management, sensor, and metrics

pub mod state;
// pub mod task_store;  // TODO: Create this module when needed
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

// Re-export sensor items - FIXED IMPORTS
pub use sensor::{
    // Functions with prefixed names to avoid conflicts
    error as sensor_error,
    warn as sensor_warn,
    info as sensor_info,
    debug as sensor_debug,
    critical as sensor_critical,
    success as sensor_success,
    sys as sensor_sys,
    // Types
    LogLevel,
    Sensor,
    SensorBuilder,
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
    response as metrics_response, // metrics_state::response()
    status as metrics_status, // metrics_state::status()
    // Types
    BreathingUpdate,
    SystemUpdate,
    StoreUpdate,
    SystemMetrics,
    BreathingState,
    PerformanceMetrics,
    SystemStress,
    StoreMetrics,
    QuantumState,
};
