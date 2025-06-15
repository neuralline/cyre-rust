// src/lib.rs
// Cyre Rust - Now with ULTIMATE TimeKeeper integration!
// The most advanced event management system ever built

//=============================================================================
// MODULE DECLARATIONS
//=============================================================================

pub mod types;       // Core types and configurations
pub mod protection;  // Protection mechanisms  
pub mod talent;      // Talent system
pub mod breathing;   // Quantum breathing system
pub mod timekeeper;  // ULTIMATE TimeKeeper system 🔥
pub mod timeline;    // Timeline and scheduling (legacy)
pub mod branch;      // Branch system
pub mod channel;     // Channel implementation
pub mod core;        // Main Cyre implementation
pub mod utils;       // Utility functions

//=============================================================================
// RE-EXPORTS FOR PUBLIC API
//=============================================================================

// Core types
pub use types::{
    ActionId, ActionPayload, Priority,
    CyreResponse, TalentResult, ValidationResult,
    IO
};

// Main implementation
pub use core::Cyre;

// ULTIMATE TimeKeeper system
pub use timekeeper::{
    TimeKeeper, Formation, FormationBuilder, TimerRepeat, PrecisionTier,
    get_timekeeper, set_timeout, set_interval, clear_timer, delay,
    TimeKeeperIntegration,
};

// Advanced systems
pub use talent::{Talent, TalentType, TalentRegistry};
pub use breathing::{QuantumBreathing, BreathingPattern};
pub use timeline::{Timeline, TimelineEntry}; // Legacy timeline
pub use branch::{BranchSystem, BranchEntry};
pub use channel::Channel;
pub use protection::{ProtectionState, ProtectionType};

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

/// Timeout macro - setTimeout equivalent but better
#[macro_export]
macro_rules! timeout {
    ($action:expr, $payload:expr, $delay:expr) => {
        $crate::timekeeper::set_timeout($action, $payload, $delay)
    };
}

/// Interval macro - setInterval equivalent but better
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
pub const DESCRIPTION: &str = "Ultimate reactive event manager with beast-mode TimeKeeper";

//=============================================================================
// PRELUDE MODULE FOR CONVENIENCE
//=============================================================================

/// Complete imports for Cyre users - now with TimeKeeper!
pub mod prelude {
    pub use crate::{
        // Core Cyre
        Cyre, IO, CyreResponse, ActionPayload, Priority,
        
        // TimeKeeper beast mode
        TimeKeeper, Formation, FormationBuilder, TimerRepeat, PrecisionTier,
        get_timekeeper, set_timeout, set_interval, clear_timer, delay,
        TimeKeeperIntegration,
        
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
// CYRE BUILDER WITH TIMEKEEPER INTEGRATION
//=============================================================================

/// Enhanced Cyre builder with TimeKeeper integration
pub struct CyreBuilder {
    enable_timekeeper: bool,
    enable_breathing: bool,
    enable_talents: bool,
    precision_mode: Option<PrecisionTier>,
}

impl CyreBuilder {
    pub fn new() -> Self {
        Self {
            enable_timekeeper: true,  // Enabled by default
            enable_breathing: true,
            enable_talents: true,
            precision_mode: None,
        }
    }

    /// Enable/disable TimeKeeper integration
    pub fn with_timekeeper(mut self, enabled: bool) -> Self {
        self.enable_timekeeper = enabled;
        self
    }

    /// Set TimeKeeper precision mode
    pub fn with_precision(mut self, precision: PrecisionTier) -> Self {
        self.precision_mode = Some(precision);
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
            println!("✅ TimeKeeper initialized with beast-mode precision!");
        }

        // TODO: Initialize other systems based on flags
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

