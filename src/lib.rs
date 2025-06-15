// src/lib.rs
// Cyre Rust - Fixed module structure and exports

//=============================================================================
// MODULE DECLARATIONS - FIXED
//=============================================================================

// Core types module (using mod.rs structure)
pub mod types;

// Other modules
pub mod protection;  // Protection mechanisms  
pub mod talent;      // Talent system
pub mod breathing;   // Quantum breathing system
pub mod timekeeper;  // TimeKeeper system
pub mod timeline;    // Timeline and scheduling (legacy)
pub mod branch;      // Branch system
pub mod channel;     // Channel implementation
pub mod core;        // Main Cyre implementation
pub mod context;     // Context and state management - ADDED
pub mod utils;       // Utility functions

//=============================================================================
// RE-EXPORTS FOR PUBLIC API - FIXED
//=============================================================================

// Core types
pub use types::{
    ActionId, ActionPayload, Priority,
    CyreResponse, TalentResult, ValidationResult,
    IO
};

// Main implementation
pub use core::Cyre;

// TimeKeeper system (fixed exports)
pub use timekeeper::{
    TimeKeeper, Formation, FormationBuilder, TimerRepeat,
    get_timekeeper, set_timeout, set_interval, clear_timer, delay,
};

// Advanced systems
pub use talent::{Talent, TalentType, TalentRegistry};
pub use breathing::{QuantumBreathing, BreathingPattern};
pub use timeline::{Timeline, TimelineEntry};
pub use branch::{BranchSystem, BranchEntry};
pub use channel::Channel;
pub use protection::{ProtectionState, ProtectionType};

// Context and state
pub use context::{TimelineStore};

// Utility functions
pub use utils::current_timestamp;

//=============================================================================
// FEATURE FLAGS AND CONDITIONAL COMPILATION
//=============================================================================

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "benchmarks")]
pub mod benchmarks;

//=============================================================================
// CONVENIENCE MACROS
//=============================================================================

/// Timeout macro - setTimeout equivalent
#[macro_export]
macro_rules! timeout {
    ($action:expr, $payload:expr, $delay:expr) => {
        $crate::timekeeper::set_timeout($action, $payload, $delay)
    };
}

/// Interval macro - setInterval equivalent
#[macro_export]
macro_rules! interval {
    ($action:expr, $payload:expr, $interval:expr) => {
        $crate::timekeeper::set_interval($action, $payload, $interval)
    };
}

/// Sleep macro - async delay
#[macro_export]
macro_rules! sleep {
    ($duration:expr) => {
        $crate::timekeeper::delay($duration)
    };
}

//=============================================================================
// LIBRARY METADATA
//=============================================================================

/// Cyre Rust version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library description
pub const DESCRIPTION: &str = "High-performance reactive event manager with TimeKeeper";

//=============================================================================
// PRELUDE MODULE FOR CONVENIENCE - FIXED
//=============================================================================

/// Complete imports for Cyre users
pub mod prelude {
    pub use crate::{
        // Core Cyre
        Cyre, IO, CyreResponse, ActionPayload, Priority,
        
        // TimeKeeper
        TimeKeeper, Formation, FormationBuilder, TimerRepeat,
        get_timekeeper, set_timeout, set_interval, clear_timer, delay,
        
        // Advanced systems
        Talent, TalentType, QuantumBreathing,
        
        // Utilities
        current_timestamp,
    };
    
    // Macros
    pub use crate::{timeout, interval, sleep};
    
    // Common traits and types for async programming
    pub use std::future::Future;
    pub use std::pin::Pin;
    pub use serde_json::{json, Value};
}

//=============================================================================
// CYRE BUILDER - SIMPLIFIED
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
        let mut cyre = Cyre::new();

        if self.enable_timekeeper {
            cyre.init_timekeeper().await?;
            println!("✅ TimeKeeper initialized!");
        }

        if self.enable_breathing {
            println!("✅ Quantum breathing system initialized");
        }

        if self.enable_talents {
            println!("✅ Talent system initialized");
        }

        Ok(cyre)
    }
}

impl Default for CyreBuilder {
    fn default() -> Self {
        Self::new()
    }
}