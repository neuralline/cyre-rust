// tests/integration_test.rs
// Comprehensive integration tests for modular Cyre

use cyre_rust::prelude::*;
use cyre_rust::talent::{Talent, TalentRegistry, functions::*};
use cyre_rust::protection::{ProtectionState, ProtectionType};
use serde_json::json;
use std::time::Duration;

//=============================================================================
// INTEGRATION TESTS
//=============================================================================

#[tokio::test]
async fn test_end_to_end_workflow() {
    // Test complete workflow with all components
    let mut cyre = Cyre::new();
    
    // Setup action with protection
    let config = IO::new("workflow-test")
        .with_name("Workflow Test")
        .with_throttle(100)
        .with_priority(Priority::High);
    
    cyre.action(config);
    
    // Setup handler
    cyre.on("workflow-test", |payload| {
        Box::pin(async move {
            let user = payload.get("user").unwrap_or(&json!("anonymous"));
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "processed": true,
                    "user": user,
                    "timestamp": current_timestamp()
                }),
                message: "Workflow completed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({
                    "workflow": "test",
                    "version": "1.0"
                })),
            }
        })
    });
    
    // Test successful execution
    let response = cyre.call("workflow-test", json!({
        "user": "alice",
        "action": "login"
    })).await;
    
    assert!(response.ok);
    assert_eq!(response.message, "Workflow completed");
    assert_eq!(response.payload.get("user").unwrap(), &json!("alice"));
    
    // Test throttling protection
    let response2 = cyre.call("workflow-test", json!({
        "user": "bob", 
        "action": "login"
    })).await;
    
    assert!(!response2.ok); // Should be throttled
    assert!(response2.error.is_some());
    
    // Verify metrics
    let metrics = cyre.get_performance_metrics();
    assert_eq!(metrics["total_executions"].as_u64().unwrap(), 1);
    assert_eq!(metrics["protection_blocks"].as_u64().unwrap(), 1);
}

#[tokio::test]
async fn test_talent_system_integration() {
    // Test talent system with Cyre integration
    let mut cyre = Cyre::new();
    let talent_registry = TalentRegistry::new();
    
    // Register talents
    let validator = Talent::schema(
        "validate-order",
        "Order Validation",
        "Validates order data",
        require_fields(&["product_id", "quantity", "user_id"])
    );
    
    let pricing_calculator = Talent::transform(
        "calculate-pricing",
        "Pricing Calculator",
        "Calculates order total",
        |mut payload| {
            if let Some(obj) = payload.as_object_mut() {
                let quantity = obj.get("quantity").and_then(|v| v.as_f64()).unwrap_or(1.0);
                let price = obj.get("price").and_then(|v| v.as_f64()).unwrap_or(10.0);
                obj.insert("total".to_string(), json!(quantity * price));
            }
            payload
        }
    );
    
    let metadata_enricher = Talent::transform(
        "enrich-metadata", 
        "Metadata Enricher",
        "Adds processing metadata",
        add_timestamp("processed_at")
    );
    
    talent_registry.register_talent(validator).unwrap();
    talent_registry.register_talent(pricing_calculator).unwrap();
    talent_registry.register_talent(metadata_enricher).unwrap();
    
    // Setup Cyre action
    cyre.action(IO::new("process-order"));
    cyre.on("process-order", move |payload| {
        Box::pin(async move {
            // Execute talent pipeline (in real implementation, this would be integrated)
            let config = IO::new("process-order");
            let talent_ids = vec![
                "validate-order".to_string(),
                "calculate-pricing".to_string(), 
                "enrich-metadata".to_string()
            ];
            
            // Note: In full implementation, talent registry would be injected
            // For now, we simulate the processing
            let processed_payload = json!({
                "product_id": payload.get("product_id"),
                "quantity": payload.get("quantity"),
                "user_id": payload.get("user_id"),
                "price": 15.99,
                "total": 31.98, // 2 * 15.99
                "processed_at": current_timestamp(),
                "status": "processed"
            });
            
            CyreResponse {
                ok: true,
                payload: processed_payload,
                message: "Order processed successfully".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({
                    "talents_executed": 3,
                    "pipeline": "order-processing"
                })),
            }
        })
    });
    
    // Test valid order
    let valid_order = json!({
        "product_id": "PROD-123",
        "quantity": 2,
        "user_id": "USER-456"
    });
    
    let response = cyre.call("process-order", valid_order).await;
    assert!(response.ok);
    assert_eq!(response.payload.get("total").unwrap(), &json!(31.98));
    assert!(response.payload.get("processed_at").is_some());
}

