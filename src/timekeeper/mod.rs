// src/timekeeper/mod.rs
// TimeKeeper integration for task store

use std::sync::{ Arc, OnceLock };
use std::time::Duration;
use std::future::Future;
use std::pin::Pin;
use tokio::sync::RwLock;
use serde::{ Serialize, Deserialize };

use crate::types::CyreResponse;

//=============================================================================
// TIMEKEEPER TYPES
//=============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimerRepeat {
    Once,
    Forever,
    Count(u32),
}

#[derive(Debug, Clone)]
pub struct Formation {
    pub id: String,
    pub action_id: String,
    pub interval: u64,
    pub delay: Option<u64>,
    pub repeat: TimerRepeat,
    pub created_at: u64,
    pub last_execution: Option<u64>,
    pub execution_count: u32,
    pub is_active: bool,
}

/// TimeKeeper callback type
pub type TimeKeeperCallback = Box<
    dyn (Fn() -> Pin<Box<dyn Future<Output = CyreResponse> + Send>>) + Send + Sync
>;

//=============================================================================
// TIMEKEEPER IMPLEMENTATION
//=============================================================================

pub struct TimeKeeper {
    formations: Arc<RwLock<std::collections::HashMap<String, Formation>>>,
}

impl TimeKeeper {
    pub fn new() -> Self {
        Self {
            formations: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Schedule a timeout (runs once after delay)
    pub async fn schedule_timeout(
        &self,
        id: &str,
        callback: TimeKeeperCallback,
        delay: Duration
    ) -> Result<String, String> {
        let formation_id = format!("timeout-{}-{}", id, current_timestamp());

        let formation = Formation {
            id: formation_id.clone(),
            action_id: id.to_string(),
            interval: delay.as_millis() as u64,
            delay: Some(delay.as_millis() as u64),
            repeat: TimerRepeat::Once,
            created_at: current_timestamp(),
            last_execution: None,
            execution_count: 0,
            is_active: true,
        };

        // Store formation
        {
            let mut formations = self.formations.write().await;
            formations.insert(formation_id.clone(), formation);
        }

        // Execute the timeout
        let formations_ref = self.formations.clone();
        let formation_id_clone = formation_id.clone();

        tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            // Execute callback
            let result = callback().await;

            // Update formation
            {
                let mut formations = formations_ref.write().await;
                if let Some(formation) = formations.get_mut(&formation_id_clone) {
                    formation.execution_count += 1;
                    formation.last_execution = Some(current_timestamp());
                    formation.is_active = false; // Timeout only runs once
                }
            }

            println!("[TIMEKEEPER] Timeout {} executed: {}", formation_id_clone, result.message);
        });

        Ok(formation_id)
    }

    /// Schedule an interval (runs repeatedly)
    pub async fn schedule_interval(
        &self,
        id: &str,
        callback: TimeKeeperCallback,
        interval: Duration,
        repeat: TimerRepeat
    ) -> Result<String, String> {
        let formation_id = format!("interval-{}-{}", id, current_timestamp());

        let formation = Formation {
            id: formation_id.clone(),
            action_id: id.to_string(),
            interval: interval.as_millis() as u64,
            delay: None,
            repeat,
            created_at: current_timestamp(),
            last_execution: None,
            execution_count: 0,
            is_active: true,
        };

        // Store formation
        {
            let mut formations = self.formations.write().await;
            formations.insert(formation_id.clone(), formation);
        }

        // Execute the interval
        let formations_ref = self.formations.clone();
        let formation_id_clone = formation_id.clone();

        tokio::spawn(async move {
            let mut execution_count = 0;
            let max_executions = match repeat {
                TimerRepeat::Once => 1,
                TimerRepeat::Count(n) => n,
                TimerRepeat::Forever => u32::MAX,
            };

            let mut interval_timer = tokio::time::interval(interval);
            interval_timer.tick().await; // Skip first immediate tick

            while execution_count < max_executions {
                // Check if formation is still active
                let is_active = {
                    let formations = formations_ref.read().await;
                    formations
                        .get(&formation_id_clone)
                        .map(|f| f.is_active)
                        .unwrap_or(false)
                };

                if !is_active {
                    break;
                }

                interval_timer.tick().await;
                execution_count += 1;

                // Execute callback
                let result = callback().await;

                // Update formation
                {
                    let mut formations = formations_ref.write().await;
                    if let Some(formation) = formations.get_mut(&formation_id_clone) {
                        formation.execution_count = execution_count;
                        formation.last_execution = Some(current_timestamp());

                        // Deactivate if finished
                        if execution_count >= max_executions {
                            formation.is_active = false;
                        }
                    }
                }

                println!(
                    "[TIMEKEEPER] Interval {} execution {}: {}",
                    formation_id_clone,
                    execution_count,
                    result.message
                );
            }
        });

        Ok(formation_id)
    }

