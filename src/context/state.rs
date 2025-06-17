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
use crate::types::{ ActionId, IO, AsyncHandler, Timer };
use crate::context::sensor;
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
            max_history_size: 1000,
        }
    }

    fn cleanup(&mut self) {
        if self.data.len() > self.max_history_size {
            let keys_to_delete: Vec<StateKey> = self.data
                .keys()
                .take(self.data.len() - self.max_history_size)
                .cloned()
                .collect();

            for key in keys_to_delete {
                self.data.remove(&key);
            }
        }
    }
}

impl<T: Clone> StateStore<T> for Store<T> {
    fn get(&self, key: &StateKey) -> Option<T> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: StateKey, value: T) {
        self.data.insert(key, value);
        self.cleanup();
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
// METRICS STATE
//=============================================================================

/// Metrics state structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsState {
    pub active_formations: u64,
    pub total_executions: u64,
    pub fast_path_hits: u64,
    pub protection_blocks: u64,
    pub last_update: u64,
}

impl Default for MetricsState {
    fn default() -> Self {
        Self {
            active_formations: 0,
            total_executions: 0,
            fast_path_hits: 0,
            protection_blocks: 0,
            last_update: current_timestamp(),
        }
    }
}

impl MetricsState {
    pub fn update(&mut self, updates: MetricsUpdate) {
        if let Some(active_formations) = updates.active_formations {
            self.active_formations = active_formations;
        }
        if let Some(total_executions) = updates.total_executions {
            self.total_executions = total_executions;
        }
        if let Some(fast_path_hits) = updates.fast_path_hits {
            self.fast_path_hits = fast_path_hits;
        }
        if let Some(protection_blocks) = updates.protection_blocks {
            self.protection_blocks = protection_blocks;
        }
        self.last_update = current_timestamp();
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Update struct for partial metrics updates
#[derive(Debug, Default)]
pub struct MetricsUpdate {
    pub active_formations: Option<u64>,
    pub total_executions: Option<u64>,
    pub fast_path_hits: Option<u64>,
    pub protection_blocks: Option<u64>,
}

//=============================================================================
// ACTION METRICS
//=============================================================================

/// Per-action metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateActionMetrics {
    pub last_execution_time: u64,
    pub execution_count: u64,
    pub errors: Vec<String>,
}

//=============================================================================
// SUBSCRIBER TYPE
//=============================================================================

/// Subscriber interface for event handlers
#[derive(Clone)]
pub struct ISubscriber {
    pub id: ActionId,
    pub handler: AsyncHandler,
    pub created_at: u64,
}

// Manual Debug implementation since AsyncHandler doesn't implement Debug
impl std::fmt::Debug for ISubscriber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ISubscriber")
            .field("id", &self.id)
            .field("handler", &"<AsyncHandler>")
            .field("created_at", &self.created_at)
            .finish()
    }
}

//=============================================================================
// BRANCH STORE TYPE
//=============================================================================

/// Branch store for branching logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchStore {
    pub id: ActionId,
    pub conditions: Vec<String>,
    pub created_at: u64,
}

//=============================================================================
// PAYLOAD STATE INTEGRATION
//=============================================================================

/// Payload state management (separate from IO config)
#[derive(Debug)]
pub struct PayloadState {
    store: Arc<RwLock<Store<JsonValue>>>,
    max_payload_size: usize, // Maximum size in bytes
}

