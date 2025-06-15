// src/core/mod.rs
// Core Cyre implementation - fixed async Send issues

use std::sync::{Arc, RwLock, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::collections::HashMap;

use crate::types::{
    ActionId, ActionPayload, AsyncHandler, CyreResponse, IO, likely
};
use crate::channel::Channel;
use crate::utils::current_timestamp;

//=============================================================================
// COMPILED PIPELINE FOR PERFORMANCE
//=============================================================================

#[derive(Debug, Clone)]
pub struct CompiledPipeline {
    pub action_id: String,
    pub has_protection: bool,
    pub has_talents: bool,
    pub has_middleware: bool,
    pub fast_path_eligible: bool,
    pub compiled_at: u64,
}

//=============================================================================
// MAIN CYRE IMPLEMENTATION
//=============================================================================

/// High-performance reactive event manager
pub struct Cyre {
    // Core stores (performance optimized)
    channels: Arc<RwLock<HashMap<ActionId, Arc<Channel>>>>,
    configurations: Arc<RwLock<HashMap<ActionId, IO>>>,
    fast_path_channels: Arc<RwLock<HashMap<ActionId, Arc<Channel>>>>, // Performance cache
    
    // Pipeline optimization
    pipeline_cache: Arc<RwLock<HashMap<ActionId, CompiledPipeline>>>,
    
    // Global performance counters
    total_executions: AtomicU64,
    fast_path_hits: AtomicU64,
    protection_blocks: AtomicU64,
    
    // System state
    initialized: AtomicBool,
    start_time: u64,
}

impl Cyre {
    /// Create a new Cyre instance
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            configurations: Arc::new(RwLock::new(HashMap::new())),
            fast_path_channels: Arc::new(RwLock::new(HashMap::new())),
            pipeline_cache: Arc::new(RwLock::new(HashMap::new())),
            total_executions: AtomicU64::new(0),
            fast_path_hits: AtomicU64::new(0),
            protection_blocks: AtomicU64::new(0),
            initialized: AtomicBool::new(false),
            start_time: current_timestamp(),
        }
    }

    /// Check if a channel exists (public API for timekeeper)
    pub fn has_channel(&self, id: &str) -> bool {
        self.channels.read().unwrap().contains_key(id)
    }

    /// Initialize TimeKeeper integration
    pub async fn init_timekeeper(&mut self) -> Result<(), String> {
        println!("🕒 TimeKeeper initialized for Cyre instance");
        Ok(())
    }

    /// Execute an action directly (for TimeKeeper integration)
    pub async fn execute_action(&self, id: &str, payload: ActionPayload) -> CyreResponse {
        self.call(id, payload).await
    }

    /// Register an action configuration
    pub fn action(&mut self, config: IO) -> CyreResponse {
        let id = config.id.clone();
        
        // Store configuration
        {
            let mut configs = self.configurations.write().unwrap();
            configs.insert(id.clone(), config.clone());
        }

        // Create compiled pipeline for optimization
        let pipeline = CompiledPipeline {
            action_id: id.clone(),
            has_protection: config.has_protection(),
            has_talents: config.has_advanced_features(),
            has_middleware: !config.middleware.is_empty(),
            fast_path_eligible: config.is_fast_path_eligible(),
            compiled_at: current_timestamp(),
        };

        {
            let mut cache = self.pipeline_cache.write().unwrap();
            cache.insert(id.clone(), pipeline);
        }

        CyreResponse {
            ok: true,
            payload: serde_json::json!({
                "channel_id": id,
                "fast_path_eligible": config.is_fast_path_eligible(),
                "has_protection": config.has_protection()
            }),
            message: "Action registered with pipeline optimization".to_string(),
            error: None,
            timestamp: current_timestamp(),
            metadata: None,
        }
    }

    /// Register a handler for an action
    pub fn on<F>(&mut self, id: &str, handler: F) -> CyreResponse 
    where
        F: Fn(ActionPayload) -> std::pin::Pin<Box<dyn std::future::Future<Output = CyreResponse> + Send>> + Send + Sync + 'static,
    {
        let handler: AsyncHandler = Arc::new(handler);
        
        // Get or create configuration
        let config = {
            let configs = self.configurations.read().unwrap();
            configs.get(id).cloned().unwrap_or_else(|| IO::new(id))
        };
        
        // Create channel
        let channel = Channel::new(id.to_string(), handler, config.clone());
        let channel_arc = Arc::new(channel);

        // Store in appropriate channel cache
        {
            let mut channels = self.channels.write().unwrap();
            channels.insert(id.to_string(), channel_arc.clone());
        }

        if config.is_fast_path_eligible() {
            let mut fast_channels = self.fast_path_channels.write().unwrap();
            fast_channels.insert(id.to_string(), channel_arc);
        }
        
        CyreResponse {
            ok: true,
            payload: serde_json::json!({
                "subscriber_id": id,
                "registered": true,
                "fast_path": config.is_fast_path_eligible()
            }),
            message: "Handler registered".to_string(),
            error: None,
            timestamp: current_timestamp(),
            metadata: None,
        }
    }

    /// Call an action (hot path optimized) - FIXED for Send
    #[inline(always)]
    pub async fn call(&self, id: &str, payload: ActionPayload) -> CyreResponse {
        // FIXED: Clone channel outside of lock to ensure Send
        let fast_channel = {
            let fast_channels = self.fast_path_channels.read().unwrap();
            fast_channels.get(id).cloned()
        };

        // Try fast path first - critical optimization
        if let Some(channel) = fast_channel {
            if likely(channel.is_fast_path()) {
                self.fast_path_hits.fetch_add(1, Ordering::Relaxed);
                self.total_executions.fetch_add(1, Ordering::Relaxed);
                return channel.execute_fast_path(payload).await;
            }
        }

        // FIXED: Clone channel outside of lock for protected execution
        let protected_channel = {
            let channels = self.channels.read().unwrap();
            channels.get(id).cloned()
        };

        // Fall back to protected execution
        if let Some(channel) = protected_channel {
            if let Some(result) = channel.execute_with_protection(payload).await {
                self.total_executions.fetch_add(1, Ordering::Relaxed);
                return result;
            } else {
                self.protection_blocks.fetch_add(1, Ordering::Relaxed);
                return CyreResponse {
                    ok: false,
                    payload: serde_json::Value::Null,
                    message: "Call blocked by protection".to_string(),
                    error: Some("Protected".to_string()),
                    timestamp: current_timestamp(),
                    metadata: None,
                };
            }
        }

        // Channel not found
        CyreResponse {
            ok: false,
            payload: serde_json::Value::Null,
            message: "Channel not found".to_string(),
            error: Some("Not found".to_string()),
            timestamp: current_timestamp(),
            metadata: None,
        }
    }

    /// Remove an action and its handler
    pub fn forget(&mut self, id: &str) -> bool {
        let mut removed = false;
        
        // Remove from all stores
        {
            let mut channels = self.channels.write().unwrap();
            removed |= channels.remove(id).is_some();
        }
        
        {
            let mut fast_channels = self.fast_path_channels.write().unwrap();
            fast_channels.remove(id);
        }
        
        {
            let mut configs = self.configurations.write().unwrap();
            configs.remove(id);
        }
        
        {
            let mut cache = self.pipeline_cache.write().unwrap();
            cache.remove(id);
        }
        
        if removed {
            println!("🗑️ Action forgotten: {}", id);
        }
        
        removed
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> serde_json::Value {
        let total_calls = self.total_executions.load(Ordering::Relaxed);
        let fast_path_hits = self.fast_path_hits.load(Ordering::Relaxed);
        let protection_blocks = self.protection_blocks.load(Ordering::Relaxed);
        
        serde_json::json!({
            "total_executions": total_calls,
            "fast_path_hits": fast_path_hits,
            "protection_blocks": protection_blocks,
            "fast_path_ratio": if total_calls > 0 { 
                (fast_path_hits as f64 / total_calls as f64) * 100.0 
            } else { 0.0 },
            "active_channels": self.channels.read().unwrap().len(),
            "fast_path_channels": self.fast_path_channels.read().unwrap().len(),
            "uptime_ms": current_timestamp() - self.start_time,
        })
    }

    /// Get comprehensive system metrics
    pub fn get_comprehensive_metrics(&self) -> serde_json::Value {
        let total_calls = self.total_executions.load(Ordering::Relaxed);
        let fast_path_hits = self.fast_path_hits.load(Ordering::Relaxed);
        let protection_blocks = self.protection_blocks.load(Ordering::Relaxed);
        
        serde_json::json!({
            "performance": {
                "total_executions": total_calls,
                "fast_path_hits": fast_path_hits,
                "fast_path_ratio": if total_calls > 0 { 
                    (fast_path_hits as f64 / total_calls as f64) * 100.0 
                } else { 0.0 },
                "protection_blocks": protection_blocks,
                "uptime_ms": current_timestamp() - self.start_time
            },
            "system": {
                "active_channels": self.channels.read().unwrap().len(),
                "fast_path_channels": self.fast_path_channels.read().unwrap().len(),
                "compiled_pipelines": self.pipeline_cache.read().unwrap().len(),
            }
        })
    }

    /// Clear all actions and handlers
    pub fn clear(&mut self) {
        {
            let mut channels = self.channels.write().unwrap();
            channels.clear();
        }
        
        {
            let mut fast_channels = self.fast_path_channels.write().unwrap();
            fast_channels.clear();
        }
        
        {
            let mut configs = self.configurations.write().unwrap();
            configs.clear();
        }
        
        {
            let mut cache = self.pipeline_cache.write().unwrap();
            cache.clear();
        }
        
        // Reset counters
        self.total_executions.store(0, Ordering::Relaxed);
        self.fast_path_hits.store(0, Ordering::Relaxed);
        self.protection_blocks.store(0, Ordering::Relaxed);
    }

    /// Get system uptime in milliseconds
    pub fn uptime_ms(&self) -> u64 {
        current_timestamp() - self.start_time
    }

    /// Check if system is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Relaxed)
    }

    /// Get number of active channels
    pub fn channel_count(&self) -> usize {
        self.channels.read().unwrap().len()
    }

    /// Get number of fast path channels
    pub fn fast_path_channel_count(&self) -> usize {
        self.fast_path_channels.read().unwrap().len()
    }

    /// Get total execution count
    pub fn execution_count(&self) -> u64 {
        self.total_executions.load(Ordering::Relaxed)
    }

    /// Get fast path hit count
    pub fn fast_path_hits(&self) -> u64 {
        self.fast_path_hits.load(Ordering::Relaxed)
    }

    /// Get protection block count
    pub fn protection_blocks(&self) -> u64 {
        self.protection_blocks.load(Ordering::Relaxed)
    }
}

impl std::fmt::Debug for Cyre {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cyre")
            .field("total_executions", &self.total_executions.load(Ordering::Relaxed))
            .field("fast_path_hits", &self.fast_path_hits.load(Ordering::Relaxed))
            .field("protection_blocks", &self.protection_blocks.load(Ordering::Relaxed))
            .field("initialized", &self.initialized.load(Ordering::Relaxed))
            .field("start_time", &self.start_time)
            .field("channel_count", &self.channels.read().unwrap().len())
            .finish()
    }
}

impl Default for Cyre {
    fn default() -> Self {
        Self::new()
    }
}