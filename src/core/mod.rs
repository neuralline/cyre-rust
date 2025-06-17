// src/core/mod.rs
// FIXED: Complete module integration with performance optimization

use std::sync::{ Arc, atomic::{ AtomicU64, AtomicBool, Ordering } };
use dashmap::DashMap; // High-performance concurrent HashMap

use crate::types::{ ActionId, ActionPayload, AsyncHandler, CyreResponse, IO };
use crate::channel::Channel;
use crate::utils::current_timestamp;

//=============================================================================
// PERFORMANCE-OPTIMIZED DATA STRUCTURES
//=============================================================================

/// Fast channel cache for hot paths (replaces RwLock<HashMap>)
type ChannelCache = DashMap<ActionId, Arc<Channel>>;

//=============================================================================
// MAIN CYRE IMPLEMENTATION - LOCK-FREE WHERE POSSIBLE
//=============================================================================

/// High-performance reactive event manager
pub struct Cyre {
    // Lock-free concurrent data structures (PERFORMANCE FIX)
    channels: ChannelCache,
    fast_path_cache: ChannelCache,
    handlers: DashMap<ActionId, AsyncHandler>,

    // Atomic counters only (no locks for metrics)
    total_executions: AtomicU64,
    fast_path_hits: AtomicU64,
    protection_blocks: AtomicU64,
    timekeeper_executions: AtomicU64,

    // System state
    initialized: AtomicBool,
    timekeeper_enabled: AtomicBool,
    start_time: u64,
}

impl Cyre {
    /// Create a new Cyre instance
    pub fn new() -> Self {
        Self {
            channels: DashMap::new(),
            fast_path_cache: DashMap::new(),
            handlers: DashMap::new(),
            total_executions: AtomicU64::new(0),
            fast_path_hits: AtomicU64::new(0),
            protection_blocks: AtomicU64::new(0),
            timekeeper_executions: AtomicU64::new(0),
            initialized: AtomicBool::new(false),
            timekeeper_enabled: AtomicBool::new(false),
            start_time: current_timestamp(),
        }
    }

    /// Register an action (creates channel + caches if fast path)
    pub fn action(&self, config: IO) {
        let channel = Arc::new(Channel::from_config(&config));

        // Store in main channels
        self.channels.insert(config.id.clone(), channel.clone());

        // Cache in fast path if eligible
        if self.is_fast_path_eligible(&config) {
            self.fast_path_cache.insert(config.id.clone(), channel);
        }

        println!(
            "✅ Action registered: {} (fast_path: {})",
            config.id,
            self.is_fast_path_eligible(&config)
        );
    }

    /// Register a handler for an action
    pub fn on<F, Fut>(&self, action_id: &str, handler: F)
        where
            F: Fn(ActionPayload) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = CyreResponse> + Send + 'static
    {
        let wrapped_handler: AsyncHandler = Arc::new(move |payload| { Box::pin(handler(payload)) });

        self.handlers.insert(action_id.to_string(), wrapped_handler);
        println!("✅ Handler registered: {}", action_id);
    }

    /// FIXED: Complete call execution flow
    pub async fn call(&self, action_id: &str, payload: ActionPayload) -> CyreResponse {
        // Fast path optimization (no locks!)
        if let Some(channel) = self.fast_path_cache.get(action_id) {
            if let Some(handler) = self.handlers.get(action_id) {
                self.fast_path_hits.fetch_add(1, Ordering::Relaxed);
                self.total_executions.fetch_add(1, Ordering::Relaxed);

                return self.execute_fast_path(&channel, handler.clone(), payload).await;
            }
        }

        // Protected path with channel lookup
        if let Some(channel) = self.channels.get(action_id) {
            if let Some(handler) = self.handlers.get(action_id) {
                self.total_executions.fetch_add(1, Ordering::Relaxed);

                return self.execute_with_protection(&channel, handler.clone(), payload).await;
            }
        }

        // Channel/handler not found
        CyreResponse {
            ok: false,
            payload: serde_json::Value::Null,
            message: format!("Action '{}' not found", action_id),
            error: Some("NotFound".to_string()),
            timestamp: current_timestamp(),
            metadata: None,
        }
    }