impl PayloadState {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(Store::new())),
            max_payload_size: 1024 * 1024, // 1MB default limit
        }
    }

    pub fn set(&self, key: StateKey, payload: JsonValue) -> Result<(), String> {
        // Check payload size
        let payload_size = serde_json
            ::to_string(&payload)
            .map_err(|e| format!("Failed to serialize payload: {}", e))?
            .len();

        if payload_size > self.max_payload_size {
            return Err(
                format!(
                    "Payload size {} bytes exceeds maximum allowed size of {} bytes",
                    payload_size,
                    self.max_payload_size
                )
            );
        }

        if let Ok(mut store) = self.store.write() {
            store.set(key, payload);
            Ok(())
        } else {
            Err("Failed to write to payload store".to_string())
        }
    }

    pub fn get(&self, key: &StateKey) -> Option<JsonValue> {
        if let Ok(store) = self.store.read() { store.get(key) } else { None }
    }

    pub fn forget(&self, key: &StateKey) -> bool {
        if let Ok(mut store) = self.store.write() { store.forget(key) } else { false }
    }

    pub fn clear(&self) {
        if let Ok(mut store) = self.store.write() {
            store.clear();
        }
    }

    pub fn get_total_size(&self) -> usize {
        if let Ok(store) = self.store.read() {
            store
                .get_all()
                .iter()
                .filter_map(|p| serde_json::to_string(p).ok())
                .map(|s| s.len())
                .sum()
        } else {
            0
        }
    }

    pub fn get_largest_payload(&self) -> Option<(StateKey, usize)> {
        if let Ok(store) = self.store.read() {
            store
                .get_all()
                .iter()
                .filter_map(|payload| {
                    serde_json
                        ::to_string(payload)
                        .ok()
                        .map(|s| (
                            payload
                                .get("id")
                                .and_then(|id| id.as_str())
                                .unwrap_or("")
                                .to_string(),
                            s.len(),
                        ))
                })
                .max_by_key(|&(_, size)| size)
        } else {
            None
        }
    }
}

//=============================================================================
// GLOBAL STORES - Thread-safe singletons
//=============================================================================

/// Global stores using OnceLock for thread-safe initialization
static IO_STORE: OnceLock<Arc<RwLock<Store<IO>>>> = OnceLock::new();
static SUBSCRIBER_STORE: OnceLock<Arc<RwLock<Store<ISubscriber>>>> = OnceLock::new();
static TIMELINE_STORE: OnceLock<Arc<RwLock<Store<Timer>>>> = OnceLock::new();
static BRANCH_STORE: OnceLock<Arc<RwLock<Store<BranchStore>>>> = OnceLock::new();
static METRICS_STATE: OnceLock<Arc<RwLock<MetricsState>>> = OnceLock::new();
static PAYLOAD_STATE: OnceLock<PayloadState> = OnceLock::new();

/// Get or initialize the IO store
fn get_io_store() -> &'static Arc<RwLock<Store<IO>>> {
    IO_STORE.get_or_init(|| Arc::new(RwLock::new(Store::new())))
}

/// Get or initialize the subscriber store
fn get_subscriber_store() -> &'static Arc<RwLock<Store<ISubscriber>>> {
    SUBSCRIBER_STORE.get_or_init(|| Arc::new(RwLock::new(Store::new())))
}

/// Get or initialize the timeline store
fn get_timeline_store() -> &'static Arc<RwLock<Store<Timer>>> {
    TIMELINE_STORE.get_or_init(|| Arc::new(RwLock::new(Store::new())))
}

/// Get or initialize the branch store
fn get_branch_store() -> &'static Arc<RwLock<Store<BranchStore>>> {
    BRANCH_STORE.get_or_init(|| Arc::new(RwLock::new(Store::new())))
}

/// Get or initialize the metrics state
fn get_metrics_state() -> &'static Arc<RwLock<MetricsState>> {
    METRICS_STATE.get_or_init(|| Arc::new(RwLock::new(MetricsState::default())))
}

/// Get or initialize the payload state
fn get_payload_state() -> &'static PayloadState {
    PAYLOAD_STATE.get_or_init(|| PayloadState::new())
}

//=============================================================================
// IO OPERATIONS - Configuration only, no payload data
//=============================================================================

/// IO operations - channel information store
pub struct IOState;

