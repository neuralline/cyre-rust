// src/context/state.rs
// File location: src/context/state.rs
// Cyre state management with clean separation - Rust implementation

//=============================================================================
// IMPORTS
//=============================================================================

use std::sync::{ Arc, RwLock, OnceLock };
use std::collections::HashMap;
use serde::{ Serialize, Deserialize };
use serde_json::{ Value as JsonValue, json };
use crate::types::{ IO, AsyncHandler };
use crate::utils::current_timestamp;

//=============================================================================
// STATE KEY TYPE
//=============================================================================

/// State key type for consistent identification
pub type StateKey = String;

//=============================================================================
// STORE BLUEPRINT - Generic Store Interface
//=============================================================================

/// Generic store interface - like your createStore blueprint
pub trait StateStore<T> {
  fn get(&self, key: &StateKey) -> Option<T>;
  fn set(&mut self, key: StateKey, value: T);
  fn forget(&mut self, key: &StateKey) -> bool;
  fn clear(&mut self);
  fn get_all(&self) -> Vec<T>;
  fn size(&self) -> usize;
}

/// Concrete implementation of StateStore
#[derive(Debug)]
pub struct Store<T> {
  data: HashMap<StateKey, T>,
  max_history_size: usize,
}

impl<T: Clone> Store<T> {
  pub fn new() -> Self {
    Self {
      data: HashMap::new(),
      max_history_size: 1000, // Default limit
    }
  }

  pub fn with_max_size(max_size: usize) -> Self {
    Self {
      data: HashMap::new(),
      max_history_size: max_size,
    }
  }
}

impl<T: Clone> StateStore<T> for Store<T> {
  fn get(&self, key: &StateKey) -> Option<T> {
    self.data.get(key).cloned()
  }

  fn set(&mut self, key: StateKey, value: T) {
    self.data.insert(key, value);

    // Prune if exceeding max size
    if self.data.len() > self.max_history_size {
      // Simple pruning - remove oldest entries
      let keys_to_remove: Vec<_> = self.data
        .keys()
        .take(self.data.len() - self.max_history_size)
        .cloned()
        .collect();

      for key in keys_to_remove {
        self.data.remove(&key);
      }
    }
  }

  fn forget(&mut self, key: &StateKey) -> bool {
    self.data.remove(key).is_some()
  }

  fn clear(&mut self) {
    self.data.clear();
  }

  fn get_all(&self) -> Vec<T> {
    self.data.values().cloned().collect()
  }

  fn size(&self) -> usize {
    self.data.len()
  }
}

//=============================================================================
// GLOBAL STATE STORES
//=============================================================================

static IO_STORE: OnceLock<Arc<RwLock<Store<IO>>>> = OnceLock::new();
static SUBSCRIBER_STORE: OnceLock<Arc<RwLock<Store<ISubscriber>>>> = OnceLock::new();
static TIMELINE_STORE: OnceLock<Arc<RwLock<Store<TimelineEntry>>>> = OnceLock::new();
static BRANCH_STORE: OnceLock<Arc<RwLock<Store<BranchStore>>>> = OnceLock::new();

fn get_io_store() -> &'static Arc<RwLock<Store<IO>>> {
  IO_STORE.get_or_init(|| Arc::new(RwLock::new(Store::new())))
}

fn get_subscriber_store() -> &'static Arc<RwLock<Store<ISubscriber>>> {
  SUBSCRIBER_STORE.get_or_init(|| Arc::new(RwLock::new(Store::new())))
}

fn get_timeline_store() -> &'static Arc<RwLock<Store<TimelineEntry>>> {
  TIMELINE_STORE.get_or_init(|| Arc::new(RwLock::new(Store::new())))
}

fn get_branch_store() -> &'static Arc<RwLock<Store<BranchStore>>> {
  BRANCH_STORE.get_or_init(|| Arc::new(RwLock::new(Store::new())))
}

//=============================================================================
// TYPE DEFINITIONS
//=============================================================================

/// Timeline entry for tracking action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
  pub id: String,
  pub action_id: String,
  pub timestamp: u64,
  pub payload: JsonValue,
  pub success: bool,
  pub execution_time: Option<u64>,
  pub error: Option<String>,
}

/// Subscriber interface for action handlers
#[derive(Clone)]
pub struct ISubscriber {
  pub id: String,
  pub handler: AsyncHandler,
  pub active: bool,
  pub created_at: u64,
}

impl ISubscriber {
  pub fn new(id: String, handler: AsyncHandler) -> Self {
    Self {
      id,
      handler,
      active: true,
      created_at: current_timestamp(),
    }
  }
}

// Manual Debug implementation for ISubscriber (handler can't be debugged)
impl std::fmt::Debug for ISubscriber {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ISubscriber")
      .field("id", &self.id)
      .field("handler", &"<async_handler>")
      .field("active", &self.active)
      .field("created_at", &self.created_at)
      .finish()
  }
}