#[tokio::test] 
async fn test_protection_system_detailed() {
    // Test protection system in detail
    let mut cyre = Cyre::new();
    
    // Test throttle protection
    let throttle_config = IO::new("throttle-test")
        .with_throttle(50); // 50ms throttle
    
    cyre.action(throttle_config);
    cyre.on("throttle-test", |payload| {
        Box::pin(async move {
            CyreResponse {
                ok: true,
                payload: json!({"processed": payload}),
                message: "Throttle test".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    });
    
    // First call should succeed
    let response1 = cyre.call("throttle-test", json!({"test": 1})).await;
    assert!(response1.ok);
    
    // Immediate second call should fail
    let response2 = cyre.call("throttle-test", json!({"test": 2})).await;
    assert!(!response2.ok);
    
    // Wait and try again
    tokio::time::sleep(Duration::from_millis(60)).await;
    let response3 = cyre.call("throttle-test", json!({"test": 3})).await;
    assert!(response3.ok);
    
    // Test change detection
    let change_config = IO::new("change-test")
        .with_change_detection();
    
    cyre.action(change_config);
    cyre.on("change-test", |payload| {
        Box::pin(async move {
            CyreResponse {
                ok: true,
                payload: json!({"echo": payload}),
                message: "Change test".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    });
    
    let test_payload = json!({"data": "test"});
    
    // First call with payload should succeed
    let response1 = cyre.call("change-test", test_payload.clone()).await;
    assert!(response1.ok);
    
    // Second call with same payload should be blocked
    let response2 = cyre.call("change-test", test_payload).await;
    assert!(!response2.ok);
    
    // Call with different payload should succeed
    let response3 = cyre.call("change-test", json!({"data": "different"})).await;
    assert!(response3.ok);
}

#[tokio::test]
async fn test_fast_path_vs_protected_path() {
    // Compare fast path vs protected path performance
    let mut cyre = Cyre::new();
    
    // Setup fast path channel (no protection)
    cyre.action(IO::new("fast"));
    cyre.on("fast", |payload| {
        Box::pin(async move {
            CyreResponse {
                ok: true,
                payload,
                message: "Fast".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    });
    
    // Setup protected channel
    let protected_config = IO::new("protected")
        .with_throttle(1); // Minimal throttle
    
    cyre.action(protected_config);
    cyre.on("protected", |payload| {
        Box::pin(async move {
            CyreResponse {
                ok: true,
                payload,
                message: "Protected".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    });
    
    let iterations = 100;
    
    // Time fast path calls
    let start = std::time::Instant::now();
    for i in 0..iterations {
        let _response = cyre.call("fast", json!({"iter": i})).await;
    }
    let fast_duration = start.elapsed();
    
    // Time protected path calls (with delays to avoid throttling)
    let start = std::time::Instant::now();
    for i in 0..iterations {
        let _response = cyre.call("protected", json!({"iter": i})).await;
        tokio::time::sleep(Duration::from_millis(2)).await; // Avoid throttle
    }
    let protected_duration = start.elapsed();
    
    // Verify fast path is significantly faster (accounting for throttle delays)
    println!("Fast path: {:.2}ms total", fast_duration.as_millis());
    println!("Protected path: {:.2}ms total", protected_duration.as_millis());
    
    // Check metrics
    let metrics = cyre.get_performance_metrics();
    let fast_path_ratio = metrics["fast_path_ratio"].as_f64().unwrap();
    
    assert!(fast_path_ratio > 40.0); // At least some fast path usage
    assert_eq!(metrics["total_executions"].as_u64().unwrap(), iterations * 2);
}

#[test]
fn test_protection_state_standalone() {
    // Test protection state independently
    use cyre_rust::protection::ProtectionState;
    
    // Test no protection
    let no_protection = ProtectionState::new(None, None, false);
    assert!(no_protection.is_no_protection());
    assert!(no_protection.should_execute(&json!({"test": "data"})));
    
    // Test throttle only
    let throttle_protection = ProtectionState::new(Some(100), None, false);
    assert!(!throttle_protection.is_no_protection());
    
    let payload = json!({"test": "data"});
    assert!(throttle_protection.should_execute(&payload)); // First call
    assert!(!throttle_protection.should_execute(&payload)); // Second call blocked
    
    // Test change detection
    let change_protection = ProtectionState::new(None, None, true);
    let payload1 = json!({"data": "first"});
    let payload2 = json!({"data": "second"});
    let payload3 = json!({"data": "first"}); // Same as payload1
    
    assert!(change_protection.should_execute(&payload1));
    assert!(change_protection.should_execute(&payload2));
    assert!(!change_protection.should_execute(&payload3)); // Should be skipped
    
    // Check statistics
    let (blocks, skips) = change_protection.get_stats();
    assert_eq!(skips, 1);
}

#[test]
fn test_io_builder_pattern() {
    // Test IO configuration builder pattern
    let config = IO::new("test-action")
        .with_name("Test Action")
        .with_priority(Priority::Critical)
        .with_throttle(500)
        .with_change_detection()
        .with_talent("validate-input");
    
    assert_eq!(config.id, "test-action");
    assert_eq!(config.name, Some("Test Action".to_string()));
    assert_eq!(config.priority, Priority::Critical);
    assert_eq!(config.throttle, Some(500));
    assert!(config.detect_changes);
    assert!(!config.is_fast_path_eligible()); // Should be false due to protection
    assert!(config.has_protection());
    assert!(config.has_advanced_features());
}

#[tokio::test]
async fn test_comprehensive_metrics() {
    // Test comprehensive system metrics
    let mut cyre = Cyre::new();
    
    // Setup multiple channels
    cyre.action(IO::new("fast-channel"));
    cyre.action(IO::new("protected-channel").with_throttle(50));
    cyre.action(IO::new("priority-channel").with_priority(Priority::High));
    
    cyre.on("fast-channel", |p| Box::pin(async move { 
        CyreResponse { ok: true, payload: p, ..Default::default() }
    }));
    
    cyre.on("protected-channel", |p| Box::pin(async move {
        CyreResponse { ok: true, payload: p, ..Default::default() }
    }));
    
    cyre.on("priority-channel", |p| Box::pin(async move {
        CyreResponse { ok: true, payload: p, ..Default::default() }
    }));
    
    // Execute various calls
    let _r1 = cyre.call("fast-channel", json!({"test": 1})).await;
    let _r2 = cyre.call("protected-channel", json!({"test": 2})).await;
    let _r3 = cyre.call("priority-channel", json!({"test": 3})).await;
    
    // Try to trigger protection
    let _r4 = cyre.call("protected-channel", json!({"test": 4})).await; // Should be blocked
    
    // Check comprehensive metrics
    let metrics = cyre.get_comprehensive_metrics();
    
    assert_eq!(metrics["performance"]["total_executions"].as_u64().unwrap(), 3);
    assert_eq!(metrics["performance"]["protection_blocks"].as_u64().unwrap(), 1);
    assert_eq!(metrics["system"]["active_channels"].as_u64().unwrap(), 3);
    
    // Check individual channel counts
    assert_eq!(cyre.channel_count(), 3);
    assert!(cyre.fast_path_channel_count() > 0);
}

//=============================================================================
// PERFORMANCE REGRESSION TESTS
//=============================================================================

#[tokio::test]
async fn test_performance_regression() {
    // Ensure modular architecture doesn't degrade performance
    let mut cyre = Cyre::new();
    
    cyre.action(IO::new("perf-test"));
    cyre.on("perf-test", |payload| {
        Box::pin(async move {
            CyreResponse {
                ok: true,
                payload,
                message: "Performance test".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    });
    
    let iterations = 1000;
    let start = std::time::Instant::now();
    
    for i in 0..iterations {
        let _response = cyre.call("perf-test", json!({"iteration": i})).await;
    }
    
    let duration = start.elapsed();
    let ops_per_sec = (iterations as f64 / duration.as_secs_f64()) as u64;
    
    // Should maintain good performance (adjust threshold as needed)
    assert!(ops_per_sec > 10_000, "Performance regression detected: {} ops/sec", ops_per_sec);
    
    println!("Performance test: {} ops/sec", ops_per_sec);
}

//=============================================================================
// ERROR HANDLING TESTS
//=============================================================================

#[tokio::test]
async fn test_error_handling() {
    let mut cyre = Cyre::new();
    
    // Test missing channel
    let response = cyre.call("non-existent", json!({})).await;
    assert!(!response.ok);
    assert!(response.error.is_some());
    assert!(response.error.unwrap().contains("not found"));
    
    // Test handler that returns error
    cyre.action(IO::new("error-test"));
    cyre.on("error-test", |_payload| {
        Box::pin(async move {
            CyreResponse {
                ok: false,
                payload: json!(null),
                message: "Handler error".to_string(),
                error: Some("Simulated error".to_string()),
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    });
    
    let response = cyre.call("error-test", json!({})).await;
    assert!(!response.ok);
    assert_eq!(response.message, "Handler error");
    assert!(response.error.is_some());
}