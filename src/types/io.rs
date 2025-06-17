// src/types/io.rs - COMPLETE FILE - Fixed ownership issues

use serde::{ Serialize, Deserialize };
use std::collections::HashMap;

//=============================================================================
// LOCAL TYPE DEFINITIONS (to avoid circular imports)
//=============================================================================

/// Action identifier type
pub type ActionId = String;

/// Priority levels for action execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 1,
    Normal = 2, // FIXED: was Medium in some places
    High = 3,
    Critical = 4,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

//=============================================================================
// SUPPORTING TYPES
//=============================================================================

/// Required field type - matches TypeScript boolean | 'non-empty'
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequiredType {
    Basic(bool),
    NonEmpty,
}

/// Repeat type - matches TypeScript number | boolean
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepeatType {
    Count(u32),
    Infinite,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub mode: AuthMode,
    pub token: Option<String>,
    pub allowed_callers: Option<Vec<String>>,
    pub group_policy: Option<String>,
    pub session_timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMode {
    #[serde(rename = "token")]
    Token,
    #[serde(rename = "context")]
    Context,
    #[serde(rename = "group")]
    Group,
    #[serde(rename = "disabled")]
    Disabled,
}

//=============================================================================
// COMPLETE IO CONFIGURATION - MATCHES TYPESCRIPT INTERFACE
//=============================================================================

/// Complete IO configuration matching the TypeScript interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IO {
    // ===== CORE IDENTIFICATION =====
    /// Unique identifier for this action
    pub id: ActionId,

    /// Human-readable name for the channel
    pub name: Option<String>,

    /// Channel category/classification (defaults to id if not specified)
    #[serde(rename = "type")]
    pub channel_type: Option<String>,

    /// Hierarchical path for organization (e.g., "sensors/temperature/room1")
    pub path: Option<String>,

    /// Group membership for bulk operations
    pub group: Option<String>,

    /// Tags for filtering and organization
    pub tags: Vec<String>,

    /// Description of what this channel does
    pub description: Option<String>,

    /// Version for channel evolution tracking
    pub version: Option<String>,

    // ===== VALIDATION & REQUIREMENTS =====
    /// Require payload to be provided
    pub required: Option<RequiredType>,

    /// Schema validation for payload
    pub schema: Option<String>, // Schema name/ID for now

    /// Block this action from execution
    pub block: bool,

    // ===== TIMING & SCHEDULING =====
    /// Milliseconds between executions for repeated actions
    pub interval: Option<u64>,

    /// Number of times to repeat execution (None = once, Some(0) = infinite)
    pub repeat: Option<RepeatType>,

    /// Milliseconds to delay before first execution
    pub delay: Option<u64>,

    // ===== PROTECTION MECHANISMS =====
    /// Minimum milliseconds between executions (rate limiting)
    pub throttle: Option<u64>,

    /// Collapse rapid calls within this window (milliseconds)
    pub debounce: Option<u64>,

    /// Maximum wait for debounce
    pub max_wait: Option<u64>,

    /// Only execute if payload has changed from previous execution
    pub detect_changes: bool,

    // ===== SYSTEM FEATURES =====
    /// Enable logging for this action
    pub log: bool,

    /// Priority level for execution during system stress
    pub priority: Priority,

    /// Middleware functions to process action before execution
    pub middleware: Vec<String>,

    // ===== STATE REACTIVITY =====
    /// Only execute when this condition returns true (talent name)
    pub condition: Option<String>,

    /// Select specific part of payload to watch for changes (talent name)
    pub selector: Option<String>,

    /// Transform payload before execution (talent name)
    pub transform: Option<String>,

    // ===== AUTHENTICATION =====
    pub auth: Option<AuthConfig>,

    // ===== INTERNAL OPTIMIZATION FIELDS =====
    /// Pre-computed blocking state for instant rejection
    pub _is_blocked: bool,

    /// Reason for blocking if _is_blocked is true
    pub _block_reason: Option<String>,

    /// True if action has no protections and can use fast path
    pub _has_fast_path: bool,

    /// Pre-compiled protection pipeline function names
    pub _pipeline: Vec<String>,

    /// Active debounce timer ID
    pub _debounce_timer: Option<String>,

    /// Flag to bypass debounce protection for internal use
    pub _bypass_debounce: bool,

    /// Flag to indicate if action is scheduled for execution
    pub _is_scheduled: bool,

    /// First debounce call timestamp for maxWait calculation
    pub _first_debounce_call: Option<u64>,

    /// Branch ID if this channel belongs to a branch
    pub _branch_id: Option<String>,

    // ===== COMPILED PIPELINES =====
    /// Fusion pipeline function names
    pub _fusion_pipeline: Vec<String>,

    /// Pattern pipeline function names
    pub _pattern_pipeline: Vec<String>,

    /// Pre-computed flags for optimization
    pub _has_protections: bool,
    pub _has_processing: bool,
    pub _has_scheduling: bool,
    pub _processing_talents: Vec<String>,
    pub _has_change_detection: bool,

    // ===== METADATA FIELDS =====
    pub timestamp: Option<u64>,
    pub time_of_creation: Option<u64>,
    pub _last_exec_time: Option<u64>,
    pub _execution_time: Option<u64>,
    pub _execution_count: u64,

    // ===== ADDITIONAL PROPERTIES =====
    /// Additional dynamic properties
    pub additional_properties: HashMap<String, serde_json::Value>,
}