/// Branch store for hierarchical organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchStore {
  pub id: String,
  pub name: String,
  pub parent_id: Option<String>,
  pub children: Vec<String>,
  pub created_at: u64,
}

/// Timeline store alias for legacy compatibility
pub type Timeline = Vec<TimelineEntry>;
pub type TimelineStore = Store<TimelineEntry>;

//=============================================================================
// IO MODULE - Action Configuration Management
//=============================================================================

pub mod io {
  use super::*;

  /// Get IO configuration by ID
  pub fn get(id: &str) -> Option<IO> {
    get_io_store().read().ok()?.get(&id.to_string())
  }

  /// Set IO configuration
  pub fn set(id: String, config: IO) -> Result<(), String> {
    get_io_store()
      .write()
      .map_err(|e| format!("Failed to acquire write lock: {}", e))?
      .set(id, config);
    Ok(())
  }

  /// Remove IO configuration
  pub fn forget(id: &str) -> bool {
    get_io_store()
      .write()
      .map(|mut store| store.forget(&id.to_string()))
      .unwrap_or(false)
  }

  /// Clear all IO configurations
  pub fn clear() {
    if let Ok(mut store) = get_io_store().write() {
      store.clear();
    }
  }

  /// Get all IO configurations
  pub fn get_all() -> Vec<IO> {
    get_io_store()
      .read()
      .map(|store| store.get_all())
      .unwrap_or_default()
  }

  /// Get size of IO store
  pub fn size() -> usize {
    get_io_store()
      .read()
      .map(|store| store.size())
      .unwrap_or(0)
  }

  /// Get status of IO store
  pub fn status() -> JsonValue {
    json!({
            "type": "io_store",
            "size": size(),
            "timestamp": current_timestamp()
        })
  }
}

//=============================================================================
// SUBSCRIBERS MODULE - Handler Management
//=============================================================================

pub mod subscribers {
  use super::*;

  /// Get subscriber by ID
  pub fn get(id: &str) -> Option<ISubscriber> {
    get_subscriber_store().read().ok()?.get(&id.to_string())
  }

  /// Set subscriber
  pub fn set(id: String, subscriber: ISubscriber) -> Result<(), String> {
    get_subscriber_store()
      .write()
      .map_err(|e| format!("Failed to acquire write lock: {}", e))?
      .set(id, subscriber);
    Ok(())
  }

  /// Remove subscriber
  pub fn forget(id: &str) -> bool {
    get_subscriber_store()
      .write()
      .map(|mut store| store.forget(&id.to_string()))
      .unwrap_or(false)
  }

  /// Clear all subscribers
  pub fn clear() {
    if let Ok(mut store) = get_subscriber_store().write() {
      store.clear();
    }
  }

  /// Get all subscribers
  pub fn get_all() -> Vec<ISubscriber> {
    get_subscriber_store()
      .read()
      .map(|store| store.get_all())
      .unwrap_or_default()
  }

  /// Get size of subscriber store
  pub fn size() -> usize {
    get_subscriber_store()
      .read()
      .map(|store| store.size())
      .unwrap_or(0)
  }

  /// Get status of subscriber store
  pub fn status() -> JsonValue {
    json!({
            "type": "subscriber_store",
            "size": size(),
            "active_handlers": get_all().iter().filter(|s| s.active).count(),
            "timestamp": current_timestamp()
        })
  }
}

//=============================================================================
// TIMELINE MODULE - Execution History
//=============================================================================

pub mod timeline {
  use super::*;

  /// Add timeline entry
  pub fn add(entry: TimelineEntry) -> Result<(), String> {
    get_timeline_store()
      .write()
      .map_err(|e| format!("Failed to acquire write lock: {}", e))?
      .set(entry.id.clone(), entry);
    Ok(())
  }

  /// Get timeline entry by ID
  pub fn get(id: &str) -> Option<TimelineEntry> {
    get_timeline_store().read().ok()?.get(&id.to_string())
  }

  /// Remove timeline entry
  pub fn forget(id: &str) -> bool {
    get_timeline_store()
      .write()
      .map(|mut store| store.forget(&id.to_string()))
      .unwrap_or(false)
  }

  /// Clear timeline
  pub fn clear() {
    if let Ok(mut store) = get_timeline_store().write() {
      store.clear();
    }
  }

  /// Get all timeline entries
  pub fn get_all() -> Vec<TimelineEntry> {
    get_timeline_store()
      .read()
      .map(|store| store.get_all())
      .unwrap_or_default()
  }

  /// Get recent timeline entries (last n)
  pub fn get_recent(count: usize) -> Vec<TimelineEntry> {
    let mut entries = get_all();
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    entries.into_iter().take(count).collect()
  }

  /// Get size of timeline store
  pub fn size() -> usize {
    get_timeline_store()
      .read()
      .map(|store| store.size())
      .unwrap_or(0)
  }

