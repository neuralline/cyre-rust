// src/channel/channel.rs
// File location: src/channel/channel.rs
// Unified channel management using centralized context/state IO store

//=============================================================================
// IMPORTS
//=============================================================================

use crate::types::{ IO, AsyncHandler, CyreResponse, ActionPayload };
use crate::context::state::{ io, subscribers, ISubscriber };
use crate::context::sensor;
use crate::utils::current_timestamp;
use std::sync::Arc;
use serde_json::json;

//=============================================================================
// CHANNEL INFORMATION
//=============================================================================

/// Channel information structure
#[derive(Debug, Clone)]
pub struct ChannelInfo {
    pub id: String,
    pub name: Option<String>,
    pub has_handler: bool,
    pub config: IO,
    pub created_at: u64,
    pub last_execution: Option<u64>,
    pub execution_count: u64,
}

//=============================================================================
// UNIFIED CHANNEL MANAGER
//=============================================================================

/// Unified channel manager using centralized state only
pub struct ChannelManager;

impl ChannelManager {
    //=========================================================================
    // CHANNEL CREATION - USING CENTRALIZED IO STORE
    //=========================================================================

    /// Create new channel in centralized IO store
    pub fn create(config: IO) -> Result<(), String> {
        let action_id = config.id.clone();

        // Log creation attempt
        sensor::debug("channel", &format!("Creating channel: {}", action_id), false);

        // Validate configuration
        Self::validate_config(&config)?;

        // Store in centralized IO store
        match io::set(config) {
            Ok(_) => {
                sensor::success(
                    "channel",
                    &format!("Channel '{}' created successfully", action_id),
                    false
                );
                Ok(())
            }
            Err(e) => {
                sensor::error(
                    "channel",
                    &format!("Failed to create channel '{}': {}", action_id, e),
                    Some("ChannelManager::create"),
                    None
                );
                Err(e)
            }
        }
    }

