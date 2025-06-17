// src/context/sensor.rs
// File location: src/context/sensor.rs
// Cyre Sensor Logging System - Terminal logging with your color theme

//=============================================================================
// IMPORTS
//=============================================================================

use std::fmt;
use serde::{ Deserialize, Serialize };
use serde_json::Value as JsonValue;

//=============================================================================
// COLOR DEFINITIONS - YOUR THEME
//=============================================================================

/// Color codes with semantic names - matching your TypeScript theme
pub struct Colors;

impl Colors {
    pub const RESET: &'static str = "\x1b[0m";
    pub const MAGENTA: &'static str = "\x1b[35m";
    pub const MAGENTA_BRIGHT: &'static str = "\x1b[95m";
    pub const RED: &'static str = "\x1b[31m";
    pub const RED_BRIGHT: &'static str = "\x1b[91m";
    pub const GREEN: &'static str = "\x1b[32m";
    pub const GREEN_BRIGHT: &'static str = "\x1b[92m";
    pub const CYAN: &'static str = "\x1b[36m";
    pub const CYAN_BRIGHT: &'static str = "\x1b[96m";
    pub const YELLOW: &'static str = "\x1b[33m";
    pub const YELLOW_BRIGHT: &'static str = "\x1b[93m";
    pub const WHITE: &'static str = "\x1b[37m";
    pub const WHITE_BRIGHT: &'static str = "\x1b[97m";
    pub const BLUE: &'static str = "\x1b[34m";
    pub const BLUE_BRIGHT: &'static str = "\x1b[94m";

    // Background colors
    pub const BG_RED: &'static str = "\x1b[41m";
    pub const BG_YELLOW: &'static str = "\x1b[43m";
    pub const BG_BLUE: &'static str = "\x1b[44m";
    pub const BG_MAGENTA: &'static str = "\x1b[45m";

    // Text styles
    pub const BOLD: &'static str = "\x1b[1m";
    pub const DIM: &'static str = "\x1b[2m";
    pub const ITALIC: &'static str = "\x1b[3m";
    pub const UNDERLINE: &'static str = "\x1b[4m";
}

//=============================================================================
// LOG LEVELS
//=============================================================================

/// Log levels for categorizing events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    SUCCESS,
    CRITICAL,
    SYS,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    /// Get color styling for log level - matching your theme exactly
    pub fn color_style(&self) -> &'static str {
        match self {
            LogLevel::DEBUG => Colors::DIM, // dim, cyan
            LogLevel::INFO => Colors::CYAN, // cyan, bold
            LogLevel::WARN => Colors::YELLOW_BRIGHT, // yellowBright, bold
            LogLevel::ERROR => Colors::RED_BRIGHT, // redBright, bold
            LogLevel::SUCCESS => Colors::GREEN_BRIGHT, // greenBright, bold, dim
            LogLevel::CRITICAL => Colors::BG_RED, // bgRed, whiteBright, bold
            LogLevel::SYS => Colors::BG_MAGENTA, // bgMagenta, white
        }
    }

    /// Get additional styling
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

    /// Check if this log level should log to terminal by default
    pub fn logs_by_default(&self) -> bool {
        matches!(self, LogLevel::ERROR | LogLevel::CRITICAL)
    }
}

//=============================================================================
// SENSOR CORE FUNCTIONALITY
//=============================================================================

/// C.Y.R.E - S.E.N.S.O.R
///
/// Simplified sensor system for terminal logging:
/// - Only error and critical log by default
/// - Others need manual enable
/// - Uses your exact color theme
/// - Format: [timestamp] LEVEL: message
pub struct Sensor;

impl Sensor {
    /// General purpose log method
    pub fn log(
        action_id: impl AsRef<str>,
        level: LogLevel,
        message: impl AsRef<str>,
        location: Option<impl AsRef<str>>,
        force_log: bool,
        metadata: Option<JsonValue>
    ) {
        let should_log = force_log || level.logs_by_default();

        if should_log {
            Self::log_to_terminal(
                action_id.as_ref(),
                level,
                message.as_ref(),
                location.as_ref().map(|l| l.as_ref()),
                metadata
            );
        }

        #[cfg(debug_assertions)]
        if !should_log {
            // In debug mode, show suppressed logs dimly
            println!(
                "{}{}[SUPPRESSED] {} {}: {}{}",
                Colors::DIM,
                Colors::CYAN,
                Self::format_timestamp(),
                level,
                message.as_ref(),
                Colors::RESET
            );
        }
    }

    /// Success logging
    pub fn success(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
        Self::log(action_id, LogLevel::SUCCESS, message, None::<&str>, force_log, None);
    }

    /// Error logging - logs by default
    pub fn error(
        action_id: impl AsRef<str>,
        message: impl AsRef<str>,
        location: Option<impl AsRef<str>>,
        metadata: Option<JsonValue>
    ) {
        Self::log(action_id, LogLevel::ERROR, message, location, false, metadata);
    }

    /// Warning logging
    pub fn warn(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
        Self::log(action_id, LogLevel::WARN, message, None::<&str>, force_log, None);
    }

    /// Info logging
    pub fn info(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
        Self::log(action_id, LogLevel::INFO, message, None::<&str>, force_log, None);
    }

    /// Debug logging
    pub fn debug(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
        Self::log(action_id, LogLevel::DEBUG, message, None::<&str>, force_log, None);
    }

    /// Critical logging - logs by default
    pub fn critical(
        action_id: impl AsRef<str>,
        message: impl AsRef<str>,
        location: Option<impl AsRef<str>>,
        metadata: Option<JsonValue>
    ) {
        Self::log(action_id, LogLevel::CRITICAL, message, location, false, metadata);
    }

