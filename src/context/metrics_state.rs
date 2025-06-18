// src/context/metrics_state.rs
// File location: src/context/metrics_state.rs
// QuantumState - The brain of the system using state store pattern

//=============================================================================
// IMPORTS
//=============================================================================

use std::sync::{ Arc, RwLock, OnceLock };
use serde::{ Serialize, Deserialize };
use crate::utils::current_timestamp;
use crate::config::{ Protection, Breathing, default_metrics };

//=============================================================================
// UPDATE TYPES - FOR FUNCTION PARAMETERS
//=============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreathingUpdate {
    pub stress_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemUpdate {
    pub cpu: f64,
    pub memory: f64,
    pub event_loop: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreUpdate {
    pub channels: u32,
    pub branches: u32,
    pub tasks: u32,
    pub subscribers: u32,
    pub timeline: u32,
}

//=============================================================================
// QUANTUM STATE TYPES
//=============================================================================

/// System metrics for monitoring performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu: f64,
    pub memory: f64,
    pub event_loop: f64,
    pub is_overloaded: bool,
}

/// Breathing state for quantum adaptive performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreathingState {
    pub breath_count: u64,
    pub current_rate: u64,
    pub last_breath: u64,
    pub stress: f64,
    pub is_recuperating: bool,
    pub recuperation_depth: u32,
    pub pattern: String,
    pub next_breath_due: u64,
}

/// Performance metrics tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub calls_total: u64,
    pub calls_per_second: f64,
    pub last_call_timestamp: u64,
    pub active_queues: ActiveQueues,
    pub queue_depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveQueues {
    pub critical: u32,
    pub high: u32,
    pub medium: u32,
    pub low: u32,
    pub background: u32,
}

/// System stress indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStress {
    pub cpu: f64,
    pub memory: f64,
    pub event_loop: f64,
    pub call_rate: f64,
    pub combined: f64,
}

/// Store metrics for resource tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreMetrics {
    pub channels: u32,
    pub branches: u32,
    pub tasks: u32,
    pub subscribers: u32,
    pub timeline: u32,
}

/// QuantumState - The comprehensive system brain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumState {
    pub system: SystemMetrics,
    pub breathing: BreathingState,
    pub performance: PerformanceMetrics,
    pub stress: SystemStress,
    pub store: StoreMetrics,
    pub last_update: u64,
    pub in_recuperation: bool,
    pub hibernating: bool,
    pub active_formations: u32,
    pub _locked: bool,
    pub _init: bool,
    pub _shutdown: bool,
}

