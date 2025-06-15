// src/channel/channel.rs
// Channel implementation for action execution

use std::{
    sync::{Arc, atomic::{AtomicU64, Ordering}},
    time::Instant,
};

use crate::types::{ActionId, ActionPayload, AsyncHandler, CyreResponse, Priority, IO};
use crate::protection::ProtectionState;
use crate::utils::current_timestamp;

//=============================================================================
// CHANNEL IMPLEMENTATION
//=============================================================================

/// A channel represents a single action with its handler and configuration
pub struct Channel {
    pub id: ActionId,
    handler: AsyncHandler,
    protection: Arc<ProtectionState>,
    priority: Priority,
    fast_path: bool,
    config: IO,
    
    // Performance counters
    execution_count: AtomicU64,
    total_duration: AtomicU64,
    last_execution: AtomicU64,
    error_count: AtomicU64,
}

impl Channel {
    /// Create a new channel with the given configuration
    pub fn new(id: ActionId, handler: AsyncHandler, config: IO) -> Self {
        let protection = Arc::new(ProtectionState::new(
            config.throttle,
            config.debounce,
            config.detect_changes,
        ));
        
        let fast_path = config.is_fast_path_eligible() && protection.is_no_protection();

        Self {
            id,
            handler,
            protection,
            priority: config.priority,
            fast_path,
            config,
            execution_count: AtomicU64::new(0),
            total_duration: AtomicU64::new(0),
            last_execution: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }

    /// Execute via fast path (zero overhead)
    #[inline(always)]
    pub async fn execute_fast_path(&self, payload: ActionPayload) -> CyreResponse {
        let start = Instant::now();
        let result = (self.handler)(payload).await;
        let duration = start.elapsed().as_micros() as u64;
        
        // Minimal metrics update
        self.execution_count.fetch_add(1, Ordering::Relaxed);
        self.total_duration.fetch_add(duration, Ordering::Relaxed);
        self.last_execution.store(current_timestamp(), Ordering::Relaxed);
        
        result
    }

    /// Execute with protection mechanisms
    pub async fn execute_with_protection(&self, payload: ActionPayload) -> Option<CyreResponse> {
        if !self.protection.should_execute(&payload) {
            return None; // Protected - don't execute
        }

        let start = Instant::now();
        let result = (self.handler)(payload).await;
        let duration = start.elapsed().as_micros() as u64;
        
        self.execution_count.fetch_add(1, Ordering::Relaxed);
        self.total_duration.fetch_add(duration, Ordering::Relaxed);
        self.last_execution.store(current_timestamp(), Ordering::Relaxed);
        
        if !result.ok {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }
        
        Some(result)
    }

    /// Execute with full feature support (talents, middleware, etc.)
    pub async fn execute_full_pipeline(&self, payload: ActionPayload) -> Option<CyreResponse> {
        // For now, this is the same as execute_with_protection
        // In the future, this would handle talents, middleware, etc.
        self.execute_with_protection(payload).await
    }

    /// Get channel performance metrics
    pub fn get_metrics(&self) -> serde_json::Value {
        let executions = self.execution_count.load(Ordering::Relaxed);
        let total_duration = self.total_duration.load(Ordering::Relaxed);
        let errors = self.error_count.load(Ordering::Relaxed);
        let (blocks, skips) = self.protection.get_stats();
        
        serde_json::json!({
            "id": self.id,
            "executions": executions,
            "total_duration_us": total_duration,
            "avg_duration_us": if executions > 0 { total_duration / executions } else { 0 },
            "errors": errors,
            "error_rate": if executions > 0 { errors as f64 / executions as f64 } else { 0.0 },
            "protection_blocks": blocks,
            "protection_skips": skips,
            "fast_path": self.fast_path,
            "priority": self.priority,
            "last_execution": self.last_execution.load(Ordering::Relaxed)
        })
    }

    /// Check if this channel uses fast path
    #[inline(always)]
    pub fn is_fast_path(&self) -> bool {
        self.fast_path
    }

    /// Get channel priority
    pub fn priority(&self) -> Priority {
        self.priority
    }

    /// Get channel ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get execution count
    pub fn execution_count(&self) -> u64 {
        self.execution_count.load(Ordering::Relaxed)
    }

    /// Get error count
    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }

    /// Get average execution duration in microseconds
    pub fn avg_duration_us(&self) -> u64 {
        let executions = self.execution_count.load(Ordering::Relaxed);
        let total_duration = self.total_duration.load(Ordering::Relaxed);
        
        if executions > 0 {
            total_duration / executions
        } else {
            0
        }
    }

    /// Get last execution timestamp
    pub fn last_execution(&self) -> u64 {
        self.last_execution.load(Ordering::Relaxed)
    }

    /// Get protection statistics
    pub fn protection_stats(&self) -> (u64, u64) {
        self.protection.get_stats()
    }

    /// Check if channel has protection enabled
    pub fn has_protection(&self) -> bool {
        !self.protection.is_no_protection()
    }

    /// Get channel configuration
    pub fn config(&self) -> &IO {
        &self.config
    }

    /// Reset channel statistics
    pub fn reset_stats(&self) {
        self.execution_count.store(0, Ordering::Relaxed);
        self.total_duration.store(0, Ordering::Relaxed);
        self.last_execution.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        self.protection.reset();
    }

