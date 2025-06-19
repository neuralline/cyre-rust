// src/timekeeper/timekeeper.rs - Clean TimeKeeper implementation
// File location: src/timekeeper/timekeeper.rs

use std::sync::{ Arc, atomic::{ AtomicU64, AtomicBool, Ordering } };
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::types::Priority;
use crate::utils::current_timestamp;
use crate::context::state::timeline;

//=============================================================================
// TIMER REPEAT CONFIGURATION - SUPPORTS BOOLEAN AND NUMBER
//=============================================================================

/// Timer repeat configuration matching TypeScript API
#[derive(Debug, Clone)]
pub enum TimerRepeat {
  Once,
  Forever,
  Count(u64),
}

impl From<bool> for TimerRepeat {
  fn from(repeat: bool) -> Self {
    if repeat { TimerRepeat::Forever } else { TimerRepeat::Once }
  }
}

impl From<u64> for TimerRepeat {
  fn from(count: u64) -> Self {
    if count == 0 {
      TimerRepeat::Forever // 0 = infinite like in JavaScript
    } else {
      TimerRepeat::Count(count)
    }
  }
}

impl From<u32> for TimerRepeat {
  fn from(count: u32) -> Self {
    TimerRepeat::from(count as u64)
  }
}

impl From<i32> for TimerRepeat {
  fn from(count: i32) -> Self {
    if count <= 0 { TimerRepeat::Forever } else { TimerRepeat::Count(count as u64) }
  }
}

impl From<i64> for TimerRepeat {
  fn from(count: i64) -> Self {
    if count <= 0 { TimerRepeat::Forever } else { TimerRepeat::Count(count as u64) }
  }
}

//=============================================================================
// FORMATION (TIMER) DEFINITION
//=============================================================================

/// Formation represents a scheduled callback execution
#[derive(Debug, Clone)]
pub struct Formation {
  pub id: String,
  pub callback_id: String, // For async callback handling
  pub interval: u64,
  pub repeat: TimerRepeat,
  pub delay: Option<u64>,
  pub priority: Priority,
  pub created_at: u64,
  pub next_execution: u64,
  pub execution_count: u64,
  pub has_executed_once: bool,
  pub is_active: bool,
  pub original_duration: u64,
}

impl Formation {
  pub fn new(
    id: String,
    interval: u64,
    repeat: TimerRepeat,
    delay: Option<u64>,
    priority: Priority
  ) -> Self {
    let now = current_timestamp();
    let next_time = now + delay.unwrap_or(interval);
    let callback_id = format!("cb_{}", &id);

    Self {
      id,
      callback_id,
      interval,
      repeat,
      delay,
      priority,
      created_at: now,
      next_execution: next_time,
      execution_count: 0,
      has_executed_once: false,
      is_active: true,
      original_duration: interval,
    }
  }

  pub fn should_continue(&self) -> bool {
    match self.repeat {
      TimerRepeat::Once => false,
      TimerRepeat::Forever => true,
      TimerRepeat::Count(count) => self.execution_count < count,
    }
  }

  pub fn update_for_next_execution(&mut self) {
    let now = current_timestamp();
    self.execution_count += 1;
    self.has_executed_once = true;

    // Clear delay after first execution
    let interval = if self.has_executed_once {
      self.interval
    } else {
      self.delay.unwrap_or(self.interval)
    };

    self.next_execution = now + interval;
    self.delay = None; // Clear delay after first execution
  }
}

//=============================================================================
// CALLBACK REGISTRY
//=============================================================================

type AsyncCallback = Arc<
  dyn (Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>) + Send + Sync
>;

struct CallbackRegistry {
  callbacks: Arc<RwLock<HashMap<String, AsyncCallback>>>,
}

impl CallbackRegistry {
  fn new() -> Self {
    Self {
      callbacks: Arc::new(RwLock::new(HashMap::new())),
    }
  }

  async fn register(&self, id: String, callback: AsyncCallback) {
    let mut callbacks = self.callbacks.write().await;
    callbacks.insert(id, callback);
  }

  async fn get(&self, id: &str) -> Option<AsyncCallback> {
    let callbacks = self.callbacks.read().await;
    callbacks.get(id).cloned()
  }

  async fn remove(&self, id: &str) {
    let mut callbacks = self.callbacks.write().await;
    callbacks.remove(id);
  }

  async fn clear(&self) {
    let mut callbacks = self.callbacks.write().await;
    callbacks.clear();
  }
}

//=============================================================================
// QUARTZ ENGINE - SIMPLIFIED EXECUTION COORDINATOR
//=============================================================================