//=============================================================================
// DEFAULT IMPLEMENTATION
//=============================================================================

impl Default for IO {
    fn default() -> Self {
        let now = crate::utils::current_timestamp();

        Self {
            // Core identification
            id: String::new(),
            name: None,
            channel_type: None,
            path: None,
            group: None,
            tags: Vec::new(),
            description: None,
            version: None,

            // Validation & requirements
            required: None,
            schema: None,
            block: false,

            // Timing & scheduling
            interval: None,
            repeat: None,
            delay: None,

            // Protection mechanisms
            throttle: None,
            debounce: None,
            max_wait: None,
            detect_changes: false,

            // System features
            log: false,
            priority: Priority::Normal,
            middleware: Vec::new(),

            // State reactivity
            condition: None,
            selector: None,
            transform: None,

            // Authentication
            auth: None,

            // Internal optimization
            _is_blocked: false,
            _block_reason: None,
            _has_fast_path: true,
            _pipeline: Vec::new(),
            _debounce_timer: None,
            _bypass_debounce: false,
            _is_scheduled: false,
            _first_debounce_call: None,
            _branch_id: None,

            // Compiled pipelines
            _fusion_pipeline: Vec::new(),
            _pattern_pipeline: Vec::new(),
            _has_protections: false,
            _has_processing: false,
            _has_scheduling: false,
            _processing_talents: Vec::new(),
            _has_change_detection: false,

            // Metadata
            timestamp: Some(now),
            time_of_creation: Some(now),
            _last_exec_time: None,
            _execution_time: None,
            _execution_count: 0,

            // Additional properties
            additional_properties: HashMap::new(),
        }
    }
}

//=============================================================================
// CORE BUILDER METHODS
//=============================================================================

