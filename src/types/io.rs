// src/types/io.rs
// IO configuration types

use serde::{Serialize, Deserialize};
use super::{ActionId, ActionPayload, Priority, SmallTags, SmallMiddleware};

//=============================================================================
// IO CONFIGURATION
//=============================================================================

/// Complete configuration for actions and channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IO {
    // Basic identification
    pub id: ActionId,
    pub name: Option<String>,
    pub tags: SmallTags,
    pub payload: Option<ActionPayload>,
    
    // Timing system
    pub interval: Option<u64>,
    pub delay: Option<u64>,
    pub repeat: Option<i32>,
    
    // Protection system
    pub throttle: Option<u64>,
    pub debounce: Option<u64>,
    pub detect_changes: bool,
    pub required: bool,
    pub max_wait: Option<u64>,
    pub block: bool,
    
    // Advanced features
    pub priority: Priority,
    pub middleware: SmallMiddleware,
    pub log: bool,
    pub immutable: bool,
    pub no_dispatch: bool,
    
    // Performance optimization flags
    pub fast_path_eligible: bool,
    pub compiled_pipeline: bool,
    
    // Talent system integration
    pub talents: Vec<String>,
    pub schema_validation: bool,
    
    // Branch system
    pub branch_id: Option<String>,
    pub parent_branch: Option<String>,
}

impl Default for IO {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: None,
            tags: Vec::new(),
            payload: None,
            interval: None,
            delay: None,
            repeat: None,
            throttle: None,
            debounce: None,
            detect_changes: false,
            required: false,
            max_wait: None,
            block: false,
            priority: Priority::Medium,
            middleware: Vec::new(),
            log: false,
            immutable: false,
            no_dispatch: false,
            fast_path_eligible: true,
            compiled_pipeline: false,
            talents: Vec::new(),
            schema_validation: false,
            branch_id: None,
            parent_branch: None,
        }
    }
}

impl IO {
    /// Create a new IO configuration with just an ID
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Default::default()
        }
    }
    
    /// Builder pattern: Set name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    /// Builder pattern: Set priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Builder pattern: Set throttle
    pub fn with_throttle(mut self, throttle_ms: u64) -> Self {
        self.throttle = Some(throttle_ms);
        self.fast_path_eligible = false; // Throttle disables fast path
        self
    }
    
    /// Builder pattern: Set debounce
    pub fn with_debounce(mut self, debounce_ms: u64) -> Self {
        self.debounce = Some(debounce_ms);
        self.fast_path_eligible = false; // Debounce disables fast path
        self
    }
    
    /// Builder pattern: Enable change detection
    pub fn with_change_detection(mut self) -> Self {
        self.detect_changes = true;
        self.fast_path_eligible = false; // Change detection disables fast path
        self
    }
    
    /// Builder pattern: Add talent
    pub fn with_talent(mut self, talent_id: impl Into<String>) -> Self {
        self.talents.push(talent_id.into());
        self.fast_path_eligible = false; // Talents disable fast path
        self
    }
    
    /// Builder pattern: Set interval
    pub fn with_interval(mut self, interval_ms: u64) -> Self {
        self.interval = Some(interval_ms);
        self
    }
    
    /// Builder pattern: Set delay
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay = Some(delay_ms);
        self
    }
    
    /// Check if this configuration is eligible for fast path optimization
    pub fn is_fast_path_eligible(&self) -> bool {
        self.fast_path_eligible &&
        self.throttle.is_none() &&
        self.debounce.is_none() &&
        !self.detect_changes &&
        self.talents.is_empty() &&
        self.middleware.is_empty() &&
        !self.log &&
        !self.schema_validation
    }
    
    /// Check if this configuration has any protection mechanisms
    pub fn has_protection(&self) -> bool {
        self.throttle.is_some() || 
        self.debounce.is_some() || 
        self.detect_changes ||
        self.block
    }
    
    /// Check if this configuration has advanced features
    pub fn has_advanced_features(&self) -> bool {
        !self.talents.is_empty() ||
        !self.middleware.is_empty() ||
        self.schema_validation ||
        self.branch_id.is_some()
    }
}