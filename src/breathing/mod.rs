// src/breathing/mod.rs
// File location: src/breathing/mod.rs
// Quantum Breathing System - Adaptive performance management

//=============================================================================
// IMPORTS
//=============================================================================

use std::sync::{ Arc, RwLock, OnceLock };
use tokio::time::interval;
use crate::context::{ metrics_state, sensor };
use crate::config::{ Breathing, Messages };
use crate::utils::current_timestamp;

// Re-export the BreathingUpdate type from metrics_state
pub use crate::context::metrics_state::BreathingUpdate;

//=============================================================================
// BREATHING SYSTEM STATE
//=============================================================================

#[derive(Debug)]
struct BreathingSystem {
    is_running: bool,
    task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl BreathingSystem {
    fn new() -> Self {
        Self {
            is_running: false,
            task_handle: None,
        }
    }
}

static BREATHING_SYSTEM: OnceLock<Arc<RwLock<BreathingSystem>>> = OnceLock::new();

fn get_breathing_system() -> &'static Arc<RwLock<BreathingSystem>> {
    BREATHING_SYSTEM.get_or_init(|| Arc::new(RwLock::new(BreathingSystem::new())))
}

//=============================================================================
// PUBLIC API
//=============================================================================

/// Start the quantum breathing system
pub fn start_breathing() {
    let system = get_breathing_system();

    if let Ok(mut breathing) = system.write() {
        if breathing.is_running {
            return; // Already running
        }

        breathing.is_running = true;

        // Spawn the breathing task
        let system_clone = Arc::clone(system);
        let handle = tokio::spawn(async move {
            breathing_loop(system_clone).await;
        });

        breathing.task_handle = Some(handle);

        sensor::sys("breathing", Messages::BREATHING_SYSTEM_STARTING, true);
    }
}

/// Stop the quantum breathing system
pub fn stop_breathing() {
    let system = get_breathing_system();

    if let Ok(mut breathing) = system.write() {
        if !breathing.is_running {
            return; // Already stopped
        }

        breathing.is_running = false;

        // Cancel the task if it exists
        if let Some(handle) = breathing.task_handle.take() {
            handle.abort();
        }

        sensor::sys("breathing", Messages::BREATHING_SYSTEM_STOPPED, true);
    }
}

/// Check if breathing system is running
pub fn is_breathing() -> bool {
    get_breathing_system()
        .read()
        .map(|breathing| breathing.is_running)
        .unwrap_or(false)
}

/// Force a breathing cycle (useful for testing or immediate response)
pub async fn force_breath() -> Result<(), String> {
    if !is_breathing() {
        return Err("Breathing system not running".to_string());
    }

    perform_breathing_cycle().await
}

//=============================================================================
// INTERNAL BREATHING LOGIC
//=============================================================================

/// Main breathing loop
async fn breathing_loop(system: Arc<RwLock<BreathingSystem>>) {
    let mut interval = interval(std::time::Duration::from_millis(Breathing::BREATH_CHECK_INTERVAL));

    loop {
        interval.tick().await;

        // Check if we should continue running
        let should_continue = system
            .read()
            .map(|breathing| breathing.is_running)
            .unwrap_or(false);

        if !should_continue {
            break;
        }

        // Perform breathing cycle
        if let Err(e) = perform_breathing_cycle().await {
            sensor::error(
                "breathing",
                &format!("Breathing cycle failed: {}", e),
                Some("breathing_loop"),
                None
            );
        }
    }
}

/// Perform a single breathing cycle
async fn perform_breathing_cycle() -> Result<(), String> {
    // Get current metrics state
    let current_state = metrics_state::get().ok_or("Metrics state not available")?;

    let now = current_timestamp();

    // Check if it's time for a breath
    let needs_breath =
        now >= current_state.breathing.next_breath_due || metrics_state::needs_breathing();

    if !needs_breath {
        return Ok(());
    }

    // Calculate new stress level based on system metrics
    let new_stress = calculate_system_stress(&current_state);

    // Update breathing state
    metrics_state::breath(BreathingUpdate {
        stress_level: new_stress,
    })?;

    // Handle stress level changes
    handle_stress_level_changes(new_stress, &current_state).await?;

    Ok(())
}

