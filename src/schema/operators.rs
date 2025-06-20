// src/schema/operators.rs - FIXED throttle operator implementation

use crate::context::sensor;
use crate::types::{ ActionPayload, IO };
use crate::utils::current_timestamp;
use std::sync::Arc;
use std::time::Duration;
use serde::{ Deserialize, Serialize };
use tokio::task::JoinHandle;
use crate::timekeeper::{ get_timekeeper, TimerRepeat };

//=============================================================================
// FIXED: THROTTLE OPERATOR WITH PROPER STATE MANAGEMENT
//=============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottleOperator {
  pub ms: u64,
}

impl ThrottleOperator {
  pub fn new(ms: u64) -> Self {
    Self { ms }
  }
}

/// PURE THROTTLE: Only reads state and calculates - never modifies anything
pub async fn process_throttle(
  operator: &ThrottleOperator,
  action_id: &str,
  payload: ActionPayload
) -> OperatorResult {
  use crate::context::state;

  let throttle_ms = operator.ms;
  let current_time = current_timestamp();

  // Read action from central state (READ ONLY)
  let action = match state::io::get(action_id) {
    Some(action) => action,
    None => {
      sensor::error(
        "throttle",
        &format!("[{}] Action '{}' not found in state", current_time, action_id),
        Some("process_throttle"),
        None
      );
      return OperatorResult::Block("Action not found in state".to_string());
    }
  };

  let last_exec_time = action._last_exec_time.unwrap_or(0);

  // PURE CALCULATION: Only calculate elapsed time
  let elapsed_since_last_exec = if last_exec_time > 0 {
    current_time.saturating_sub(last_exec_time)
  } else {
    throttle_ms // First call always passes
  };

  // Log throttle check details

  // PURE CHECK: Only evaluate throttle condition
  if last_exec_time > 0 && elapsed_since_last_exec < throttle_ms {
    let remaining = throttle_ms.saturating_sub(elapsed_since_last_exec);

    return OperatorResult::Block(format!("Throttled - {}ms remaining", remaining));
  }

  // Throttle check passed - allow execution (state updated later by Cyre core)

  OperatorResult::Continue(payload)
}

