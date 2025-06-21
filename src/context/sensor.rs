//! # Sensor Module - src/sensor.rs
//!
//! Ultra-simplified logging with message-first API.
//! Priority order: message → location → event_type → action_id → metadata
//! Only message is required, everything else is optional.

use serde_json::Value as JsonValue;
use std::time::{ SystemTime, UNIX_EPOCH };

//=============================================================================
// LOG LEVELS & COLORS (keeping existing)
//=============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    SUCCESS,
    CRITICAL,
    SYS,
}

pub struct Colors;
impl Colors {
    pub const RESET: &'static str = "\x1b[0m";
    pub const BOLD: &'static str = "\x1b[1m";
    pub const DIM: &'static str = "\x1b[2m";
    pub const CYAN: &'static str = "\x1b[36m";
    pub const YELLOW_BRIGHT: &'static str = "\x1b[93m";
    pub const RED_BRIGHT: &'static str = "\x1b[91m";
    pub const GREEN_BRIGHT: &'static str = "\x1b[92m";
    pub const WHITE_BRIGHT: &'static str = "\x1b[97m";
    pub const WHITE: &'static str = "\x1b[37m";
    pub const BG_RED: &'static str = "\x1b[41m";
    pub const BG_MAGENTA: &'static str = "\x1b[45m";
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::DEBUG => write!(f, "DEBUG"),
            LogLevel::INFO => write!(f, "INFO"),
            LogLevel::WARN => write!(f, "WARN"),
            LogLevel::ERROR => write!(f, "ERROR"),
            LogLevel::SUCCESS => write!(f, "SUCCESS"),
            LogLevel::CRITICAL => write!(f, "CRITICAL"),
            LogLevel::SYS => write!(f, "SYS"),
        }
    }
}

impl LogLevel {
    pub fn color_style(&self) -> &'static str {
        match self {
            LogLevel::DEBUG => Colors::DIM,
            LogLevel::INFO => Colors::CYAN,
            LogLevel::WARN => Colors::YELLOW_BRIGHT,
            LogLevel::ERROR => Colors::RED_BRIGHT,
            LogLevel::SUCCESS => Colors::GREEN_BRIGHT,
            LogLevel::CRITICAL => Colors::BG_RED,
            LogLevel::SYS => Colors::BG_MAGENTA,
        }
    }

    pub fn additional_style(&self) -> &'static str {
        match self {
            LogLevel::DEBUG => Colors::CYAN,
            LogLevel::INFO => Colors::BOLD,
            LogLevel::WARN => Colors::BOLD,
            LogLevel::ERROR => Colors::BOLD,
            LogLevel::SUCCESS => Colors::BOLD,
            LogLevel::CRITICAL => Colors::WHITE_BRIGHT,
            LogLevel::SYS => Colors::WHITE,
        }
    }

    pub fn logs_by_default(&self) -> bool {
        matches!(self, LogLevel::ERROR | LogLevel::CRITICAL)
    }
}

//=============================================================================
// SENSOR BUILDER PATTERN FOR ULTIMATE FLEXIBILITY
//=============================================================================

pub struct SensorBuilder {
    message: String,
    location: Option<String>,
    event_type: Option<String>,
    action_id: Option<String>,
    metadata: Option<JsonValue>,
    force_log: bool,
}

impl SensorBuilder {
    pub fn new(message: impl AsRef<str>) -> Self {
        Self {
            message: message.as_ref().to_string(),
            location: None,
            event_type: None,
            action_id: None,
            metadata: None,
            force_log: false,
        }
    }

    pub fn location(mut self, location: impl AsRef<str>) -> Self {
        self.location = Some(location.as_ref().to_string());
        self
    }

    pub fn event_type(mut self, event_type: impl AsRef<str>) -> Self {
        self.event_type = Some(event_type.as_ref().to_string());
        self
    }

    pub fn action_id(mut self, action_id: impl AsRef<str>) -> Self {
        self.action_id = Some(action_id.as_ref().to_string());
        self
    }

    pub fn metadata(mut self, metadata: JsonValue) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn force_log(mut self) -> Self {
        self.force_log = true;
        self
    }

