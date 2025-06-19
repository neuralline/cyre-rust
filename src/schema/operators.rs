// src/schema/operators.rs - FIXED operator implementations

use crate::context::sensor;
use crate::types::ActionPayload;
use crate::utils::current_timestamp;
use std::sync::atomic::{ AtomicU64, Ordering };
use std::sync::Arc;
use std::time::Duration;
use serde::{ Deserialize, Serialize };
use tokio::task::JoinHandle;
use crate::timekeeper::{ get_timekeeper, TimerRepeat };

//=============================================================================
// FIXED: THROTTLE OPERATOR WITH PROPER STATE MANAGEMENT
//=============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ThrottleOperator {
  ms: u64,
  #[serde(skip)]
  #[serde(default = "default_atomic_u64")]
  last_execution: AtomicU64,
}

fn default_atomic_u64() -> AtomicU64 {
  AtomicU64::new(0)
}

impl Clone for ThrottleOperator {
  fn clone(&self) -> Self {
    ThrottleOperator {
      ms: self.ms,
      last_execution: AtomicU64::new(self.last_execution.load(Ordering::Relaxed)),
    }
  }
}

impl ThrottleOperator {
  pub fn new(ms: u64) -> Self {
    Self {
      ms,
      last_execution: AtomicU64::new(0),
    }
  }

  pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
    let now = current_timestamp();
    let last = self.last_execution.load(Ordering::Relaxed);

    // FIXED: Proper throttle logic
    if last > 0 && now - last < self.ms {
      let remaining = self.ms - (now - last);
      OperatorResult::Block(format!("Throttled: {}ms remaining", remaining))
    } else {
      self.last_execution.store(now, Ordering::Relaxed);
      OperatorResult::Continue(payload)
    }
  }
}

//=============================================================================
// FIXED: DEBOUNCE OPERATOR WITH ACTUAL DELAY
//=============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct DebounceOperator {
  ms: u64,
  #[serde(skip)]
  #[serde(default = "default_arc_mutex_option_u64")]
  last_call: Arc<std::sync::Mutex<Option<u64>>>,
  #[serde(skip)]
  #[serde(default = "default_arc_mutex_option_join_handle")]
  pending_timer: Arc<std::sync::Mutex<Option<JoinHandle<()>>>>,
}

fn default_arc_mutex_option_u64() -> Arc<std::sync::Mutex<Option<u64>>> {
  Arc::new(std::sync::Mutex::new(None))
}

fn default_arc_mutex_option_join_handle() -> Arc<std::sync::Mutex<Option<JoinHandle<()>>>> {
  Arc::new(std::sync::Mutex::new(None))
}

impl Clone for DebounceOperator {
  fn clone(&self) -> Self {
    DebounceOperator {
      ms: self.ms,
      last_call: Arc::new(std::sync::Mutex::new(*self.last_call.lock().unwrap())),
      pending_timer: Arc::new(std::sync::Mutex::new(None)), // Don't clone running timers
    }
  }
}

impl DebounceOperator {
  pub fn new(ms: u64) -> Self {
    Self {
      ms,
      last_call: Arc::new(std::sync::Mutex::new(None)),
      pending_timer: Arc::new(std::sync::Mutex::new(None)),
    }
  }

  pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
    let now = current_timestamp();

    // Cancel previous timer if exists
    {
      let mut timer_guard = self.pending_timer.lock().unwrap();
      if let Some(handle) = timer_guard.take() {
        handle.abort();
      }
    }

    // Check if we should debounce
    {
      let mut last_guard = self.last_call.lock().unwrap();
      if let Some(last) = *last_guard {
        if now - last < self.ms {
          *last_guard = Some(now);
          return OperatorResult::Defer(payload, Duration::from_millis(self.ms));
        }
      }
      *last_guard = Some(now);
    }

    OperatorResult::Continue(payload)
  }
}

//=============================================================================
// FIXED: REQUIRED OPERATOR WITH PROPER VALIDATION
//=============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredOperator {
  non_empty: bool,
}

impl RequiredOperator {
  pub fn new() -> Self {
    Self { non_empty: false }
  }

  pub fn new_non_empty() -> Self {
    Self { non_empty: true }
  }

  pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
    // FIXED: Proper required validation
    if payload.is_null() {
      return OperatorResult::Block("Required payload missing".to_string());
    }

    if self.non_empty {
      match &payload {
        serde_json::Value::String(s) if s.is_empty() => {
          return OperatorResult::Block("Non-empty string required".to_string());
        }
        serde_json::Value::Array(arr) if arr.is_empty() => {
          return OperatorResult::Block("Non-empty array required".to_string());
        }
        serde_json::Value::Object(obj) if obj.is_empty() => {
          return OperatorResult::Block("Non-empty object required".to_string());
        }
        _ => {}
      }
    }

    OperatorResult::Continue(payload)
  }
}

//=============================================================================
// OTHER OPERATORS (EXISTING BUT FIXED)
//=============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockOperator;

impl BlockOperator {
  pub fn new() -> Self {
    Self
  }

