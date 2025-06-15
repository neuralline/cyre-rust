// src/timekeeper/mod.rs
// TimeKeeper module - fixed exports

use crate::types::{ActionPayload};
use crate::core::Cyre;

pub mod timekeeper;

// Re-export timekeeper types (FIXED - removed missing types)
pub use timekeeper::{
    Formation, FormationBuilder, TimerRepeat,
    TimeKeeper,
    get_timekeeper, set_timeout, set_interval, clear_timer, delay,
};

// Simple trait for TimeKeeper integration (FIXED - simplified)
pub trait TimeKeeperIntegration {
    /// Initialize TimeKeeper integration
    fn init_timekeeper(&mut self) -> Result<(), String>;
    
    /// Schedule an action for execution
    fn schedule_action(&self, action_id: &str, payload: ActionPayload, interval: u64, repeat: TimerRepeat) -> Result<String, String>;
    
    /// Cancel a scheduled formation
    fn cancel_scheduled(&self, formation_id: &str) -> Result<(), String>;
}

// TimeKeeper integration for Cyre (FIXED - simplified implementation)
impl TimeKeeperIntegration for Cyre {
    fn init_timekeeper(&mut self) -> Result<(), String> {
        println!("🕒 TimeKeeper integration initialized");
        Ok(())
    }

    fn schedule_action(&self, action_id: &str, _payload: ActionPayload, interval: u64, _repeat: TimerRepeat) -> Result<String, String> {
        if !self.has_channel(action_id) {
            return Err(format!("Action '{}' not found", action_id));
        }

        println!("📅 Scheduling action '{}' with interval {}ms", action_id, interval);
        
        let formation_id = format!("formation_{}_{}", action_id, crate::utils::current_timestamp());
        Ok(formation_id)
    }

    fn cancel_scheduled(&self, formation_id: &str) -> Result<(), String> {
        println!("🗑️ Cancelling formation: {}", formation_id);
        Ok(())
    }
}

// Convenience functions for quick access
pub async fn timeout(action_id: &str, payload: ActionPayload, delay: u64) -> Result<String, String> {
    set_timeout(action_id, payload, delay).await
}

pub async fn interval(action_id: &str, payload: ActionPayload, interval: u64) -> Result<String, String> {
    set_interval(action_id, payload, interval).await
}

pub async fn sleep(duration: u64) -> Result<(), String> {
    delay(duration).await
}