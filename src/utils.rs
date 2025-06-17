// src/utils.rs
// FIXED: Complete utility functions

use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hash, Hasher };
use crate::types::ActionPayload;

/// Get current timestamp in milliseconds
pub fn current_timestamp() -> u64 {
    std::time::SystemTime
        ::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Hash a payload for change detection
pub fn hash_payload(payload: &ActionPayload) -> u64 {
    let mut hasher = DefaultHasher::new();
    // Convert payload to string for hashing
    payload.to_string().hash(&mut hasher);
    hasher.finish()
}

/// Branch prediction hint for likely conditions
#[inline(always)]
pub fn likely(condition: bool) -> bool {
    // For now, just return the condition
    // In the future, could use compiler intrinsics for optimization
    condition
}

/// Branch prediction hint for unlikely conditions
#[inline(always)]
pub fn unlikely(condition: bool) -> bool {
    // For now, just return the condition
    // In the future, could use compiler intrinsics for optimization
    condition
}

/// Format duration in a human-readable way
pub fn format_duration(duration_ms: u64) -> String {
    if duration_ms < 1000 {
        format!("{}ms", duration_ms)
    } else if duration_ms < 60_000 {
        format!("{:.1}s", (duration_ms as f64) / 1000.0)
    } else {
        format!("{:.1}m", (duration_ms as f64) / 60_000.0)
    }
}

/// Generate a unique ID
pub fn generate_id() -> String {
    format!("cyre_{}", current_timestamp())
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::types::CyreResponse;
    use serde_json::{ json, Value };

    /// Create a test payload
    pub fn test_payload(value: i32) -> ActionPayload {
        json!({"test": true, "value": value})
    }

    /// Create a success response for testing
    pub fn success_response(payload: ActionPayload) -> CyreResponse {
        CyreResponse {
            ok: true,
            payload,
            message: "Success".to_string(),
            error: None,
            timestamp: current_timestamp(),
            metadata: None,
        }
    }

    /// Create an error response for testing
    pub fn error_response(message: &str) -> CyreResponse {
        CyreResponse {
            ok: false,
            payload: Value::Null,
            message: message.to_string(),
            error: Some(message.to_string()),
            timestamp: current_timestamp(),
            metadata: None,
        }
    }
}