//=============================================================================
// SUMMARY TYPES FOR STATUS QUERIES
//=============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub overall: String,
    pub stress_level: f64,
    pub breathing_rate: u64,
    pub is_recuperating: bool,
    pub uptime: u64,
    pub system_locked: bool,
    pub hibernating: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreathingInfo {
    pub pattern: String,
    pub current_rate: u64,
    pub stress_level: f64,
    pub breath_count: u64,
    pub next_breath_due: u64,
    pub is_recuperating: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_calls: u64,
    pub calls_per_second: f64,
    pub queue_depth: u32,
    pub active_formations: u32,
    pub system_cpu: f64,
    pub system_memory: f64,
    pub is_overloaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreSummary {
    pub channels: u32,
    pub branches: u32,
    pub tasks: u32,
    pub subscribers: u32,
    pub timeline: u32,
    pub total_resources: u32,
}

//=============================================================================
// DEFAULT IMPLEMENTATION USING CONFIG
//=============================================================================

/// Default metrics matching your TypeScript defaultMetrics
impl Default for QuantumState {
    fn default() -> Self {
        // Import defaultMetrics from config as starting value!
        let default_config = default_metrics();

        Self {
            system: SystemMetrics {
                cpu: default_config.system.cpu,
                memory: default_config.system.memory,
                event_loop: default_config.system.event_loop,
                is_overloaded: default_config.system.is_overloaded,
            },
            breathing: BreathingState {
                breath_count: default_config.breathing.breath_count,
                current_rate: default_config.breathing.current_rate,
                last_breath: default_config.breathing.last_breath,
                stress: default_config.breathing.stress,
                is_recuperating: default_config.breathing.is_recuperating,
                recuperation_depth: default_config.breathing.recuperation_depth,
                pattern: default_config.breathing.pattern,
                next_breath_due: default_config.breathing.next_breath_due,
            },
            performance: PerformanceMetrics {
                calls_total: default_config.performance.calls_total,
                calls_per_second: default_config.performance.calls_per_second,
                last_call_timestamp: default_config.performance.last_call_timestamp,
                active_queues: ActiveQueues {
                    critical: default_config.performance.active_queues.critical,
                    high: default_config.performance.active_queues.high,
                    medium: default_config.performance.active_queues.medium,
                    low: default_config.performance.active_queues.low,
                    background: default_config.performance.active_queues.background,
                },
                queue_depth: default_config.performance.queue_depth,
            },
            stress: SystemStress {
                cpu: default_config.stress.cpu,
                memory: default_config.stress.memory,
                event_loop: default_config.stress.event_loop,
                call_rate: default_config.stress.call_rate,
                combined: default_config.stress.combined,
            },
            store: StoreMetrics {
                channels: default_config.store.channels,
                branches: default_config.store.branches,
                tasks: default_config.store.tasks,
                subscribers: default_config.store.subscribers,
                timeline: default_config.store.timeline,
            },
            last_update: default_config.last_update,
            in_recuperation: default_config.in_recuperation,
            hibernating: default_config.hibernating,
            active_formations: default_config.active_formations,
            _locked: default_config._locked,
            _init: default_config._init,
            _shutdown: default_config._shutdown,
        }
    }
}

//=============================================================================
// GLOBAL METRICS STATE STORE - USING STATE STORE PATTERN
//=============================================================================

static METRICS_STATE: OnceLock<Arc<RwLock<QuantumState>>> = OnceLock::new();

/// Get the global metrics state store
fn get_metrics_store() -> &'static Arc<RwLock<QuantumState>> {
    METRICS_STATE.get_or_init(|| Arc::new(RwLock::new(QuantumState::default())))
}

//=============================================================================
// METRICS STATE OPERATIONS - PROPER NAMING LIKE .breath() .set() .get() .lock()
//=============================================================================

/// Get current metrics state
pub fn get() -> Option<QuantumState> {
    let store = get_metrics_store();
    store
        .read()
        .ok()
        .map(|state| state.clone())
}

/// Set metrics state
pub fn set(new_state: QuantumState) -> Result<(), String> {
    let store = get_metrics_store();
    match store.write() {
        Ok(mut state) => {
            *state = new_state;
            Ok(())
        }
        Err(e) => Err(format!("Failed to update metrics state: {}", e)),
    }
}

/// Update specific fields
pub fn update<F>(updater: F) -> Result<(), String> where F: FnOnce(&mut QuantumState) {
    let store = get_metrics_store();
    match store.write() {
        Ok(mut state) => {
            updater(&mut state);
            state.last_update = current_timestamp();
            Ok(())
        }
        Err(e) => Err(format!("Failed to update metrics state: {}", e)),
    }
}

/// Reset metrics to default
pub fn reset() {
    let _ = set(QuantumState::default());
}

/// Initialize the metrics system
pub fn init() -> Result<(), String> {
    update(|state| {
        state._init = true;
        state._locked = false;
        state._shutdown = false;
        state.last_update = current_timestamp();
    })
}

/// Lock the system temporarily
pub fn lock() -> Result<(), String> {
    update(|state| {
        state._locked = true;
    })
}

/// Unlock the system
pub fn unlock() -> Result<(), String> {
    update(|state| {
        state._locked = false;
    })
}

/// Shutdown the system
pub fn shutdown() -> Result<(), String> {
    update(|state| {
        state._shutdown = true;
        state._locked = true;
        state.hibernating = true;
    })
}

//=============================================================================
// STATUS QUERIES
//=============================================================================

pub fn status() -> QuantumState {
    get().unwrap_or_default()
}

/// Check if initialized
pub fn is_init() -> bool {
    get()
        .map(|state| state._init)
        .unwrap_or(false)
}

/// Check if locked
pub fn is_locked() -> bool {
    get()
        .map(|state| state._locked)
        .unwrap_or(false)
}

/// Check if shutdown
pub fn is_shutdown() -> bool {
    get()
        .map(|state| state._shutdown)
        .unwrap_or(false)
}