    /// Check if channel is healthy (low error rate)
    pub fn is_healthy(&self) -> bool {
        let executions = self.execution_count.load(Ordering::Relaxed);
        let errors = self.error_count.load(Ordering::Relaxed);
        
        if executions == 0 {
            return true; // No executions, assume healthy
        }
        
        let error_rate = errors as f64 / executions as f64;
        error_rate < 0.05 // Less than 5% error rate
    }

    /// Get channel performance rating (0.0 to 1.0)
    pub fn performance_rating(&self) -> f64 {
        let executions = self.execution_count.load(Ordering::Relaxed);
        if executions == 0 {
            return 1.0; // No data, assume perfect
        }
        
        let errors = self.error_count.load(Ordering::Relaxed);
        let error_rate = errors as f64 / executions as f64;
        
        // Simple rating: 1.0 - error_rate, clamped to [0.0, 1.0]
        (1.0 - error_rate).max(0.0).min(1.0)
    }
}

impl std::fmt::Debug for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Channel")
            .field("id", &self.id)
            .field("priority", &self.priority)
            .field("fast_path", &self.fast_path)
            .field("execution_count", &self.execution_count.load(Ordering::Relaxed))
            .field("error_count", &self.error_count.load(Ordering::Relaxed))
            .finish()
    }
}

//=============================================================================
// CHANNEL BUILDER
//=============================================================================

/// Builder for creating channels with fluent API
pub struct ChannelBuilder {
    id: String,
    config: IO,
}

impl ChannelBuilder {
    /// Create a new channel builder
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            config: IO::new(id.clone()),
            id,
        }
    }

    /// Set priority
    pub fn priority(mut self, priority: Priority) -> Self {
        self.config.priority = priority;
        self
    }

    /// Add throttle protection
    pub fn throttle(mut self, ms: u64) -> Self {
        self.config.throttle = Some(ms);
        self.config.fast_path_eligible = false;
        self
    }

    /// Add debounce protection
    pub fn debounce(mut self, ms: u64) -> Self {
        self.config.debounce = Some(ms);
        self.config.fast_path_eligible = false;
        self
    }

    /// Enable change detection
    pub fn detect_changes(mut self) -> Self {
        self.config.detect_changes = true;
        self.config.fast_path_eligible = false;
        self
    }

    /// Enable logging
    pub fn with_logging(mut self) -> Self {
        self.config.log = true;
        self.config.fast_path_eligible = false;
        self
    }

    /// Build the channel with the given handler
    pub fn build<F>(self, handler: F) -> Channel
    where
        F: Fn(ActionPayload) -> std::pin::Pin<Box<dyn std::future::Future<Output = CyreResponse> + Send>> + Send + Sync + 'static,
    {
        let handler: AsyncHandler = Arc::new(handler);
        Channel::new(self.id, handler, self.config)
    }
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_fast_path_execution() {
        let channel = ChannelBuilder::new("test")
            .build(|payload| {
                Box::pin(async move {
                    success_response(payload)
                })
            });

        assert!(channel.is_fast_path());
        
        let response = channel.execute_fast_path(test_payload(42)).await;
        assert!(response.ok);
        assert_eq!(channel.execution_count(), 1);
    }

    #[tokio::test]
    async fn test_protection_execution() {
        let channel = ChannelBuilder::new("test")
            .throttle(100)
            .build(|payload| {
                Box::pin(async move {
                    success_response(payload)
                })
            });

        assert!(!channel.is_fast_path());
        assert!(channel.has_protection());
        
        // First call should succeed
        let response = channel.execute_with_protection(test_payload(1)).await;
        assert!(response.is_some());
        assert!(response.unwrap().ok);
        
        // Second call should be blocked
        let response = channel.execute_with_protection(test_payload(2)).await;
        assert!(response.is_none());
        
        let (blocks, _) = channel.protection_stats();
        assert_eq!(blocks, 1);
    }

    #[tokio::test]
    async fn test_error_tracking() {
        let channel = ChannelBuilder::new("test")
            .build(|_payload| {
                Box::pin(async move {
                    error_response("Test error")
                })
            });

        let response = channel.execute_fast_path(test_payload(1)).await;
        assert!(!response.ok);
        assert_eq!(channel.error_count(), 1);
        assert!(!channel.is_healthy());
    }

    #[test]
    fn test_channel_metrics() {
        let channel = ChannelBuilder::new("test")
            .priority(Priority::High)
            .build(|payload| {
                Box::pin(async move {
                    success_response(payload)
                })
            });

        let metrics = channel.get_metrics();
        assert_eq!(metrics["id"].as_str().unwrap(), "test");
        assert_eq!(metrics["priority"].as_str().unwrap(), "High");
        assert_eq!(metrics["fast_path"].as_bool().unwrap(), true);
        assert_eq!(metrics["executions"].as_u64().unwrap(), 0);
    }

    #[test]
    fn test_performance_rating() {
        let channel = ChannelBuilder::new("test")
            .build(|payload| {
                Box::pin(async move {
                    success_response(payload)
                })
            });

        // No executions should give perfect rating
        assert_eq!(channel.performance_rating(), 1.0);
        
        // Simulate some successful executions
        channel.execution_count.store(100, Ordering::Relaxed);
        channel.error_count.store(5, Ordering::Relaxed); // 5% error rate
        
        let rating = channel.performance_rating();
        assert_eq!(rating, 0.95); // 1.0 - 0.05
    }
}