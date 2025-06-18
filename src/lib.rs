// src/lib.rs
// Cyre Rust - Main library with proper module structure

//=============================================================================
// MODULE DECLARATIONS
//=============================================================================

// Core types module
pub mod types;
pub mod config;

// Other modules
pub mod breathing; // Quantum breathing system
pub mod timekeeper; // TimeKeeper system
pub mod branch; // Branch system
pub mod channel; // Channel implementation
pub mod core; // Main Cyre implementation
pub mod pipeline; // Pipeline system for enhanced Cyre
pub mod context; // Context and state management with task store + sensor
pub mod orchestration; // Orchestration system
pub mod utils; // Utility functions

//=============================================================================
// RE-EXPORTS FOR PUBLIC API
//=============================================================================

// Core types
pub use types::{
    ActionId,
    ActionPayload,
    Priority,
    CyreResponse,
    TalentResult,
    ValidationResult,
    IO,
    SchemaFunction,
    ConditionFunction,
    TransformFunction,
    SelectorFunction,
};

// Main implementation
pub use core::Cyre;

// TimeKeeper system
pub use timekeeper::{
    TimeKeeper,
    Formation,
    FormationBuilder,
    TimerRepeat,
    get_timekeeper,
    set_timeout,
    set_interval,
    clear_timer,
    delay,
};

// Context and state management
pub use context::{
    // Task store functions
    keep as task_keep,
    forget as task_forget,
    activate as task_activate,
    get as task_get,
    list as task_list,
    stats as task_stats,
    timeout as task_timeout,
    interval as task_interval,
    complex as task_complex,
    // Task store types
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
    // Enhanced state system - NEW
    io,
    subscribers,
    timeline,
    stores,
    StateKey,
    MetricsState,
    MetricsUpdate,
    StateActionMetrics,
    ISubscriber,
    BranchStore,
    MetricsOps,
    // Legacy compatibility
    get_timeline,
    Timeline,
    TimelineStore,
    // Sensor system
    sensor_log,
    sensor_success,
    sensor_error,
    sensor_warn,
    sensor_info,
    sensor_debug,
    sensor_critical,
    sensor_sys,
    LogLevel,
    Sensor,
    SENSOR,
    // Metrics state system
    metrics_get,
    metrics_set,
    metrics_update,
    metrics_reset,
    metrics_init,
    metrics_lock,
    metrics_unlock,
    metrics_shutdown,
    metrics_is_init,
    metrics_is_locked,
    metrics_is_shutdown,
    metrics_breath,
    metrics_system,
    metrics_response,
    metrics_store,
    metrics_status,
    metrics_summary,
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

// Breathing system
pub use breathing::{
    start_breathing,
    stop_breathing,
    is_breathing,
    force_breath,
    get_breathing_status,
    is_in_recuperation,
    is_hibernating,
    get_stress_level,
};

// Utility functions
pub use utils::{ current_timestamp, generate_id };

//=============================================================================
// MAIN LIBRARY INITIALIZATION FUNCTION
//=============================================================================

/// Initialize a new Cyre instance with full system setup
pub async fn init() -> Result<Cyre, Box<dyn std::error::Error>> {
    let mut cyre = Cyre::new();

    // Initialize the core system
    match cyre.init().await {
        Ok(_response) => {
            // Cyre initialization includes:
            // - Metrics system initialization
            // - Breathing system startup
            // - State stores initialization
            Ok(cyre)
        }
        Err(e) => Err(e.into()),
    }
}

/// Create a new Cyre instance (alias for convenience)
pub fn new() -> Cyre {
    Cyre::new()
}

/// Quick setup function for common use cases
pub async fn quick_setup() -> Result<Cyre, Box<dyn std::error::Error>> {
    init().await
}

//=============================================================================
// VERSION AND METADATA
//=============================================================================

/// Current Cyre version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library metadata
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Get runtime information
pub fn runtime_info() -> serde_json::Value {
    serde_json::json!({
        "version": VERSION,
        "description": DESCRIPTION,
        "features": [
            "reactive_events",
            "quantum_breathing",
            "state_management",
            "protection_systems",
            "timeline_tracking",
            "metrics_monitoring"
        ],
        "timestamp": utils::current_timestamp()
    })
}

//=============================================================================
// PRELUDE MODULE FOR CONVENIENT IMPORTS
//=============================================================================

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        // Core types
        Cyre,
        IO,
        ActionPayload,
        CyreResponse,
        Priority,
        // Main functions
        init,
        new,
        quick_setup,
        // State management
        io,
        subscribers,
        timeline,
        stores,
        // Metrics
        metrics_init,
        metrics_status,
        metrics_summary,
        // Breathing
        start_breathing,
        stop_breathing,
        is_breathing,
        // Task store
        task_keep,
        task_forget,
        task_activate,
        task_get,
        // Sensor
        sensor_log,
        sensor_error,
        sensor_success,
        // Utilities
        current_timestamp,
        generate_id,
        // Pipeline functions for tests
    };
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_library_initialization() {
        let cyre = init().await;
        assert!(cyre.is_ok());

        let cyre = cyre.unwrap();
        assert!(cyre.is_initialized());
    }

    #[test]
    fn test_runtime_info() {
        let info = runtime_info();
        assert!(info["version"].is_string());
        assert!(info["features"].is_array());
        assert!(info["timestamp"].is_number());
    }

    #[test]
    fn test_version_constants() {
        assert!(!VERSION.is_empty());
        assert!(!DESCRIPTION.is_empty());
    }

    #[tokio::test]
    async fn test_quick_setup() {
        let result = quick_setup().await;
        assert!(result.is_ok());
    }
}