    /// Schedule complex (delay + interval + repeat)
    pub async fn schedule_complex(
        &self,
        id: &str,
        callback: TimeKeeperCallback,
        delay: Option<Duration>,
        interval: Duration,
        repeat: TimerRepeat
    ) -> Result<String, String> {
        let formation_id = format!("complex-{}-{}", id, current_timestamp());

        let formation = Formation {
            id: formation_id.clone(),
            action_id: id.to_string(),
            interval: interval.as_millis() as u64,
            delay: delay.map(|d| d.as_millis() as u64),
            repeat,
            created_at: current_timestamp(),
            last_execution: None,
            execution_count: 0,
            is_active: true,
        };

        // Store formation
        {
            let mut formations = self.formations.write().await;
            formations.insert(formation_id.clone(), formation);
        }

        // Execute with initial delay if specified
        let formations_ref = self.formations.clone();
        let formation_id_clone = formation_id.clone();

        tokio::spawn(async move {
            // Initial delay if specified
            if let Some(delay) = delay {
                tokio::time::sleep(delay).await;
            }

            let mut execution_count = 0;
            let max_executions = match repeat {
                TimerRepeat::Once => 1,
                TimerRepeat::Count(n) => n,
                TimerRepeat::Forever => u32::MAX,
            };

            while execution_count < max_executions {
                // Check if formation is still active
                let is_active = {
                    let formations = formations_ref.read().await;
                    formations
                        .get(&formation_id_clone)
                        .map(|f| f.is_active)
                        .unwrap_or(false)
                };

                if !is_active {
                    break;
                }

                execution_count += 1;

                // Execute callback
                let result = callback().await;

                // Update formation
                {
                    let mut formations = formations_ref.write().await;
                    if let Some(formation) = formations.get_mut(&formation_id_clone) {
                        formation.execution_count = execution_count;
                        formation.last_execution = Some(current_timestamp());

                        // Deactivate if finished
                        if execution_count >= max_executions {
                            formation.is_active = false;
                        }
                    }
                }

                println!(
                    "[TIMEKEEPER] Complex {} execution {}: {}",
                    formation_id_clone,
                    execution_count,
                    result.message
                );

                // Wait for next interval (unless this was the last execution)
                if execution_count < max_executions {
                    tokio::time::sleep(interval).await;
                }
            }
        });

        Ok(formation_id)
    }

    /// Cancel a formation
    pub async fn cancel_formation(&self, formation_id: &str) -> Result<(), String> {
        let mut formations = self.formations.write().await;
        if let Some(formation) = formations.get_mut(formation_id) {
            formation.is_active = false;
            println!("[TIMEKEEPER] Formation {} cancelled", formation_id);
            Ok(())
        } else {
            Err(format!("Formation {} not found", formation_id))
        }
    }

    /// Get formation status
    pub async fn get_formation(&self, formation_id: &str) -> Option<Formation> {
        let formations = self.formations.read().await;
        formations.get(formation_id).cloned()
    }

    /// List all active formations
    pub async fn list_active_formations(&self) -> Vec<Formation> {
        let formations = self.formations.read().await;
        formations
            .values()
            .filter(|f| f.is_active)
            .cloned()
            .collect()
    }
}

//=============================================================================
// GLOBAL TIMEKEEPER INSTANCE
//=============================================================================

