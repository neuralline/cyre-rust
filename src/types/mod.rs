// src/types/mod.rs - FIXED: Remove duplicate definitions

use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{ Serialize, Deserialize };
use serde_json::Value as JsonValue;

//=============================================================================
// RE-EXPORT THE COMPLETE IO IMPLEMENTATION
//=============================================================================

mod io;
pub use io::*; // This brings in IO, RequiredType, RepeatType, AuthConfig, AuthMode, Priority

//=============================================================================
// CORE TYPE ALIASES (Only define ONCE - no duplicates)
//=============================================================================

/// Action identifier type
pub type ActionId = String;

/// Action payload type (JSON)
pub type ActionPayload = JsonValue;

/// Async handler function type
pub type AsyncHandler = Arc<
    dyn (Fn(ActionPayload) -> Pin<Box<dyn Future<Output = CyreResponse> + Send>>) + Send + Sync
>;

/// Small vectors for performance optimization
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
    pub priority: Priority, // Use the Priority from io.rs
    pub metrics: Option<JsonValue>,
}

impl Timer {
    pub fn new(id: String) -> Self {
        let now = std::time::SystemTime
            ::now()
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
            priority: Priority::Normal,
            metrics: None,
        }
    }
}

//=============================================================================
// RESPONSE TYPES (Only define ONCE)
//=============================================================================

/// Standard response format for all Cyre operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyreResponse {
    /// Whether the operation succeeded
    pub ok: bool,

    /// Response payload data
    pub payload: ActionPayload,

    /// Human-readable message
    pub message: String,

    /// Error information if ok is false
    pub error: Option<String>,

    /// Response timestamp
    pub timestamp: u64,

    /// Additional metadata
    pub metadata: Option<JsonValue>,
}

impl CyreResponse {
    /// Create a successful response
    pub fn success(payload: ActionPayload, message: impl Into<String>) -> Self {
        Self {
            ok: true,
            payload,
            message: message.into(),
            error: None,
            timestamp: crate::utils::current_timestamp(),
            metadata: None,
        }
    }

    /// Create an error response
    pub fn error(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            payload: JsonValue::Null,
            message: message.into(),
            error: Some(error.into()),
            timestamp: crate::utils::current_timestamp(),
            metadata: None,
        }
    }

    /// Add metadata to response
    pub fn with_metadata(mut self, metadata: JsonValue) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Legacy compatibility - redirect to success
    pub fn ok(payload: ActionPayload) -> Self {
        Self::success(payload, "Success")
    }
}

impl Default for CyreResponse {
    fn default() -> Self {
        Self {
            ok: true,
            payload: serde_json::Value::Null,
            message: String::new(),
            error: None,
            timestamp: std::time::SystemTime
                ::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            metadata: None,
        }
    }
}

//=============================================================================
// TALENT SYSTEM TYPES (FIXED - correct structure)
//=============================================================================

/// Result type for talent execution - FIXED STRUCTURE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalentResult {
    pub success: bool,
    pub payload: ActionPayload, // Main result field
    pub error: Option<String>,
    pub metadata: Option<JsonValue>,
}

impl TalentResult {
    pub fn success(payload: ActionPayload) -> Self {
        Self {
            success: true,
            payload,
            error: None,
            metadata: None,
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            payload: JsonValue::Null,
            error: Some(error.into()),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: JsonValue) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Result type for schema validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }
}

//=============================================================================
// FUNCTION TYPES FOR TALENTS
//=============================================================================

/// Function type for schema validation
pub type SchemaFunction = Arc<dyn (Fn(&ActionPayload) -> ValidationResult) + Send + Sync>;

/// Function type for conditional execution
pub type ConditionFunction = Arc<dyn (Fn(&ActionPayload) -> bool) + Send + Sync>;

/// Function type for data transformation
pub type TransformFunction = Arc<dyn (Fn(ActionPayload) -> ActionPayload) + Send + Sync>;

/// Function type for data selection/filtering
pub type SelectorFunction = Arc<dyn (Fn(&ActionPayload) -> ActionPayload) + Send + Sync>;

/// Generic talent function type
pub type TalentFunction = Arc<dyn (Fn(&IO, ActionPayload) -> TalentResult) + Send + Sync>;

//=============================================================================
// ERROR TYPES (Only define ONCE)
//=============================================================================

/// Comprehensive error types for Cyre operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CyreError {
    /// Action not found
    ActionNotFound {
        action_id: String,
    },

    /// Handler not registered
    HandlerNotFound {
        action_id: String,
    },

    /// Protection violation (throttled, blocked, etc.)
    ProtectionViolation {
        reason: String,
        action_id: String,
    },

    /// Pipeline compilation error
    PipelineError {
        error: String,
        action_id: String,
    },

    /// Validation error
    ValidationError {
        errors: Vec<String>,
        action_id: String,
    },

    /// Authentication error
    AuthenticationError {
        reason: String,
    },

    /// TimeKeeper integration error
    TimeKeeperError {
        error: String,
    },

    /// Talent system error
    TalentError {
        talent_name: String,
        error: String,
    },

    /// Configuration error
    ConfigurationError {
        field: String,
        reason: String,
    },

    /// System error
    SystemError {
        error: String,
    },
}