    /// Fast path execution (zero overhead)
    #[inline(always)]
    async fn execute_fast_path(
        &self,
        _channel: &Channel,
        handler: AsyncHandler,
        payload: ActionPayload
    ) -> CyreResponse {
        // Direct execution - no protection checks
        handler(payload).await
    }

    /// Protected execution with throttling/debouncing
    async fn execute_with_protection(
        &self,
        channel: &Channel,
        handler: AsyncHandler,
        payload: ActionPayload
    ) -> CyreResponse {
        // Check protection mechanisms
        if !channel.can_execute() {
            self.protection_blocks.fetch_add(1, Ordering::Relaxed);
            return CyreResponse {
                ok: false,
                payload: serde_json::Value::Null,
                message: "Blocked by protection mechanism".to_string(),
                error: Some("Protected".to_string()),
                timestamp: current_timestamp(),
                metadata: None,
            };
        }

        // Execute with protection
        let result = handler(payload).await;
        channel.record_execution(&result);
        result
    }

    /// Initialize TimeKeeper integration
    pub async fn init_timekeeper(&self) -> Result<(), String> {
        self.timekeeper_enabled.store(true, Ordering::Relaxed);
        println!("🕒 TimeKeeper integration initialized");
        Ok(())
    }

    /// Check if action exists
    pub fn has_channel(&self, id: &str) -> bool {
        self.channels.contains_key(id)
    }

    /// Get performance metrics (lock-free)
    pub fn get_performance_metrics(&self) -> serde_json::Value {
        let total = self.total_executions.load(Ordering::Relaxed);
        let fast_hits = self.fast_path_hits.load(Ordering::Relaxed);
        let blocks = self.protection_blocks.load(Ordering::Relaxed);
        let timekeeper = self.timekeeper_executions.load(Ordering::Relaxed);

        serde_json::json!({
            "total_executions": total,
            "fast_path_hits": fast_hits,
            "fast_path_ratio": if total > 0 { (fast_hits as f64 / total as f64) * 100.0 } else { 0.0 },
            "protection_blocks": blocks,
            "timekeeper_executions": timekeeper,
            "active_channels": self.channels.len(),
            "uptime_ms": current_timestamp() - self.start_time,
        })
    }

    /// Remove an action
    pub fn forget(&self, action_id: &str) -> bool {
        let removed_channel = self.channels.remove(action_id).is_some();
        let _removed_fast = self.fast_path_cache.remove(action_id).is_some();
        let removed_handler = self.handlers.remove(action_id).is_some();

        if removed_channel || removed_handler {
            println!("🗑️ Action forgotten: {}", action_id);
        }

        removed_channel || removed_handler
    }

    /// Get channel count
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }
}

//=============================================================================
// ENHANCED IO IMPLEMENTATION
//=============================================================================

impl Cyre {
    /// Check if IO config is eligible for fast path caching (moved to Cyre)
    fn is_fast_path_eligible(&self, config: &IO) -> bool {
        // Fast path only for actions with no protection or scheduling
        config.throttle.is_none() &&
            config.debounce.is_none() &&
            config.interval.is_none() &&
            config.delay.is_none() &&
            config.repeat.is_none()
    }

    /// Check if this IO has scheduling properties
    fn has_scheduling(&self, config: &IO) -> bool {
        config.interval.is_some() || config.repeat.is_some() || config.delay.is_some()
    }
}

//=============================================================================
// DEFAULT IMPLEMENTATIONS
//=============================================================================

impl Default for Cyre {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Cyre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cyre")
            .field("total_executions", &self.total_executions.load(Ordering::Relaxed))
            .field("fast_path_hits", &self.fast_path_hits.load(Ordering::Relaxed))
            .field("protection_blocks", &self.protection_blocks.load(Ordering::Relaxed))
            .field("timekeeper_enabled", &self.timekeeper_enabled.load(Ordering::Relaxed))
            .field("channel_count", &self.channels.len())
            .finish()
    }
}