/// Calculate overall system stress
fn calculate_system_stress(state: &metrics_state::QuantumState) -> f64 {
    // Weight different stress factors
    let cpu_stress = (state.system.cpu / 100.0).min(1.0);
    let memory_stress = (state.system.memory / 100.0).min(1.0);
    let event_loop_stress = (state.system.event_loop / 1000.0).min(1.0);
    let call_rate_stress = (state.performance.calls_per_second / 100.0).min(1.0);

    // Combined stress calculation
    let combined = (
        cpu_stress * 0.3 +
        memory_stress * 0.3 +
        event_loop_stress * 0.2 +
        call_rate_stress * 0.2
    ).min(1.0);

    // Apply some smoothing to prevent rapid oscillations
    let current_stress = state.breathing.stress;
    let smoothing_factor = 0.3;

    current_stress * (1.0 - smoothing_factor) + combined * smoothing_factor
}

/// Handle stress level changes and trigger appropriate responses
async fn handle_stress_level_changes(
    new_stress: f64,
    current_state: &metrics_state::QuantumState
) -> Result<(), String> {
    // High stress - enter recuperation
    if new_stress > Breathing::STRESS_HIGH && !current_state.in_recuperation {
        metrics_state::enter_recuperation()?;
        sensor::warn("breathing", Messages::BREATHING_STRESS_HIGH, true);
    } else if
        // Stress normalized - exit recuperation
        new_stress <= Breathing::STRESS_MEDIUM &&
        current_state.in_recuperation
    {
        metrics_state::exit_recuperation()?;
        sensor::info("breathing", Messages::BREATHING_STRESS_NORMALIZED, true);
    }

    // Very high stress - enter hibernation
    if new_stress > Breathing::STRESS_CRITICAL && !current_state.hibernating {
        metrics_state::enter_hibernation()?;
        sensor::critical(
            "breathing",
            Messages::BREATHING_HIBERNATION_ENTER,
            Some("handle_stress_level_changes"),
            None
        );
    } else if
        // Stress very low - exit hibernation
        new_stress <= Breathing::CALM_THRESHOLD &&
        current_state.hibernating
    {
        metrics_state::exit_hibernation()?;
        sensor::info("breathing", Messages::BREATHING_HIBERNATION_EXIT, true);
    }

    Ok(())
}

//=============================================================================
// BREATHING SYSTEM MONITORING
//=============================================================================

/// Get current breathing status
pub fn get_breathing_status() -> Option<metrics_state::BreathingInfo> {
    metrics_state::get_breathing_info()
}

/// Check if system is in recuperation mode
pub fn is_in_recuperation() -> bool {
    metrics_state
        ::get()
        .map(|state| state.in_recuperation)
        .unwrap_or(false)
}

/// Check if system is hibernating
pub fn is_hibernating() -> bool {
    metrics_state
        ::get()
        .map(|state| state.hibernating)
        .unwrap_or(false)
}

/// Get current stress level
pub fn get_stress_level() -> f64 {
    metrics_state
        ::get()
        .map(|state| state.breathing.stress)
        .unwrap_or(0.0)
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{ sleep, Duration };

    #[tokio::test]
    async fn test_breathing_lifecycle() {
        // Initialize metrics first
        let _ = metrics_state::init();

        // Test starting breathing
        assert!(!is_breathing());
        start_breathing();
        assert!(is_breathing());

        // Give it a moment to start
        sleep(Duration::from_millis(10)).await;

        // Test stopping breathing
        stop_breathing();
        assert!(!is_breathing());
    }

    #[tokio::test]
    async fn test_force_breath() {
        let _ = metrics_state::init();

        // Should fail when not running
        assert!(force_breath().await.is_err());

        start_breathing();

        // Should succeed when running
        let result = force_breath().await;
        assert!(result.is_ok());

        stop_breathing();
    }

    #[test]
    fn test_stress_calculation() {
        let _ = metrics_state::init();

        // Create a test state
        let mut state = metrics_state::QuantumState::default();
        state.system.cpu = 50.0;
        state.system.memory = 60.0;
        state.system.event_loop = 100.0;
        state.performance.calls_per_second = 10.0;

        let stress = calculate_system_stress(&state);
        assert!(stress >= 0.0 && stress <= 1.0);
    }

    #[test]
    fn test_breathing_status_functions() {
        let _ = metrics_state::init();

        // Initially should be calm
        assert!(!is_in_recuperation());
        assert!(!is_hibernating());
        assert_eq!(get_stress_level(), 0.0);

        // The breathing status might be None initially
        // but shouldn't panic
        let _status = get_breathing_status();
    }
}
