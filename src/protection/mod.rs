// src/protection/mod.rs
// Protection mechanisms module - FIXED exports

pub mod state;

// Re-export all protection types explicitly
pub use state::{ ProtectionState, ProtectionType, ProtectionBuilder };