impl IOState {
    /// Set action configuration (no payload)
    pub fn set(mut action: IO) -> Result<(), String> {
        // Validate action
        if action.id.trim().is_empty() {
            let error = "IO state: Channel must have an id";
            sensor::critical("io-state", error, Some("IOState::set"), None);
            return Err(error.to_string());
        }

        // Add timestamp
        action.timestamp = Some(current_timestamp());

        // Store in IO store
        let store = get_io_store();
        match store.write() {
            Ok(mut store) => {
                store.set(action.id.clone(), action.clone());
                sensor::debug("io-state", &format!("Channel '{}' configured", action.id), false);
                Ok(())
            }
            Err(e) => {
                let error = format!("IO state corruption detected: {}", e);
                sensor::critical("io-state", &error, Some("IOState::set"), None);
                Err(error)
            }
        }
    }

    /// Get action configuration
    pub fn get(id: &StateKey) -> Option<IO> {
        let store = get_io_store();
        match store.read() {
            Ok(store) => store.get(id),
            Err(_) => {
                sensor::error("io-state", "Failed to read IO store", Some("IOState::get"), None);
                None
            }
        }
    }

    /// Remove action configuration
    pub fn forget(id: &StateKey) -> bool {
        // Also remove payload
        get_payload_state().forget(id);

        // Remove from IO store
        let store = get_io_store();
        match store.write() {
            Ok(mut store) => {
                let result = store.forget(id);
                if result {
                    sensor::debug("io-state", &format!("Channel '{}' forgotten", id), false);
                }
                result
            }
            Err(_) => {
                sensor::error(
                    "io-state",
                    &format!("Failed to forget channel '{}'", id),
                    Some("IOState::forget"),
                    None
                );
                false
            }
        }
    }

    /// Clear all action configurations
    pub fn clear() {
        // Clear payload state
        get_payload_state().clear();

        // Clear IO store
        let store = get_io_store();
        if let Ok(mut store) = store.write() {
            store.clear();
        }

        // Reset metrics
        let metrics = get_metrics_state();
        if let Ok(mut metrics) = metrics.write() {
            metrics.reset();
        }

        sensor::info("io-state", "All channels cleared", true);
    }

    /// Get all action configurations
    pub fn get_all() -> Vec<IO> {
        let store = get_io_store();
        match store.read() {
            Ok(store) => store.get_all(),
            Err(_) => {
                sensor::error(
                    "io-state",
                    "Failed to read all IO configs",
                    Some("IOState::get_all"),
                    None
                );
                Vec::new()
            }
        }
    }

    /// Get action metrics
    pub fn get_metrics(id: &str) -> Option<StateActionMetrics> {
        if let Some(action) = IOState::get(&id.to_string()) {
            Some(StateActionMetrics {
                last_execution_time: action._last_exec_time.unwrap_or(0),
                execution_count: action._execution_count,
                errors: Vec::new(), // IO doesn't have direct errors field, using empty vec
            })
        } else {
            None
        }
    }
}

//=============================================================================
// SUBSCRIBERS OPERATIONS
//=============================================================================

/// Subscribers management
pub struct Subscribers;

impl Subscribers {
    pub fn add(subscriber: ISubscriber) -> Result<(), String> {
        if subscriber.id.trim().is_empty() {
            return Err("Invalid subscriber format: ID required".to_string());
        }

        let store = get_subscriber_store();
        match store.write() {
            Ok(mut store) => {
                store.set(subscriber.id.clone(), subscriber);
                Ok(())
            }
            Err(e) => Err(format!("Failed to add subscriber: {}", e)),
        }
    }

    pub fn get(id: &str) -> Option<ISubscriber> {
        let store = get_subscriber_store();
        store.read().ok()?.get(&id.to_string())
    }

    pub fn forget(id: &str) -> bool {
        let store = get_subscriber_store();
        store
            .write()
            .map(|mut s| s.forget(&id.to_string()))
            .unwrap_or(false)
    }

    pub fn clear() {
        let store = get_subscriber_store();
        if let Ok(mut store) = store.write() {
            store.clear();
        }
    }

