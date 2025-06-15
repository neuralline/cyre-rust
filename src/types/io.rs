// src/types/io.rs - Enhanced with TimeKeeper integration
// Complete IO configuration with delay, interval, and repeat support

use serde::{Serialize, Deserialize};
use super::{ActionId, ActionPayload, Priority, SmallTags, SmallMiddleware};

//=============================================================================
// IO CONFIGURATION WITH TIMEKEEPER SUPPORT
//=============================================================================

/// Complete configuration for actions and channels with TimeKeeper integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IO {
    // Basic identification
    pub id: ActionId,
    pub name: Option<String>,
    pub tags: SmallTags,
    pub payload: Option<ActionPayload>,
    
    // TimeKeeper timing system
    pub delay: Option<u64>,           // Initial delay before first execution (setTimeout)
    pub interval: Option<u64>,        // Repeat interval (setInterval)
    pub repeat: Option<i32>,          // Repeat count (-1 = infinite, 0 = once, n = n times)
    pub timeout: Option<u64>,         // Maximum execution time
    
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
    pub timekeeper_enabled: bool,     // Indicates TimeKeeper management
    
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
            delay: None,
            interval: None,
            repeat: None,
            timeout: None,
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
            timekeeper_enabled: false,
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
    
    //=============================================================================
    // TIMEKEEPER INTEGRATION METHODS
    //=============================================================================
    
    /// Set initial delay (setTimeout equivalent)
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay = Some(delay_ms);
        self.timekeeper_enabled = true;
        self.compiled_pipeline = true;
        self.fast_path_eligible = false; // TimeKeeper disables fast path
        self
    }
    
    /// Set repeat interval (setInterval equivalent)
    pub fn with_interval(mut self, interval_ms: u64) -> Self {
        self.interval = Some(interval_ms);
        self.repeat = Some(-1); // Infinite by default
        self.timekeeper_enabled = true;
        self.compiled_pipeline = true;
        self.fast_path_eligible = false;
        self
    }
    
    /// Set finite repeat count
    pub fn with_repeat(mut self, count: u32) -> Self {
        self.repeat = Some(count as i32);
        if self.interval.is_none() {
            self.interval = Some(1000); // Default 1 second interval
        }
        self.timekeeper_enabled = true;
        self.compiled_pipeline = true;
        self.fast_path_eligible = false;
        self
    }
    
    /// Set infinite repeat (runs forever)
    pub fn with_infinite_repeat(mut self) -> Self {
        self.repeat = Some(-1);
        if self.interval.is_none() {
            self.interval = Some(1000); // Default 1 second interval
        }
        self.timekeeper_enabled = true;
        self.compiled_pipeline = true;
        self.fast_path_eligible = false;
        self
    }
    
    /// Set timeout (maximum execution time)
    pub fn timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout = Some(timeout_ms);
        self
    }
    
    /// Alternative timeout method
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout = Some(timeout_ms);
        self
    }
    
    /// Complex scheduling: delay + interval + repeat
    pub fn schedule_complex(mut self, delay_ms: u64, interval_ms: u64, repeat_count: u32) -> Self {
        self.delay = Some(delay_ms);
        self.interval = Some(interval_ms);
        self.repeat = Some(repeat_count as i32);
        self.timekeeper_enabled = true;
        self.compiled_pipeline = true;
        self.fast_path_eligible = false;
        self
    }
    
    //=============================================================================
    // COMPILED PIPELINE DETECTION
    //=============================================================================
    
    /// Check if this configuration requires TimeKeeper
    pub fn needs_timekeeper(&self) -> bool {
        self.delay.is_some() || 
        self.interval.is_some() || 
        self.repeat.is_some()
    }
    
    /// Check if this configuration has scheduling
    pub fn has_scheduling(&self) -> bool {
        self.timekeeper_enabled || self.needs_timekeeper()
    }
    
    /// Get scheduling type for pipeline compilation
    pub fn get_scheduling_type(&self) -> SchedulingType {
        match (self.delay.is_some(), self.interval.is_some(), self.repeat) {
            (true, false, _) => SchedulingType::DelayOnly,
            (false, true, Some(-1)) => SchedulingType::IntervalInfinite,
            (false, true, Some(n)) if n > 0 => SchedulingType::IntervalFinite,
            (true, true, Some(-1)) => SchedulingType::ComplexInfinite,
            (true, true, Some(n)) if n > 0 => SchedulingType::ComplexFinite,
            _ => SchedulingType::Immediate
        }
    }
    
    /// Check if this configuration is eligible for fast path optimization
    pub fn is_fast_path_eligible(&self) -> bool {
        self.fast_path_eligible &&
        !self.has_scheduling() &&
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
        self.branch_id.is_some() ||
        self.has_scheduling()
    }
    
    /// Get compiled pipeline priority
    pub fn get_pipeline_priority(&self) -> PipelinePriority {
        if self.has_scheduling() {
            PipelinePriority::TimeKeeper
        } else if self.has_protection() {
            PipelinePriority::Protected
        } else if self.has_advanced_features() {
            PipelinePriority::Advanced
        } else {
            PipelinePriority::FastPath
        }
    }
}