    pub fn log(self, level: LogLevel) {
        Sensor::log_internal(
            &self.message,
            self.location.as_deref(),
            self.event_type.as_deref(),
            self.action_id.as_deref(),
            level,
            self.force_log,
            self.metadata
        );
    }
}

//=============================================================================
// SENSOR CORE - ULTRA SIMPLE API
//=============================================================================

pub struct Sensor;

impl Sensor {
    //=========================================================================
    // SIMPLE ONE-LINER METHODS (message only)
    //=========================================================================

    /// sensor.critical("Database connection lost!")
    pub fn critical(message: impl AsRef<str>) -> SensorBuilder {
        let builder = SensorBuilder::new(message);
        builder.log(LogLevel::CRITICAL);
        SensorBuilder::new("") // Return empty builder for chaining if needed
    }

    /// sensor.error("Failed to process request")
    pub fn error(message: impl AsRef<str>) -> SensorBuilder {
        let builder = SensorBuilder::new(message);
        builder.log(LogLevel::ERROR);
        SensorBuilder::new("")
    }

    /// sensor.warn("High memory usage detected").force_log()
    pub fn warn(message: impl AsRef<str>) -> SensorBuilder {
        SensorBuilder::new(message)
    }

    /// sensor.info("User logged in successfully").force_log()
    pub fn info(message: impl AsRef<str>) -> SensorBuilder {
        SensorBuilder::new(message)
    }

    /// sensor.success("Operation completed").force_log()
    pub fn success(message: impl AsRef<str>) -> SensorBuilder {
        SensorBuilder::new(message)
    }

    /// sensor.debug("Cache hit for key: user_123").force_log()
    pub fn debug(message: impl AsRef<str>) -> SensorBuilder {
        SensorBuilder::new(message)
    }

    /// sensor.sys("Quantum Breathing System activated").force_log()
    pub fn sys(message: impl AsRef<str>) -> SensorBuilder {
        SensorBuilder::new(message)
    }

    //=========================================================================
    // INTERNAL LOGGING ENGINE
    //=========================================================================

    fn log_internal(
        message: &str,
        location: Option<&str>,
        event_type: Option<&str>,
        action_id: Option<&str>,
        level: LogLevel,
        force_log: bool,
        metadata: Option<JsonValue>
    ) {
        let should_log = force_log || level.logs_by_default();

        if should_log {
            Self::log_to_terminal(message, location, event_type, action_id, level, metadata);
        }

        #[cfg(debug_assertions)]
        if !should_log {
            println!(
                "{}{}[SUPPRESSED] {} {}: {}{}",
                Colors::DIM,
                Colors::CYAN,
                Self::format_timestamp(),
                level,
                message,
                Colors::RESET
            );
        }
    }

    fn log_to_terminal(
        message: &str,
        location: Option<&str>,
        event_type: Option<&str>,
        action_id: Option<&str>,
        level: LogLevel,
        metadata: Option<JsonValue>
    ) {
        let timestamp = Self::format_timestamp();
        let color_style = level.color_style();
        let additional_style = level.additional_style();

        let mut log_line = String::new();
        log_line.push_str(&format!("{}{}", color_style, additional_style));

        // Base format: [timestamp] LEVEL: message
        log_line.push_str(&format!("[{}] {}: {}", timestamp, level, message));

        // Optional enrichment in logical order
        if let Some(loc) = location {
            log_line.push_str(&format!(" @{}", loc));
        }
        if let Some(event) = event_type {
            log_line.push_str(&format!(" [{}]", event));
        }
        if let Some(action) = action_id {
            log_line.push_str(&format!(" ({}) ", action));
        }
        if let Some(meta) = metadata {
            log_line.push_str(&format!(" {}", meta));
        }

        // Reset colors
        log_line.push_str(Colors::RESET);

        println!("{}", log_line);
    }

    fn format_timestamp() -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let secs = now.as_secs() % 86400; // Seconds within the day
        let millis = now.subsec_millis();
        format!("{}.{:03}Z", secs, millis)
    }
}

//=============================================================================
// BACKWARD COMPATIBILITY - EXISTING API PRESERVED
//=============================================================================

