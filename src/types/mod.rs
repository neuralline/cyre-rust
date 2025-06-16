// src/types/mod.rs
// Core types for Cyre Rust with proper CyreResponse implementation

use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;
use serde::{ Serialize, Deserialize };
use serde_json::Value;

//=============================================================================
// CORE TYPE ALIASES
//=============================================================================

pub type ActionId = String;
pub type ActionPayload = Value;
pub type AsyncHandler = Arc<
    dyn (Fn(ActionPayload) -> Pin<Box<dyn Future<Output = CyreResponse> + Send>>) + Send + Sync
>;

// Fast hash map type for performance
pub type FastMap<K, V> = HashMap<K, V>;

//=============================================================================
// CYRE RESPONSE TYPE
//=============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyreResponse {
    pub ok: bool,
    pub payload: Value,
    pub message: String,
    pub error: Option<String>,
    pub timestamp: u64,
    pub metadata: Option<Value>,
}

impl CyreResponse {
    /// Create a successful response
    pub fn ok(payload: Value) -> Self {
        Self {
            ok: true,
            payload,
            message: "Success".to_string(),
            error: None,
            timestamp: std::time::SystemTime
                ::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: None,
        }
    }

    /// Create a successful response with message
    pub fn ok_with_message(payload: Value, message: String) -> Self {
        Self {
            ok: true,
            payload,
            message,
            error: None,
            timestamp: std::time::SystemTime
                ::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: None,
        }
    }

    /// Create an error response
    pub fn error(message: String) -> Self {
        Self {
            ok: false,
            payload: Value::Null,
            message: "Error".to_string(),
            error: Some(message),
            timestamp: std::time::SystemTime
                ::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: None,
        }
    }

    /// Create an error response with payload
    pub fn error_with_payload(payload: Value, error_message: String) -> Self {
        Self {
            ok: false,
            payload,
            message: "Error".to_string(),
            error: Some(error_message),
            timestamp: std::time::SystemTime
                ::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: None,
        }
    }

    /// Add metadata to the response
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set custom message
    pub fn with_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }
}

impl Default for CyreResponse {
    fn default() -> Self {
        Self::ok(Value::Null)
    }
}

//=============================================================================
// PRIORITY ENUM
//=============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Priority {
    Low = 1,
    Normal = 2,
    Medium = 3,
    High = 4,
    Critical = 5,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

impl Priority {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Priority::Low),
            2 => Some(Priority::Normal),
            3 => Some(Priority::Medium),
            4 => Some(Priority::High),
            5 => Some(Priority::Critical),
            _ => None,
        }
    }
}

//=============================================================================
// IO CONFIGURATION STRUCT
//=============================================================================

#[derive(Debug, Clone)]
pub struct IO {
    // Core identification
    pub id: ActionId,
    pub name: Option<String>,
    pub description: Option<String>,

    // Performance and behavior
    pub priority: Priority,
    pub throttle: Option<u64>,
    pub debounce: Option<u64>,
    pub detect_changes: bool,

    // Scheduling (for TimeKeeper integration)
    pub delay: Option<u64>,
    pub interval: Option<u64>,
    pub repeat: Option<u32>,

    // Resource limits
    pub timeout: Option<u64>,
    pub memory_limit: Option<u64>,

    // Advanced features
    pub fast_path_eligible: bool,
    pub log: bool,
    pub middleware: Vec<String>,

    // Metadata
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl IO {
    /// Create a new IO configuration
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: None,
            description: None,
            priority: Priority::Normal,
            throttle: None,
            debounce: None,
            detect_changes: false,
            delay: None,
            interval: None,
            repeat: None,
            timeout: None,
            memory_limit: None,
            fast_path_eligible: true,
            log: false,
            middleware: Vec::new(),
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set human-readable name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Set throttle (minimum time between executions in ms)
    pub fn with_throttle(mut self, throttle_ms: u64) -> Self {
        self.throttle = Some(throttle_ms);
        self
    }

    /// Set debounce (delay execution until no more calls for N ms)
    pub fn with_debounce(mut self, debounce_ms: u64) -> Self {
        self.debounce = Some(debounce_ms);
        self
    }

    /// Enable change detection
    pub fn with_change_detection(mut self) -> Self {
        self.detect_changes = true;
        self
    }

    /// Set execution delay (for TimeKeeper)
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay = Some(delay_ms);
        self
    }

    /// Set execution interval (for TimeKeeper)
    pub fn with_interval(mut self, interval_ms: u64) -> Self {
        self.interval = Some(interval_ms);
        self
    }

    /// Set repeat count (for TimeKeeper)
    pub fn with_repeat(mut self, repeat_count: u32) -> Self {
        self.repeat = Some(repeat_count);
        self
    }

    /// Set execution timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout = Some(timeout_ms);
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if this IO has fast path eligibility
    pub fn is_fast_path_eligible(&self) -> bool {
        self.fast_path_eligible &&
            self.throttle.is_none() &&
            self.debounce.is_none() &&
            !self.detect_changes &&
            self.delay.is_none() &&
            self.interval.is_none()
    }

