// src/types/core.rs
// Core type definitions

use std::{future::Future, pin::Pin, sync::Arc};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

//=============================================================================
// FUNDAMENTAL TYPES
//=============================================================================

/// Unique identifier for actions and channels
pub type ActionId = String;

/// JSON payload type for action data
pub type ActionPayload = JsonValue;

/// Async handler type for processing actions
pub type AsyncHandler = Arc<dyn Fn(ActionPayload) -> Pin<Box<dyn Future<Output = CyreResponse> + Send>> + Send + Sync>;

/// Performance optimized HashMap (can be upgraded later)
pub type FastMap<K, V> = HashMap<K, V>;

/// Stack-optimized string collections (can be upgraded to SmallVec later)
pub type SmallTags = Vec<String>;
pub type SmallMiddleware = Vec<String>;

//=============================================================================
// RESPONSE TYPES
//=============================================================================

/// Standard response type for all Cyre operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyreResponse {
    pub ok: bool,
    pub payload: ActionPayload,
    pub message: String,
    pub error: Option<String>,
    pub timestamp: u64,
    pub metadata: Option<JsonValue>,
}

impl Default for CyreResponse {
    fn default() -> Self {
        Self {
            ok: true,
            payload: serde_json::Value::Null,
            message: String::new(),
            error: None,
            timestamp: crate::utils::current_timestamp(),
            metadata: None,
        }
    }
}

/// Result type for talent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalentResult {
    pub success: bool,
    pub payload: ActionPayload,
    pub error: Option<String>,
    pub metadata: Option<JsonValue>,
}

/// Result type for schema validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

//=============================================================================
// FUNCTION TYPES FOR TALENTS
//=============================================================================

/// Function type for schema validation
pub type SchemaFunction = fn(ActionPayload) -> ValidationResult;

/// Function type for conditional execution
pub type ConditionFunction = fn(ActionPayload) -> bool;

/// Function type for data transformation
pub type TransformFunction = fn(ActionPayload) -> ActionPayload;

/// Function type for data selection/filtering
pub type SelectorFunction = fn(ActionPayload) -> ActionPayload;

/// Generic talent function type
pub type TalentFunction = fn(&crate::types::IO, ActionPayload) -> TalentResult;

//=============================================================================
// PERFORMANCE OPTIMIZATION HELPERS
//=============================================================================

/// Simple likely hint replacement (until stable)
#[inline(always)]
pub fn likely(b: bool) -> bool { b }

/// Simple unlikely hint replacement (until stable)
#[inline(always)]
pub fn unlikely(b: bool) -> bool { b }