impl IO {
    /// Create a new IO configuration with just an ID
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Default::default()
        }
    }

    // ===== IDENTIFICATION BUILDERS =====

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_type(mut self, channel_type: impl Into<String>) -> Self {
        self.channel_type = Some(channel_type.into());
        self
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    // ===== VALIDATION BUILDERS =====

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = Some(RequiredType::Basic(required));
        self
    }

    pub fn with_required_non_empty(mut self) -> Self {
        self.required = Some(RequiredType::NonEmpty);
        self
    }

    pub fn with_schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    pub fn with_block(mut self, block: bool) -> Self {
        self.block = block;
        self._is_blocked = block;
        if block {
            self._block_reason = Some("Manually blocked".to_string());
            self._has_fast_path = false;
        }
        self
    }

    // ===== TIMING BUILDERS =====

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay = Some(delay_ms);
        self._has_scheduling = true;
        self._has_fast_path = false;
        self
    }

    pub fn with_interval(mut self, interval_ms: u64) -> Self {
        self.interval = Some(interval_ms);
        self._has_scheduling = true;
        self._has_fast_path = false;
        self
    }

    pub fn with_repeat_count(mut self, count: u32) -> Self {
        self.repeat = Some(RepeatType::Count(count));
        self._has_scheduling = true;
        self._has_fast_path = false;
        self
    }

    pub fn with_repeat_infinite(mut self) -> Self {
        self.repeat = Some(RepeatType::Infinite);
        self._has_scheduling = true;
        self._has_fast_path = false;
        self
    }

    // ===== PROTECTION BUILDERS =====

    pub fn with_throttle(mut self, throttle_ms: u64) -> Self {
        self.throttle = Some(throttle_ms);
        self._has_protections = true;
        self._has_fast_path = false;
        self
    }

    pub fn with_debounce(mut self, debounce_ms: u64) -> Self {
        self.debounce = Some(debounce_ms);
        self._has_protections = true;
        self._has_fast_path = false;
        self
    }

    pub fn with_max_wait(mut self, max_wait_ms: u64) -> Self {
        self.max_wait = Some(max_wait_ms);
        self
    }

    pub fn with_change_detection(mut self) -> Self {
        self.detect_changes = true;
        self._has_change_detection = true;
        self._has_protections = true;
        self._has_fast_path = false;
        self
    }

    // ===== SYSTEM BUILDERS =====

    pub fn with_logging(mut self, enabled: bool) -> Self {
        self.log = enabled;
        self
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_middleware(mut self, middleware: impl Into<String>) -> Self {
        self.middleware.push(middleware.into());
        self._has_processing = true;
        self._has_fast_path = false;
        self
    }

    // ===== REACTIVITY BUILDERS - FIXED OWNERSHIP ISSUES =====

    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        let condition_str = condition.into(); // Convert ONCE
        self.condition = Some(condition_str.clone()); // Clone for storage
        self._has_processing = true;
        self._processing_talents.push(condition_str); // Use the same string
        self._has_fast_path = false;
        self
    }

    pub fn with_selector(mut self, selector: impl Into<String>) -> Self {
        let selector_str = selector.into(); // Convert ONCE
        self.selector = Some(selector_str.clone()); // Clone for storage
        self._has_processing = true;
        self._processing_talents.push(selector_str); // Use the same string
        self._has_fast_path = false;
        self
    }

    pub fn with_transform(mut self, transform: impl Into<String>) -> Self {
        let transform_str = transform.into(); // Convert ONCE
        self.transform = Some(transform_str.clone()); // Clone for storage
        self._has_processing = true;
        self._processing_talents.push(transform_str); // Use the same string
        self._has_fast_path = false;
        self
    }

    // ===== AUTHENTICATION BUILDERS =====

    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = Some(auth);
        self
    }

    pub fn with_token_auth(mut self, token: impl Into<String>) -> Self {
        self.auth = Some(AuthConfig {
            mode: AuthMode::Token,
            token: Some(token.into()),
            allowed_callers: None,
            group_policy: None,
            session_timeout: None,
        });
        self
    }

    // ===== CONVENIENCE BUILDERS =====

    pub fn with_property(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.additional_properties.insert(key.into(), value);
        self
    }
}

//=============================================================================
// CONVENIENT STATIC CONSTRUCTORS
//=============================================================================

impl IO {
    /// Create a delayed action (setTimeout equivalent)
    pub fn delayed(id: impl Into<String>, delay_ms: u64) -> Self {
        Self::new(id).with_delay(delay_ms)
    }

    /// Create an interval action (setInterval equivalent)
    pub fn interval(id: impl Into<String>, interval_ms: u64) -> Self {
        Self::new(id).with_interval(interval_ms).with_repeat_infinite()
    }

    /// Create a finite repeat action
    pub fn repeat(id: impl Into<String>, interval_ms: u64, count: u32) -> Self {
        Self::new(id).with_interval(interval_ms).with_repeat_count(count)
    }

    /// Create a complex scheduled action (delay + interval + repeat)
    pub fn complex(id: impl Into<String>, delay_ms: u64, interval_ms: u64, count: u32) -> Self {
        Self::new(id).with_delay(delay_ms).with_interval(interval_ms).with_repeat_count(count)
    }

