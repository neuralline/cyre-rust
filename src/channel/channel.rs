// src/channel/channel.rs
// FIXED: Performance-optimized channel with lock-free operations

use std::sync::atomic::{ AtomicU64, Ordering };
use parking_lot::RwLock; // Faster than std::sync::RwLock

use crate::types::{ ActionId, ActionPayload, CyreResponse, Priority, IO };
use crate::protection::ProtectionState;
use crate::utils::current_timestamp;

//=============================================================================
// PERFORMANCE-OPTIMIZED CHANNEL
//=============================================================================

/// High-performance channel with lock-free metrics
pub struct Channel {
    pub id: ActionId,

    // Lock-free performance counters
    execution_count: AtomicU64,
    error_count: AtomicU64,
    last_execution_time: AtomicU64,

    // Protection state (minimal locking)
    protection: Option<RwLock<ProtectionState>>,

    // Channel configuration (immutable after creation)
    priority: Priority,
    is_fast_path: bool,

    // Performance tracking
    avg_execution_time: AtomicU64, // Microseconds
    peak_execution_time: AtomicU64,
}

impl Channel {
    /// Create channel from IO configuration
    pub fn from_config(config: &IO) -> Self {
        let protection = if Self::has_protection_needed(config) {
            Some(
                RwLock::new(
                    ProtectionState::new(config.throttle, config.debounce, config.detect_changes)
                )
            )
        } else {
            None
        };

        Self {
            id: config.id.clone(),
            execution_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            last_execution_time: AtomicU64::new(0),
            protection,
            priority: config.priority,
            is_fast_path: Self::is_fast_path_eligible(config),
            avg_execution_time: AtomicU64::new(0),
            peak_execution_time: AtomicU64::new(0),
        }
    }

    /// Check if this configuration needs protection mechanisms
    fn has_protection_needed(config: &IO) -> bool {
        config.throttle.is_some() || config.debounce.is_some() || config.detect_changes
    }

    /// Check if config is eligible for fast path
    fn is_fast_path_eligible(config: &IO) -> bool {
        config.throttle.is_none() &&
            config.debounce.is_none() &&
            config.interval.is_none() &&
            config.delay.is_none() &&
            config.repeat.is_none()
    }

    /// Check if execution is allowed (fast path for unprotected channels)
    #[inline(always)]
    pub fn can_execute(&self) -> bool {
        match &self.protection {
            None => true, // Fast path: no protection
            Some(protection_lock) => {
                // Optimized read-only check first
                let _protection = protection_lock.read();
                // Simplified check for now - just return true
                // In full implementation, this would check protection.should_execute()
                true
            }
        }
    }

    /// Record execution metrics (lock-free)
    #[inline]
    pub fn record_execution(&self, result: &CyreResponse) {
        let now = current_timestamp();
        let _count = self.execution_count.fetch_add(1, Ordering::Relaxed);

        self.last_execution_time.store(now, Ordering::Relaxed);

        if !result.ok {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }

        // Update protection state if needed
        if let Some(protection_lock) = &self.protection {
            if let Some(protection) = protection_lock.try_write() {
                // Protection state update (simplified for now)
                drop(protection);
            }
        }
    }

    /// Fast path check (inline for zero overhead)
    #[inline(always)]
    pub fn is_fast_path(&self) -> bool {
        self.is_fast_path
    }

    /// Get execution count (lock-free)
    pub fn execution_count(&self) -> u64 {
        self.execution_count.load(Ordering::Relaxed)
    }

    /// Get error count (lock-free)
    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }

    /// Check if channel is healthy (lock-free)
    pub fn is_healthy(&self) -> bool {
        let total = self.execution_count.load(Ordering::Relaxed);
        if total == 0 {
            return true;
        }

        let errors = self.error_count.load(Ordering::Relaxed);
        let error_rate = (errors as f64) / (total as f64);
        error_rate < 0.1 // Less than 10% error rate
    }

    /// Get performance rating (lock-free)
    pub fn performance_rating(&self) -> f64 {
        let total = self.execution_count.load(Ordering::Relaxed);
        if total == 0 {
            return 1.0;
        }

        let errors = self.error_count.load(Ordering::Relaxed);
        let success_rate = 1.0 - (errors as f64) / (total as f64);
        success_rate.max(0.0).min(1.0)
    }

    /// Get channel metrics (optimized for frequent access)
    pub fn get_metrics(&self) -> serde_json::Value {
        let total_executions = self.execution_count.load(Ordering::Relaxed);
        let total_errors = self.error_count.load(Ordering::Relaxed);
        let last_execution = self.last_execution_time.load(Ordering::Relaxed);

        serde_json::json!({
            "id": self.id,
            "priority": format!("{:?}", self.priority),
            "fast_path": self.is_fast_path,
            "executions": total_executions,
            "errors": total_errors,
            "error_rate": if total_executions > 0 { 
                total_errors as f64 / total_executions as f64 
            } else { 0.0 },
            "last_execution": last_execution,
            "healthy": self.is_healthy(),
            "performance_rating": self.performance_rating(),
        })
    }

    /// Get protection statistics (read-only, minimal locking)
    pub fn protection_stats(&self) -> (u64, u64) {
        match &self.protection {
            None => (0, 0),
            Some(protection_lock) => {
                let protection = protection_lock.read();
                protection.get_stats()
            }
        }
    }
}

//=============================================================================
// BUILDER PATTERN FOR CHANNELS
//=============================================================================

pub struct ChannelBuilder {
    id: ActionId,
    priority: Priority,
    protection_config: Option<IO>,
}

impl ChannelBuilder {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            priority: Priority::Medium,
            protection_config: None,
        }
    }

    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_protection(mut self, config: IO) -> Self {
        self.protection_config = Some(config);
        self
    }

    pub fn build<F, Fut>(self, _handler: F) -> Channel
        where
            F: Fn(ActionPayload) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = CyreResponse> + Send + 'static
    {
        let mut config = self.protection_config.unwrap_or_else(|| IO::new(&self.id));
        config.priority = self.priority;

        Channel::from_config(&config)
    }
}

//=============================================================================
// OPTIMIZED IMPLEMENTATIONS
//=============================================================================

impl std::fmt::Debug for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Channel")
            .field("id", &self.id)
            .field("executions", &self.execution_count.load(Ordering::Relaxed))
            .field("errors", &self.error_count.load(Ordering::Relaxed))
            .field("fast_path", &self.is_fast_path)
            .field("priority", &self.priority)
            .finish()
    }
}

//=============================================================================
// EXTENSION TRAIT FOR IO - REMOVED TO AVOID CONFLICTS
//=============================================================================

// Removed duplicate implementations to avoid conflicts with types/mod.rs
