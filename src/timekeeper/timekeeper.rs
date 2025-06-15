// src/timekeeper/timekeeper.rs - FIXED VERSION
// Simplified TimeKeeper without missing dependencies

use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use crate::types::{ActionPayload, Priority};
use crate::utils::current_timestamp;

//=============================================================================
// FORMATION TYPES
//=============================================================================

/// Timer repeat configuration
#[derive(Debug, Clone)]
pub enum TimerRepeat {
    Once,
    Forever,
    Count(u64),
}

/// Formation represents a scheduled action execution
#[derive(Debug, Clone)]
pub struct Formation {
    pub id: String,
    pub action_id: String,
    pub payload: ActionPayload,
    pub interval: u64,
    pub repeat: TimerRepeat,
    pub delay: Option<u64>,
    pub priority: Priority,
    pub created_at: u64,
    pub next_execution: u64,
    pub execution_count: u64,
    pub is_active: bool,
}

impl Formation {
    pub fn new(
        id: String,
        action_id: String,
        payload: ActionPayload,
        interval: u64,
        repeat: TimerRepeat,
        delay: Option<u64>,
        priority: Priority,
    ) -> Self {
        let now = current_timestamp();
        let next_time = now + delay.unwrap_or(interval);
        
        Self {
            id,
            action_id,
            payload,
            interval,
            repeat,
            delay,
            priority,
            created_at: now,
            next_execution: next_time,
            execution_count: 0,
            is_active: true,
        }
    }
}

//=============================================================================
// SIMPLIFIED TIMEKEEPER (No external dependencies)
//=============================================================================

/// TimeKeeper for scheduling actions
#[derive(Debug)]
pub struct TimeKeeper {
    total_formations: AtomicU64,
    active_formations: AtomicU64,
    is_running: std::sync::atomic::AtomicBool,
    formations: Arc<std::sync::Mutex<std::collections::HashMap<String, Formation>>>,
}

impl TimeKeeper {
    pub fn new() -> Self {
        Self {
            total_formations: AtomicU64::new(0),
            active_formations: AtomicU64::new(0),
            is_running: std::sync::atomic::AtomicBool::new(false),
            formations: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Schedule a formation
    pub async fn schedule_formation(&self, formation: Formation) -> Result<(), String> {
        let formation_id = formation.id.clone();
        
        // Store formation
        {
            let mut formations = self.formations.lock().unwrap();
            formations.insert(formation_id.clone(), formation);
        }
        
        self.total_formations.fetch_add(1, Ordering::Relaxed);
        self.active_formations.fetch_add(1, Ordering::Relaxed);
        
        // Start ticker if needed
        if !self.is_running.load(Ordering::Relaxed) {
            self.start_ticker().await;
        }
        
        println!("📅 Formation {} scheduled", formation_id);
        Ok(())
    }

    /// Start the execution ticker
    async fn start_ticker(&self) {
        if self.is_running.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            let formations = Arc::clone(&self.formations);
            
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
                
                loop {
                    interval.tick().await;
                    
                    let now = current_timestamp();
                    let mut to_execute = Vec::new();
                    
                    // Check for due formations
                    {
                        let formations_guard = formations.lock().unwrap();
                        for formation in formations_guard.values() {
                            if formation.is_active && now >= formation.next_execution {
                                to_execute.push(formation.clone());
                            }
                        }
                    }
                    
                    // Execute due formations
                    for mut formation in to_execute {
                        println!("⏰ Executing formation: {}", formation.id);
                        
                        formation.execution_count += 1;
                        formation.next_execution = now + formation.interval;
                        
                        // Handle repeat logic
                        match formation.repeat {
                            TimerRepeat::Once => {
                                formation.is_active = false;
                            },
                            TimerRepeat::Forever => {
                                // Continue running
                            },
                            TimerRepeat::Count(count) => {
                                if formation.execution_count >= count {
                                    formation.is_active = false;
                                }
                            }
                        }
                        
                        // Update stored formation
                        {
                            let mut formations_guard = formations.lock().unwrap();
                            if formation.is_active {
                                formations_guard.insert(formation.id.clone(), formation);
                            } else {
                                formations_guard.remove(&formation.id);
                            }
                        }
                    }
                    
                    // Check if we should stop
                    let active_count = {
                        let formations_guard = formations.lock().unwrap();
                        formations_guard.values().filter(|f| f.is_active).count()
                    };
                    
                    if active_count == 0 {
                        break;
                    }
                }
            });
        }
    }

    /// Cancel a scheduled formation
    pub async fn cancel_formation(&self, formation_id: &str) -> Result<(), String> {
        let mut formations = self.formations.lock().unwrap();
        
        if formations.remove(formation_id).is_some() {
            self.active_formations.fetch_sub(1, Ordering::Relaxed);
            Ok(())
        } else {
            Err(format!("Formation {} not found", formation_id))
        }
    }

    /// Get formation status
    pub async fn get_formation_status(&self, formation_id: &str) -> Option<Formation> {
        let formations = self.formations.lock().unwrap();
        formations.get(formation_id).cloned()
    }

    /// Get all active formations
    pub async fn get_active_formations(&self) -> Vec<Formation> {
        let formations = self.formations.lock().unwrap();
        formations.values()
            .filter(|f| f.is_active)
            .cloned()
            .collect()
    }

    /// Get TimeKeeper statistics
    pub fn get_stats(&self) -> serde_json::Value {
        let formations = self.formations.lock().unwrap();
        let active_count = formations.values().filter(|f| f.is_active).count();
        
        serde_json::json!({
            "total_formations": self.total_formations.load(Ordering::Relaxed),
            "active_formations": active_count,
            "completed_formations": formations.len() - active_count,
            "is_running": self.is_running.load(Ordering::Relaxed)
        })
    }
}

