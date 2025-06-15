// src/timekeeper/timekeeper.rs - FIXED VERSION
// TimeKeeper using centralized timeline store

use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use crate::types::{ActionPayload, Priority};
use crate::utils::current_timestamp;

//=============================================================================
// FORMATION TYPES (Updated for centralized state)
//=============================================================================

/// Timer repeat configuration
#[derive(Debug, Clone)]
pub enum TimerRepeat {
    Once,
    Forever,
    Count(u64),
}

/// Formation represents a scheduled action execution (matches Timer interface)
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
    
    // Match Timer interface from state.ts
    pub start_time: u64,
    pub duration: u64,
    pub original_duration: u64,
    pub callback: Option<String>, // Store action_id instead of function
    pub last_execution_time: u64,
    pub next_execution_time: u64,
    pub status: String, // "active", "paused", "completed"
    pub has_executed_once: bool,
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
            id: id.clone(),
            action_id: action_id.clone(),
            payload,
            interval,
            repeat,
            delay,
            priority,
            created_at: now,
            next_execution: next_time,
            execution_count: 0,
            is_active: true,
            
            // Timer interface compatibility
            start_time: now,
            duration: interval,
            original_duration: interval,
            callback: Some(action_id),
            last_execution_time: 0,
            next_execution_time: next_time,
            status: "active".to_string(),
            has_executed_once: false,
        }
    }
}

//=============================================================================
// CENTRALIZED TIMEKEEPER (No local state!)
//=============================================================================

/// TimeKeeper that uses centralized timeline store
#[derive(Debug)]
pub struct TimeKeeper {
    // ✅ NO local state - everything goes through centralized stores
    total_formations: AtomicU64,
    active_formations: AtomicU64,
    is_running: std::sync::atomic::AtomicBool,
}