    pub fn get_all() -> Vec<ISubscriber> {
        let store = get_subscriber_store();
        store
            .read()
            .map(|s| s.get_all())
            .unwrap_or_default()
    }
}

//=============================================================================
// TIMELINE OPERATIONS - Scheduled task store
//=============================================================================

/// Timeline management for scheduled tasks
pub struct Timeline;

impl Timeline {
    pub fn add(timer: Timer) {
        if timer.id.trim().is_empty() {
            return;
        }

        let store = get_timeline_store();
        if let Ok(mut store) = store.write() {
            store.set(timer.id.clone(), timer);

            // Update active formations count in metrics
            let active_count = store
                .get_all()
                .iter()
                .filter(|t| t.status == "active")
                .count() as u64;

            let metrics = get_metrics_state();
            if let Ok(mut metrics) = metrics.write() {
                metrics.update(MetricsUpdate {
                    active_formations: Some(active_count),
                    ..Default::default()
                });
            }
        }
    }

    pub fn get(id: &StateKey) -> Option<Timer> {
        let store = get_timeline_store();
        store.read().ok()?.get(id)
    }

    pub fn forget(id: &StateKey) -> bool {
        let store = get_timeline_store();
        let result = if let Ok(mut store) = store.write() {
            // Clear any active timers (this would integrate with actual timer cleanup)
            if let Some(_timer) = store.get(id) {
                sensor::debug("timeline", &format!("Forgetting timer '{}'", id), false);
            }

            let result = store.forget(id);

            // Update active formations count
            let active_count = store
                .get_all()
                .iter()
                .filter(|t| t.status == "active")
                .count() as u64;

            let metrics = get_metrics_state();
            if let Ok(mut metrics) = metrics.write() {
                metrics.update(MetricsUpdate {
                    active_formations: Some(active_count),
                    ..Default::default()
                });
            }

            result
        } else {
            false
        };

        result
    }

    pub fn clear() {
        let store = get_timeline_store();
        if let Ok(mut store) = store.write() {
            // Clear all timers (this would integrate with actual timer cleanup)
            store.clear();

            // Reset metrics
            let metrics = get_metrics_state();
            if let Ok(mut metrics) = metrics.write() {
                metrics.update(MetricsUpdate {
                    active_formations: Some(0),
                    ..Default::default()
                });
            }
        }
    }

    pub fn get_all() -> Vec<Timer> {
        let store = get_timeline_store();
        store
            .read()
            .map(|s| s.get_all())
            .unwrap_or_default()
    }

    pub fn get_active() -> Vec<Timer> {
        let store = get_timeline_store();
        store
            .read()
            .map(|s|
                s
                    .get_all()
                    .into_iter()
                    .filter(|t| t.status == "active")
                    .collect()
            )
            .unwrap_or_default()
    }
}

//=============================================================================
// PAYLOAD STATE OPERATIONS
//=============================================================================

/// Payload state operations (separate from IO config)
pub struct PayloadStateOps;

impl PayloadStateOps {
    pub fn set(id: StateKey, payload: JsonValue) -> Result<(), String> {
        get_payload_state().set(id, payload)
    }

    pub fn get(id: &StateKey) -> Option<JsonValue> {
        get_payload_state().get(id)
    }

    pub fn forget(id: &StateKey) -> bool {
        get_payload_state().forget(id)
    }

    pub fn clear() {
        get_payload_state().clear()
    }

    pub fn get_size_metrics() -> JsonValue {
        let state = get_payload_state();
        json!({
            "total_size_bytes": state.get_total_size(),
            "payload_count": state.store.read().map(|s| s.size()).unwrap_or(0),
            "largest_payload": state.get_largest_payload()
                .map(|(id, size)| json!({
                    "id": id,
                    "size_bytes": size
                }))
        })
    }
}

//=============================================================================
// METRICS OPERATIONS
//=============================================================================