struct QuartzEngine {
  is_running: Arc<AtomicBool>,
  tick_interval: u64,
  callback_registry: CallbackRegistry,
}

impl QuartzEngine {
  fn new() -> Self {
    Self {
      is_running: Arc::new(AtomicBool::new(false)),
      tick_interval: 10, // 10ms precision
      callback_registry: CallbackRegistry::new(),
    }
  }

  async fn start(&self) {
    if self.is_running.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
      let callback_registry = self.callback_registry.callbacks.clone();
      let tick_interval = self.tick_interval;
      let is_running = self.is_running.clone();

      tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(tick_interval));

        while is_running.load(Ordering::SeqCst) {
          interval.tick().await;
          Self::tick(callback_registry.clone()).await;
        }
      });
    }
  }

  async fn stop(&self) {
    self.is_running.store(false, Ordering::SeqCst);
  }

  async fn tick(callback_registry: Arc<RwLock<HashMap<String, AsyncCallback>>>) {
    let current_time = current_timestamp();
    let mut formations_to_execute = Vec::new();

    // Get formations due for execution from timeline
    {
      let all_entries = timeline::get_all();
      for entry in all_entries {
        // Parse formation data from timeline entry
        if entry.success && entry.error.is_none() && current_time >= entry.timestamp {
          if let Some(formation) = Self::entry_to_formation(&entry) {
            if formation.is_active {
              formations_to_execute.push(formation);
            }
          }
        }
      }
    }

    // Execute formations
    for mut formation in formations_to_execute {
      // Get and execute callback
      if
        let Some(callback) = ({
          let callbacks = callback_registry.read().await;
          callbacks.get(&formation.callback_id).cloned()
        })
      {
        // Execute callback in background
        tokio::spawn(async move {
          callback().await;
        });
      }

      // Update formation state
      formation.update_for_next_execution();

      if formation.should_continue() {
        // Save updated formation to timeline
        let entry = Self::formation_to_entry(&formation);
        let _ = timeline::add(entry);
      } else {
        // Formation completed - remove from timeline
        timeline::forget(&formation.id);
      }
    }
  }

  // Helper to convert timeline entry to formation
  fn entry_to_formation(entry: &crate::context::state::TimelineEntry) -> Option<Formation> {
    let payload = &entry.payload;

    let repeat = match payload.get("repeat")?.as_str()? {
      "once" => TimerRepeat::Once,
      "forever" => TimerRepeat::Forever,
      s if s.starts_with("count:") => {
        let count = s.strip_prefix("count:")?.parse().ok()?;
        TimerRepeat::Count(count)
      }
      _ => {
        return None;
      }
    };

    Some(Formation {
      id: entry.id.clone(),
      callback_id: entry.action_id.clone(),
      interval: payload.get("interval")?.as_u64()?,
      repeat,
      delay: payload.get("delay").and_then(|v| v.as_u64()),
      priority: Priority::Normal,
      created_at: payload.get("created_at")?.as_u64()?,
      next_execution: entry.timestamp,
      execution_count: payload.get("execution_count")?.as_u64()?,
      has_executed_once: payload.get("execution_count")?.as_u64()? > 0,
      is_active: payload.get("is_active")?.as_bool()?,
      original_duration: payload.get("original_duration")?.as_u64()?,
    })
  }

  // Helper to convert formation to timeline entry
  fn formation_to_entry(formation: &Formation) -> crate::context::state::TimelineEntry {
    crate::context::state::TimelineEntry {
      id: formation.id.clone(),
      action_id: formation.callback_id.clone(),
      timestamp: formation.next_execution,
      payload: serde_json::json!({
                "formation_id": formation.id,
                "interval": formation.interval,
                "repeat": match &formation.repeat {
                    TimerRepeat::Once => "once".to_string(),
                    TimerRepeat::Forever => "forever".to_string(),
                    TimerRepeat::Count(n) => format!("count:{}", n)
                },
                "delay": formation.delay,
                "execution_count": formation.execution_count,
                "is_active": formation.is_active,
                "created_at": formation.created_at,
                "original_duration": formation.original_duration
            }),
      success: formation.is_active,
      execution_time: None,
      error: None,
    }
  }

  async fn register_callback(&self, id: String, callback: AsyncCallback) {
    self.callback_registry.register(id, callback).await;
  }

  async fn remove_callback(&self, id: &str) {
    self.callback_registry.remove(id).await;
  }

  async fn clear_callbacks(&self) {
    self.callback_registry.clear().await;
  }
}

//=============================================================================
// GLOBAL TIMEKEEPER INSTANCE
//=============================================================================

