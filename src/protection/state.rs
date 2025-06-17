// src/protection/state.rs
// Protection mechanism implementation - FIXED: Remove unnecessary parentheses

use std::sync::atomic::{ AtomicU64, Ordering };
use crate::utils::current_timestamp;

//=============================================================================
// PROTECTION TYPES
//=============================================================================

/// Protection mechanism types for compile-time optimization
#[derive(Debug, Clone, Copy)]
pub enum ProtectionType {
    None, // Zero overhead
    ThrottleOnly, // Single check
    DebounceOnly, // Single check
    Combined, // Full protection
}

/// Builder for creating protection configurations
#[derive(Debug)]
pub struct ProtectionBuilder {
    throttle: Option<u64>,
    debounce: Option<u64>,
    detect_changes: bool,
}

impl ProtectionBuilder {
    /// Create new protection builder
    pub fn new() -> Self {
        Self {
            throttle: None,
            debounce: None,
            detect_changes: false,
        }
    }

    /// Set throttle duration
    pub fn throttle(mut self, ms: u64) -> Self {
        self.throttle = Some(ms);
        self
    }

    /// Set debounce duration
    pub fn debounce(mut self, ms: u64) -> Self {
        self.debounce = Some(ms);
        self
    }

    /// Enable change detection
    pub fn detect_changes(mut self) -> Self {
        self.detect_changes = true;
        self
    }

    /// Build protection state
    pub fn build(self) -> ProtectionState {
        ProtectionState::new(self.throttle, self.debounce, self.detect_changes)
    }
}

impl Default for ProtectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

//=============================================================================
// PROTECTION STATE
//=============================================================================

/// Thread-safe protection state for channels
#[derive(Debug)]
pub struct ProtectionState {
    protection_type: ProtectionType,
    last_execution: AtomicU64,
    last_payload_hash: AtomicU64,
    throttle_ms: u64,
    debounce_ms: u64,
    detect_changes: bool,

    // Statistics
    total_requests: AtomicU64,
    blocked_requests: AtomicU64,
    successful_executions: AtomicU64,
}

impl ProtectionState {
    /// Create new protection state with specified configuration
    pub fn new(throttle: Option<u64>, debounce: Option<u64>, detect_changes: bool) -> Self {
        let protection_type = match (throttle, debounce, detect_changes) {
            (None, None, false) => ProtectionType::None,
            (Some(_), None, false) => ProtectionType::ThrottleOnly,
            (None, Some(_), false) => ProtectionType::DebounceOnly,
            _ => ProtectionType::Combined,
        };

        Self {
            protection_type,
            last_execution: AtomicU64::new(0),
            last_payload_hash: AtomicU64::new(0),
            throttle_ms: throttle.unwrap_or(0),
            debounce_ms: debounce.unwrap_or(0),
            detect_changes,
            total_requests: AtomicU64::new(0),
            blocked_requests: AtomicU64::new(0),
            successful_executions: AtomicU64::new(0),
        }
    }

    /// Check if execution can proceed
    pub fn check_can_execute(&self) -> bool {
        let now = current_timestamp();
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        // Check throttling - FIXED: remove unnecessary parentheses
        if self.throttle_ms > 0 {
            let last_exec = self.last_execution.load(Ordering::Relaxed);
            if now.saturating_sub(last_exec) < self.throttle_ms {
                self.blocked_requests.fetch_add(1, Ordering::Relaxed);
                return false;
            }
        }

        // Check debouncing - FIXED: remove unnecessary parentheses
        if self.debounce_ms > 0 {
            let debounce_time = self.last_execution.load(Ordering::Relaxed);
            if now.saturating_sub(debounce_time) < self.debounce_ms {
                self.blocked_requests.fetch_add(1, Ordering::Relaxed);
                return false;
            }
        }

        true
    }

    /// Record execution result
    pub fn record_execution(&self, _result: &crate::types::CyreResponse) {
        let now = current_timestamp();
        self.last_execution.store(now, Ordering::Relaxed);
        self.successful_executions.fetch_add(1, Ordering::Relaxed);
    }

    /// Get blocked request count
    pub fn get_blocked_count(&self) -> u64 {
        self.blocked_requests.load(Ordering::Relaxed)
    }

    /// Get total request count
    pub fn get_total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }

    /// Check if no protection is enabled
    pub fn is_no_protection(&self) -> bool {
        matches!(self.protection_type, ProtectionType::None)
    }

    /// Reset statistics
    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.blocked_requests.store(0, Ordering::Relaxed);
        self.successful_executions.store(0, Ordering::Relaxed);
        self.last_execution.store(0, Ordering::Relaxed);
        self.last_payload_hash.store(0, Ordering::Relaxed);
    }

    /// Get protection statistics
    pub fn get_stats(&self) -> (u64, u64) {
        (self.blocked_requests.load(Ordering::Relaxed), self.total_requests.load(Ordering::Relaxed))
    }

    /// Get protection type
    pub fn protection_type(&self) -> ProtectionType {
        self.protection_type
    }
}