//=============================================================================
// SCHEDULING TYPES
//=============================================================================

/// Types of scheduling for pipeline compilation optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingType {
    Immediate,           // No scheduling, execute immediately
    DelayOnly,          // setTimeout equivalent (delay only)
    IntervalInfinite,   // setInterval equivalent (infinite)
    IntervalFinite,     // setInterval with count (finite)
    ComplexInfinite,    // delay + infinite interval
    ComplexFinite,      // delay + finite interval
}

impl SchedulingType {
    /// Check if this scheduling type requires TimeKeeper
    pub fn needs_timekeeper(&self) -> bool {
        !matches!(self, SchedulingType::Immediate)
    }
    
    /// Get the execution pattern description
    pub fn description(&self) -> &'static str {
        match self {
            SchedulingType::Immediate => "Execute immediately",
            SchedulingType::DelayOnly => "Execute once after delay",
            SchedulingType::IntervalInfinite => "Execute repeatedly forever",
            SchedulingType::IntervalFinite => "Execute repeatedly for count",
            SchedulingType::ComplexInfinite => "Delay then repeat forever",
            SchedulingType::ComplexFinite => "Delay then repeat for count",
        }
    }
}

/// Pipeline priority levels for compilation optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PipelinePriority {
    FastPath = 0,    // Highest priority - zero overhead
    Protected = 1,   // Protection mechanisms only
    Advanced = 2,    // Advanced features (talents, middleware)
    TimeKeeper = 3,  // TimeKeeper scheduling (lowest for compilation)
}

impl PipelinePriority {
    /// Get optimization level description
    pub fn optimization_level(&self) -> &'static str {
        match self {
            PipelinePriority::FastPath => "Zero-overhead fast path",
            PipelinePriority::Protected => "Protection-enabled path",
            PipelinePriority::Advanced => "Feature-rich path",
            PipelinePriority::TimeKeeper => "Scheduled execution path",
        }
    }
}

//=============================================================================
// TIMEKEEPER CONVERSION HELPERS
//=============================================================================

impl IO {
    /// Convert to TimeKeeper TimerRepeat
    pub fn to_timer_repeat(&self) -> crate::timekeeper::TimerRepeat {
        match self.repeat {
            Some(-1) => crate::timekeeper::TimerRepeat::Forever,
            Some(1) | None => crate::timekeeper::TimerRepeat::Once,
            Some(n) if n > 1 => crate::timekeeper::TimerRepeat::Count(n as u64),
            _ => crate::timekeeper::TimerRepeat::Once,
        }
    }

    /// Get interval with default
    pub fn get_interval_ms(&self) -> u64 {
        self.interval.unwrap_or(0)
    }

    /// Get delay
    pub fn get_delay_ms(&self) -> Option<u64> {
        self.delay
    }
    
    /// Get repeat count
    pub fn get_repeat_count(&self) -> Option<i32> {
        self.repeat
    }
    
    /// Create a TimeKeeper formation builder from this IO config
    pub fn to_formation_builder(&self, payload: crate::types::ActionPayload) -> crate::timekeeper::FormationBuilder {
        let mut builder = crate::timekeeper::FormationBuilder::new(&self.id, payload)
            .priority(self.priority)
            .repeat(self.to_timer_repeat());
            
        if let Some(interval) = self.interval {
            builder = builder.interval(interval);
        }
        
        if let Some(delay) = self.delay {
            builder = builder.delay(delay);
        }
        
        builder
    }
}

//=============================================================================
// CONVENIENT BUILDER METHODS
//=============================================================================

