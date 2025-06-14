// Cyre Rust Core Implementation - FIXED VERSION
// Functional TypeScript-to-Rust translation with performance focus

use std::{
    collections::HashMap,
    sync::{Arc, RwLock, atomic::{AtomicU64, AtomicBool, Ordering}},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    future::Future,
    pin::Pin,
};
use tokio::{
    sync::Mutex,
    time::sleep,
    task::JoinHandle,
};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

//=============================================================================
// CORE TYPES - Matching Cyre's TypeScript interfaces
//=============================================================================

pub type ActionId = String;
pub type ActionPayload = JsonValue;
pub type AsyncHandler = Arc<dyn Fn(ActionPayload) -> Pin<Box<dyn Future<Output = CyreResponse> + Send>> + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High, 
    Medium,
    Low,
    Background,
}

#[derive(Debug, Clone)]
pub struct CyreResponse<T = ActionPayload> {
    pub ok: bool,
    pub payload: T,
    pub message: String,
    pub error: Option<String>,
    pub timestamp: u64,
    pub metadata: Option<ResponseMetadata>,
}

#[derive(Debug, Clone)]
pub struct ResponseMetadata {
    pub execution_time: u64,
    pub scheduled: bool,
    pub delayed: bool,
    pub duration: Option<u64>,
    pub interval: Option<u64>,
    pub repeat: Option<RepeatConfig>,
    pub validation_passed: bool,
    pub condition_met: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepeatConfig {
    Count(u32),
    Infinite,
    None,
}

//=============================================================================
// IO INTERFACE - Core action configuration  
//=============================================================================

#[derive(Debug, Clone)]
pub struct IO {
    pub id: ActionId,
    pub name: Option<String>,
    pub path: Option<String>,
    pub payload: Option<ActionPayload>,
    
    // Protection mechanisms
    pub throttle: Option<u64>,        // milliseconds
    pub debounce: Option<u64>,        // milliseconds
    pub max_wait: Option<u64>,        // milliseconds
    pub detect_changes: Option<bool>,
    
    // Timing & scheduling
    pub delay: Option<u64>,           // milliseconds
    pub interval: Option<u64>,        // milliseconds
    pub repeat: Option<RepeatConfig>,
    
    // Processing
    pub priority: Option<Priority>,
    pub required: Option<bool>,
    pub block: Option<bool>,
    
    // Talents (conditions/transforms)
    pub condition: Option<String>,    // Simplified for now
    pub transform: Option<String>,    // Simplified for now
    
    // Branch metadata
    pub branch_id: Option<String>,
    pub tags: Vec<String>,
}

//=============================================================================
// SUBSCRIPTION & RESPONSE TYPES
//=============================================================================

pub struct SubscriptionResponse {
    pub ok: bool,
    pub message: String,
    // Note: Removed unsubscribe function to avoid Debug trait issues
}

#[derive(Debug)]
pub struct ActionResult {
    pub ok: bool,
    pub message: String,
}

//=============================================================================
// STORES - Core data management matching Cyre's architecture
//=============================================================================

// IO Store - Action configurations
type IOStore = Arc<RwLock<HashMap<ActionId, IO>>>;

// Subscriber Store - Event handlers
#[derive(Clone)]
struct Subscriber {
    id: ActionId,
    handler: AsyncHandler,
    created_at: Instant,
}

type SubscriberStore = Arc<RwLock<HashMap<ActionId, Subscriber>>>;

// Payload Store - Current state management  
#[derive(Debug)]
struct PayloadEntry {
    current: ActionPayload,
    previous: Option<ActionPayload>,
    last_updated: u64,
    update_count: AtomicU64,
    frozen: AtomicBool,
}

// Manual Clone implementation for PayloadEntry
impl Clone for PayloadEntry {
    fn clone(&self) -> Self {
        Self {
            current: self.current.clone(),
            previous: self.previous.clone(),
            last_updated: self.last_updated,
            update_count: AtomicU64::new(self.update_count.load(Ordering::SeqCst)),
            frozen: AtomicBool::new(self.frozen.load(Ordering::SeqCst)),
        }
    }
}

type PayloadStore = Arc<RwLock<HashMap<ActionId, PayloadEntry>>>;

// Timeline Store - Scheduled executions
#[derive(Debug)]
struct TimelineEntry {
    id: ActionId,
    next_execution: u64,
    interval: Option<u64>,
    repeat_count: Option<u32>,
    timer_handle: Option<JoinHandle<()>>,
}

type TimelineStore = Arc<RwLock<HashMap<String, TimelineEntry>>>;

// Branch Store - Hierarchical organization
#[derive(Debug)]
struct BranchEntry {
    id: String,
    path: String,
    parent_path: Option<String>,
    depth: u32,
    created_at: u64,
    is_active: AtomicBool,
    channel_count: AtomicU64,
}

// Manual Clone implementation for BranchEntry
impl Clone for BranchEntry {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            path: self.path.clone(),
            parent_path: self.parent_path.clone(),
            depth: self.depth,
            created_at: self.created_at,
            is_active: AtomicBool::new(self.is_active.load(Ordering::SeqCst)),
            channel_count: AtomicU64::new(self.channel_count.load(Ordering::SeqCst)),
        }
    }
}