//=============================================================================
// MAIN UPDATE FUNCTIONS - PROPER NAMING LIKE metricsState.breath()
//=============================================================================

/// Breathing update - proper naming like metricsState.breath({new data})
pub fn breath(breathing_data: BreathingUpdate) -> Result<(), String> {
    let now = current_timestamp();
    update(|state| {
        state.breathing.stress = breathing_data.stress_level;
        state.breathing.last_breath = now;
        state.breathing.breath_count += 1;

        // Update breathing rate based on stress
        let base_rate = Breathing::BASE_RATE as f64;
        let stress_multiplier = if breathing_data.stress_level > Breathing::STRESS_HIGH {
            0.5 // Breathe faster under stress
        } else if breathing_data.stress_level < Breathing::STRESS_LOW {
            2.0 // Breathe slower when calm
        } else {
            1.0
        };

        state.breathing.current_rate = (base_rate * stress_multiplier) as u64;
        state.breathing.next_breath_due = now + state.breathing.current_rate;

        // Update pattern
        state.breathing.pattern = if breathing_data.stress_level > Breathing::STRESS_CRITICAL {
            "EMERGENCY".to_string()
        } else if state.breathing.is_recuperating {
            "RECOVERY".to_string()
        } else {
            "NORMAL".to_string()
        };
    })
}

/// System metrics update
pub fn system(system_data: SystemUpdate) -> Result<(), String> {
    update(|state| {
        state.system.cpu = system_data.cpu;
        state.system.memory = system_data.memory;
        state.system.event_loop = system_data.event_loop;

        // Check if system is overloaded
        state.system.is_overloaded =
            system_data.cpu > Protection::CPU_WARNING ||
            system_data.memory > Protection::MEMORY_WARNING ||
            system_data.event_loop > Protection::EVENT_LOOP_WARNING;

        // Update stress levels
        state.stress.cpu = system_data.cpu / 100.0;
        state.stress.memory = system_data.memory / 100.0;
        state.stress.event_loop = system_data.event_loop / 1000.0;
        state.stress.call_rate = (state.performance.calls_per_second / 1000.0).min(1.0);

        // Calculate combined stress
        state.stress.combined = (
            state.stress.cpu * 0.3 +
            state.stress.memory * 0.3 +
            state.stress.event_loop * 0.2 +
            state.stress.call_rate * 0.2
        ).min(1.0);
    })
}

/// Record call - renamed from call() to avoid confusion
pub fn response(_ok: bool, _execution_time: u64) -> Result<(), String> {
    let now = current_timestamp();
    update(|state| {
        state.performance.calls_total += 1;
        state.performance.last_call_timestamp = now;

        // Calculate calls per second (simple moving average)
        let time_diff = if state.performance.last_call_timestamp > 0 {
            ((now - state.performance.last_call_timestamp) as f64) / 1000.0
        } else {
            1.0
        };

        if time_diff > 0.0 {
            state.performance.calls_per_second = 1.0 / time_diff;
        }
    })
}

/// Store metrics update
pub fn store(store_data: StoreUpdate) -> Result<(), String> {
    update(|state| {
        state.store.channels = store_data.channels;
        state.store.branches = store_data.branches;
        state.store.tasks = store_data.tasks;
        state.store.subscribers = store_data.subscribers;
        state.store.timeline = store_data.timeline;
    })
}

//=============================================================================
// LIFECYCLE MANAGEMENT
//=============================================================================

/// Enter recuperation mode
pub fn enter_recuperation() -> Result<(), String> {
    update(|state| {
        state.in_recuperation = true;
        state.breathing.is_recuperating = true;
        state.breathing.pattern = "RECOVERY".to_string();
        state.breathing.recuperation_depth = (state.breathing.recuperation_depth + 1).min(
            Breathing::MAX_RECUPERATION_DEPTH
        );
    })
}

/// Exit recuperation mode
pub fn exit_recuperation() -> Result<(), String> {
    update(|state| {
        state.in_recuperation = false;
        state.breathing.is_recuperating = false;
        state.breathing.pattern = "NORMAL".to_string();
        state.breathing.recuperation_depth = 0;
    })
}

