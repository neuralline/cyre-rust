// src/types/io.rs - PURE INTERFACE - No state management

use serde::{ Serialize, Deserialize };
use std::collections::HashMap;
use crate::schema::operators::Operator;

//=============================================================================
// SUPPORTING TYPES
//=============================================================================

/// Action identifier type
pub type ActionId = String;

/// Priority levels for action execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
  Low = 1,
  Normal = 2,
  High = 3,
  Critical = 4,
}

impl Default for Priority {
  fn default() -> Self {
    Priority::Normal
  }
}

/// Required field type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequiredType {
  Basic(bool),
  NonEmpty,
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
// IO CONFIGURATION - PURE INTERFACE (NO STATE LOGIC)
//=============================================================================

/// Pure IO configuration interface - configuration data only
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IO {
  // ===== CORE IDENTIFICATION =====
  pub id: ActionId,
  pub name: Option<String>,
  #[serde(rename = "type")]
  pub channel_type: Option<String>,
  pub path: Option<String>,
  pub group: Option<String>,
  pub tags: Vec<String>,
  pub description: Option<String>,
  pub version: Option<String>,

  // ===== DEFAULT PAYLOAD =====
  pub payload: Option<serde_json::Value>,

  // ===== VALIDATION & REQUIREMENTS =====
  pub required: Option<RequiredType>,
  pub schema: Option<String>,
  pub block: bool,

  // ===== TIMING & SCHEDULING =====
  pub interval: Option<u64>,
  pub repeat: Option<serde_json::Value>,
  pub delay: Option<u64>,

  // ===== PROTECTION MECHANISMS =====
  pub throttle: Option<u64>,
  pub debounce: Option<u64>,
  pub max_wait: Option<u64>,
  pub detect_changes: bool,

  // ===== SYSTEM FEATURES =====
  pub log: bool,
  pub priority: Priority,
  pub middleware: Vec<String>,

  // ===== STATE REACTIVITY =====
  pub condition: Option<String>,
  pub selector: Option<String>,
  pub transform: Option<String>,

  // ===== AUTHENTICATION =====
  pub auth: Option<AuthConfig>,

  // ===== COMPILED FIELDS (DATA ONLY - NO STORAGE LOGIC) =====
  pub _is_blocked: bool,
  pub _block_reason: Option<String>,
  pub _has_fast_path: bool,
  #[serde(skip)] // Skip serialization for operators
  pub _pipeline: Vec<Operator>,
  pub _has_protections: bool,
  pub _has_processing: bool,
  pub _has_scheduling: bool,

  // ===== METADATA =====
  pub timestamp: Option<u64>,
  pub time_of_creation: Option<u64>,
  pub _last_exec_time: Option<u64>,
  pub _execution_time: Option<u64>,
  pub _execution_count: u64,

  // ===== ADDITIONAL PROPERTIES =====
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

      // Default payload
      payload: None,

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

      // Compiled flags (defaults only - compiler will set these)
      _is_blocked: false,
      _block_reason: None,
      _has_fast_path: true, // Default assumption
      _pipeline: Vec::new(),
      _has_protections: false,
      _has_processing: false,
      _has_scheduling: false,

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
// BUILDER METHODS - PURE DATA MANIPULATION (NO STORAGE)
//=============================================================================

impl IO {
  /// Create new IO configuration
  pub fn new(id: impl Into<String>) -> Self {
    Self {
      id: id.into(),
      ..Default::default()
    }
  }

  // ===== IDENTIFICATION =====
  pub fn with_name(mut self, name: impl Into<String>) -> Self {
    self.name = Some(name.into());
    self
  }

  pub fn with_type(mut self, channel_type: impl Into<String>) -> Self {
    self.channel_type = Some(channel_type.into());
    self
  }

  pub fn with_description(mut self, description: impl Into<String>) -> Self {
    self.description = Some(description.into());
    self
  }

  pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
    self.tags.push(tag.into());
    self
  }

  // ===== PAYLOAD =====
  pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
    self.payload = Some(payload);
    self
  }

  // ===== VALIDATION =====
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
    self
  }

  // ===== TIMING =====
  pub fn with_delay(mut self, delay_ms: u64) -> Self {
    self.delay = Some(delay_ms);
    self
  }

  pub fn with_interval(mut self, interval_ms: u64) -> Self {
    self.interval = Some(interval_ms);
    self
  }

  pub fn with_repeat_count(mut self, count: u32) -> Self {
    self.repeat = Some(serde_json::Value::Number(count.into()));
    self
  }

  pub fn with_repeat_infinite(mut self) -> Self {
    self.repeat = Some(serde_json::Value::Bool(true));
    self
  }

  // ===== PROTECTION =====
  pub fn with_throttle(mut self, throttle_ms: u64) -> Self {
    self.throttle = Some(throttle_ms);
    self
  }

  pub fn with_debounce(mut self, debounce_ms: u64) -> Self {
    self.debounce = Some(debounce_ms);
    self
  }

  pub fn with_max_wait(mut self, max_wait_ms: u64) -> Self {
    self.max_wait = Some(max_wait_ms);
    self
  }

  pub fn with_change_detection(mut self) -> Self {
    self.detect_changes = true;
    self
  }

  // ===== SYSTEM =====
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
    self
  }

  // ===== REACTIVITY =====
  pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
    self.condition = Some(condition.into());
    self
  }

  pub fn with_selector(mut self, selector: impl Into<String>) -> Self {
    self.selector = Some(selector.into());
    self
  }

  pub fn with_transform(mut self, transform: impl Into<String>) -> Self {
    self.transform = Some(transform.into());
    self
  }

  // ===== AUTHENTICATION =====
  pub fn with_auth(mut self, auth: AuthConfig) -> Self {
    self.auth = Some(auth);
    self
  }

  // ===== CONVENIENCE =====
  pub fn with_property(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
    self.additional_properties.insert(key.into(), value);
    self
  }
}

//=============================================================================
// STATIC CONSTRUCTORS - CONVENIENCE METHODS
//=============================================================================

impl IO {
  /// Create delayed action
  pub fn delayed(id: impl Into<String>, delay_ms: u64) -> Self {
    Self::new(id).with_delay(delay_ms)
  }

  /// Create interval action
  pub fn interval(id: impl Into<String>, interval_ms: u64) -> Self {
    Self::new(id).with_interval(interval_ms).with_repeat_infinite()
  }

  /// Create API endpoint
  pub fn api(id: impl Into<String>, throttle_ms: u64) -> Self {
    Self::new(id).with_throttle(throttle_ms).with_logging(true)
  }

  /// Create search action
  pub fn search(id: impl Into<String>, debounce_ms: u64) -> Self {
    Self::new(id).with_debounce(debounce_ms).with_priority(Priority::High)
  }
}

//=============================================================================
// DISPLAY
//=============================================================================

impl std::fmt::Display for IO {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "IO[{}]", self.id)
  }
}