/// Metrics state operations
pub struct MetricsOps;

impl MetricsOps {
    pub fn update(updates: MetricsUpdate) {
        let metrics = get_metrics_state();
        if let Ok(mut metrics) = metrics.write() {
            metrics.update(updates);
        }
    }

    pub fn get() -> MetricsState {
        let metrics = get_metrics_state();
        metrics
            .read()
            .map(|m| m.clone())
            .unwrap_or_default()
    }

    pub fn reset() {
        let metrics = get_metrics_state();
        if let Ok(mut metrics) = metrics.write() {
            metrics.reset();
        }
    }
}

//=============================================================================
// READONLY STORES EXPORT
//=============================================================================

/// Readonly access to stores for external use
pub struct Stores;

impl Stores {
    pub fn io_size() -> usize {
        get_io_store()
            .read()
            .map(|s| s.size())
            .unwrap_or(0)
    }

    pub fn subscribers_size() -> usize {
        get_subscriber_store()
            .read()
            .map(|s| s.size())
            .unwrap_or(0)
    }

    pub fn timeline_size() -> usize {
        get_timeline_store()
            .read()
            .map(|s| s.size())
            .unwrap_or(0)
    }

    pub fn payload_size() -> usize {
        get_payload_state()
            .store.read()
            .map(|s| s.size())
            .unwrap_or(0)
    }

    pub fn get_system_stats() -> JsonValue {
        serde_json::json!({
            "io_channels": Self::io_size(),
            "subscribers": Self::subscribers_size(),
            "active_timers": Self::timeline_size(),
            "payloads": Self::payload_size(),
            "metrics": MetricsOps::get()
        })
    }
}

//=============================================================================
// PUBLIC API - Mirroring your TypeScript structure
//=============================================================================

/// IO operations export (matching your TypeScript API)
pub mod io {
    use super::*;

    pub fn set(action: crate::types::IO) -> Result<(), String> {
        IOState::set(action)
    }

    pub fn get(id: &str) -> Option<crate::types::IO> {
        IOState::get(&id.to_string())
    }

    pub fn forget(id: &str) -> bool {
        IOState::forget(&id.to_string())
    }

    pub fn clear() {
        IOState::clear()
    }

    pub fn get_all() -> Vec<crate::types::IO> {
        IOState::get_all()
    }

    pub fn get_metrics(id: &str) -> Option<StateActionMetrics> {
        IOState::get_metrics(id)
    }
}

/// Subscribers operations export
pub mod subscribers {
    use super::*;

    pub fn add(subscriber: ISubscriber) -> Result<(), String> {
        Subscribers::add(subscriber)
    }

    pub fn get(id: &str) -> Option<ISubscriber> {
        Subscribers::get(id)
    }

    pub fn forget(id: &str) -> bool {
        Subscribers::forget(id)
    }

    pub fn clear() {
        Subscribers::clear()
    }

    pub fn get_all() -> Vec<ISubscriber> {
        Subscribers::get_all()
    }
}

/// Timeline operations export
pub mod timeline {
    use super::*;

    pub fn add(timer: crate::types::Timer) {
        Timeline::add(timer)
    }

    pub fn get(id: &str) -> Option<crate::types::Timer> {
        Timeline::get(&id.to_string())
    }

    pub fn forget(id: &str) -> bool {
        Timeline::forget(&id.to_string())
    }

    pub fn clear() {
        Timeline::clear()
    }

    pub fn get_all() -> Vec<crate::types::Timer> {
        Timeline::get_all()
    }

    pub fn get_active() -> Vec<crate::types::Timer> {
        Timeline::get_active()
    }
}

/// Stores export
pub mod stores {
    pub use super::Stores;
}

//=============================================================================
// BACKWARD COMPATIBILITY FUNCTIONS
//=============================================================================

/// Compatibility functions for existing code that expects the old API

// IO Config operations (for channel.rs)