/// Enter hibernation mode
pub fn enter_hibernation() -> Result<(), String> {
    update(|state| {
        state.hibernating = true;
        state.breathing.pattern = "HIBERNATION".to_string();
        state.breathing.current_rate = Breathing::MAX_RATE;
    })
}

/// Exit hibernation mode
pub fn exit_hibernation() -> Result<(), String> {
    update(|state| {
        state.hibernating = false;
        state.breathing.pattern = "NORMAL".to_string();
        state.breathing.current_rate = Breathing::BASE_RATE;
    })
}

//=============================================================================
// INFORMATION QUERIES - RENAMED FOR BETTER API
//=============================================================================

/// Get health summary
pub fn get_summary() -> HealthSummary {
    get()
        .map(|state| {
            let overall_health = if state.stress.combined < 0.3 {
                "excellent"
            } else if state.stress.combined < 0.6 {
                "good"
            } else if state.stress.combined < 0.8 {
                "warning"
            } else {
                "critical"
            };

            HealthSummary {
                overall: overall_health.to_string(),
                stress_level: state.stress.combined,
                breathing_rate: state.breathing.current_rate,
                is_recuperating: state.in_recuperation,
                uptime: current_timestamp() - state.last_update,
                system_locked: state._locked,
                hibernating: state.hibernating,
            }
        })
        .unwrap_or_else(|| HealthSummary {
            overall: "unknown".to_string(),
            stress_level: 0.0,
            breathing_rate: Breathing::BASE_RATE,
            is_recuperating: false,
            uptime: 0,
            system_locked: false,
            hibernating: false,
        })
}

/// Get breathing info
pub fn get_breathing_info() -> Option<BreathingInfo> {
    get().map(|state| {
        BreathingInfo {
            pattern: state.breathing.pattern.clone(),
            current_rate: state.breathing.current_rate,
            stress_level: state.breathing.stress,
            breath_count: state.breathing.breath_count,
            next_breath_due: state.breathing.next_breath_due,
            is_recuperating: state.breathing.is_recuperating,
        }
    })
}

/// Get performance summary
pub fn get_performance_summary() -> Option<PerformanceSummary> {
    get().map(|state| {
        PerformanceSummary {
            total_calls: state.performance.calls_total,
            calls_per_second: state.performance.calls_per_second,
            queue_depth: state.performance.queue_depth,
            active_formations: state.active_formations,
            system_cpu: state.system.cpu,
            system_memory: state.system.memory,
            is_overloaded: state.system.is_overloaded,
        }
    })
}

/// Get store summary - renamed from get_store_summary for consistency
pub fn get_store_summary() -> Option<StoreSummary> {
    get().map(|state| {
        StoreSummary {
            channels: state.store.channels,
            branches: state.store.branches,
            tasks: state.store.tasks,
            subscribers: state.store.subscribers,
            timeline: state.store.timeline,
            total_resources: state.store.channels +
            state.store.branches +
            state.store.tasks +
            state.store.subscribers +
            state.store.timeline,
        }
    })
}

/// Check if breathing is needed
pub fn needs_breathing() -> bool {
    get()
        .map(|state| {
            let now = current_timestamp();
            state.breathing.stress > Breathing::STRESS_HIGH ||
                now >= state.breathing.next_breath_due
        })
        .unwrap_or(false)
}

//=============================================================================
// INTEGRATION HELPERS
//=============================================================================

/// Helper to update metrics from other modules
pub fn record_channel_operation() -> Result<(), String> {
    response(true, 0)
}

/// Helper to record system stress from protection systems
pub fn record_protection_trigger(stress_increase: f64) -> Result<(), String> {
    if let Some(current_state) = get() {
        let new_stress = (current_state.stress.combined + stress_increase).min(1.0);
        breath(BreathingUpdate { stress_level: new_stress })
    } else {
        Err("Metrics state not initialized".to_string())
    }
}

/// Helper to get current system load for adaptive behavior
pub fn get_current_load() -> f64 {
    get()
        .map(|state| state.stress.combined)
        .unwrap_or(0.0)
}

/// Helper to check if system can handle more load
pub fn can_handle_load() -> bool {
    if let Some(state) = get() {
        !state.system.is_overloaded &&
            !state._locked &&
            !state.hibernating &&
            state.stress.combined < Breathing::STRESS_HIGH
    } else {
        false
    }
}