type BranchStore = Arc<RwLock<HashMap<String, BranchEntry>>>;

//=============================================================================
// PROTECTION MECHANISMS - Core to Cyre's performance
//=============================================================================

#[derive(Debug)]
struct ProtectionState {
    last_execution: AtomicU64,
    last_throttle: AtomicU64,
    debounce_timer: Arc<Mutex<Option<JoinHandle<()>>>>,
    debounce_payload: Arc<Mutex<Option<ActionPayload>>>,
    execution_count: AtomicU64,
}

// Manual Clone implementation for ProtectionState
impl Clone for ProtectionState {
    fn clone(&self) -> Self {
        Self {
            last_execution: AtomicU64::new(self.last_execution.load(Ordering::SeqCst)),
            last_throttle: AtomicU64::new(self.last_throttle.load(Ordering::SeqCst)),
            debounce_timer: Arc::new(Mutex::new(None)), // Reset timer on clone
            debounce_payload: Arc::new(Mutex::new(None)), // Reset payload on clone
            execution_count: AtomicU64::new(self.execution_count.load(Ordering::SeqCst)),
        }
    }
}

impl ProtectionState {
    fn new() -> Self {
        Self {
            last_execution: AtomicU64::new(0),
            last_throttle: AtomicU64::new(0),
            debounce_timer: Arc::new(Mutex::new(None)),
            debounce_payload: Arc::new(Mutex::new(None)),
            execution_count: AtomicU64::new(0),
        }
    }
}

type ProtectionStore = Arc<RwLock<HashMap<ActionId, Arc<ProtectionState>>>>;

//=============================================================================
// TIMEKEEPER - Precise scheduling system
//=============================================================================

pub struct TimeKeeper {
    timers: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    counter: AtomicU64,
}

impl TimeKeeper {
    pub fn new() -> Self {
        Self {
            timers: Arc::new(RwLock::new(HashMap::new())),
            counter: AtomicU64::new(0),
        }
    }
    
    pub async fn keep<F, Fut>(&self, duration_ms: u64, callback: F) -> String 
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let timer_id = format!("timer_{}", self.counter.fetch_add(1, Ordering::SeqCst));
        let timers = Arc::clone(&self.timers);
        let timer_id_clone = timer_id.clone();
        
        let handle = tokio::spawn(async move {
            sleep(Duration::from_millis(duration_ms)).await;
            callback().await;
            
            // Clean up timer after execution
            if let Ok(mut timers) = timers.write() {
                timers.remove(&timer_id_clone);
            }
        });
        
        if let Ok(mut timers) = self.timers.write() {
            timers.insert(timer_id.clone(), handle);
        }
        
        timer_id
    }
    
    pub fn forget(&self, timer_id: &str) -> bool {
        if let Ok(mut timers) = self.timers.write() {
            if let Some(handle) = timers.remove(timer_id) {
                handle.abort();
                return true;
            }
        }
        false
    }
}

//=============================================================================
// QUANTUM BREATHING - Stress adaptation system
//=============================================================================

#[derive(Debug, Clone)]
pub struct BreathingState {
    pub stress_level: f64,
    pub breathing_rate: u64,
    pub pattern: BreathingPattern,
    pub last_measurement: Instant,
}

#[derive(Debug, Clone)]
pub enum BreathingPattern {
    Normal,    // 1:1:0.5 ratio
    Recovery,  // 2:2:1 ratio  
}

impl BreathingState {
    fn new() -> Self {
        Self {
            stress_level: 0.0,
            breathing_rate: 200, // Base 200ms
            pattern: BreathingPattern::Normal,
            last_measurement: Instant::now(),
        }
    }
    