    /// System logging - special category
    pub fn sys(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
        Self::log(action_id, LogLevel::SYS, message, None::<&str>, force_log, None);
    }

    //=========================================================================
    // PRIVATE HELPER METHODS
    //=========================================================================

    /// Log to terminal with your exact format: [timestamp] LEVEL: message
    fn log_to_terminal(
        action_id: &str,
        level: LogLevel,
        message: &str,
        location: Option<&str>,
        metadata: Option<JsonValue>
    ) {
        let timestamp = Self::format_timestamp();
        let color_style = level.color_style();
        let additional_style = level.additional_style();

        // Build the log line: [timestamp] LEVEL: action_id - message
        let mut log_line = String::new();

        // [timestamp]
        log_line.push_str(&format!("[{}] ", timestamp));

        // LEVEL: (with colors)
        log_line.push_str(&format!("{}{}{}: ", color_style, additional_style, level));

        // action_id - message
        log_line.push_str(&format!("{} - {}", action_id, message));

        // Location if provided
        if let Some(loc) = location {
            log_line.push_str(&format!(" @ {}", loc));
        }

        // Reset colors
        log_line.push_str(Colors::RESET);

        println!("{}", log_line);

        // Print metadata if available (indented, without colors for readability)
        if let Some(metadata) = metadata {
            if let Ok(pretty_metadata) = serde_json::to_string_pretty(&metadata) {
                for line in pretty_metadata.lines() {
                    println!("    {}", line);
                }
            }
        }

        // Add extra emphasis for critical
        if level == LogLevel::CRITICAL {
            println!(
                "{}{}🚨 CRITICAL EVENT - IMMEDIATE ATTENTION REQUIRED 🚨{}",
                Colors::BG_RED,
                Colors::WHITE_BRIGHT,
                Colors::RESET
            );
        }
    }

    /// Format timestamp to ISO format like your example
    fn format_timestamp() -> String {
        use std::time::{ SystemTime, UNIX_EPOCH };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        // Convert to ISO-like format (simplified)
        // For now, just return timestamp - you can enhance this to proper ISO format
        let secs = timestamp / 1000;
        let millis = timestamp % 1000;

        // This is a simplified version - you might want to use chrono crate for proper ISO format
        format!("{}.{:03}Z", secs, millis)
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        use std::time::{ SystemTime, UNIX_EPOCH };
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
    }
}

//=============================================================================
// MODULE-LEVEL CONVENIENCE FUNCTIONS
//=============================================================================

/// General purpose log function
pub fn log(
    action_id: impl AsRef<str>,
    level: LogLevel,
    message: impl AsRef<str>,
    location: Option<impl AsRef<str>>,
    force_log: bool,
    metadata: Option<JsonValue>
) {
    Sensor::log(action_id, level, message, location, force_log, metadata);
}

/// Success logging (manual enable required)
pub fn success(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
    Sensor::success(action_id, message, force_log);
}

/// Error logging (auto-enabled)
pub fn error(
    action_id: impl AsRef<str>,
    message: impl AsRef<str>,
    location: Option<impl AsRef<str>>,
    metadata: Option<JsonValue>
) {
    Sensor::error(action_id, message, location, metadata);
}

/// Warning logging (manual enable required)
pub fn warn(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
    Sensor::warn(action_id, message, force_log);
}

/// Info logging (manual enable required)
pub fn info(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
    Sensor::info(action_id, message, force_log);
}

/// Debug logging (manual enable required)
pub fn debug(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
    Sensor::debug(action_id, message, force_log);
}

/// Critical logging (auto-enabled)
pub fn critical(
    action_id: impl AsRef<str>,
    message: impl AsRef<str>,
    location: Option<impl AsRef<str>>,
    metadata: Option<JsonValue>
) {
    Sensor::critical(action_id, message, location, metadata);
}

/// System logging (manual enable required)
pub fn sys(action_id: impl AsRef<str>, message: impl AsRef<str>, force_log: bool) {
    Sensor::sys(action_id, message, force_log);
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_levels() {
        assert_eq!(LogLevel::ERROR.logs_by_default(), true);
        assert_eq!(LogLevel::CRITICAL.logs_by_default(), true);
        assert_eq!(LogLevel::INFO.logs_by_default(), false);
        assert_eq!(LogLevel::DEBUG.logs_by_default(), false);
        assert_eq!(LogLevel::SUCCESS.logs_by_default(), false);
        assert_eq!(LogLevel::WARN.logs_by_default(), false);
        assert_eq!(LogLevel::SYS.logs_by_default(), false);
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(Colors::RESET, "\x1b[0m");
        assert_eq!(Colors::BG_MAGENTA, "\x1b[45m");
        assert_eq!(Colors::RED_BRIGHT, "\x1b[91m");
    }

    #[test]
    fn test_log_level_colors() {
        assert_eq!(LogLevel::SYS.color_style(), Colors::BG_MAGENTA);
        assert_eq!(LogLevel::CRITICAL.color_style(), Colors::BG_RED);
        assert_eq!(LogLevel::ERROR.color_style(), Colors::RED_BRIGHT);
    }

    #[tokio::test]
    async fn test_sensor_methods() {
        // These won't actually log unless force_log is true or they're error/critical
        success("test", "Success message", false);
        info("test", "Info message", false);
        warn("test", "Warning message", false);
        debug("test", "Debug message", false);

        // These will log by default
        error("test", "Error message", None::<&str>, None);
        critical("test", "Critical message", Some("test_location"), None);

        // Force logging
        success("test", "Forced success", true);
        sys("test", "System message", true);
    }
}

//=============================================================================
// SENSOR INSTANCE
//=============================================================================

/// Default sensor instance for easy access
pub static SENSOR: Sensor = Sensor;