//=============================================================================
// CONVENIENCE FUNCTIONS FOR UPDATING INDIVIDUAL METRICS
//=============================================================================

/// Update system metrics
pub fn update_system_metrics(cpu: f64, memory: f64, event_loop: f64) -> Result<(), String> {
    system(SystemUpdate { cpu, memory, event_loop })
}

/// Update breathing with stress level
pub fn update_breathing(stress_level: f64) -> Result<(), String> {
    breath(BreathingUpdate { stress_level })
}

/// Update store metrics
pub fn update_store_metrics(
    channels: u32,
    branches: u32,
    tasks: u32,
    subscribers: u32,
    timeline: u32
) -> Result<(), String> {
    store(StoreUpdate { channels, branches, tasks, subscribers, timeline })
}

/// Record a function call
pub fn record_call() -> Result<(), String> {
    response(true, 0)
}

//=============================================================================
// INITIALIZE FUNCTION (ALIAS FOR CONSISTENCY)
//=============================================================================

/// Initialize function (alias for init for consistency)
pub fn initialize() -> Result<(), String> {
    init()
}

/// Check if initialized (alias for is_init for consistency)
pub fn is_initialized() -> bool {
    is_init()
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        reset();
        let result = initialize();
        assert!(result.is_ok());
        assert!(is_initialized());
        assert!(!is_locked());
        assert!(!is_shutdown());
    }

    #[test]
    fn test_breathing_update() {
        reset();
        let _ = initialize();

        let result = update_breathing(0.7);
        assert!(result.is_ok());

        if let Some(state) = get() {
            assert_eq!(state.breathing.stress, 0.7);
            assert!(state.breathing.breath_count > 0);
        }
    }

    #[test]
    fn test_system_update() {
        reset();
        let _ = initialize();

        let result = update_system_metrics(50.0, 60.0, 100.0);
        assert!(result.is_ok());

        if let Some(state) = get() {
            assert_eq!(state.system.cpu, 50.0);
            assert_eq!(state.system.memory, 60.0);
            assert_eq!(state.system.event_loop, 100.0);
            assert!(!state.system.is_overloaded); // Below thresholds
        }
    }

    #[test]
    fn test_system_overload_detection() {
        reset();
        let _ = initialize();

        // Test with values above thresholds
        let result = update_system_metrics(90.0, 90.0, 300.0);
        assert!(result.is_ok());

        if let Some(state) = get() {
            assert!(state.system.is_overloaded);
        }
    }

    #[test]
    fn test_recuperation_lifecycle() {
        reset();
        let _ = initialize();

        let result = enter_recuperation();
        assert!(result.is_ok());
        assert!(get().unwrap().in_recuperation);
        assert_eq!(get().unwrap().breathing.pattern, "RECOVERY");

        let result = exit_recuperation();
        assert!(result.is_ok());
        assert!(!get().unwrap().in_recuperation);
        assert_eq!(get().unwrap().breathing.pattern, "NORMAL");
    }

    #[test]
    fn test_hibernation_lifecycle() {
        reset();
        let _ = initialize();

        let result = enter_hibernation();
        assert!(result.is_ok());
        assert!(get().unwrap().hibernating);
        assert_eq!(get().unwrap().breathing.pattern, "HIBERNATION");

        let result = exit_hibernation();
        assert!(result.is_ok());
        assert!(!get().unwrap().hibernating);
        assert_eq!(get().unwrap().breathing.pattern, "NORMAL");
    }

    #[test]
    fn test_shutdown() {
        reset();
        let _ = initialize();
        assert!(is_initialized());
        assert!(!is_shutdown());

        let result = shutdown();
        assert!(result.is_ok());
        assert!(is_shutdown());
        assert!(is_locked());
        assert!(get().unwrap().hibernating);
    }

    #[test]
    fn test_store_metrics_update() {
        reset();
        let _ = initialize();

        let result = update_store_metrics(10, 5, 15, 20, 8);
        assert!(result.is_ok());

        if let Some(state) = get() {
            assert_eq!(state.store.channels, 10);
            assert_eq!(state.store.branches, 5);
            assert_eq!(state.store.tasks, 15);
            assert_eq!(state.store.subscribers, 20);
            assert_eq!(state.store.timeline, 8);
        }
    }
}
