// src/config/mod.rs
// Cyre system configurations and messages - Pure constants only

use serde::{ Serialize, Deserialize };

//=============================================================================
// PAYLOAD CONFIGURATION
//=============================================================================

pub struct PayloadConfig;
impl PayloadConfig {
  pub const MAX_HISTORY_PER_CHANNEL: usize = 10;
}

//=============================================================================
// TIMING CONFIGURATION
//=============================================================================

pub struct Timing;
impl Timing {
  /// 60fps - For smooth animations (requestAnimationFrame preferred)
  pub const ANIMATION: u64 = 17; // ~16.67ms rounded up
  /// User interface updates
  pub const UI_UPDATE: u64 = 100;
  /// Input handling (typing, scrolling)
  pub const INPUT_DEBOUNCE: u64 = 150;
  /// API polling/data refresh
  pub const API_POLLING: u64 = 1000;
  /// Background operations
  pub const BACKGROUND_TASK: u64 = 5000;

  // System timing constants
  /// 1 minute - Recuperation check interval
  pub const RECUPERATION: u64 = 60 * 1000;
  /// 1 hour - Long timer handling
  pub const LONG_INTERVAL: u64 = 60 * 60 * 1000;
  /// Max safe timeout value
  pub const MAX_TIMEOUT: u64 = (2u64).pow(31) - 1;
}

//=============================================================================
// PROTECTION CONFIGURATION
//=============================================================================

pub struct Protection;
impl Protection {
  pub const CALL_THRESHOLD: u32 = 100;
  pub const MIN_DEBOUNCE: u64 = 50;
  pub const MIN_THROTTLE: u64 = 50;
  pub const MAX_DELAY: u64 = 2000;
  pub const WINDOW: u64 = 1000;
  pub const INITIAL_DELAY: u64 = 25;
  pub const SYSTEM_LOAD_DELAY: u64 = 250;

  // System warning thresholds
  pub const CPU_WARNING: f64 = 85.0;
  pub const CPU_CRITICAL: f64 = 95.0;
  pub const MEMORY_WARNING: f64 = 85.0;
  pub const MEMORY_CRITICAL: f64 = 95.0;
  pub const EVENT_LOOP_WARNING: f64 = 200.0;
  pub const EVENT_LOOP_CRITICAL: f64 = 1000.0;
}

pub struct SystemThresholds;
impl SystemThresholds {
  // CPU thresholds
  pub const CPU_WARNING: f64 = 85.0;
  pub const CPU_CRITICAL: f64 = 95.0;

  // Memory thresholds
  pub const MEMORY_WARNING: f64 = 85.0;
  pub const MEMORY_CRITICAL: f64 = 95.0;

  // Event loop thresholds
  pub const EVENT_LOOP_WARNING: u64 = 200;
  pub const EVENT_LOOP_CRITICAL: u64 = 1000;

  pub const OVERLOAD_THRESHOLD: u32 = 4;
}

//=============================================================================
// BREATHING CONFIGURATION
//=============================================================================

pub struct Breathing;
impl Breathing {
  // Core breathing rates (in ms)
  pub const MIN_RATE: u64 = 50;
  pub const BASE_RATE: u64 = 200;
  pub const MAX_RATE: u64 = 5000;

  // Stress thresholds (0.0 - 1.0)
  pub const STRESS_LOW: f64 = 0.2;
  pub const STRESS_MEDIUM: f64 = 0.5;
  pub const STRESS_HIGH: f64 = 0.8;
  pub const STRESS_CRITICAL: f64 = 0.95;

  // Recuperation parameters
  pub const RECUPERATION_THRESHOLD: f64 = 0.75;
  pub const HIBERNATION_THRESHOLD: f64 = 0.95;
  pub const CALM_THRESHOLD: f64 = 0.3;
  pub const MAX_RECUPERATION_DEPTH: u32 = 10;

  // Timing constants
  pub const BREATH_CHECK_INTERVAL: u64 = 1000;
  pub const RECUPERATION_TIME: u64 = 5000;
  pub const HIBERNATION_TIME: u64 = 30000;
}

//=============================================================================
// MESSAGES CONFIGURATION
//=============================================================================

pub struct Messages;
impl Messages {
  // Welcome and initialization messages
  pub const WELCOME: &'static str = "🦀 CYRE - Reactive Event Management System";
  pub const INIT_START: &'static str = "Initializing Cyre system components";
  pub const INIT_COMPLETE: &'static str = "System initialization complete";

