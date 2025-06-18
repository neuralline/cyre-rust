// src/channel/channel.rs
// File location: src/channel/channel.rs
// Simplified channel management using centralized context/state

//=============================================================================
// IMPORTS
//=============================================================================

use crate::types::{ IO, AsyncHandler, CyreResponse, ActionPayload };
use crate::context::state::{ io, subscribers, ISubscriber };
use crate::context::sensor;
use crate::utils::current_timestamp;
use std::sync::Arc;
use serde_json::{ Value as JsonValue };

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
    pub latest_payload: Option<JsonValue>,
}

//=============================================================================
// UNIFIED CHANNEL MANAGER
//=============================================================================

/// Unified channel manager using centralized state only
pub struct ChannelManager;

impl ChannelManager {
    /// Create new channel in centralized IO store
    pub fn create(config: IO) -> Result<(), String> {
        let action_id = config.id.clone();

        // Log creation attempt
        sensor::debug("channel", &format!("Creating channel: {}", action_id), false);

        // Validate configuration
        Self::validate_config(&config)?;

        // Store in centralized IO store with correct signature
        match io::set(action_id.clone(), config) {
            Ok(_) => Ok(()),
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

        // Create subscriber with all required fields
        let subscriber = ISubscriber {
            id: action_id.to_string(),
            handler: async_handler,
            active: true,
            created_at: current_timestamp(),
        };

        // Store in centralized subscriber store
        match subscribers::set(action_id.to_string(), subscriber) {
            Ok(_) => Ok(()),
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
            config: config.clone(),
            created_at: config.timestamp.unwrap_or(current_timestamp()),
            last_execution: config._last_exec_time,
            execution_count: config._execution_count,
            latest_payload: None, // Simplified - no payload tracking for now
        })
    }

    /// Update channel configuration in centralized IO store
    pub fn update(config: IO) -> Result<(), String> {
        let action_id = config.id.clone();

        // Validate configuration
        Self::validate_config(&config)?;

        // Update in centralized IO store
        match io::set(action_id.clone(), config) {
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

    /// Check if channel exists in centralized IO store
    pub fn exists(action_id: &str) -> bool {
        io::get(action_id).is_some()
    }

    /// Check if handler exists in centralized subscriber store
    pub fn has_handler(action_id: &str) -> bool {
        subscribers::get(action_id).is_some()
    }

    /// List all channels
    pub fn list_all_channels() -> Vec<ChannelInfo> {
        io::get_all()
            .into_iter()
            .filter_map(|config| Self::get_info(&config.id))
            .collect()
    }

    /// Validate channel configuration
    fn validate_config(config: &IO) -> Result<(), String> {
        if config.id.trim().is_empty() {
            return Err("Channel ID cannot be empty".to_string());
        }
        Ok(())
    }
}

//=============================================================================
// PUBLIC API FUNCTIONS
//=============================================================================

/// Create a new channel
pub fn create_channel(config: IO) -> Result<(), String> {
    ChannelManager::create(config)
}

/// Create channel and register handler
pub fn channel<F>(config: IO, handler: F) -> Result<(), String>
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
    ChannelManager::create(config)?;

    // Register handler
    ChannelManager::register_handler(&action_id, handler)?;

    Ok(())
}

/// Get channel info
pub fn get_channel_info(action_id: &str) -> Option<ChannelInfo> {
    ChannelManager::get_info(action_id)
}

/// Check if channel exists
pub fn channel_exists(action_id: &str) -> bool {
    ChannelManager::exists(action_id)
}

/// List all channels
pub fn list_all_channels() -> Vec<ChannelInfo> {
    ChannelManager::list_all_channels()
}

/// Remove a channel
pub fn remove_channel(action_id: &str) -> Result<(), String> {
    ChannelManager::forget(action_id)
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CyreResponse;
    use std::pin::Pin;
    use std::future::Future;

    fn mock_handler() -> AsyncHandler {
        Arc::new(|_payload| {
            Box::pin(async move { CyreResponse {
                    ok: true,
                    payload: serde_json::json!({}),
                    message: "Mock response".to_string(),
                    error: None,
                    timestamp: current_timestamp(),
                    metadata: None,
                } }) as Pin<Box<dyn Future<Output = CyreResponse> + Send>>
        })
    }

    #[test]
    fn test_channel_creation() {
        // Clear state first
        io::clear();
        subscribers::clear();

        let config = IO::new("test-channel");
        let result = ChannelManager::create(config);
        assert!(result.is_ok());
        assert!(ChannelManager::exists("test-channel"));
    }

    #[test]
    fn test_channel_info() {
        // Clear state first
        io::clear();
        subscribers::clear();

        let config = IO::new("test-info-channel");
        let _ = ChannelManager::create(config);

        let info = ChannelManager::get_info("test-info-channel");
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.id, "test-info-channel");
        assert!(!info.has_handler); // No handler registered yet
    }

    #[test]
    fn test_channel_with_handler() {
        // Clear state first
        io::clear();
        subscribers::clear();

        let config = IO::new("test-handler-channel");
        let result = channel(config, |_payload| {
            Box::pin(async move { CyreResponse {
                    ok: true,
                    payload: serde_json::json!({}),
                    message: "Test response".to_string(),
                    error: None,
                    timestamp: current_timestamp(),
                    metadata: None,
                } })
        });

        assert!(result.is_ok());
        assert!(ChannelManager::exists("test-handler-channel"));
        assert!(ChannelManager::has_handler("test-handler-channel"));
    }
}
