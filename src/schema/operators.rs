// src/schema/operators.rs
// File location: src/schema/operators.rs
// Pure operators - answer when executor calls

//=============================================================================
// IMPORTS
//=============================================================================

use crate::types::ActionPayload;
use crate::utils::current_timestamp;
use std::sync::atomic::{ AtomicU64, Ordering };
use std::sync::Mutex;
use std::time::{ Duration, Instant };
use tokio::task::JoinHandle;

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

//=============================================================================
// OPERATOR ENUM
//=============================================================================

#[derive(Debug)]
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
// PROTECTION OPERATORS
//=============================================================================

#[derive(Debug)]
pub struct BlockOperator;

impl BlockOperator {
  pub fn new() -> Self {
    Self
  }

  pub async fn process(&self, _payload: ActionPayload) -> OperatorResult {
    OperatorResult::Block("Action is blocked".to_string())
  }
}

#[derive(Debug)]
pub struct ThrottleOperator {
  ms: u64,
  last_execution: AtomicU64,
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

    if now.saturating_sub(last) < self.ms {
      let remaining = self.ms - (now - last);
      OperatorResult::Block(format!("Throttled: {}ms remaining", remaining))
    } else {
      self.last_execution.store(now, Ordering::Relaxed);
      OperatorResult::Continue(payload)
    }
  }
}

#[derive(Debug)]
pub struct DebounceOperator {
  ms: u64,
  last_call: Mutex<Option<Instant>>,
  pending_timer: Mutex<Option<JoinHandle<()>>>,
}

impl DebounceOperator {
  pub fn new(ms: u64) -> Self {
    Self {
      ms,
      last_call: Mutex::new(None),
      pending_timer: Mutex::new(None),
    }
  }

  pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
    let now = Instant::now();

    // Cancel previous timer if exists
    {
      let mut timer_guard = self.pending_timer.lock().unwrap();
      if let Some(handle) = timer_guard.take() {
        handle.abort();
      }
    }

    // Check if we're within debounce window
    {
      let mut last_guard = self.last_call.lock().unwrap();
      if let Some(last) = *last_guard {
        if now.duration_since(last) < Duration::from_millis(self.ms) {
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
// VALIDATION OPERATORS
//=============================================================================

#[derive(Debug)]
pub struct RequiredOperator;

impl RequiredOperator {
  pub fn new() -> Self {
    Self
  }

  pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
    if payload.is_null() {
      OperatorResult::Block("Required payload missing".to_string())
    } else {
      OperatorResult::Continue(payload)
    }
  }
}

#[derive(Debug)]
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
    // TODO: Implement actual schema validation with talent system
    OperatorResult::Continue(payload)
  }
}

//=============================================================================
// PROCESSING OPERATORS
//=============================================================================

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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
        serde_json::Value::Number(crate::utils::current_timestamp().into())
      );
    }

    println!("✅ Transform completed: {}", result_payload);
    OperatorResult::Continue(result_payload)
  }
}

#[derive(Debug)]
pub struct DetectChangesOperator;

impl DetectChangesOperator {
  pub fn new() -> Self {
    Self
  }

  pub async fn process(&self, payload: ActionPayload) -> OperatorResult {
    // Simple pass-through for now
    OperatorResult::Continue(payload)
  }
}

//=============================================================================
// SCHEDULE OPERATOR
//=============================================================================

#[derive(Debug)]
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
    OperatorResult::Schedule(payload, ScheduleConfig {
      delay: self.delay,
      interval: self.interval,
      repeat: self.repeat,
    })
  }
}
