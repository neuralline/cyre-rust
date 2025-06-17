// src/talent/mod.rs
// Complete talent system implementation

pub mod registry;
pub mod types;
pub mod functions;

// Re-export all talent types
pub use registry::*;
pub use types::*;
pub use functions::*;