static GLOBAL_TIMEKEEPER: tokio::sync::OnceCell<Arc<TimeKeeper>> = tokio::sync::OnceCell::const_new();

/// TimeKeeper for managing scheduled callbacks
pub struct TimeKeeper {
  quartz: QuartzEngine,
  total_formations: AtomicU64,
  is_hibernating: AtomicBool,
}

impl TimeKeeper {
  fn new() -> Self {
    Self {
      quartz: QuartzEngine::new(),
      total_formations: AtomicU64::new(0),
      is_hibernating: AtomicBool::new(false),
    }
  }

  /// Keep a timer with callback - main API
  pub async fn keep<F>(
    &self,
    interval: u64,
    callback: F,
    repeat: impl Into<TimerRepeat>,
    id: impl Into<String>,
    delay: Option<u64>
  ) -> Result<String, String>
    where
      F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> +
        Send +
        Sync +
        'static
  {
    if self.is_hibernating.load(Ordering::SeqCst) {
      return Err("TimeKeeper is hibernating".to_string());
    }

    let formation_id = id.into();
    let repeat_config = repeat.into();

    // Remove existing formation with same ID
    self.forget(&formation_id).await;

    // Create formation
    let formation = Formation::new(
      formation_id.clone(),
      interval,
      repeat_config,
      delay,
      Priority::Normal
    );

    // Register callback
    let callback_arc: AsyncCallback = Arc::new(move || callback());
    self.quartz.register_callback(formation.callback_id.clone(), callback_arc).await;

    // Add to timeline
    let timeline_entry = QuartzEngine::formation_to_entry(&formation);
    timeline::add(timeline_entry)?;
    self.total_formations.fetch_add(1, Ordering::SeqCst);

    // Start quartz if not running
    self.quartz.start().await;

    Ok(formation_id)
  }

  /// Remove a timer
  pub async fn forget(&self, id: &str) {
    timeline::forget(id);
    self.quartz.remove_callback(id).await;
  }

  /// Wait for a duration (Promise-based delay)
  pub async fn wait(&self, duration_ms: u64) -> Result<(), String> {
    if duration_ms == 0 {
      return Ok(());
    }

    // Simple approach using tokio::time::sleep
    tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms)).await;
    Ok(())
  }

  /// Reset all timers
  pub async fn reset(&self) {
    self.quartz.stop().await;
    self.quartz.clear_callbacks().await;
    timeline::clear();
    self.total_formations.store(0, Ordering::SeqCst);
    self.is_hibernating.store(false, Ordering::SeqCst);
  }

  /// Hibernate - stop all timers and prevent new ones
  pub async fn hibernate(&self) {
    self.is_hibernating.store(true, Ordering::SeqCst);
    self.quartz.stop().await;
    self.quartz.clear_callbacks().await;
    timeline::clear();
  }

  /// Get status information
  pub fn status(&self) -> serde_json::Value {
    let formations = timeline::get_all();
    let active_count = formations
      .iter()
      .filter(|f| {
        // Check if formation is still active based on timeline entry
        f.success && f.error.is_none()
      })
      .count();

    serde_json::json!({
            "active_formations": active_count,
            "total_formations": self.total_formations.load(Ordering::SeqCst),
            "hibernating": self.is_hibernating.load(Ordering::SeqCst),
            "quartz_running": self.quartz.is_running.load(Ordering::SeqCst),
            "timeline_entries": formations.len(),
            "formations": formations
        })
  }
}

/// Get the global TimeKeeper instance
pub async fn get_timekeeper() -> Arc<TimeKeeper> {
  GLOBAL_TIMEKEEPER.get_or_init(|| async { Arc::new(TimeKeeper::new()) }).await.clone()
}

//=============================================================================
// CONVENIENCE FUNCTIONS
//=============================================================================

/// Set a timeout (equivalent to setTimeout)
pub async fn set_timeout<F>(
  callback: F,
  delay_ms: u64,
  id: Option<String>
) -> Result<String, String>
  where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> +
      Send +
      Sync +
      'static
{
  let timekeeper = get_timekeeper().await;
  let timer_id = id.unwrap_or_else(||
    format!("timeout_{}_{}", current_timestamp(), current_timestamp() % 10000)
  );

  timekeeper.keep(delay_ms, callback, TimerRepeat::Once, timer_id, None).await
}