    /// Register handler in centralized subscriber store
    pub fn register_handler<F>(action_id: &str, handler: F) -> Result<(), String>
        where
            F: Fn(
                ActionPayload
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = CyreResponse> + Send>> +
                Send +
                Sync +
                'static
    {
        // Check if channel exists in centralized IO store
        if io::get(action_id).is_none() {
            let error = format!("Channel '{}' does not exist", action_id);
            sensor::error("channel", &error, Some("ChannelManager::register_handler"), None);
            return Err(error);
        }

        // Wrap handler with proper type
        let async_handler: AsyncHandler = Arc::new(handler);

        // Create subscriber
        let subscriber = ISubscriber {
            id: action_id.to_string(),
            handler: async_handler,
            created_at: current_timestamp(),
        };

        // Store in centralized subscriber store
        match subscribers::add(subscriber) {
            Ok(_) => {
                sensor::success(
                    "channel",
                    &format!("Handler registered for '{}'", action_id),
                    false
                );
                Ok(())
            }
            Err(e) => {
                sensor::error(
                    "channel",
                    &format!("Failed to register handler for '{}': {}", action_id, e),
                    Some("ChannelManager::register_handler"),
                    None
                );
                Err(e)
            }
        }
    }

    /// Create channel and register handler in one operation
    pub fn create_with_handler<F>(config: IO, handler: F) -> Result<(), String>
        where
            F: Fn(
                ActionPayload
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = CyreResponse> + Send>> +
                Send +
                Sync +
                'static
    {
        let action_id = config.id.clone();

        // Create channel first
        Self::create(config)?;

        // Register handler
        Self::register_handler(&action_id, handler)?;

        sensor::success("channel", &format!("Channel '{}' created with handler", action_id), false);
        Ok(())
    }

    //=========================================================================
    // CHANNEL MANAGEMENT - USING CENTRALIZED STORES
    //=========================================================================

    /// Get channel information from centralized stores
    pub fn get_info(action_id: &str) -> Option<ChannelInfo> {
        // Get config from centralized IO store
        let config = io::get(action_id)?;

        // Check if handler exists in centralized subscriber store
        let has_handler = subscribers::get(action_id).is_some();

        Some(ChannelInfo {
            id: action_id.to_string(),
            name: config.name.clone(),
            has_handler,
            created_at: config.timestamp.unwrap_or(current_timestamp()),
            last_execution: config._last_exec_time,
            execution_count: config._execution_count,
            config,
        })
    }

    /// Update channel configuration in centralized IO store
    pub fn update(config: IO) -> Result<(), String> {
        let action_id = config.id.clone();

        // Validate configuration
        Self::validate_config(&config)?;

        // Update in centralized IO store
        match io::set(config) {
            Ok(_) => {
                sensor::info(
                    "channel",
                    &format!("Channel '{}' updated successfully", action_id),
                    false
                );
                Ok(())
            }
            Err(e) => {
                sensor::error(
                    "channel",
                    &format!("Failed to update channel '{}': {}", action_id, e),
                    Some("ChannelManager::update"),
                    None
                );
                Err(e)
            }
        }
    }

    /// Remove channel from all centralized stores
    pub fn forget(action_id: &str) -> Result<(), String> {
        sensor::debug("channel", &format!("Forgetting channel: {}", action_id), false);

        // Remove handler from centralized subscriber store
        subscribers::forget(action_id);

        // Remove config from centralized IO store
        let removed = io::forget(action_id);

        if removed {
            sensor::success(
                "channel",
                &format!("Channel '{}' forgotten successfully", action_id),
                false
            );
            Ok(())
        } else {
            let error = format!("Channel '{}' not found", action_id);
            sensor::error("channel", &error, Some("ChannelManager::forget"), None);
            Err(error)
        }
    }

    /// Modify channel configuration using a function
    pub fn modify<F>(action_id: &str, modifier: F) -> Result<(), String> where F: FnOnce(&mut IO) {
        // Get current config from centralized store
        let mut config = match io::get(action_id) {
            Some(config) => config,
            None => {
                let error = format!("Channel '{}' not found", action_id);
                sensor::error("channel", &error, Some("ChannelManager::modify"), None);
                return Err(error);
            }
        };

        // Apply modifier
        modifier(&mut config);

        // Update timestamp
        config.timestamp = Some(current_timestamp());

        // Store back in centralized store
        Self::update(config)
    }

    //=========================================================================
    // EXISTENCE CHECKS - USING CENTRALIZED STORES
    //=========================================================================

    /// Check if channel exists in centralized IO store
    pub fn exists(action_id: &str) -> bool {
        io::get(action_id).is_some()
    }

    /// Check if handler exists in centralized subscriber store
    pub fn has_handler(action_id: &str) -> bool {
        subscribers::get(action_id).is_some()
    }

    //=========================================================================
    // LISTING OPERATIONS - USING CENTRALIZED STORES
    //=========================================================================

    /// List all channel IDs from centralized IO store
    pub fn list_all() -> Vec<String> {
        io::get_all()
            .into_iter()
            .map(|config| config.id)
            .collect()
    }

    /// List channels by group from centralized IO store
    pub fn list_by_group(group: &str) -> Vec<String> {
        io::get_all()
            .into_iter()
            .filter(|config| config.group.as_ref().map_or(false, |g| g == group))
            .map(|config| config.id)
            .collect()
    }

    /// List channels by path prefix from centralized IO store
    pub fn list_by_path(path_prefix: &str) -> Vec<String> {
        io::get_all()
            .into_iter()
            .filter(|config| config.path.as_ref().map_or(false, |p| p.starts_with(path_prefix)))
            .map(|config| config.id)
            .collect()
    }

    /// Get all channel information from centralized stores
    pub fn get_all_info() -> Vec<ChannelInfo> {
        io::get_all()
            .into_iter()
            .filter_map(|config| Self::get_info(&config.id))
            .collect()
    }

    //=========================================================================
    // SYSTEM SUMMARY - USING CENTRALIZED STORES
    //=========================================================================

    /// Get system summary from centralized stores
    pub fn get_system_summary() -> serde_json::Value {
        let all_configs = io::get_all();
        let total_channels = all_configs.len();
        let total_handlers = subscribers::get_all().len();

        // Analyze channel types
        let with_throttle = all_configs
            .iter()
            .filter(|c| c.throttle.is_some())
            .count();
        let with_debounce = all_configs
            .iter()
            .filter(|c| c.debounce.is_some())
            .count();
        let with_change_detection = all_configs
            .iter()
            .filter(|c| c._has_change_detection)
            .count();
        let fast_path_eligible = all_configs
            .iter()
            .filter(|c| c.is_fast_path_eligible())
            .count();

        json!({
            "total_channels": total_channels,
            "total_handlers": total_handlers,
            "coverage": if total_channels > 0 {
                (total_handlers as f64 / total_channels as f64) * 100.0
            } else {
                0.0
            },
            "protection_analysis": {
                "with_throttle": with_throttle,
                "with_debounce": with_debounce,
                "with_change_detection": with_change_detection,
                "fast_path_eligible": fast_path_eligible,
                "fast_path_percentage": if total_channels > 0 {
                    (fast_path_eligible as f64 / total_channels as f64) * 100.0
                } else {
                    0.0
                }
            }
        })
    }

    //=========================================================================
    // VALIDATION - PRIVATE HELPER
    //=========================================================================

    /// Validate channel configuration
    fn validate_config(config: &IO) -> Result<(), String> {
        // Check ID
        if config.id.trim().is_empty() {
            return Err("Channel ID cannot be empty".to_string());
        }

        // Check for invalid characters in ID
        if config.id.contains(char::is_whitespace) {
            return Err("Channel ID cannot contain whitespace".to_string());
        }

        // Validate throttle value
        if let Some(throttle) = config.throttle {
            if throttle == 0 {
                return Err("Throttle value must be greater than 0".to_string());
            }
        }

        // Validate debounce value
        if let Some(debounce) = config.debounce {
            if debounce == 0 {
                return Err("Debounce value must be greater than 0".to_string());
            }
        }

        // Validate max_wait with debounce
        if let (Some(_), Some(max_wait)) = (config.debounce, config.max_wait) {
            if max_wait == 0 {
                return Err("Max wait value must be greater than 0".to_string());
            }
        }

        Ok(())
    }
}

//=============================================================================
// BACKWARD COMPATIBILITY FUNCTIONS
//=============================================================================

/// Backward compatibility functions that delegate to the unified implementation

pub fn create_channel(config: IO) -> Result<(), String> {
    ChannelManager::create(config)
}

pub fn register_handler<F>(action_id: &str, handler: F) -> Result<(), String>
    where
        F: Fn(
            ActionPayload
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = CyreResponse> + Send>> +
            Send +
            Sync +
            'static
{
    ChannelManager::register_handler(action_id, handler)
}

pub fn get_channel_info(action_id: &str) -> Option<ChannelInfo> {
    ChannelManager::get_info(action_id)
}

pub fn channel_exists(action_id: &str) -> bool {
    ChannelManager::exists(action_id)
}

pub fn handler_exists(action_id: &str) -> bool {
    ChannelManager::has_handler(action_id)
}

pub fn list_all_channels() -> Vec<String> {
    ChannelManager::list_all()
}

pub fn get_system_summary() -> serde_json::Value {
    ChannelManager::get_system_summary()
}