/// EXISTING API: sensor::error(action_id, message, location, metadata)
pub fn error(
    action_id: impl AsRef<str>,
    message: impl AsRef<str>,
    location: Option<impl AsRef<str>>,
    metadata: Option<JsonValue>
) {
    Sensor::log_internal(
        message.as_ref(),
        location.as_ref().map(|l| l.as_ref()),
        None, // event_type
        Some(action_id.as_ref()),
        LogLevel::ERROR,
        false, // Use default logging rules
        metadata
    );
}

/// EXISTING API: sensor::critical(action_id, message, location, metadata)
pub fn critical(
    action_id: impl AsRef<str>,
    message: impl AsRef<str>,
    location: Option<impl AsRef<str>>,
    metadata: Option<JsonValue>
) {
    Sensor::log_internal(
        message.as_ref(),
        location.as_ref().map(|l| l.as_ref()),
        None, // event_type
        Some(action_id.as_ref()),
        LogLevel::CRITICAL,
        false, // Use default logging rules
        metadata
    );
}

/// EXISTING API: sensor::warn(action_id, message, force_log)
pub fn warn(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
    Sensor::log_internal(
        message.as_ref(),
        None, // location
        None, // event_type
        Some(action_id.as_ref()),
        LogLevel::WARN,
        force_log,
        None // metadata
    );
}

/// EXISTING API: sensor::info(action_id, message, force_log)
pub fn info(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
    Sensor::log_internal(
        message.as_ref(),
        None, // location
        None, // event_type
        Some(action_id.as_ref()),
        LogLevel::INFO,
        force_log,
        None // metadata
    );
}

/// EXISTING API: sensor::success(action_id, message, force_log)
pub fn success(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
    Sensor::log_internal(
        message.as_ref(),
        None, // location
        None, // event_type
        Some(action_id.as_ref()),
        LogLevel::SUCCESS,
        force_log,
        None // metadata
    );
}

/// EXISTING API: sensor::debug(action_id, message, force_log)
pub fn debug(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
    Sensor::log_internal(
        message.as_ref(),
        None, // location
        None, // event_type
        Some(action_id.as_ref()),
        LogLevel::DEBUG,
        force_log,
        None // metadata
    );
}

/// EXISTING API: sensor::sys(action_id, message, force_log)
pub fn sys(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
    Sensor::log_internal(
        message.as_ref(),
        None, // location
        None, // event_type
        Some(action_id.as_ref()),
        LogLevel::SYS,
        force_log,
        None // metadata
    );
}

//=============================================================================
// NEW SIMPLIFIED API (message-first, everything optional)
//=============================================================================

/// NEW API: sensor::msg::critical("Database down!")
pub mod msg {
    use super::*;

    /// msg::critical("Database connection lost!")
    pub fn critical(message: impl AsRef<str>) -> SensorBuilder {
        let builder = SensorBuilder::new(message);
        builder.log(LogLevel::CRITICAL);
        SensorBuilder::new("") // Return empty for potential chaining
    }

    /// msg::error("Failed to process request")
    pub fn error(message: impl AsRef<str>) -> SensorBuilder {
        let builder = SensorBuilder::new(message);
        builder.log(LogLevel::ERROR);
        SensorBuilder::new("")
    }

    /// msg::warn("High memory usage").force_log()
    pub fn warn(message: impl AsRef<str>) -> SensorBuilder {
        SensorBuilder::new(message)
    }

    /// msg::info("User logged in successfully").force_log()
    pub fn info(message: impl AsRef<str>) -> SensorBuilder {
        SensorBuilder::new(message)
    }

    /// msg::success("Operation completed").force_log()
    pub fn success(message: impl AsRef<str>) -> SensorBuilder {
        SensorBuilder::new(message)
    }

    /// msg::debug("Cache hit for key: user_123").force_log()
    pub fn debug(message: impl AsRef<str>) -> SensorBuilder {
        SensorBuilder::new(message)
    }

    /// msg::sys("Quantum Breathing System activated").force_log()
    pub fn sys(message: impl AsRef<str>) -> SensorBuilder {
        SensorBuilder::new(message)
    }
}
