// examples/enhanced_state_demo.rs
// Demo of the enhanced Rust state management system

use cyre_rust::prelude::*;
use cyre_rust::context::state::{
    io,
    subscribers,
    timeline,
    stores,
    MetricsUpdate,
    MetricsOps,
    PayloadStateOps,
};
use cyre_rust::context::sensor;
use serde_json::json;
use std::time::Duration;

/*

      C.Y.R.E - S.T.A.T.E - D.E.M.O
      
      Enhanced state management demonstration:
      - Clean separation of concerns (IO config vs payload data)
      - Thread-safe stores with sensor logging
      - Metrics tracking and health monitoring
      - Beautiful organization like TypeScript version

*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 ENHANCED CYRE STATE SYSTEM DEMO");
    println!("==================================");
    println!("Demonstrating clean state separation with sensor logging\n");

    // =======================================================================
    // PART 1: IO CONFIGURATION MANAGEMENT
    // =======================================================================

    println!("⚙️ === PART 1: IO CONFIGURATION MANAGEMENT ===");

    // Create IO configurations
    let api_config = IO::new("api-endpoint")
        .with_throttle(1000)
        .with_priority(Priority::High)
        .with_name("API Endpoint");

    let user_config = IO::new("user-action")
        .with_debounce(300)
        .with_change_detection()
        .with_name("User Interaction");

    let background_config = IO::new("background-job")
        .with_priority(Priority::Low)
        .with_name("Background Processing");

    // Store configurations (with sensor logging)
    println!("📝 Storing IO configurations...");
    io::set(api_config)?;
    io::set(user_config)?;
    io::set(background_config)?;

    // Retrieve and display configurations
    println!("📖 Retrieving configurations:");
    if let Some(config) = io::get("api-endpoint") {
        println!(
            "   API Endpoint: throttle={}ms, priority={:?}",
            config.throttle.unwrap_or(0),
            config.priority
        );
    }

    if let Some(config) = io::get("user-action") {
        println!(
            "   User Action: debounce={}ms, change_detection={}",
            config.debounce.unwrap_or(0),
            config.change_detection
        );
    }

    println!("   Total IO configs: {}", io::get_all().len());

    // =======================================================================
    // PART 2: PAYLOAD STATE MANAGEMENT (SEPARATE)
    // =======================================================================

    println!("\n💾 === PART 2: PAYLOAD STATE MANAGEMENT ===");

    // Store payloads separately from configurations
    PayloadStateOps::set(
        "api-endpoint".to_string(),
        json!({
        "last_request": {
            "timestamp": current_timestamp(),
            "method": "POST",
            "endpoint": "/api/users"
        }
    })
    );

    PayloadStateOps::set(
        "user-action".to_string(),
        json!({
        "user_id": "user_123",
        "action": "click_button",
        "element": "submit-form"
    })
    );

    // Retrieve payloads
    if let Some(payload) = PayloadStateOps::get("api-endpoint") {
        println!("📦 API Endpoint payload: {}", payload);
    }

    if let Some(payload) = PayloadStateOps::get("user-action") {
        println!("📦 User Action payload: {}", payload);
    }

    // =======================================================================
    // PART 3: SUBSCRIBER MANAGEMENT
    // =======================================================================

    println!("\n👥 === PART 3: SUBSCRIBER MANAGEMENT ===");

    // Create mock subscribers (handlers)
    let api_subscriber = crate::context::state::ISubscriber {
        id: "api-endpoint".to_string(),
        handler: std::sync::Arc::new(|payload| {
            Box::pin(async move { CyreResponse::success(payload, "API handler executed") })
        }),
        created_at: current_timestamp(),
    };

    let user_subscriber = crate::context::state::ISubscriber {
        id: "user-action".to_string(),
        handler: std::sync::Arc::new(|payload| {
            Box::pin(async move { CyreResponse::success(payload, "User handler executed") })
        }),
        created_at: current_timestamp(),
    };

    // Add subscribers
    subscribers::add(api_subscriber)?;
    subscribers::add(user_subscriber)?;

    println!("👥 Added subscribers for API and User actions");
    println!("   Total subscribers: {}", subscribers::get_all().len());

    // =======================================================================
    // PART 4: TIMELINE MANAGEMENT (SCHEDULED TASKS)
    // =======================================================================

    println!("\n⏰ === PART 4: TIMELINE MANAGEMENT ===");

    // Create scheduled tasks
    let cleanup_timer = Timer {
        id: "cleanup-job".to_string(),
        start_time: current_timestamp(),
        duration: 3600000, // 1 hour
        original_duration: 3600000,
        repeat: Some(-1), // Forever
        execution_count: 0,
        last_execution_time: 0,
        next_execution_time: current_timestamp() + 3600000,
        is_in_recuperation: false,
        status: "active".to_string(),
        is_active: true,
        delay: None,
        interval: Some(3600000),
        has_executed_once: false,
        priority: Priority::Low,
        metrics: Some(json!({"type": "cleanup"})),
    };

    let reminder_timer = Timer {
        id: "reminder".to_string(),
        start_time: current_timestamp(),
        duration: 60000, // 1 minute
        original_duration: 60000,
        repeat: Some(3), // 3 times
        execution_count: 0,
        last_execution_time: 0,
        next_execution_time: current_timestamp() + 60000,
        is_in_recuperation: false,
        status: "active".to_string(),
        is_active: true,
        delay: None,
        interval: None,
        has_executed_once: false,
        priority: Priority::Medium,
        metrics: Some(json!({"type": "reminder"})),
    };

    // Add to timeline
    timeline::add(cleanup_timer);
    timeline::add(reminder_timer);

    println!("⏰ Added scheduled tasks to timeline");
    println!("   Total timers: {}", timeline::get_all().len());
    println!("   Active timers: {}", timeline::get_active().len());

    // =======================================================================
    // PART 5: METRICS TRACKING
    // =======================================================================

    println!("\n📊 === PART 5: METRICS TRACKING ===");

    // Update system metrics
    MetricsOps::update(MetricsUpdate {
        total_executions: Some(150),
        fast_path_hits: Some(120),
        protection_blocks: Some(5),
        active_formations: Some(timeline::get_active().len() as u64),
    });

    let metrics = MetricsOps::get();
    println!("📈 System Metrics:");
    println!("   Total Executions: {}", metrics.total_executions);
    println!("   Fast Path Hits: {}", metrics.fast_path_hits);
    println!("   Protection Blocks: {}", metrics.protection_blocks);
    println!("   Active Formations: {}", metrics.active_formations);
    println!("   Last Update: {}", metrics.last_update);

    // =======================================================================
    // PART 6: ACTION METRICS
    // =======================================================================

    println!("\n📋 === PART 6: ACTION METRICS ===");

    // Get action-specific metrics
    if let Some(api_metrics) = io::get_metrics("api-endpoint") {
        println!("📊 API Endpoint Metrics:");
        println!("   Execution Count: {}", api_metrics.execution_count);
        println!("   Last Execution: {}", api_metrics.last_execution_time);
        println!("   Errors: {:?}", api_metrics.errors);
    }

    // =======================================================================
    // PART 7: SYSTEM HEALTH & STATS
    // =======================================================================

    println!("\n🏥 === PART 7: SYSTEM HEALTH & STATS ===");

    let system_stats = stores::Stores::get_system_stats();
    println!("🔍 System Statistics:");
    println!("{}", serde_json::to_string_pretty(&system_stats)?);

    // Individual store sizes
    println!("\n📦 Store Sizes:");
    println!("   IO Channels: {}", stores::Stores::io_size());
    println!("   Subscribers: {}", stores::Stores::subscribers_size());
    println!("   Timeline: {}", stores::Stores::timeline_size());

    // =======================================================================
    // PART 8: ERROR HANDLING & SENSOR INTEGRATION
    // =======================================================================

    println!("\n🚨 === PART 8: ERROR HANDLING & SENSOR INTEGRATION ===");

    // Test error scenarios with sensor logging
    println!("🔍 Testing error scenarios...");

    // Try to create invalid IO config
    let invalid_config = IO::new(""); // Empty ID should fail
    match io::set(invalid_config) {
        Ok(_) => println!("❌ Should have failed!"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }

    // Try to access non-existent config
    if let Some(_) = io::get("non-existent") {
        println!("❌ Should return None!");
    } else {
        println!("✅ Correctly returned None for non-existent config");
    }

    // =======================================================================
    // PART 9: CLEANUP OPERATIONS
    // =======================================================================

    println!("\n🧹 === PART 9: CLEANUP OPERATIONS ===");

    // Forget specific items
    println!("🗑️ Forgetting specific items...");
    let forgot_api = io::forget("api-endpoint");
    let forgot_user = subscribers::forget("user-action");
    let forgot_timer = timeline::forget("reminder");

    println!("   Forgot API config: {}", forgot_api);
    println!("   Forgot user subscriber: {}", forgot_user);
    println!("   Forgot reminder timer: {}", forgot_timer);

    // Check remaining items
    println!("\n📊 Remaining items after cleanup:");
    println!("   IO configs: {}", io::get_all().len());
    println!("   Subscribers: {}", subscribers::get_all().len());
    println!("   Timers: {}", timeline::get_all().len());

    // =======================================================================
    // PART 10: INTEGRATION WITH CYRE ACTIONS
    // =======================================================================

    println!("\n⚡ === PART 10: INTEGRATION WITH CYRE ACTIONS ===");

    // Initialize Cyre and show integration
    let mut cyre = Cyre::new();

    // Register action using state system
    let integration_config = IO::new("state-integration")
        .with_throttle(500)
        .with_name("State Integration Demo");

    io::set(integration_config.clone())?;
    cyre.action(integration_config)?;

    // Register handler and add to subscribers
    cyre.on("state-integration", |payload| {
        Box::pin(async move {
            // Log execution using sensor
            sensor::success("state-integration", "Handler executed successfully", true);

            // Store result in payload state
            PayloadStateOps::set(
                "state-integration-result".to_string(),
                json!({
                "executed_at": current_timestamp(),
                "payload": payload,
                "result": "success"
            })
            );

            CyreResponse::success(payload, "State integration successful")
        })
    });

    // Execute action
    let result = cyre.call(
        "state-integration",
        json!({
        "test": "integration",
        "timestamp": current_timestamp()
    })
    ).await;

    println!("🎯 Integration result: {} - {}", result.ok, result.message);

    // Check stored result
    if let Some(stored_result) = PayloadStateOps::get("state-integration-result") {
        println!("💾 Stored result: {}", stored_result);
    }

    // =======================================================================
    // PART 11: PERFORMANCE TESTING
    // =======================================================================

    println!("\n🚀 === PART 11: PERFORMANCE TESTING ===");

    let start_time = std::time::Instant::now();

    // Create many IO configs rapidly
    for i in 0..1000 {
        let config = IO::new(&format!("perf-test-{}", i)).with_priority(Priority::Medium);
        let _ = io::set(config);
    }

    let creation_time = start_time.elapsed();
    println!("⚡ Created 1000 IO configs in: {:.2}ms", creation_time.as_millis());

    // Access configs rapidly
    let start_time = std::time::Instant::now();
    let mut found = 0;

    for i in 0..1000 {
        if io::get(&format!("perf-test-{}", i)).is_some() {
            found += 1;
        }
    }

    let access_time = start_time.elapsed();
    println!("🔍 Accessed 1000 configs in: {:.2}ms (found: {})", access_time.as_millis(), found);

    // Cleanup performance test configs
    let start_time = std::time::Instant::now();
    for i in 0..1000 {
        io::forget(&format!("perf-test-{}", i));
    }
    let cleanup_time = start_time.elapsed();
    println!("🧹 Cleaned up 1000 configs in: {:.2}ms", cleanup_time.as_millis());

    // =======================================================================
    // PART 12: FINAL COMPREHENSIVE CLEAR
    // =======================================================================

    println!("\n🔄 === PART 12: FINAL SYSTEM CLEAR ===");

    // Clear all stores
    io::clear();
    subscribers::clear();
    timeline::clear();
    PayloadStateOps::clear();
    MetricsOps::reset();

    println!("🧹 All stores cleared");

    // Verify everything is clean
    let final_stats = stores::Stores::get_system_stats();
    println!("📊 Final system state:");
    println!("{}", serde_json::to_string_pretty(&final_stats)?);

    // =======================================================================
    // SUMMARY
    // =======================================================================

    println!("\n🎉 === DEMO SUMMARY ===");

    sensor::success("state-demo", "Enhanced state system demo completed successfully", true);

    println!("\n✅ FEATURES DEMONSTRATED:");
    println!("   🔧 Clean separation: IO config vs payload data");
    println!("   🧵 Thread-safe stores with RwLock protection");
    println!("   📊 Comprehensive metrics tracking");
    println!("   👥 Subscriber management");
    println!("   ⏰ Timeline/scheduler integration");
    println!("   🚨 Error handling with sensor logging");
    println!("   🧹 Proper cleanup and resource management");
    println!("   ⚡ High-performance operations");
    println!("   🔗 Seamless Cyre integration");

    println!("\n💡 KEY RUST ADVANTAGES:");
    println!("   🦀 Memory safety with zero-cost abstractions");
    println!("   🔒 Thread safety enforced at compile time");
    println!("   ⚡ Performance optimized HashMap operations");
    println!("   🎯 Type safety prevents runtime state corruption");
    println!("   📦 Modular design with clear boundaries");

    println!("\n📚 USAGE PATTERNS:");
    println!("   use cyre_rust::context::state::{{io, subscribers, timeline}};");
    println!("   io::set(config)?; // Store IO configuration");
    println!("   subscribers::add(handler)?; // Register event handler");
    println!("   timeline::add(timer); // Schedule task");
    println!("   PayloadStateOps::set(id, data); // Store payload separately");

    println!("\n🔗 INTEGRATION:");
    println!("   - Sensor logging for all operations");
    println!("   - Metrics tracking and health monitoring");
    println!("   - Backward compatibility with existing code");
    println!("   - Clean API matching TypeScript structure");

    Ok(())
}

//=============================================================================
// HELPER FUNCTIONS FOR DEMO
//=============================================================================

/// Demonstrate error handling patterns
async fn demonstrate_error_handling() {
    println!("🔍 Error Handling Demonstration:");

    // Invalid operations that should fail gracefully
    let invalid_operations = vec![("empty_id", ""), ("whitespace_only", "   ")];

    for (test_name, invalid_id) in invalid_operations {
        let config = IO::new(invalid_id);
        match io::set(config) {
            Ok(_) => {
                sensor::error(
                    "demo",
                    &format!("Test '{}' should have failed", test_name),
                    Some("demonstrate_error_handling"),
                    None
                );
            }
            Err(e) => {
                sensor::success(
                    "demo",
                    &format!("Test '{}' correctly failed: {}", test_name, e),
                    false
                );
            }
        }
    }
}

/// Demonstrate concurrent access patterns
async fn demonstrate_concurrency() {
    use tokio::task;

    println!("🧵 Concurrency Demonstration:");

    // Spawn multiple tasks that access state concurrently
    let mut handles = Vec::new();

    for i in 0..10 {
        let handle = task::spawn(async move {
            let config = IO::new(&format!("concurrent-{}", i)).with_priority(Priority::Medium);

            // This should be thread-safe
            io::set(config).unwrap();

            // Read back the config
            if let Some(retrieved) = io::get(&format!("concurrent-{}", i)) {
                sensor::debug(
                    "concurrency",
                    &format!("Task {} stored and retrieved config", i),
                    false
                );
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    println!("✅ All concurrent operations completed successfully");
}