static GLOBAL_TIMEKEEPER: OnceLock<Arc<TimeKeeper>> = OnceLock::new();

/// Get the global TimeKeeper instance
pub async fn get_timekeeper() -> Arc<TimeKeeper> {
    GLOBAL_TIMEKEEPER.get_or_init(|| { Arc::new(TimeKeeper::new()) }).clone()
}

//=============================================================================
// CONVENIENCE FUNCTIONS
//=============================================================================

/// Set a timeout (equivalent to setTimeout)
pub async fn set_timeout<F, Fut>(
    action_id: &str,
    callback: F,
    delay_ms: u64
)
    -> Result<String, String>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = CyreResponse> + Send + 'static
{
    let timekeeper = get_timekeeper().await;
    let wrapped_callback: TimeKeeperCallback = Box::new(move || {
        let fut = callback();
        Box::pin(fut)
    });

    timekeeper.schedule_timeout(action_id, wrapped_callback, Duration::from_millis(delay_ms)).await
}

/// Set an interval (equivalent to setInterval)
pub async fn set_interval<F, Fut>(
    action_id: &str,
    callback: F,
    interval_ms: u64,
    repeat: TimerRepeat
)
    -> Result<String, String>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = CyreResponse> + Send + 'static
{
    let timekeeper = get_timekeeper().await;
    let wrapped_callback: TimeKeeperCallback = Box::new(move || {
        let fut = callback();
        Box::pin(fut)
    });

    timekeeper.schedule_interval(
        action_id,
        wrapped_callback,
        Duration::from_millis(interval_ms),
        repeat
    ).await
}

/// Clear a timer (cancel formation)
pub async fn clear_timer(formation_id: &str) -> Result<(), String> {
    let timekeeper = get_timekeeper().await;
    timekeeper.cancel_formation(formation_id).await
}

/// Async delay function
pub async fn delay(duration_ms: u64) -> Result<(), String> {
    tokio::time::sleep(Duration::from_millis(duration_ms)).await;
    Ok(())
}

//=============================================================================
// FORMATION BUILDER (for compatibility)
//=============================================================================

pub struct FormationBuilder {
    action_id: String,
    interval: Option<u64>,
    delay: Option<u64>,
    repeat: TimerRepeat,
}

impl FormationBuilder {
    pub fn new(action_id: &str) -> Self {
        Self {
            action_id: action_id.to_string(),
            interval: None,
            delay: None,
            repeat: TimerRepeat::Once,
        }
    }

    pub fn interval(mut self, interval_ms: u64) -> Self {
        self.interval = Some(interval_ms);
        self
    }

    pub fn delay(mut self, delay_ms: u64) -> Self {
        self.delay = Some(delay_ms);
        self
    }

    pub fn repeat(mut self, repeat: TimerRepeat) -> Self {
        self.repeat = repeat;
        self
    }

    pub async fn schedule<F, Fut>(self, callback: F) -> Result<String, String>
        where
            F: Fn() -> Fut + Send + Sync + 'static,
            Fut: Future<Output = CyreResponse> + Send + 'static
    {
        let timekeeper = get_timekeeper().await;
        let wrapped_callback: TimeKeeperCallback = Box::new(move || {
            let fut = callback();
            Box::pin(fut)
        });

        match (self.delay, self.interval) {
            (Some(delay), None) => {
                timekeeper.schedule_timeout(
                    &self.action_id,
                    wrapped_callback,
                    Duration::from_millis(delay)
                ).await
            }
            (None, Some(interval)) => {
                timekeeper.schedule_interval(
                    &self.action_id,
                    wrapped_callback,
                    Duration::from_millis(interval),
                    self.repeat
                ).await
            }
            (Some(delay), Some(interval)) => {
                timekeeper.schedule_complex(
                    &self.action_id,
                    wrapped_callback,
                    Some(Duration::from_millis(delay)),
                    Duration::from_millis(interval),
                    self.repeat
                ).await
            }
            (None, None) => { Err("Either delay or interval must be specified".to_string()) }
        }
    }
}

//=============================================================================
// UTILITY FUNCTIONS
//=============================================================================

fn current_timestamp() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64
}