    fn adapt_to_stress(&mut self, new_stress: f64) {
        self.stress_level = new_stress;
        
        // Adapt breathing rate based on stress
        self.breathing_rate = match new_stress {
            s if s < 0.3 => 50,   // Low stress, fast breathing
            s if s < 0.6 => 200,  // Normal stress
            s if s < 0.8 => 500,  // High stress
            _ => 2000,            // Critical stress, slow recovery
        };
        
        // Switch patterns at high stress
        self.pattern = if new_stress > 0.7 {
            BreathingPattern::Recovery
        } else {
            BreathingPattern::Normal
        };
    }
}

//=============================================================================
// MAIN CYRE IMPLEMENTATION
//=============================================================================

pub struct Cyre {
    // Core stores
    io_store: IOStore,
    subscriber_store: SubscriberStore,
    payload_store: PayloadStore,
    timeline_store: TimelineStore,
    branch_store: BranchStore,
    protection_store: ProtectionStore,
    
    // Systems
    timekeeper: TimeKeeper,
    breathing: Arc<RwLock<BreathingState>>,
    
    // Performance metrics
    execution_count: AtomicU64,
    total_duration: AtomicU64,
    error_count: AtomicU64,
}

impl Cyre {
    pub fn new() -> Self {
        Self {
            io_store: Arc::new(RwLock::new(HashMap::new())),
            subscriber_store: Arc::new(RwLock::new(HashMap::new())),
            payload_store: Arc::new(RwLock::new(HashMap::new())),
            timeline_store: Arc::new(RwLock::new(HashMap::new())),
            branch_store: Arc::new(RwLock::new(HashMap::new())),
            protection_store: Arc::new(RwLock::new(HashMap::new())),
            timekeeper: TimeKeeper::new(),
            breathing: Arc::new(RwLock::new(BreathingState::new())),
            execution_count: AtomicU64::new(0),
            total_duration: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }
    
    //=========================================================================
    // CORE API - Matching Cyre's TypeScript interface
    //=========================================================================
    
    pub fn action(&self, config: IO) -> ActionResult {
        // Validate configuration
        if config.id.is_empty() {
            return ActionResult {
                ok: false,
                message: "Action ID is required".to_string(),
            };
        }
        
        // Check for conflicting protections
        if config.throttle.is_some() && config.debounce.is_some() {
            return ActionResult {
                ok: false,
                message: "Throttle and debounce cannot both be active".to_string(),
            };
        }
        
        // Store IO configuration
        if let Ok(mut store) = self.io_store.write() {
            store.insert(config.id.clone(), config.clone());
        } else {
            return ActionResult {
                ok: false,
                message: "Failed to store action configuration".to_string(),
            };
        }
        
        // Initialize protection state
        if let Ok(mut protection_store) = self.protection_store.write() {
            protection_store.insert(config.id.clone(), Arc::new(ProtectionState::new()));
        }
        
        // Initialize payload if provided
        if let Some(payload) = &config.payload {
            if let Ok(mut payload_store) = self.payload_store.write() {
                payload_store.insert(config.id.clone(), PayloadEntry {
                    current: payload.clone(),
                    previous: None,
                    last_updated: self.current_timestamp(),
                    update_count: AtomicU64::new(0),
                    frozen: AtomicBool::new(false),
                });
            }
        }
        
        ActionResult {
            ok: true,
            message: format!("Action '{}' registered successfully", config.id),
        }
    }
    
    pub fn on<F, Fut>(&self, id: &str, handler: F) -> SubscriptionResponse 
    where
        F: Fn(ActionPayload) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = CyreResponse> + Send + 'static,
    {
        let action_id = id.to_string();
        
        // Create async handler with proper lifetime management
        let async_handler: AsyncHandler = Arc::new(move |payload: ActionPayload| {
            let fut = handler(payload);
            Box::pin(fut)
        });
        
        let subscriber = Subscriber {
            id: action_id.clone(),
            handler: async_handler,
            created_at: Instant::now(),
        };
        
        if let Ok(mut store) = self.subscriber_store.write() {
            store.insert(action_id.clone(), subscriber);
            
            SubscriptionResponse {
                ok: true,
                message: format!("Subscribed to '{}'", action_id),
            }
        } else {
            SubscriptionResponse {
                ok: false,
                message: "Failed to register subscriber".to_string(),
            }
        }
    }
    
    pub async fn call(&self, id: &str, payload: Option<ActionPayload>) -> CyreResponse {
        let start = Instant::now();
        let action_id = id.to_string();
        
        // Get action configuration
        let action = {
            if let Ok(store) = self.io_store.read() {
                store.get(&action_id).cloned()
            } else {
                return self.error_response("Failed to read action store".to_string());
            }
        };
        
        let action = match action {
            Some(a) => a,
            None => return self.error_response(format!("Action '{}' not found", action_id)),
        };
        
        // Prepare payload
        let call_payload = payload.unwrap_or_else(|| {
            self.get_payload(&action_id).unwrap_or(serde_json::json!(null))
        });
        
        // Apply protection pipeline
        match self.apply_protections(&action, call_payload.clone()).await {
            Ok(Some(protected_payload)) => {
                self.execute_action(&action, protected_payload, start).await
            },
            Ok(None) => {
                // Action was blocked by protection
                CyreResponse {
                    ok: false,
                    payload: serde_json::json!(null),
                    message: "Action blocked by protection mechanism".to_string(),
                    error: None,
                    timestamp: self.current_timestamp(),
                    metadata: None,
                }
            },
            Err(err) => self.error_response(err),
        }
    }
    
    pub fn get(&self, id: &str) -> Option<IO> {
        if let Ok(store) = self.io_store.read() {
            store.get(id).cloned()
        } else {
            None
        }
    }
    
    pub fn forget(&self, id: &str) -> bool {
        let action_id = id.to_string();
        let mut success = true;
        
        // Remove from all stores
        if let Ok(mut store) = self.io_store.write() {
            success &= store.remove(&action_id).is_some();
        }
        
        if let Ok(mut store) = self.subscriber_store.write() {
            store.remove(&action_id);
        }
        
        if let Ok(mut store) = self.payload_store.write() {
            store.remove(&action_id);
        }
        
        if let Ok(mut store) = self.protection_store.write() {
            store.remove(&action_id);
        }
        
        success
    }
    
    //=========================================================================
    // PROTECTION PIPELINE - Core performance features
    //=========================================================================
    
    async fn apply_protections(&self, action: &IO, payload: ActionPayload) -> Result<Option<ActionPayload>, String> {
        let action_id = &action.id;
        
        // Get protection state
        let protection_state = {
            if let Ok(store) = self.protection_store.read() {
                store.get(action_id).cloned()
            } else {
                return Err("Failed to read protection store".to_string());
            }
        };
        
        let protection_state = match protection_state {
            Some(state) => state,
            None => return Ok(Some(payload)), // No protection state, allow
        };
        
        let now = self.current_timestamp();
        
        // 1. Throttle check - Rate limiting
        if let Some(throttle_ms) = action.throttle {
            let last_throttle = protection_state.last_throttle.load(Ordering::SeqCst);
            if now - last_throttle < throttle_ms {
                return Ok(None); // Blocked by throttle
            }
            protection_state.last_throttle.store(now, Ordering::SeqCst);
        }
        
        // 2. Debounce - Call collapsing  
        if let Some(debounce_ms) = action.debounce {
            return self.apply_debounce(action, payload, debounce_ms, &protection_state).await;
        }
        
        // 3. Change detection
        if action.detect_changes.unwrap_or(false) {
            if !self.payload_changed(action_id, &payload) {
                return Ok(None); // No change, skip execution
            }
        }
        
        Ok(Some(payload))
    }
    
    async fn apply_debounce(
        &self, 
        action: &IO, 
        payload: ActionPayload, 
        debounce_ms: u64,
        protection_state: &Arc<ProtectionState>
    ) -> Result<Option<ActionPayload>, String> {
        // Store latest payload
        {
            let mut debounce_payload = protection_state.debounce_payload.lock().await;
            *debounce_payload = Some(payload.clone());
        }
        
        // Cancel existing timer
        {
            let mut timer = protection_state.debounce_timer.lock().await;
            if let Some(handle) = timer.take() {
                handle.abort();
            }
        }
        
        // Clone subscriber BEFORE the async block to avoid holding the lock
        let subscriber_opt = {
            if let Ok(store) = self.subscriber_store.read() {
                store.get(&action.id).cloned()
            } else {
                None
            }
        };
        
        // Set up new debounce timer with safe sharing
        let action_clone = action.clone();
        let protection_clone = Arc::clone(protection_state);
        let payload_store = Arc::clone(&self.payload_store);
        
        let timer_handle = tokio::spawn(async move {
            sleep(Duration::from_millis(debounce_ms)).await;
            
            // Execute with latest payload after debounce
            let latest_payload = {
                let payload_guard = protection_clone.debounce_payload.lock().await;
                payload_guard.clone()
            };
            
            if let Some(final_payload) = latest_payload {
                // Update payload store
                if let Ok(mut store) = payload_store.write() {
                    if let Some(entry) = store.get_mut(&action_clone.id) {
                        entry.previous = Some(entry.current.clone());
                        entry.current = final_payload.clone();
                        entry.last_updated = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;
                        entry.update_count.fetch_add(1, Ordering::SeqCst);
                    }
                }
                
                // Execute subscriber if we cloned it successfully
                if let Some(subscriber) = subscriber_opt {
                    let _result = (subscriber.handler)(final_payload).await;
                    println!("Debounced execution completed for: {}", action_clone.id);
                }
            }
        });
        
        // Store timer handle
        {
            let mut timer = protection_state.debounce_timer.lock().await;
            *timer = Some(timer_handle);
        }
        
        Ok(None) // Debounced, will execute later
    }
    
    //=========================================================================
    // EXECUTION ENGINE
    //=========================================================================
    
    async fn execute_action(&self, action: &IO, payload: ActionPayload, start: Instant) -> CyreResponse {
        let action_id = &action.id;
        
        // Update payload store
        self.update_payload(action_id, payload.clone());
        
        // Get subscriber
        let subscriber = {
            if let Ok(store) = self.subscriber_store.read() {
                store.get(action_id).cloned()
            } else {
                return self.error_response("Failed to read subscriber store".to_string());
            }
        };
        
        let subscriber = match subscriber {
            Some(sub) => sub,
            None => {
                return CyreResponse {
                    ok: false,
                    payload: serde_json::json!(null),
                    message: format!("No subscriber found for '{}'", action_id),
                    error: None,
                    timestamp: self.current_timestamp(),
                    metadata: None,
                }
            }
        };
        
        // Execute handler
        let result = (subscriber.handler)(payload).await;
        
        // Update metrics
        let duration = start.elapsed().as_millis() as u64;
        self.execution_count.fetch_add(1, Ordering::SeqCst);
        self.total_duration.fetch_add(duration, Ordering::SeqCst);
        
        if !result.ok {
            self.error_count.fetch_add(1, Ordering::SeqCst);
        }
        
        result
    }
    
    //=========================================================================
    // UTILITY METHODS
    //=========================================================================
    
    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
    
    fn get_payload(&self, id: &str) -> Option<ActionPayload> {
        if let Ok(store) = self.payload_store.read() {
            store.get(id).map(|entry| entry.current.clone())
        } else {
            None
        }
    }
    
    fn update_payload(&self, id: &str, payload: ActionPayload) {
        if let Ok(mut store) = self.payload_store.write() {
            if let Some(entry) = store.get_mut(id) {
                entry.previous = Some(entry.current.clone());
                entry.current = payload;
                entry.last_updated = self.current_timestamp();
                entry.update_count.fetch_add(1, Ordering::SeqCst);
            } else {
                store.insert(id.to_string(), PayloadEntry {
                    current: payload,
                    previous: None,
                    last_updated: self.current_timestamp(),
                    update_count: AtomicU64::new(1),
                    frozen: AtomicBool::new(false),
                });
            }
        }
    }
    
    fn payload_changed(&self, id: &str, new_payload: &ActionPayload) -> bool {
        if let Ok(store) = self.payload_store.read() {
            if let Some(entry) = store.get(id) {
                return &entry.current != new_payload;
            }
        }
        true // No previous payload, consider it changed
    }
    
    fn error_response(&self, message: String) -> CyreResponse {
        self.error_count.fetch_add(1, Ordering::SeqCst);
        CyreResponse {
            ok: false,
            payload: serde_json::json!(null),
            message,
            error: Some("execution_error".to_string()),
            timestamp: self.current_timestamp(),
            metadata: None,
        }
    }
    
    //=========================================================================
    // METRICS & DEBUGGING
    //=========================================================================
    
    pub fn get_metrics(&self) -> HashMap<String, u64> {
        let mut metrics = HashMap::new();
        metrics.insert("execution_count".to_string(), self.execution_count.load(Ordering::SeqCst));
        metrics.insert("total_duration".to_string(), self.total_duration.load(Ordering::SeqCst));
        metrics.insert("error_count".to_string(), self.error_count.load(Ordering::SeqCst));
        
        let avg_duration = if self.execution_count.load(Ordering::SeqCst) > 0 {
            self.total_duration.load(Ordering::SeqCst) / self.execution_count.load(Ordering::SeqCst)
        } else {
            0
        };
        metrics.insert("avg_duration".to_string(), avg_duration);
        
        metrics
    }
}

//=============================================================================
// IMPLEMENTATION TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_basic_action_flow() {
        let cyre = Cyre::new();
        
