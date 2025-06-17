// src/lib.rs
// Cyre Rust - Main library with proper module structure

//=============================================================================
// MODULE DECLARATIONS
//=============================================================================

// Core types module
pub mod types;

// Other modules
pub mod protection; // Protection mechanisms
pub mod talent; // Talent system
pub mod breathing; // Quantum breathing system
pub mod timekeeper; // TimeKeeper system
pub mod timeline; // Timeline and scheduling (legacy)
pub mod branch; // Branch system
pub mod channel; // Channel implementation
pub mod core; // Main Cyre implementation
pub mod context; // Context and state management with task store
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

// Context and task management
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
    // Timeline functions
    get_timeline,
    Timeline,
    TimelineStore,
};

// Advanced systems
pub use talent::{ Talent, TalentType, TalentRegistry };
pub use breathing::{ QuantumBreathing, BreathingPattern };
pub use timeline::{ Timeline as LegacyTimeline, TimelineEntry };
pub use branch::{ BranchSystem, BranchEntry };
pub use channel::Channel;
pub use protection::{ ProtectionState, ProtectionType };

// Utility functions
pub use utils::current_timestamp;

//=============================================================================
// CONVENIENCE MACROS
//=============================================================================

/// Timeout macro - setTimeout equivalent
#[macro_export]
macro_rules! timeout {
    ($action:expr_2021, $payload:expr_2021, $delay:expr_2021) => {
        $crate::timekeeper::set_timeout($action, $payload, $delay)
    };
}

/// Interval macro - setInterval equivalent
#[macro_export]
macro_rules! interval {
    ($action:expr_2021, $payload:expr_2021, $interval:expr_2021) => {
        $crate::timekeeper::set_interval($action, $payload, $interval)
    };
}

/// Sleep macro - async delay
#[macro_export]
macro_rules! sleep {
    ($duration:expr_2021) => {
        $crate::timekeeper::delay($duration)
    };
}

//=============================================================================
// PRELUDE MODULE FOR CONVENIENCE
//=============================================================================

/// Complete imports for Cyre users
pub mod prelude {
    pub use crate::{
        // Core Cyre
        Cyre,
        IO,
        CyreResponse,
        ActionPayload,
        Priority,
        // TimeKeeper
        TimeKeeper,
        Formation,
        FormationBuilder,
        TimerRepeat,
        get_timekeeper,
        set_timeout,
        set_interval,
        clear_timer,
        delay,
        // Task Store (from context)
        task_keep,
        task_forget,
        task_activate,
        task_get,
        task_list,
        task_stats,
        task_timeout,
        task_interval,
        task_complex,
        TaskBuilder,
        TaskStatus,
        TaskType,
        TaskPriority,
        TaskRepeat,
        Task,
        TaskConfig,
        TaskResult,
        TaskStats,
        // Timeline
        get_timeline,
        Timeline,
        TimelineStore,
        // Advanced systems
        Talent,
        TalentType,
        QuantumBreathing,
        // Utilities
        current_timestamp,
    };

    // Macros
    pub use crate::{ timeout, interval, sleep };

    // Common async traits
    pub use std::future::Future;
    pub use std::pin::Pin;
    pub use serde_json::{ json, Value };
}

//=============================================================================
// CYRE BUILDER
//=============================================================================

/// Enhanced Cyre builder
pub struct CyreBuilder {
    enable_timekeeper: bool,
    enable_breathing: bool,
    enable_talents: bool,
}

impl CyreBuilder {
    pub fn new() -> Self {
        Self {
            enable_timekeeper: true,
            enable_breathing: true,
            enable_talents: true,
        }
    }

    /// Enable/disable TimeKeeper integration
    pub fn with_timekeeper(mut self, enabled: bool) -> Self {
        self.enable_timekeeper = enabled;
        self
    }

    /// Enable/disable quantum breathing
    pub fn with_breathing(mut self, enabled: bool) -> Self {
        self.enable_breathing = enabled;
        self
    }

    /// Enable/disable talent system
    pub fn with_talents(mut self, enabled: bool) -> Self {
        self.enable_talents = enabled;
        self
    }

    /// Build the Cyre instance
    pub async fn build(self) -> Result<Cyre, String> {
        let cyre = Cyre::new();

        if self.enable_timekeeper {
            cyre.init_timekeeper().await?;
            println!("✅ TimeKeeper integration enabled");
        }

        if self.enable_breathing {
            println!("✅ Quantum breathing enabled");
        }

        if self.enable_talents {
            println!("✅ Talent system enabled");
        }

        Ok(cyre)
    }
}

//=============================================================================
// FEATURE FLAGS AND CONDITIONAL COMPILATION
//=============================================================================

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "benchmarks")]
pub mod benchmarks;

//=============================================================================
// LIBRARY METADATA
//=============================================================================

/// Cyre Rust version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library description
pub const DESCRIPTION: &str =
    "High-performance reactive event manager with TimeKeeper and Task Store";