pub fn store_io_config(config: IO) -> Result<(), String> {
    IOState::set(config)
}

pub fn get_io_config(id: &str) -> Option<IO> {
    IOState::get(&id.to_string())
}

pub fn update_io_config(config: IO) -> Result<(), String> {
    IOState::set(config) // In the new system, set handles both create and update
}

pub fn remove_io_config(id: &str) -> Result<IO, String> {
    if let Some(config) = IOState::get(&id.to_string()) {
        if IOState::forget(&id.to_string()) {
            Ok(config)
        } else {
            Err(format!("Failed to remove config for '{}'", id))
        }
    } else {
        Err(format!("Config not found for '{}'", id))
    }
}

// Handler operations (for channel.rs and core.rs) - FIXED SIGNATURES
pub fn store_handler(action_id: String, handler: AsyncHandler) -> Result<(), String> {
    let subscriber = ISubscriber {
        id: action_id,
        handler,
        created_at: current_timestamp(),
    };
    Subscribers::add(subscriber)
}

pub fn get_handler(id: &str) -> Option<AsyncHandler> {
    Subscribers::get(id).map(|s| s.handler)
}

pub fn remove_handler(id: &str) -> Option<AsyncHandler> {
    // Get the handler before removing
    let handler = Subscribers::get(id).map(|s| s.handler);
    if handler.is_some() {
        Subscribers::forget(id);
    }
    handler
}

// Existence checks
pub fn action_exists(id: &str) -> bool {
    IOState::get(&id.to_string()).is_some()
}

pub fn handler_exists(id: &str) -> bool {
    Subscribers::get(id).is_some()
}

// List operations (for channel.rs) - FIXED RETURN TYPES
pub fn list_all_actions() -> Vec<String> {
    IOState::get_all()
        .into_iter()
        .map(|action| action.id)
        .collect()
}

pub fn get_actions_by_group(group: &str) -> Vec<String> {
    IOState::get_all()
        .into_iter()
        .filter(|action| action.group.as_ref().map_or(false, |g| g == group))
        .map(|action| action.id)
        .collect()
}

pub fn get_actions_by_path(path_prefix: &str) -> Vec<String> {
    IOState::get_all()
        .into_iter()
        .filter(|action| action.path.as_ref().map_or(false, |p| p.starts_with(path_prefix)))
        .map(|action| action.id)
        .collect()
}

// Metrics operations (for core.rs) - FIXED RETURN TYPE
pub fn increment_call_count() {
    MetricsOps::update(MetricsUpdate {
        total_executions: Some(MetricsOps::get().total_executions + 1),
        ..Default::default()
    });
}

pub fn get_system_metrics() -> (u64, u64, usize, usize) {
    let metrics = MetricsOps::get();
    let config_count = Stores::io_size();
    let handler_count = Stores::subscribers_size();
    (
        metrics.total_executions, // total_actions (using total_executions)
        metrics.total_executions, // total_calls (same as total_executions)
        config_count, // config_count
        handler_count, // handler_count
    )
}

pub fn get_system_health() -> JsonValue {
    serde_json::json!({
        "status": "healthy",
        "uptime": current_timestamp(),
        "stores": {
            "io_channels": Stores::io_size(),
            "subscribers": Stores::subscribers_size(),
            "timers": Stores::timeline_size()
        },
        "metrics": {
            "total_executions": MetricsOps::get().total_executions,
            "fast_path_hits": MetricsOps::get().fast_path_hits,
            "protection_blocks": MetricsOps::get().protection_blocks,
            "active_formations": MetricsOps::get().active_formations,
        }
    })
}

//=============================================================================
// LEGACY COMPATIBILITY
//=============================================================================

/// Legacy function for backward compatibility
pub fn get_timeline() -> &'static Arc<RwLock<Store<Timer>>> {
    get_timeline_store()
}

/// Legacy TimelineStore export
pub type TimelineStore = Store<Timer>;