  // Protection system messages
  pub const PROTECTION_ACTIVE: &'static str = "Protection mechanisms active";
  pub const RATE_LIMITED: &'static str = "Request rate limited - queued for processing";
  pub const THROTTLE_ACTIVE: &'static str = "Throttling active - request delayed";
  pub const DEBOUNCE_ACTIVE: &'static str = "Debouncing active - previous request cancelled";

  // System status messages
  pub const SYSTEM_READY: &'static str = "System ready for operations";
  pub const SYS: &'static str =
    "Q0.0U0.0A0.0N0.0T0.0U0.0M0 - I0.0N0.0C0.0E0.0P0.0T0.0I0.0O0.0N0.0S0-- ";
  pub const SYSTEM_BUSY: &'static str = "System busy - request queued";
  pub const SYSTEM_OVERLOADED: &'static str = "System overloaded - shedding non-critical load";
  pub const SYSTEM_RECOVERING: &'static str = "System recovering from overload";

  // Error and warning messages
  pub const ACTION_NOT_FOUND: &'static str = "Action not found";
  pub const HANDLER_NOT_REGISTERED: &'static str = "No handler registered for action";
  pub const INVALID_CONFIGURATION: &'static str = "Invalid action configuration";
  pub const TIMEOUT_EXCEEDED: &'static str = "Operation timeout exceeded";

  // Clear and reset messages
  pub const SYSTEM_CLEAR_INITIATED: &'static str = "System clear initiated";
  pub const SYSTEM_CLEAR_COMPLETED: &'static str = "System clear completed successfully";
  pub const CLEAR_OPERATION_FAILED: &'static str = "Clear operation failed";

  // Breathing system messages
  pub const BREATHING_SYSTEM_STARTING: &'static str = "Quantum Breathing System starting";
  pub const BREATHING_SYSTEM_STOPPED: &'static str = "Quantum Breathing System stopped";
  pub const BREATHING_STRESS_HIGH: &'static str = "System stress high - entering recuperation";
  pub const BREATHING_STRESS_NORMALIZED: &'static str =
    "System stress normalized - exiting recuperation";
  pub const BREATHING_HIBERNATION_ENTER: &'static str = "System quiet - entering hibernation mode";
  pub const BREATHING_HIBERNATION_EXIT: &'static str = "System active - exiting hibernation mode";

  // Initialization messages
  pub const CYRE_NOT_INITIALIZED: &'static str = "Cyre not initialized - call cyre.init() first";
  pub const CYRE_ALREADY_INITIALIZED: &'static str = "Cyre already initialized";
  pub const CYRE_SYSTEM_INITIALIZING: &'static str = "Initializing Cyre system components";
  pub const CYRE_INITIALIZED_SUCCESS: &'static str = "Cyre system initialized successfully";
  pub const METRICS_INIT_FAILED: &'static str = "Metrics initialization failed";
  pub const BREATHING_START_FAILED: &'static str = "Breathing system failed to start";
  pub const TIMEKEEPER_INIT_WARNING: &'static str = "TimeKeeper initialization warning";

  // Shutdown messages
  pub const SYSTEM_SHUTDOWN_INITIATED: &'static str = "System shutdown initiated";
  pub const SYSTEM_SHUTDOWN_COMPLETED: &'static str = "System shutdown completed successfully";
  pub const SYSTEM_SHUTDOWN_FAILED: &'static str = "System shutdown failed";
  pub const SYSTEM_SHUTDOWN_SIGNAL_DETECTED: &'static str = "System shutdown signal detected";
  pub const SYSTEM_ALREADY_SHUTDOWN: &'static str = "System is already shutdown";
  pub const EMERGENCY_SHUTDOWN: &'static str = "Emergency shutdown initiated";
  pub const GRACEFUL_SHUTDOWN: &'static str = "Graceful shutdown in progress";

  // Lock messages
  pub const SYSTEM_LOCKED: &'static str = "System locked - new registrations blocked";
  pub const SYSTEM_UNLOCKED: &'static str = "System unlocked - registrations enabled";
  pub const SYSTEM_LOCK_FAILED: &'static str = "System lock operation failed";
  pub const SYSTEM_UNLOCK_FAILED: &'static str = "System unlock operation failed";
  pub const REGISTRATION_BLOCKED_LOCKED: &'static str = "Registration blocked - system is locked";
  pub const CANNOT_LOCK_SHUTDOWN_SYSTEM: &'static str = "Cannot lock shutdown system";
  pub const CANNOT_UNLOCK_SHUTDOWN_SYSTEM: &'static str = "Cannot unlock shutdown system";

