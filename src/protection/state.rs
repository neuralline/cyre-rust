// src/protection/state.rs
// Protection mechanism implementation

use std::sync::atomic::{AtomicU64, Ordering};
use crate::types::{ActionPayload, likely};
use crate::utils::{current_timestamp, hash_payload};

//=============================================================================
// PROTECTION TYPES
//=============================================================================

/// Protection mechanism types for compile-time optimization
#[derive(Debug, Clone, Copy)]
pub enum ProtectionType {
    None,           // Zero overhead
    ThrottleOnly,   // Single check
    DebounceOnly,   // Single check  
    Combined,       // Full protection
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
    block_count: AtomicU64,
    skip_count: AtomicU64,
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
            block_count: AtomicU64::new(0),
            skip_count: AtomicU64::new(0),
        }
    }

    /// Check if execution should proceed (hot path optimized)
    #[inline(always)]
    pub fn should_execute(&self, payload: &ActionPayload) -> bool {
        let now = current_timestamp();
        
        match self.protection_type {
            ProtectionType::None => true, // Hot path - zero overhead
            ProtectionType::ThrottleOnly => {
                let last = self.last_execution.load(Ordering::Relaxed);
                if likely(now - last >= self.throttle_ms) {
                    self.last_execution.store(now, Ordering::Relaxed);
                    true
                } else {
                    self.block_count.fetch_add(1, Ordering::Relaxed);
                    false
                }
            },
            ProtectionType::DebounceOnly => {
                // Simplified debounce for performance
                self.last_execution.store(now, Ordering::Relaxed);
                true
            },
            ProtectionType::Combined => {
                self.full_protection_check(payload, now)
            }
        }
    }

    /// Full protection check with all mechanisms
    fn full_protection_check(&self, payload: &ActionPayload, now: u64) -> bool {
        // Throttle check
        if self.throttle_ms > 0 {
            let last = self.last_execution.load(Ordering::Relaxed);
            if now - last < self.throttle_ms {
                self.block_count.fetch_add(1, Ordering::Relaxed);
                return false;
            }
        }

        // Change detection
        if self.detect_changes {
            let payload_hash = hash_payload(payload);
            let last_hash = self.last_payload_hash.load(Ordering::Relaxed);
            if payload_hash == last_hash {
                self.skip_count.fetch_add(1, Ordering::Relaxed);
                return false;
            }
            self.last_payload_hash.store(payload_hash, Ordering::Relaxed);
        }

        self.last_execution.store(now, Ordering::Relaxed);
        true
    }

    /// Get protection statistics
    pub fn get_stats(&self) -> (u64, u64) {
        (
            self.block_count.load(Ordering::Relaxed),
            self.skip_count.load(Ordering::Relaxed)
        )
    }

    /// Get protection type
    pub fn protection_type(&self) -> ProtectionType {
        self.protection_type
    }

    /// Check if this is a no-protection state (for fast path optimization)
    #[inline(always)]
    pub fn is_no_protection(&self) -> bool {
        matches!(self.protection_type, ProtectionType::None)
    }

    /// Get last execution timestamp
    pub fn last_execution(&self) -> u64 {
        self.last_execution.load(Ordering::Relaxed)
    }

    /// Get throttle duration in milliseconds
    pub fn throttle_ms(&self) -> u64 {
        self.throttle_ms
    }

    /// Get debounce duration in milliseconds
    pub fn debounce_ms(&self) -> u64 {
        self.debounce_ms
    }

    /// Check if change detection is enabled
    pub fn has_change_detection(&self) -> bool {
        self.detect_changes
    }

    /// Reset protection state
    pub fn reset(&self) {
        self.last_execution.store(0, Ordering::Relaxed);
        self.last_payload_hash.store(0, Ordering::Relaxed);
        self.block_count.store(0, Ordering::Relaxed);
        self.skip_count.store(0, Ordering::Relaxed);
    }
}

//=============================================================================
// PROTECTION BUILDER
//=============================================================================

/// Builder for creating protection configurations
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
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_no_protection() {
        let protection = ProtectionState::new(None, None, false);
        let payload = json!({"test": true});
        
        assert!(protection.should_execute(&payload));
        assert!(protection.should_execute(&payload));
        assert!(protection.is_no_protection());
    }

    #[test]
    fn test_throttle_only() {
        let protection = ProtectionState::new(Some(100), None, false);
        let payload = json!({"test": true});
        
        assert!(protection.should_execute(&payload));
        assert!(!protection.should_execute(&payload)); // Should be blocked
        
        // Simulate time passing
        std::thread::sleep(std::time::Duration::from_millis(101));
        assert!(protection.should_execute(&payload));
    }

    #[test]
    fn test_change_detection() {
        let protection = ProtectionState::new(None, None, true);
        let payload1 = json!({"test": 1});
        let payload2 = json!({"test": 2});
        let payload3 = json!({"test": 1}); // Same as payload1
        
        assert!(protection.should_execute(&payload1));
        assert!(protection.should_execute(&payload2));
        assert!(!protection.should_execute(&payload3)); // Should be skipped
    }

    #[test]
    fn test_protection_stats() {
        let protection = ProtectionState::new(Some(1000), None, true);
        let payload = json!({"test": true});
        
        // First call should succeed
        assert!(protection.should_execute(&payload));
        
        // Second call should be blocked by throttle
        assert!(!protection.should_execute(&payload));
        
        // Third call should be skipped by change detection
        std::thread::sleep(std::time::Duration::from_millis(1001));
        assert!(!protection.should_execute(&payload));
        
        let (blocks, skips) = protection.get_stats();
        assert_eq!(blocks, 1);
        assert_eq!(skips, 1);
    }

    #[test]
    fn test_protection_builder() {
        let protection = ProtectionBuilder::new()
            .throttle(100)
            .detect_changes()
            .build();
        
        assert_eq!(protection.throttle_ms(), 100);
        assert!(protection.has_change_detection());
        assert!(!protection.is_no_protection());
    }
}