/// Set an interval (equivalent to setInterval)
pub async fn set_interval<F>(
  callback: F,
  interval_ms: u64,
  id: Option<String>
) -> Result<String, String>
  where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> +
      Send +
      Sync +
      'static
{
  let timekeeper = get_timekeeper().await;
  let timer_id = id.unwrap_or_else(||
    format!("interval_{}_{}", current_timestamp(), current_timestamp() % 10000)
  );

  timekeeper.keep(interval_ms, callback, TimerRepeat::Forever, timer_id, None).await
}

/// Clear a timer (cancel formation)
pub async fn clear_timer(formation_id: &str) -> Result<(), String> {
  let timekeeper = get_timekeeper().await;
  timekeeper.forget(formation_id).await;
  Ok(())
}

/// Async delay function
pub async fn delay(duration_ms: u64) -> Result<(), String> {
  tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms)).await;
  Ok(())
}

//=============================================================================
// BUILDER PATTERN FOR COMPLEX TIMERS
//=============================================================================

pub struct FormationBuilder {
  interval: u64,
  repeat: TimerRepeat,
  delay: Option<u64>,
  priority: Priority,
  id: Option<String>,
}

impl FormationBuilder {
  pub fn new(interval: u64) -> Self {
    Self {
      interval,
      repeat: TimerRepeat::Once,
      delay: None,
      priority: Priority::Normal,
      id: None,
    }
  }

  pub fn repeat(mut self, repeat: impl Into<TimerRepeat>) -> Self {
    self.repeat = repeat.into();
    self
  }

  pub fn delay(mut self, delay_ms: u64) -> Self {
    self.delay = Some(delay_ms);
    self
  }

  pub fn priority(mut self, priority: Priority) -> Self {
    self.priority = priority;
    self
  }

  pub fn id(mut self, id: impl Into<String>) -> Self {
    self.id = Some(id.into());
    self
  }

  pub async fn schedule<F>(self, callback: F) -> Result<String, String>
    where
      F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> +
        Send +
        Sync +
        'static
  {
    let timekeeper = get_timekeeper().await;
    let timer_id = self.id.unwrap_or_else(|| {
      format!("formation_{}_{}", current_timestamp(), current_timestamp() % 10000)
    });

    timekeeper.keep(self.interval, callback, self.repeat, timer_id, self.delay).await
  }
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use tokio::time::{ sleep, Duration };
  use std::sync::atomic::{ AtomicU32, Ordering };

  #[tokio::test]
  async fn test_keep_and_forget() {
    let timekeeper = get_timekeeper().await;
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let timer_id = timekeeper
      .keep(
        100, // 100ms interval
        move || {
          let counter = counter_clone.clone();
          Box::pin(async move {
            counter.fetch_add(1, Ordering::SeqCst);
          })
        },
        TimerRepeat::Count(3),
        "test-timer",
        None
      ).await
      .expect("Failed to create timer");

    // Wait for executions
    sleep(Duration::from_millis(350)).await;

    // Should have executed 3 times
    assert_eq!(counter.load(Ordering::SeqCst), 3);

    // Clean up
    timekeeper.forget(&timer_id).await;
  }

  #[tokio::test]
  async fn test_wait() {
    let timekeeper = get_timekeeper().await;
    let start = current_timestamp();

    timekeeper.wait(100).await.expect("Wait failed");

    let elapsed = current_timestamp() - start;
    assert!(elapsed >= 100);
    assert!(elapsed < 150); // Some tolerance for timing
  }

  #[tokio::test]
  async fn test_boolean_repeat() {
    let timekeeper = get_timekeeper().await;
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    // Test true = forever
    let _timer_id = timekeeper
      .keep(
        50,
        move || {
          let counter = counter_clone.clone();
          Box::pin(async move {
            counter.fetch_add(1, Ordering::SeqCst);
          })
        },
        true, // Boolean: forever
        "bool-test",
        None
      ).await
      .expect("Failed to create timer");

    // Let it run briefly
    sleep(Duration::from_millis(200)).await;

    let count = counter.load(Ordering::SeqCst);
    assert!(count >= 3); // Should have run multiple times

    // Clean up
    timekeeper.forget("bool-test").await;
  }

  #[tokio::test]
  async fn test_number_repeat() {
    let timekeeper = get_timekeeper().await;
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    // Test number = specific count
    let _timer_id = timekeeper
      .keep(
        50,
        move || {
          let counter = counter_clone.clone();
          Box::pin(async move {
            counter.fetch_add(1, Ordering::SeqCst);
          })
        },
        5u32, // Number: 5 times
        "number-test",
        None
      ).await
      .expect("Failed to create timer");

    // Wait for completion
    sleep(Duration::from_millis(300)).await;

    assert_eq!(counter.load(Ordering::SeqCst), 5);
  }
}