//=============================================================================
// FORMATION BUILDER
//=============================================================================

pub struct FormationBuilder {
    action_id: String,
    payload: ActionPayload,
    interval: u64,
    repeat: TimerRepeat,
    delay: Option<u64>,
    priority: Priority,
}

impl FormationBuilder {
    pub fn new(action_id: impl Into<String>, payload: ActionPayload) -> Self {
        Self {
            action_id: action_id.into(),
            payload,
            interval: 0,
            repeat: TimerRepeat::Once,
            delay: None,
            priority: Priority::Medium,
        }
    }

    pub fn interval(mut self, interval_ms: u64) -> Self {
        self.interval = interval_ms;
        self
    }

    pub fn repeat(mut self, repeat: TimerRepeat) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn delay(mut self, delay_ms: u64) -> Self {
        self.delay = Some(delay_ms);
        self
    }

    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub async fn schedule(self) -> Result<String, String> {
        let formation_id = format!("formation_{}_{}", self.action_id, current_timestamp());
        
        let formation = Formation::new(
            formation_id.clone(),
            self.action_id,
            self.payload,
            self.interval,
            self.repeat,
            self.delay,
            self.priority,
        );

        let timekeeper = get_timekeeper().await;
        timekeeper.schedule_formation(formation).await?;
        
        Ok(formation_id)
    }
}

//=============================================================================
// GLOBAL TIMEKEEPER INSTANCE
//=============================================================================

static GLOBAL_TIMEKEEPER: tokio::sync::OnceCell<Arc<TimeKeeper>> = tokio::sync::OnceCell::const_new();

/// Get the global TimeKeeper instance
pub async fn get_timekeeper() -> Arc<TimeKeeper> {
    GLOBAL_TIMEKEEPER.get_or_init(|| async {
        Arc::new(TimeKeeper::new())
    }).await.clone()
}

//=============================================================================
// CONVENIENCE FUNCTIONS
//=============================================================================

/// Set a timeout (equivalent to setTimeout)
pub async fn set_timeout(action_id: &str, payload: ActionPayload, delay_ms: u64) -> Result<String, String> {
    FormationBuilder::new(action_id, payload)
        .delay(delay_ms)
        .repeat(TimerRepeat::Once)
        .schedule()
        .await
}

/// Set an interval (equivalent to setInterval)
pub async fn set_interval(action_id: &str, payload: ActionPayload, interval_ms: u64) -> Result<String, String> {
    FormationBuilder::new(action_id, payload)
        .interval(interval_ms)
        .repeat(TimerRepeat::Forever)
        .schedule()
        .await
}

/// Clear a timer (cancel formation)
pub async fn clear_timer(formation_id: &str) -> Result<(), String> {
    let timekeeper = get_timekeeper().await;
    timekeeper.cancel_formation(formation_id).await
}

/// Async delay function
pub async fn delay(duration_ms: u64) -> Result<(), String> {
    tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms)).await;
    Ok(())
}