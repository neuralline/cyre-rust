// src/lib.rs - MINIMAL VERSION
#![allow(dead_code)]

pub mod core;
pub mod types;
pub mod context;
pub mod breathing;
pub mod config;
pub mod utils;
pub mod schema;
pub mod timekeeper;

pub mod prelude {
    pub use crate::core::Cyre;
    pub use crate::types::*;
    pub use crate::utils::current_timestamp;
    pub use serde_json::{ json, Value };
}