impl std::fmt::Display for CyreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CyreError::ActionNotFound { action_id } => {
                write!(f, "Action '{}' not found", action_id)
            }
            CyreError::HandlerNotFound { action_id } => {
                write!(f, "Handler for action '{}' not registered", action_id)
            }
            CyreError::ProtectionViolation { reason, action_id } => {
                write!(f, "Protection violation for '{}': {}", action_id, reason)
            }
            CyreError::PipelineError { error, action_id } => {
                write!(f, "Pipeline error for '{}': {}", action_id, error)
            }
            CyreError::ValidationError { errors, action_id } => {
                write!(f, "Validation failed for '{}': {}", action_id, errors.join(", "))
            }
            CyreError::AuthenticationError { reason } => {
                write!(f, "Authentication error: {}", reason)
            }
            CyreError::TimeKeeperError { error } => { write!(f, "TimeKeeper error: {}", error) }
            CyreError::TalentError { talent_name, error } => {
                write!(f, "Talent '{}' error: {}", talent_name, error)
            }
            CyreError::ConfigurationError { field, reason } => {
                write!(f, "Configuration error in '{}': {}", field, reason)
            }
            CyreError::SystemError { error } => { write!(f, "System error: {}", error) }
        }
    }
}

impl std::error::Error for CyreError {}

// Convert CyreError to String (fixes lib.rs error)
impl From<CyreError> for String {
    fn from(error: CyreError) -> Self {
        error.to_string()
    }
}

// Convert CyreError to CyreResponse for easy error handling
impl From<CyreError> for CyreResponse {
    fn from(error: CyreError) -> Self {
        CyreResponse::error(error.to_string(), "Operation failed")
    }
}

//=============================================================================
// CONVENIENT TYPE RESULT ALIASES
//=============================================================================

/// Standard result type for Cyre operations
pub type CyreResult<T> = Result<T, CyreError>;

/// Result type for async operations
pub type AsyncCyreResult<T> = Pin<Box<dyn Future<Output = CyreResult<T>> + Send>>;

//=============================================================================
// OTHER SUPPORTING TYPES
//=============================================================================

/// Timeline entry for tracking action history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub id: String,
    pub action_id: String,
    pub payload: ActionPayload,
    pub response: CyreResponse,
    pub timestamp: u64,
    pub execution_time_ms: u64,
}

/// Branch store entry for hierarchical organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchStore {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub created_at: u64,
}

/// Pipeline execution context
#[derive(Debug, Clone)]
pub struct PipelineContext {
    pub action_id: ActionId,
    pub start_time: u64,
    pub execution_count: u64,
    pub metadata: HashMap<String, JsonValue>,
}

impl PipelineContext {
    pub fn new(action_id: ActionId) -> Self {
        Self {
            action_id,
            start_time: crate::utils::current_timestamp(),
            execution_count: 0,
            metadata: HashMap::new(),
        }
    }
}

/// Subscriber interface for action handlers
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
            created_at: crate::utils::current_timestamp(),
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

//=============================================================================
// INTEGRATION HELPERS
//=============================================================================

/// Helper trait for converting various types to ActionPayload
pub trait IntoActionPayload {
    fn into_payload(self) -> ActionPayload;
}

impl IntoActionPayload for ActionPayload {
    fn into_payload(self) -> ActionPayload {
        self
    }
}

impl IntoActionPayload for &str {
    fn into_payload(self) -> ActionPayload {
        JsonValue::String(self.to_string())
    }
}

impl IntoActionPayload for String {
    fn into_payload(self) -> ActionPayload {
        JsonValue::String(self)
    }
}

impl IntoActionPayload for i32 {
    fn into_payload(self) -> ActionPayload {
        JsonValue::Number(self.into())
    }
}

impl IntoActionPayload for u64 {
    fn into_payload(self) -> ActionPayload {
        JsonValue::Number(self.into())
    }
}

impl IntoActionPayload for bool {
    fn into_payload(self) -> ActionPayload {
        JsonValue::Bool(self)
    }
}

//=============================================================================
// CONSTANTS
//=============================================================================

/// Default timeout for operations (30 seconds)
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// Current Cyre version
pub const CYRE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information - FIXED to avoid missing env var
pub const BUILD_INFO: &str = concat!(
    "Cyre Rust v",
    env!("CARGO_PKG_VERSION"),
    " - Production Build"
);
