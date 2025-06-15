// src/types/priority.rs
// Priority system implementation

use serde::{Serialize, Deserialize};

//=============================================================================
// PRIORITY SYSTEM
//=============================================================================

/// Execution priority levels for actions and channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Background = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl Priority {
    /// Check if this is a high priority level
    #[inline(always)]
    pub fn is_high_priority(self) -> bool {
        matches!(self, Priority::High | Priority::Critical)
    }
    
    /// Check if this is a critical priority level
    #[inline(always)]
    pub fn is_critical(self) -> bool {
        matches!(self, Priority::Critical)
    }
    
    /// Get priority as numeric value
    #[inline(always)]
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    
    /// Create priority from numeric value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Priority::Background),
            1 => Some(Priority::Low),
            2 => Some(Priority::Medium),
            3 => Some(Priority::High),
            4 => Some(Priority::Critical),
            _ => None,
        }
    }
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Background => write!(f, "Background"),
            Priority::Low => write!(f, "Low"),
            Priority::Medium => write!(f, "Medium"),
            Priority::High => write!(f, "High"),
            Priority::Critical => write!(f, "Critical"),
        }
    }
}