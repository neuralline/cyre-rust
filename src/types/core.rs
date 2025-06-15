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
// TIMER TYPE (for TimeKeeper integration)
//=============================================================================

/// Timer interface for timeline store
#[derive(Debug, Clone)]
pub struct Timer {
    pub id: String,
    pub start_time: u64,
    pub duration: u64,
    pub original_duration: u64,
    pub repeat: Option<i32>, // None = once, Some(-1) = forever, Some(n) = n times
    pub execution_count: u64,
    pub last_execution_time: u64,
    pub next_execution_time: u64,
    pub is_in_recuperation: bool,
    pub status: String,
    pub is_active: bool,
    pub delay: Option<u64>,
    pub interval: Option<u64>,
    pub has_executed_once: bool,
    pub priority: super::Priority,
    pub metrics: Option<JsonValue>,
}

impl Timer {
    pub fn new(id: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            id,
            start_time: now,
            duration: 0,
            original_duration: 0,
            repeat: None,
            execution_count: 0,
            last_execution_time: 0,
            next_execution_time: now,
            is_in_recuperation: false,
            status: "active".to_string(),
            is_active: true,
            delay: None,
            interval: None,
            has_executed_once: false,
            priority: super::Priority::Medium,
            metrics: None,
        }
    }
}

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
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
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
// SUBSCRIBER TYPES (for state management)
//=============================================================================

/// Subscriber interface for action handlers - FIXED: no Debug derive
#[derive(Clone)]
pub struct ISubscriber {
    pub id: String,
    pub handler: AsyncHandler,
    pub active: bool,
    pub created_at: u64,
}

impl ISubscriber {
    pub fn new(id: String, handler: AsyncHandler) -> Self {
        Self {
            id,
            handler,
            active: true,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
}

// Manual Debug implementation for ISubscriber (handler can't be debugged)
impl std::fmt::Debug for ISubscriber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ISubscriber")
            .field("id", &self.id)
            .field("handler", &"<async_handler>")
            .field("active", &self.active)
            .field("created_at", &self.created_at)
            .finish()
    }
}

/// Branch store entry
#[derive(Debug, Clone)]
pub struct BranchStore {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub created_at: u64,
}

/// State action metrics
#[derive(Debug, Clone)]
pub struct StateActionMetrics {
    pub last_execution_time: u64,
    pub execution_count: u64,
    pub errors: Vec<String>,
}

/// State key type
pub type StateKey = String;

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