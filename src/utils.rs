// src/utils.rs
// Utility functions

use std::time::{SystemTime, UNIX_EPOCH};

//=============================================================================
// TIME UTILITIES
//=============================================================================

/// Get current timestamp in milliseconds since Unix epoch
#[inline(always)]
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Get current timestamp in microseconds since Unix epoch
#[inline(always)]
pub fn current_timestamp_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

/// Get current timestamp in nanoseconds since Unix epoch
#[inline(always)]
pub fn current_timestamp_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

//=============================================================================
// PERFORMANCE UTILITIES
//=============================================================================

/// Calculate operations per second
pub fn calculate_ops_per_sec(operations: u64, duration_ms: u64) -> f64 {
    if duration_ms == 0 {
        return 0.0;
    }
    (operations as f64 * 1000.0) / duration_ms as f64
}

/// Calculate average from a slice of numbers
pub fn calculate_average(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Calculate percentile from a sorted slice
pub fn calculate_percentile(sorted_values: &[f64], percentile: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    
    let index = (sorted_values.len() as f64 * percentile / 100.0) as usize;
    let index = index.min(sorted_values.len() - 1);
    sorted_values[index]
}

//=============================================================================
// HASH UTILITIES
//=============================================================================

/// Simple hash function for payload comparison
pub fn simple_hash(data: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

/// Hash JSON payload for change detection
pub fn hash_payload(payload: &serde_json::Value) -> u64 {
    simple_hash(&payload.to_string())
}

//=============================================================================
// FORMATTING UTILITIES
//=============================================================================

/// Format duration in milliseconds to human readable string
pub fn format_duration_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else if ms < 3_600_000 {
        format!("{:.1}m", ms as f64 / 60_000.0)
    } else {
        format!("{:.1}h", ms as f64 / 3_600_000.0)
    }
}

/// Format bytes to human readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Format operations per second with appropriate units
pub fn format_ops_per_sec(ops: f64) -> String {
    if ops >= 1_000_000.0 {
        format!("{:.1}M ops/sec", ops / 1_000_000.0)
    } else if ops >= 1_000.0 {
        format!("{:.1}K ops/sec", ops / 1_000.0)
    } else {
        format!("{:.0} ops/sec", ops)
    }
}

//=============================================================================
// VALIDATION UTILITIES
//=============================================================================

/// Validate action ID format
pub fn validate_action_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("Action ID cannot be empty".to_string());
    }
    
    if id.len() > 255 {
        return Err("Action ID cannot be longer than 255 characters".to_string());
    }
    
    // Check for valid characters (alphanumeric, dash, underscore, dot)
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err("Action ID can only contain alphanumeric characters, dashes, underscores, and dots".to_string());
    }
    
    Ok(())
}

/// Validate timing values
pub fn validate_timing(interval: Option<u64>, delay: Option<u64>) -> Result<(), String> {
    if let Some(interval) = interval {
        if interval == 0 {
            return Err("Interval cannot be zero".to_string());
        }
        if interval > 86_400_000 { // 24 hours
            return Err("Interval cannot be longer than 24 hours".to_string());
        }
    }
    
    if let Some(delay) = delay {
        if delay > 86_400_000 { // 24 hours
            return Err("Delay cannot be longer than 24 hours".to_string());
        }
    }
    
    Ok(())
}

//=============================================================================
// TESTING UTILITIES
//=============================================================================

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::types::{ActionPayload, CyreResponse};
    
    /// Create a test payload
    pub fn test_payload(value: i32) -> ActionPayload {
        serde_json::json!({
            "test": true,
            "value": value,
            "timestamp": current_timestamp()
        })
    }
    
    /// Create a success response
    pub fn success_response(payload: ActionPayload) -> CyreResponse {
        CyreResponse {
            ok: true,
            payload,
            message: "Test success".to_string(),
            error: None,
            timestamp: current_timestamp(),
            metadata: None,
        }
    }
    
    /// Create an error response
    pub fn error_response(error: impl Into<String>) -> CyreResponse {
        CyreResponse {
            ok: false,
            payload: serde_json::Value::Null,
            message: "Test error".to_string(),
            error: Some(error.into()),
            timestamp: current_timestamp(),
            metadata: None,
        }
    }
}