impl TimeKeeper {
    pub fn new() -> Self {
        Self {
            total_formations: AtomicU64::new(0),
            active_formations: AtomicU64::new(0),
            is_running: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Schedule a formation using centralized timeline store
    pub async fn schedule_formation(&self, formation: Formation) -> Result<(), String> {
        let formation_id = formation.id.clone();
        
        // ✅ Use centralized timeline store
        {
            let timeline = crate::context::state::timeline;
            
            // Convert Formation to Timer for timeline store
            let timer = crate::types::Timer {
                id: formation.id.clone(),
                start_time: formation.start_time,
                duration: formation.duration,
                original_duration: formation.original_duration,
                callback: Box::pin(async move {
                    // This will trigger actual Cyre action execution
                    self.execute_formation_action(&formation.action_id, formation.payload.clone()).await
                }),
                repeat: match formation.repeat {
                    TimerRepeat::Once => Some(1),
                    TimerRepeat::Forever => Some(-1),
                    TimerRepeat::Count(n) => Some(n as i32),
                },
                execution_count: formation.execution_count,
                last_execution_time: formation.last_execution_time,
                next_execution_time: formation.next_execution_time,
                is_in_recuperation: false,
                status: formation.status.clone(),
                is_active: formation.is_active,
                delay: formation.delay,
                interval: Some(formation.interval),
                has_executed_once: formation.has_executed_once,
                priority: formation.priority,
                // Add any other Timer fields...
                metrics: None, // Initialize with default metrics
            };
            
            timeline.add(timer);
        }
        
        self.total_formations.fetch_add(1, Ordering::Relaxed);
        self.active_formations.fetch_add(1, Ordering::Relaxed);
        
        // ✅ TimeKeeper itself starts the timeline ticker if needed
        if !self.is_running.load(Ordering::Relaxed) {
            self.start_timeline_ticker().await;
        }
        
        Ok(())
    }

    /// Start the timeline ticker (centralized timing)
    async fn start_timeline_ticker(&self) {
        if self.is_running.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(10));
                
                loop {
                    interval.tick().await;
                    
                    let now = current_timestamp();
                    
                    // ✅ Get active timers from centralized timeline store
                    let active_timers = {
                        let timeline = crate::context::state::timeline;
                        timeline.getActive()
                    };
                    
                    let mut executed_any = false;
                    
                    for timer in active_timers {
                        if timer.is_active && now >= timer.next_execution_time {
                            // Execute the timer callback (which calls Cyre action)
                            let callback = timer.callback;
                            tokio::spawn(async move {
                                callback().await;
                            });
                            
                            // Update timer for next execution
                            let mut updated_timer = timer.clone();
                            updated_timer.execution_count += 1;
                            updated_timer.last_execution_time = now;
                            updated_timer.has_executed_once = true;
                            
                            // Handle repeat logic
                            match updated_timer.repeat {
                                Some(1) => {
                                    // Execute once, then deactivate
                                    updated_timer.is_active = false;
                                    updated_timer.status = "completed".to_string();
                                },
                                Some(-1) => {
                                    // Repeat forever
                                    updated_timer.next_execution_time = now + updated_timer.duration;
                                },
                                Some(n) if n > 1 => {
                                    // Repeat n times
                                    updated_timer.repeat = Some(n - 1);
                                    updated_timer.next_execution_time = now + updated_timer.duration;
                                },
                                _ => {
                                    // Complete
                                    updated_timer.is_active = false;
                                    updated_timer.status = "completed".to_string();
                                }
                            }
                            
                            // ✅ Update centralized timeline store
                            let timeline = crate::context::state::timeline;
                            timeline.add(updated_timer);
                            
                            executed_any = true;
                        }
                    }
                    
                    // Stop ticker if no active formations
                    if !executed_any && active_timers.is_empty() {
                        break;
                    }
                }
            });
        }
    }

    /// Execute formation action through Cyre (integration point)
    async fn execute_formation_action(&self, action_id: &str, payload: ActionPayload) {
        // ✅ This is where TimeKeeper integrates with Cyre
        // Need access to Cyre instance to call the action
        
        println!("⏰ TimeKeeper executing action: {} with payload: {}", action_id, payload);
        
        // TODO: Get Cyre instance and call the action
        // This requires passing Cyre reference to TimeKeeper
        // OR using a global/static Cyre instance
        // OR using a callback registration system
    }

    /// Cancel a scheduled formation
    pub async fn cancel_formation(&self, formation_id: &str) -> Result<(), String> {
        // ✅ Use centralized timeline store
        let timeline = crate::context::state::timeline;
        let success = timeline.forget(formation_id);
        
        if success {
            self.active_formations.fetch_sub(1, Ordering::Relaxed);
            Ok(())
        } else {
            Err(format!("Formation {} not found", formation_id))
        }
    }

    /// Get formation status from centralized store
    pub async fn get_formation_status(&self, formation_id: &str) -> Option<Formation> {
        let timeline = crate::context::state::timeline;
        
        if let Some(timer) = timeline.get(formation_id) {
            // Convert Timer back to Formation
            Some(Formation {
                id: timer.id.clone(),
                action_id: timer.callback.unwrap_or_else(|| "unknown".to_string()),
                payload: serde_json::Value::Null, // Payload not stored in Timer
                interval: timer.duration,
                repeat: match timer.repeat {
                    Some(-1) => TimerRepeat::Forever,
                    Some(1) => TimerRepeat::Once,
                    Some(n) if n > 1 => TimerRepeat::Count(n as u64),
                    _ => TimerRepeat::Once,
                },
                delay: timer.delay,
                priority: timer.priority,
                created_at: timer.start_time,
                next_execution: timer.next_execution_time,
                execution_count: timer.execution_count,
                is_active: timer.is_active,
                
                // Timer interface fields
                start_time: timer.start_time,
                duration: timer.duration,
                original_duration: timer.original_duration,
                callback: Some(timer.id.clone()),
                last_execution_time: timer.last_execution_time,
                next_execution_time: timer.next_execution_time,
                status: timer.status,
                has_executed_once: timer.has_executed_once,
            })
        } else {
            None
        }
    }

    /// Get all active formations from centralized store
    pub async fn get_active_formations(&self) -> Vec<Formation> {
        let timeline = crate::context::state::timeline;
        let active_timers = timeline.getActive();
        
        active_timers.into_iter()
            .filter_map(|timer| {
                // Convert Timer to Formation
                Some(Formation {
                    id: timer.id.clone(),
                    action_id: timer.callback.unwrap_or_else(|| "unknown".to_string()),
                    payload: serde_json::Value::Null,
                    interval: timer.duration,
                    repeat: match timer.repeat {
                        Some(-1) => TimerRepeat::Forever,
                        Some(1) => TimerRepeat::Once,
                        Some(n) if n > 1 => TimerRepeat::Count(n as u64),
                        _ => TimerRepeat::Once,
                    },
                    delay: timer.delay,
                    priority: timer.priority,
                    created_at: timer.start_time,
                    next_execution: timer.next_execution_time,
                    execution_count: timer.execution_count,
                    is_active: timer.is_active,
                    
                    start_time: timer.start_time,
                    duration: timer.duration,
                    original_duration: timer.original_duration,
                    callback: Some(timer.id.clone()),
                    last_execution_time: timer.last_execution_time,
                    next_execution_time: timer.next_execution_time,
                    status: timer.status,
                    has_executed_once: timer.has_executed_once,
                })
            })
            .collect()
    }

    /// Get TimeKeeper statistics from centralized store
    pub fn get_stats(&self) -> serde_json::Value {
        let timeline = crate::context::state::timeline;
        let all_timers = timeline.getAll();
        let active_count = timeline.getActive().len();
        
        serde_json::json!({
            "total_formations": self.total_formations.load(Ordering::Relaxed),
            "active_formations": active_count,
            "completed_formations": all_timers.len() - active_count,
            "is_running": self.is_running.load(Ordering::Relaxed),
            "centralized_store": true
        })
    }
}

//=============================================================================
// FORMATION BUILDER (Updated to use centralized state)
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

        // ✅ Use centralized TimeKeeper (no local state)
        let timekeeper = get_timekeeper().await;
        timekeeper.schedule_formation(formation).await?;
        
        Ok(formation_id)
    }
}

//=============================================================================
// GLOBAL TIMEKEEPER INSTANCE (Singleton pattern)
//=============================================================================

static GLOBAL_TIMEKEEPER: tokio::sync::OnceCell<Arc<TimeKeeper>> = tokio::sync::OnceCell::const_new();

/// Get the global TimeKeeper instance
pub async fn get_timekeeper() -> Arc<TimeKeeper> {
    GLOBAL_TIMEKEEPER.get_or_init(|| async {
        Arc::new(TimeKeeper::new())
    }).await.clone()
}

//=============================================================================
// CONVENIENCE FUNCTIONS (Updated)
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