  // Reset messages
  pub const SYSTEM_RESET_INITIATED: &'static str = "System reset initiated";
  pub const SYSTEM_RESET_COMPLETED: &'static str = "System reset completed successfully";
  pub const SYSTEM_RESET_FAILED: &'static str = "System reset operation failed";
  pub const SYSTEM_RESET_PARTIAL: &'static str = "System reset completed with partial failures";
  /// Dynamic message functions
  pub fn rate_limited(delay: u64) -> String {
    format!("Request queued - processing will resume in {}ms", delay)
  }

  pub fn task_completed_with_id(task_id: &str) -> String {
    format!("Task '{}' completed successfully - anything else I can help with?", task_id)
  }

  pub fn component_init_duration(duration_ms: u128) -> String {
    format!("Component initialized in {:.2}ms", duration_ms)
  }

  pub fn system_stress_level(stress_percent: f64) -> String {
    format!("System stress level: {:.1}%", stress_percent * 100.0)
  }
}

//=============================================================================
// DEFAULT METRICS - MATCHES YOUR TYPESCRIPT defaultMetrics
//=============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultSystemMetrics {
  pub cpu: f64,
  pub memory: f64,
  pub event_loop: f64,
  pub is_overloaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultBreathingState {
  pub breath_count: u64,
  pub current_rate: u64,
  pub last_breath: u64,
  pub stress: f64,
  pub is_recuperating: bool,
  pub recuperation_depth: u32,
  pub pattern: String,
  pub next_breath_due: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultActiveQueues {
  pub critical: u32,
  pub high: u32,
  pub medium: u32,
  pub low: u32,
  pub background: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultPerformanceMetrics {
  pub calls_total: u64,
  pub calls_per_second: f64,
  pub last_call_timestamp: u64,
  pub active_queues: DefaultActiveQueues,
  pub queue_depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultSystemStress {
  pub cpu: f64,
  pub memory: f64,
  pub event_loop: f64,
  pub call_rate: f64,
  pub combined: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultStoreMetrics {
  pub channels: u32,
  pub branches: u32,
  pub tasks: u32,
  pub subscribers: u32,
  pub timeline: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultMetrics {
  pub system: DefaultSystemMetrics,
  pub breathing: DefaultBreathingState,
  pub performance: DefaultPerformanceMetrics,
  pub stress: DefaultSystemStress,
  pub store: DefaultStoreMetrics,
  pub last_update: u64,
  pub in_recuperation: bool,
  pub hibernating: bool,
  pub active_formations: u32,
  pub _locked: bool,
  pub _init: bool,
  pub _shutdown: bool,
}

/// Function to create default metrics instance
pub fn default_metrics() -> DefaultMetrics {
  use crate::utils::current_timestamp;

  DefaultMetrics {
    system: DefaultSystemMetrics {
      cpu: 0.0,
      memory: 0.0,
      event_loop: 0.0,
      is_overloaded: false,
    },
    breathing: DefaultBreathingState {
      breath_count: 0,
      current_rate: Breathing::BASE_RATE,
      last_breath: current_timestamp(),
      stress: 0.0,
      is_recuperating: false,
      recuperation_depth: 0,
      pattern: "NORMAL".to_string(),
      next_breath_due: current_timestamp() + Breathing::BASE_RATE,
    },
    performance: DefaultPerformanceMetrics {
      calls_total: 0,
      calls_per_second: 0.0,
      last_call_timestamp: current_timestamp(),
      active_queues: DefaultActiveQueues {
        critical: 0,
        high: 0,
        medium: 0,
        low: 0,
        background: 0,
      },
      queue_depth: 0,
    },
    stress: DefaultSystemStress {
      cpu: 0.0,
      memory: 0.0,
      event_loop: 0.0,
      call_rate: 0.0,
      combined: 0.0,
    },
    store: DefaultStoreMetrics {
      channels: 0,
      branches: 0,
      tasks: 0,
      subscribers: 0,
      timeline: 0,
    },
    last_update: current_timestamp(),
    in_recuperation: false,
    hibernating: false,
    active_formations: 0,
    _locked: false,
    _init: false,
    _shutdown: false,
  }
}