  pub async fn process(&self, _payload: ActionPayload) -> OperatorResult {
    OperatorResult::Block("Action is blocked".to_string())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOperator {
  talent_name: String,
}

impl TransformOperator {
  pub fn new(talent_name: &str) -> Self {
    Self {
      talent_name: talent_name.to_string(),
    }
  }

  pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
    println!("🔄 Transform operator '{}' processing payload", self.talent_name);

    let mut result_payload = payload.clone();

    if let Some(obj) = result_payload.as_object_mut() {
      // Add 2 to any numeric value
      for (key, value) in obj.iter_mut() {
        if let Some(num) = value.as_i64() {
          let new_value = num + 2;
          *value = serde_json::Value::Number(new_value.into());
          println!("   📊 {}: {} + 2 = {}", key, num, new_value);
        }
      }

      // Add transform metadata
      obj.insert(
        "_transformed_by".to_string(),
        serde_json::Value::String(self.talent_name.clone())
      );
      obj.insert(
        "_transform_timestamp".to_string(),
        serde_json::Value::Number(current_timestamp().into())
      );
    }

    println!("✅ Transform completed: {}", result_payload);
    OperatorResult::Continue(result_payload)
  }
}

//=============================================================================
// OPERATOR ENUM
//=============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operator {
  Block(BlockOperator),
  Throttle(ThrottleOperator),
  Debounce(DebounceOperator),
  Required(RequiredOperator),
  Schema(SchemaOperator),
  Condition(ConditionOperator),
  Selector(SelectorOperator),
  Transform(TransformOperator),
  Schedule(ScheduleOperator),
  DetectChanges(DetectChangesOperator),
}

impl Operator {
  pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
    match self {
      Operator::Block(op) => op.process(payload).await,
      Operator::Throttle(op) => op.process(payload).await,
      Operator::Debounce(op) => op.process(payload).await,
      Operator::Required(op) => op.process(payload).await,
      Operator::Schema(op) => op.process(payload).await,
      Operator::Condition(op) => op.process(payload).await,
      Operator::Selector(op) => op.process(payload).await,
      Operator::Transform(op) => op.process(payload).await,
      Operator::Schedule(op) => op.process(payload).await,
      Operator::DetectChanges(op) => op.process(payload).await,
    }
  }

  pub fn name(&self) -> &'static str {
    match self {
      Operator::Block(_) => "block",
      Operator::Throttle(_) => "throttle",
      Operator::Debounce(_) => "debounce",
      Operator::Required(_) => "required",
      Operator::Schema(_) => "schema",
      Operator::Condition(_) => "condition",
      Operator::Selector(_) => "selector",
      Operator::Transform(_) => "transform",
      Operator::Schedule(_) => "schedule",
      Operator::DetectChanges(_) => "detect_changes",
    }
  }
}

//=============================================================================
// PLACEHOLDER OPERATORS (TO BE IMPLEMENTED)
//=============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaOperator {
  schema_name: String,
}

impl SchemaOperator {
  pub fn new(schema_name: &str) -> Self {
    Self {
      schema_name: schema_name.to_string(),
    }
  }

  pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
    // TODO: Implement actual schema validation
    OperatorResult::Continue(payload)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionOperator {
  talent_name: String,
}

impl ConditionOperator {
  pub fn new(talent_name: &str) -> Self {
    Self {
      talent_name: talent_name.to_string(),
    }
  }

  pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
    // TODO: Implement talent system integration
    OperatorResult::Continue(payload)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorOperator {
  talent_name: String,
}

impl SelectorOperator {
  pub fn new(talent_name: &str) -> Self {
    Self {
      talent_name: talent_name.to_string(),
    }
  }

  pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
    // TODO: Implement talent system integration
    OperatorResult::Continue(payload)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectChangesOperator;

impl DetectChangesOperator {
  pub fn new() -> Self {
    Self
  }

  pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
    // TODO: Implement change detection logic
    OperatorResult::Continue(payload)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleOperator {
  delay: Option<u64>,
  interval: Option<u64>,
  repeat: Option<u32>,
}

impl ScheduleOperator {
  pub fn new(delay: Option<u64>, interval: Option<u64>, repeat: Option<u32>) -> Self {
    Self { delay, interval, repeat }
  }

  pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
    let timekeeper = get_timekeeper().await;

    let payload_clone = payload.clone();
    let callback = move || {
      let payload_inner = payload_clone.clone();
      Box::pin(async move {
        sensor::info("schedule", &format!("Scheduled execution: {}", payload_inner), true);
      }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
    };

    let _ = timekeeper.keep(
      self.interval.unwrap_or(1000),
      callback,
      self.repeat.map_or(TimerRepeat::Once, |r| {
        if r == 0 { TimerRepeat::Forever } else { TimerRepeat::Count(r as u64) }
      }),
      format!("schedule-{}", current_timestamp()),
      self.delay
    ).await;

    OperatorResult::Continue(payload)
  }
}

//=============================================================================
// OPERATOR RESULT ENUM
//=============================================================================

#[derive(Debug)]
pub enum OperatorResult {
  Continue(ActionPayload),
  Block(String),
  Defer(ActionPayload, Duration),
  Schedule(ActionPayload, ScheduleConfig),
}

#[derive(Debug, Clone)]
pub struct ScheduleConfig {
  pub delay: Option<u64>,
  pub interval: Option<u64>,
  pub repeat: Option<u32>,
}