    /// Create a protected API endpoint
    pub fn api(id: impl Into<String>, throttle_ms: u64) -> Self {
        Self::new(id).with_throttle(throttle_ms).with_change_detection().with_logging(true)
    }

    /// Create a real-time search action
    pub fn search(id: impl Into<String>, debounce_ms: u64) -> Self {
        Self::new(id).with_debounce(debounce_ms).with_max_wait(5000).with_priority(Priority::High)
    }
}

//=============================================================================
// HELPER METHODS FOR PIPELINE COMPILATION
//=============================================================================

impl IO {
    /// Check if this configuration has any scheduling
    pub fn has_scheduling(&self) -> bool {
        self._has_scheduling ||
            self.delay.is_some() ||
            self.interval.is_some() ||
            self.repeat.is_some()
    }

    /// Check if this configuration has any protection mechanisms
    pub fn has_protection(&self) -> bool {
        self._has_protections ||
            self.throttle.is_some() ||
            self.debounce.is_some() ||
            self.detect_changes ||
            self.block
    }

    /// Check if this configuration has any processing (talents/middleware)
    pub fn has_processing(&self) -> bool {
        self._has_processing ||
            !self.middleware.is_empty() ||
            self.condition.is_some() ||
            self.selector.is_some() ||
            self.transform.is_some() ||
            self.schema.is_some()
    }

    /// Check if this IO is eligible for fast path optimization
    pub fn is_fast_path_eligible(&self) -> bool {
        self._has_fast_path &&
            !self.has_scheduling() &&
            !self.has_protection() &&
            !self.has_processing() &&
            !self.log &&
            self.auth.is_none()
    }

    /// Get all talent names used by this IO
    pub fn get_all_talents(&self) -> Vec<String> {
        let mut talents = self._processing_talents.clone();

        if let Some(ref condition) = self.condition {
            talents.push(condition.clone());
        }
        if let Some(ref selector) = self.selector {
            talents.push(selector.clone());
        }
        if let Some(ref transform) = self.transform {
            talents.push(transform.clone());
        }

        talents.sort();
        talents.dedup();
        talents
    }

    /// Check if this IO requires authentication
    pub fn requires_auth(&self) -> bool {
        self.auth
            .as_ref()
            .map(|auth| !matches!(auth.mode, AuthMode::Disabled))
            .unwrap_or(false)
    }

    /// Get pipeline complexity score (for optimization decisions)
    pub fn complexity_score(&self) -> u32 {
        let mut score = 0;

        if self.has_scheduling() {
            score += 10;
        }
        if self.has_protection() {
            score += 5;
        }
        if self.has_processing() {
            score += 3;
        }
        if self.log {
            score += 1;
        }
        if self.requires_auth() {
            score += 2;
        }

        score += self.middleware.len() as u32;
        score += self.get_all_talents().len() as u32;

        score
    }

    /// Update execution metadata
    pub fn update_execution_metadata(&mut self, execution_time_ms: u64) {
        let now = crate::utils::current_timestamp();
        self._last_exec_time = Some(now);
        self._execution_time = Some(execution_time_ms);
        self._execution_count += 1;
    }

    /// Get summary for logging
    pub fn summary(&self) -> String {
        format!(
            "IO[{}] path:{} protection:{} processing:{} scheduling:{} fast_path:{}",
            self.id,
            self.path.as_deref().unwrap_or("none"),
            self.has_protection(),
            self.has_processing(),
            self.has_scheduling(),
            self.is_fast_path_eligible()
        )
    }
}

//=============================================================================
// DISPLAY IMPLEMENTATIONS
//=============================================================================

impl std::fmt::Display for IO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summary())
    }
}

impl std::fmt::Display for RequiredType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequiredType::Basic(true) => write!(f, "required"),
            RequiredType::Basic(false) => write!(f, "optional"),
            RequiredType::NonEmpty => write!(f, "required-non-empty"),
        }
    }
}

impl std::fmt::Display for RepeatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepeatType::Count(n) => write!(f, "{} times", n),
            RepeatType::Infinite => write!(f, "infinite"),
        }
    }
}