impl IO {
    /// Create a delayed action (setTimeout equivalent)
    pub fn delayed(id: impl Into<String>, delay_ms: u64) -> Self {
        Self::new(id).with_delay(delay_ms)
    }
    
    /// Create an interval action (setInterval equivalent)
    pub fn interval(id: impl Into<String>, interval_ms: u64) -> Self {
        Self::new(id).with_interval(interval_ms)
    }
    
    /// Create a finite repeat action
    pub fn repeat(id: impl Into<String>, interval_ms: u64, count: u32) -> Self {
        Self::new(id)
            .with_interval(interval_ms)
            .with_repeat(count)
    }
    
    /// Create a complex scheduled action
    pub fn complex(id: impl Into<String>, delay_ms: u64, interval_ms: u64, count: u32) -> Self {
        Self::new(id).schedule_complex(delay_ms, interval_ms, count)
    }
}

//=============================================================================
// DISPLAY IMPLEMENTATIONS
//=============================================================================

impl std::fmt::Display for SchedulingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl std::fmt::Display for PipelinePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.optimization_level())
    }
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduling_detection() {
        // No scheduling
        let immediate = IO::new("test");
        assert_eq!(immediate.get_scheduling_type(), SchedulingType::Immediate);
        assert!(!immediate.needs_timekeeper());
        assert!(immediate.is_fast_path_eligible());

        // Delay only
        let delayed = IO::new("test").with_delay(1000);
        assert_eq!(delayed.get_scheduling_type(), SchedulingType::DelayOnly);
        assert!(delayed.needs_timekeeper());
        assert!(!delayed.is_fast_path_eligible());

        // Infinite interval
        let interval = IO::new("test").with_interval(500);
        assert_eq!(interval.get_scheduling_type(), SchedulingType::IntervalInfinite);
        assert!(interval.needs_timekeeper());

        // Finite repeat
        let repeat = IO::new("test").with_interval(500).with_repeat(5);
        assert_eq!(repeat.get_scheduling_type(), SchedulingType::IntervalFinite);
        assert!(repeat.needs_timekeeper());

        // Complex scheduling
        let complex = IO::new("test").schedule_complex(1000, 500, 3);
        assert_eq!(complex.get_scheduling_type(), SchedulingType::ComplexFinite);
        assert!(complex.needs_timekeeper());
    }

    #[test]
    fn test_pipeline_priority() {
        let fast_path = IO::new("test");
        assert_eq!(fast_path.get_pipeline_priority(), PipelinePriority::FastPath);

        let protected = IO::new("test").with_throttle(1000);
        assert_eq!(protected.get_pipeline_priority(), PipelinePriority::Protected);

        let scheduled = IO::new("test").with_interval(1000);
        assert_eq!(scheduled.get_pipeline_priority(), PipelinePriority::TimeKeeper);
    }

    #[test]
    fn test_timekeeper_conversion() {
        let config = IO::new("test")
            .with_delay(500)
            .with_interval(1000)
            .with_repeat(3);

        assert_eq!(config.get_interval_ms(), 1000);
        assert_eq!(config.get_delay_ms(), Some(500));
        assert_eq!(config.get_repeat_count(), Some(3));
        
        let timer_repeat = config.to_timer_repeat();
        assert!(matches!(timer_repeat, crate::timekeeper::TimerRepeat::Count(3)));
    }

    #[test]
    fn test_convenient_builders() {
        let delayed = IO::delayed("task", 2000);
        assert_eq!(delayed.get_scheduling_type(), SchedulingType::DelayOnly);
        assert_eq!(delayed.get_delay_ms(), Some(2000));

        let interval = IO::interval("monitor", 1500);
        assert_eq!(interval.get_scheduling_type(), SchedulingType::IntervalInfinite);
        assert_eq!(interval.get_interval_ms(), 1500);

        let repeat = IO::repeat("backup", 1000, 5);
        assert_eq!(repeat.get_scheduling_type(), SchedulingType::IntervalFinite);
        assert_eq!(repeat.get_repeat_count(), Some(5));

        let complex = IO::complex("cleanup", 2000, 1000, 3);
        assert_eq!(complex.get_scheduling_type(), SchedulingType::ComplexFinite);
        assert_eq!(complex.get_delay_ms(), Some(2000));
        assert_eq!(complex.get_interval_ms(), 1000);
        assert_eq!(complex.get_repeat_count(), Some(3));
    }
}