  /// Get status of timeline store
  pub fn status() -> JsonValue {
    let entries = get_all();
    let success_count = entries
      .iter()
      .filter(|e| e.success)
      .count();
    let error_count = entries.len() - success_count;

    json!({
            "type": "timeline_store",
            "size": size(),
            "success_count": success_count,
            "error_count": error_count,
            "timestamp": current_timestamp()
        })
  }
}

//==============
pub mod payload {
  use super::*;
  use crate::types::ActionPayload;

  /// Get payload by action ID
  pub fn get(_action_id: &str) -> Option<ActionPayload> {
    // For now, return None - implement when needed
    // This would typically use another global store similar to IO_STORE
    None
  }

  /// Set default payload for action
  pub fn set(action_id: String, payload: ActionPayload) -> Result<(), String> {
    // For now, just log - implement when payload storage is needed
    println!("Payload stored for action '{}': {}", action_id, payload);
    Ok(())
  }

  /// Remove payload for action
  pub fn forget(action_id: &str) -> bool {
    // For now, return false - implement when needed
    println!("Payload forgotten for action '{}'", action_id);
    false
  }

  /// Clear all payloads
  pub fn clear() {
    // For now, do nothing - implement when needed
    println!("All payloads cleared");
  }

  /// Get all payloads
  pub fn get_all() -> Vec<(String, ActionPayload)> {
    // For now, return empty - implement when needed
    Vec::new()
  }

  /// Get size of payload store
  pub fn size() -> usize {
    // For now, return 0 - implement when needed
    0
  }

  /// Get status of payload store
  pub fn status() -> JsonValue {
    json!({
            "type": "payload_store",
            "size": size(),
            "timestamp": current_timestamp()
        })
  }
}

//=============================================================================
// STORES MODULE - Branch Management
//=============================================================================

pub mod stores {
  use super::*;

  /// Get branch by ID
  pub fn get(id: &str) -> Option<BranchStore> {
    get_branch_store().read().ok()?.get(&id.to_string())
  }

  /// Set branch
  pub fn set(id: String, branch: BranchStore) -> Result<(), String> {
    get_branch_store()
      .write()
      .map_err(|e| format!("Failed to acquire write lock: {}", e))?
      .set(id, branch);
    Ok(())
  }

  /// Remove branch
  pub fn forget(id: &str) -> bool {
    get_branch_store()
      .write()
      .map(|mut store| store.forget(&id.to_string()))
      .unwrap_or(false)
  }

  /// Clear all branches
  pub fn clear() {
    if let Ok(mut store) = get_branch_store().write() {
      store.clear();
    }
  }

  /// Get all branches
  pub fn get_all() -> Vec<BranchStore> {
    get_branch_store()
      .read()
      .map(|store| store.get_all())
      .unwrap_or_default()
  }

  /// Get size of branch store
  pub fn size() -> usize {
    get_branch_store()
      .read()
      .map(|store| store.size())
      .unwrap_or(0)
  }

  /// Get status of branch store
  pub fn status() -> JsonValue {
    json!({
            "type": "branch_store",
            "size": size(),
            "timestamp": current_timestamp()
        })
  }
}

//=============================================================================
// LEGACY COMPATIBILITY FUNCTIONS
//=============================================================================

/// Get timeline (legacy compatibility)
pub fn get_timeline() -> Timeline {
  timeline::get_all()
}

//=============================================================================
// METRICS AND STATE MANAGEMENT TRAITS
//=============================================================================

/// Metrics operations trait
pub trait MetricsOps {
  fn record_action(&self, action_id: &str, success: bool, execution_time: u64);
  fn get_action_metrics(&self, action_id: &str) -> Option<StateActionMetrics>;
}

/// Metrics state representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsState {
  pub total_actions: u64,
  pub successful_actions: u64,
  pub failed_actions: u64,
  pub average_execution_time: f64,
  pub last_updated: u64,
}

/// Metrics update structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsUpdate {
  pub action_count: u64,
  pub success_count: u64,
  pub error_count: u64,
  pub execution_time: u64,
}

/// Action-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateActionMetrics {
  pub action_id: String,
  pub call_count: u64,
  pub success_count: u64,
  pub error_count: u64,
  pub average_execution_time: f64,
  pub last_called: u64,
}

//=============================================================================
// GLOBAL STATE STATUS FUNCTION
//=============================================================================

/// Get overall state status
pub fn get_state_status() -> JsonValue {
  json!({
        "stores": {
            "io": io::status(),
            "subscribers": subscribers::status(),
            "timeline": timeline::status(),
            "branches": stores::status()
        },
        "totals": {
            "io_configs": io::size(),
            "active_handlers": subscribers::size(),
            "timeline_entries": timeline::size(),
            "branches": stores::size()
        },
        "timestamp": current_timestamp()
    })
}