        // Register action
        let result = cyre.action(IO {
            id: "test-action".to_string(),
            name: Some("Test Action".to_string()),
            payload: Some(json!({"initial": true})),
            ..Default::default()
        });
        assert!(result.ok);
        
        // Subscribe
        let sub_result = cyre.on("test-action", |payload| async move {
            CyreResponse {
                ok: true,
                payload: json!({"processed": true, "received": payload}),
                message: "Action processed successfully".to_string(),
                error: None,
                timestamp: 0,
                metadata: None,
            }
        });
        assert!(sub_result.ok);
        
        // Call
        let call_result = cyre.call("test-action", Some(json!({"data": "test"}))).await;
        assert!(call_result.ok);
        assert_eq!(call_result.message, "Action processed successfully");
    }
    
    #[tokio::test]
    async fn test_throttle_protection() {
        let cyre = Cyre::new();
        
        // Register action with throttle
        cyre.action(IO {
            id: "throttled-action".to_string(),
            throttle: Some(100), // 100ms throttle
            ..Default::default()
        });
        
        cyre.on("throttled-action", |_| async move {
            CyreResponse {
                ok: true,
                payload: json!({"executed": true}),
                message: "Executed".to_string(),
                error: None,
                timestamp: 0,
                metadata: None,
            }
        });
        
        // First call should succeed
        let result1 = cyre.call("throttled-action", None).await;
        assert!(result1.ok);
        
        // Immediate second call should be blocked
        let result2 = cyre.call("throttled-action", None).await;
        assert!(!result2.ok);
        
        // After throttle period, should succeed again
        tokio::time::sleep(Duration::from_millis(150)).await;
        let result3 = cyre.call("throttled-action", None).await;
        assert!(result3.ok);
    }
    
    #[tokio::test]
    async fn test_change_detection() {
        let cyre = Cyre::new();
        
        // Register action with change detection
        cyre.action(IO {
            id: "change-detection-action".to_string(),
            detect_changes: Some(true),
            ..Default::default()
        });
        
        cyre.on("change-detection-action", |payload| async move {
            CyreResponse {
                ok: true,
                payload: payload,
                message: "Change detected".to_string(),
                error: None,
                timestamp: 0,
                metadata: None,
            }
        });
        
        // First call should execute
        let result1 = cyre.call("change-detection-action", Some(json!({"value": 1}))).await;
        assert!(result1.ok);
        
        // Same payload should be skipped
        let result2 = cyre.call("change-detection-action", Some(json!({"value": 1}))).await;
        assert!(!result2.ok);
        
        // Different payload should execute
        let result3 = cyre.call("change-detection-action", Some(json!({"value": 2}))).await;
        assert!(result3.ok);
    }
    
    #[tokio::test]
    async fn test_debounce_protection() {
        let cyre = Cyre::new();
        
        // Register action with debounce
        cyre.action(IO {
            id: "debounced-action".to_string(),
            debounce: Some(100), // 100ms debounce
            ..Default::default()
        });
        
        cyre.on("debounced-action", |payload| async move {
            println!("Debounced handler executed with: {}", payload);
            CyreResponse {
                ok: true,
                payload: json!({"debounced": true, "data": payload}),
                message: "Debounced execution".to_string(),
                error: None,
                timestamp: 0,
                metadata: None,
            }
        });
        
        // Rapid calls - should be debounced
        let result1 = cyre.call("debounced-action", Some(json!({"call": 1}))).await;
        let result2 = cyre.call("debounced-action", Some(json!({"call": 2}))).await;
        let result3 = cyre.call("debounced-action", Some(json!({"call": 3}))).await;
        
        // All immediate calls should return None (debounced)
        assert!(!result1.ok);
        assert!(!result2.ok);
        assert!(!result3.ok);
        
        // Wait for debounce timer and a bit more
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // The debounced execution should have completed by now
        // (In a full implementation, we'd have a way to verify this)
        println!("Debounce test completed - timer should have executed");
    }
}

// Default implementations
impl Default for IO {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: None,
            path: None,
            payload: None,
            throttle: None,
            debounce: None,
            max_wait: None,
            detect_changes: None,
            delay: None,
            interval: None,
            repeat: None,
            priority: None,
            required: None,
            block: None,
            condition: None,
            transform: None,
            branch_id: None,
            tags: Vec::new(),
        }
    }
}