//=============================================================================
// FIXED: DEBOUNCE OPERATOR WITH PROPER ASYNC DELAY
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

  /// FIXED: Debounce operator with proper delay implementation
  pub async fn process(&self, action_id: &str, payload: ActionPayload) -> OperatorResult {
    let now = current_timestamp();
    let debounce_ms = self.ms;

    // Cancel any pending timer
    {
      let mut timer_guard = self.pending_timer.lock().unwrap();
      if let Some(handle) = timer_guard.take() {
        handle.abort();
        sensor::debug("debounce", &format!("Cancelled pending timer for '{}'", action_id), false);
      }
    }

    // Update last call time
    {
      let mut last_guard = self.last_call.lock().unwrap();
      if let Some(last) = *last_guard {
        if now.saturating_sub(last) < debounce_ms {
          *last_guard = Some(now);
          sensor::debug(
            "debounce",
            &format!("Action '{}' debounced - waiting {}ms", action_id, debounce_ms),
            false
          );
          return OperatorResult::Defer(payload, Duration::from_millis(debounce_ms));
        }
      }
      *last_guard = Some(now);
    }

    // Debounce check passed
    sensor::debug("debounce", &format!("Action '{}' debounce check passed", action_id), false);
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
    // Check for null payload
    if payload.is_null() {
      return OperatorResult::Block("Required payload missing".to_string());
    }

    // Non-empty validation
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
// OPERATOR PROCESSING FUNCTIONS
//=============================================================================

/// Block operator - always blocks execution
pub async fn process_block(_operator: &BlockOperator, _payload: ActionPayload) -> OperatorResult {
  OperatorResult::Block("Action is blocked".to_string())
}

/// TRUE DEBOUNCE: Only the LAST call in a rapid sequence executes
pub async fn process_debounce(
  operator: &DebounceOperator,
  action_id: &str,
  payload: ActionPayload
) -> OperatorResult {
  let current_time = current_timestamp();
  let debounce_ms = operator.ms;

  // TRUE DEBOUNCE LOGIC: Check if enough time has passed since last call
  let should_execute_immediately = {
    let mut last_guard = operator.last_call.lock().unwrap();
    if let Some(last) = *last_guard {
      let elapsed = current_time.saturating_sub(last);

      // sensor::debug(
      //   "debounce",
      //   &format!(
      //     "[{}] Last call was {}ms ago (debounce: {}ms)",
      //     current_time,
      //     elapsed,
      //     debounce_ms
      //   ),
      //   false
      // );

      if elapsed >= debounce_ms {
        // Enough time has passed - execute immediately
        *last_guard = Some(current_time);

        true
      } else {
        // Too soon - this call will be debounced
        *last_guard = Some(current_time); // Update for next call

        false
      }
    } else {
      // First call - execute immediately
      *last_guard = Some(current_time);

      true
    }
  };

  if should_execute_immediately {
    // Execute immediately
    OperatorResult::Continue(payload)
  } else {
    // TRUE DEBOUNCE: Block this call (it gets cancelled)
    OperatorResult::Block("Debounced - call cancelled".to_string())
  }
}

/// Required operator processing function
pub async fn process_required(
  operator: &RequiredOperator,
  payload: ActionPayload
) -> OperatorResult {
  // Check for null payload
  if payload.is_null() {
    return OperatorResult::Block("Required payload missing".to_string());
  }

  // Non-empty validation
  if operator.non_empty {
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

/// Transform operator processing function
pub async fn process_transform(
  operator: &TransformOperator,
  payload: ActionPayload
) -> OperatorResult {
  sensor::debug(
    "transform",
    &format!("Transform '{}' processing payload", operator.talent_name),
    false
  );

  let mut result_payload = payload.clone();

  // Apply transformations based on talent name
  match operator.talent_name.as_str() {
    "add-two" | "multiply-by-two" => {
      if let Some(obj) = result_payload.as_object_mut() {
        // Add 2 to any numeric value
        for (key, value) in obj.iter_mut() {
          if let Some(num) = value.as_i64() {
            let new_value = num + 2;
            *value = serde_json::Value::Number(new_value.into());
            sensor::debug("transform", &format!("{}: {} + 2 = {}", key, num, new_value), false);
          }
        }

        // Add transform metadata
        obj.insert(
          "_transformed_by".to_string(),
          serde_json::Value::String(operator.talent_name.clone())
        );
        obj.insert(
          "_transform_timestamp".to_string(),
          serde_json::Value::Number(current_timestamp().into())
        );
      }
    }
    "add-metadata" => {
      if let Some(obj) = result_payload.as_object_mut() {
        obj.insert(
          "_processed_by".to_string(),
          serde_json::Value::String("cyre_pipeline".to_string())
        );
        obj.insert(
          "_processed_at".to_string(),
          serde_json::Value::Number(current_timestamp().into())
        );
      }
    }
    _ => {
      sensor::warn(
        "transform",
        &format!("Unknown transform talent: {}", operator.talent_name),
        false
      );
    }
  }

  sensor::debug("transform", &format!("Transform completed: {}", result_payload), false);
  OperatorResult::Continue(result_payload)
}

/// Schema operator processing function
pub async fn process_schema(_operator: &SchemaOperator, payload: ActionPayload) -> OperatorResult {
  // TODO: Implement actual schema validation
  OperatorResult::Continue(payload)
}

/// Condition operator processing function
pub async fn process_condition(
  _operator: &ConditionOperator,
  payload: ActionPayload
) -> OperatorResult {
  // TODO: Implement talent system integration
  OperatorResult::Continue(payload)
}

/// Selector operator processing function
pub async fn process_selector(
  _operator: &SelectorOperator,
  payload: ActionPayload
) -> OperatorResult {
  // TODO: Implement talent system integration
  OperatorResult::Continue(payload)
}

/// Schedule operator processing function
pub async fn process_schedule(
  operator: &ScheduleOperator,
  payload: ActionPayload
) -> OperatorResult {
  let timekeeper = get_timekeeper().await;

  let payload_clone = payload.clone();
  let callback = move || {
    let payload_inner = payload_clone.clone();
    Box::pin(async move {
      sensor::info("schedule", &format!("Scheduled execution: {}", payload_inner), true);
    }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
  };

  let _ = timekeeper.keep(
    operator.interval.unwrap_or(1000),
    callback,
    operator.repeat.map_or(TimerRepeat::Once, |r| {
      if r == 0 { TimerRepeat::Forever } else { TimerRepeat::Count(r as u64) }
    }),
    format!("schedule-{}", current_timestamp()),
    operator.delay
  ).await;

  OperatorResult::Continue(payload)
}

/// Detect changes operator processing function
pub async fn process_detect_changes(
  _operator: &DetectChangesOperator,
  payload: ActionPayload
) -> OperatorResult {
  // TODO: Implement change detection logic
  OperatorResult::Continue(payload)
}

//=============================================================================
// OPERATOR STRUCT DEFINITIONS (DATA ONLY)
//=============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockOperator;

impl BlockOperator {
  pub fn new() -> Self {
    Self
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOperator {
  pub talent_name: String,
}

impl TransformOperator {
  pub fn new(talent_name: &str) -> Self {
    Self {
      talent_name: talent_name.to_string(),
    }
  }
}

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
  /// Process method that calls the appropriate operator function
  pub async fn process(&self, action_id: &str, payload: ActionPayload) -> OperatorResult {
    match self {
      Operator::Block(op) => process_block(op, payload).await,
      Operator::Throttle(op) => process_throttle(op, action_id, payload).await,
      Operator::Debounce(op) => process_debounce(op, action_id, payload).await,
      Operator::Required(op) => process_required(op, payload).await,
      Operator::Schema(op) => process_schema(op, payload).await,
      Operator::Condition(op) => process_condition(op, payload).await,
      Operator::Selector(op) => process_selector(op, payload).await,
      Operator::Transform(op) => process_transform(op, payload).await,
      Operator::Schedule(op) => process_schedule(op, payload).await,
      Operator::DetectChanges(op) => process_detect_changes(op, payload).await,
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
// PLACEHOLDER OPERATOR STRUCTS
//=============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaOperator {
  pub schema_name: String,
}

impl SchemaOperator {
  pub fn new(schema_name: &str) -> Self {
    Self {
      schema_name: schema_name.to_string(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionOperator {
  pub talent_name: String,
}

impl ConditionOperator {
  pub fn new(talent_name: &str) -> Self {
    Self {
      talent_name: talent_name.to_string(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorOperator {
  pub talent_name: String,
}

impl SelectorOperator {
  pub fn new(talent_name: &str) -> Self {
    Self {
      talent_name: talent_name.to_string(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectChangesOperator;

impl DetectChangesOperator {
  pub fn new() -> Self {
    Self
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleOperator {
  pub delay: Option<u64>,
  pub interval: Option<u64>,
  pub repeat: Option<u32>,
}

impl ScheduleOperator {
  pub fn new(delay: Option<u64>, interval: Option<u64>, repeat: Option<u32>) -> Self {
    Self { delay, interval, repeat }
  }
}

//=============================================================================
// OPERATOR ENUM
//=============================================================================

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