    /// Check if this IO has protection mechanisms
    pub fn has_protection(&self) -> bool {
        self.throttle.is_some() || self.debounce.is_some() || self.detect_changes
    }

    /// Check if this IO has advanced features
    pub fn has_advanced_features(&self) -> bool {
        !self.middleware.is_empty() || !self.tags.is_empty() || self.log
    }

    /// Check if this IO needs TimeKeeper
    pub fn needs_timekeeper(&self) -> bool {
        self.delay.is_some() || self.interval.is_some()
    }

    /// Get scheduling type for pipeline optimization
    pub fn get_scheduling_type(&self) -> SchedulingType {
        match (self.delay, self.interval, self.repeat) {
            (None, None, _) => SchedulingType::Immediate,
            (Some(_), None, _) => SchedulingType::DelayOnly,
            (None, Some(_), None) => SchedulingType::IntervalInfinite,
            (None, Some(_), Some(_)) => SchedulingType::IntervalFinite,
            (Some(_), Some(_), _) => SchedulingType::ComplexFinite,
        }
    }
}

//=============================================================================
// SCHEDULING TYPES
//=============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingType {
    Immediate, // No scheduling
    DelayOnly, // setTimeout equivalent
    IntervalInfinite, // setInterval infinite
    IntervalFinite, // setInterval with repeat count
    ComplexFinite, // delay + interval + repeat
}

impl SchedulingType {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Immediate => "immediate execution",
            Self::DelayOnly => "delayed execution",
            Self::IntervalInfinite => "infinite interval",
            Self::IntervalFinite => "finite interval",
            Self::ComplexFinite => "complex scheduling",
        }
    }
}

//=============================================================================
// PIPELINE OPTIMIZATION TYPES
//=============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelinePriority {
    FastPath, // Zero-overhead execution
    Protected, // With throttle/debounce
    TimeKeeper, // Scheduled execution
}

impl PipelinePriority {
    pub fn optimization_level(&self) -> &'static str {
        match self {
            Self::FastPath => "zero-cost",
            Self::Protected => "protected",
            Self::TimeKeeper => "scheduled",
        }
    }
}

//=============================================================================
// RESULT TYPES
//=============================================================================

#[derive(Debug, Clone)]
pub struct TalentResult {
    pub success: bool,
    pub value: Value,
    pub payload: Value,
    pub message: String,
    pub execution_time: u64,
    pub error: Option<String>,
    pub metadata: Option<Value>,
}

impl TalentResult {
    pub fn ok(payload: Value) -> Self {
        Self {
            success: true,
            value: payload.clone(),
            payload,
            message: "Success".to_string(),
            execution_time: 0,
            error: None,
            metadata: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            value: Value::Null,
            payload: Value::Null,
            message: "Error".to_string(),
            execution_time: 0,
            error: Some(message),
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

//=============================================================================
// FUNCTION TYPES FOR TALENT SYSTEM
//=============================================================================

pub type SchemaFunction = Arc<dyn (Fn(&Value) -> ValidationResult) + Send + Sync>;
pub type ConditionFunction = Arc<dyn (Fn(&Value) -> bool) + Send + Sync>;
pub type TransformFunction = Arc<dyn (Fn(Value) -> Value) + Send + Sync>;
pub type SelectorFunction = Arc<dyn (Fn(&Value) -> Value) + Send + Sync>;

//=============================================================================
// UTILITY FUNCTIONS
//=============================================================================

/// Likely hint for branch prediction optimization
#[inline(always)]
pub fn likely(condition: bool) -> bool {
    std::hint::black_box(condition)
}

/// Convert duration to human readable format
pub fn format_duration(duration_ms: u64) -> String {
    if duration_ms < 1000 {
        format!("{}ms", duration_ms)
    } else if duration_ms < 60_000 {
        format!("{:.1}s", (duration_ms as f64) / 1000.0)
    } else {
        format!("{:.1}m", (duration_ms as f64) / 60_000.0)
    }
}

//=============================================================================
// TIMER TYPE FOR TIMELINE INTEGRATION
//=============================================================================

#[derive(Debug, Clone)]
pub struct Timer {
    pub id: String,
    pub start_time: u64,
    pub duration: u64,
    pub callback: Option<String>, // Reference to callback (simplified)
    pub repeat: Option<u32>,
    pub execution_count: u32,
    pub last_execution_time: u64,
    pub next_execution_time: u64,
    pub is_active: bool,
    pub status: String,
    pub priority: Priority,
}

impl Timer {
    pub fn new(id: String) -> Self {
        Self {
            id,
            start_time: 0,
            duration: 0,
            callback: None,
            repeat: None,
            execution_count: 0,
            last_execution_time: 0,
            next_execution_time: 0,
            is_active: false,
            status: "inactive".to_string(),
            priority: Priority::Normal,
        }
    }
}
