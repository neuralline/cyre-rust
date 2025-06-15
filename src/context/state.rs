// src/context/state.rs
// Centralized state management - fixed naming conventions

use std::sync::{Mutex, OnceLock};
use crate::types::Timer;

/// Timeline store for managing timers
#[derive(Debug, Default)]
pub struct TimelineStore {
    timers: Vec<Timer>,
}

impl TimelineStore {
    pub fn new() -> Self {
        Self {
            timers: Vec::new(),
        }
    }

    pub fn get_all(&self) -> Vec<Timer> {
        self.timers.clone()
    }

    pub fn get_active(&self) -> Vec<Timer> {
        self.timers.iter()
            .filter(|timer| timer.is_active)
            .cloned()
            .collect()
    }

    pub fn add(&mut self, timer: Timer) {
        if let Some(pos) = self.timers.iter().position(|t| t.id == timer.id) {
            self.timers[pos] = timer;
        } else {
            self.timers.push(timer);
        }
    }

    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.timers.iter().position(|t| t.id == id) {
            self.timers.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get(&self, id: &str) -> Option<Timer> {
        self.timers.iter()
            .find(|t| t.id == id)
            .cloned()
    }

    pub fn forget(&mut self, id: &str) -> bool {
        if let Some(pos) = self.timers.iter().position(|t| t.id == id) {
            self.timers.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn update(&mut self, id: &str, timer: Timer) -> bool {
        if let Some(pos) = self.timers.iter().position(|t| t.id == id) {
            self.timers[pos] = timer;
            true
        } else {
            false
        }
    }
}

// Global timeline store instance using OnceLock (no external dependency needed)
static GLOBAL_TIMELINE: OnceLock<Mutex<TimelineStore>> = OnceLock::new();

/// Get the global timeline store
pub fn get_timeline() -> &'static Mutex<TimelineStore> {
    GLOBAL_TIMELINE.get_or_init(|| {
        Mutex::new(TimelineStore::new())
    })
}

/// Timeline interface for easy access
pub struct Timeline;

impl Timeline {
    pub fn add(timer: Timer) {
        let store = get_timeline();
        if let Ok(mut timeline_store) = store.lock() {
            timeline_store.add(timer);
        }
    }

    pub fn get(id: &str) -> Option<Timer> {
        let store = get_timeline();
        if let Ok(timeline_store) = store.lock() {
            timeline_store.get(id)
        } else {
            None
        }
    }

    pub fn forget(id: &str) -> bool {
        let store = get_timeline();
        if let Ok(mut timeline_store) = store.lock() {
            timeline_store.forget(id)
        } else {
            false
        }
    }

    pub fn clear() {
        let store = get_timeline();
        if let Ok(mut timeline_store) = store.lock() {
            timeline_store.timers.clear();
        }
    }

    pub fn get_all() -> Vec<Timer> {
        let store = get_timeline();
        if let Ok(timeline_store) = store.lock() {
            timeline_store.get_all()
        } else {
            Vec::new()
        }
    }

    pub fn get_active() -> Vec<Timer> {
        let store = get_timeline();
        if let Ok(timeline_store) = store.lock() {
            timeline_store.get_active()
        } else {
            Vec::new()
        }
    }
}

// Export the timeline instance - FIXED: uppercase static name
pub static TIMELINE: Timeline = Timeline;