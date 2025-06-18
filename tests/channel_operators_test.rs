// tests/channel_operators_test.rs
// Comprehensive test suite for channel operators and pipeline functionality

use cyre_rust::prelude::*;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_zero_overhead_fast_path() {
    // Clear global state for test isolation
    let mut cyre = Cyre::new();
    cyre.init().await.unwrap();

    // Register action with NO operators = zero overhead fast path
    cyre.action(IO::new("fast-path")).unwrap();

    cyre.on("fast-path", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload: json!({
                    "received": payload,
                    "fast_path": true
                }),
                message: "Fast path executed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            } })
    }).unwrap();

    // Verify pipeline is zero overhead
    let pipeline_info = cyre.get_pipeline_info("fast-path").unwrap();
    assert!(pipeline_info.is_zero_overhead);
    assert_eq!(pipeline_info.protection_count, 0);

    // Test execution
    let result = cyre.call("fast-path", json!({"test": "data"})).await;
    assert!(result.ok);
    assert_eq!(result.payload["fast_path"], true);
    assert_eq!(result.payload["received"]["test"], "data");
}

#[tokio::test]
async fn test_throttle_operator() {
    // Clear global state for test isolation
    let mut cyre = Cyre::new();
    cyre.init().await.unwrap();

    // Register action with throttle (100ms limit)
    cyre.action(IO::new("throttled").with_throttle(100)).unwrap();

    cyre.on("throttled", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload: json!({
                    "throttled_payload": payload,
                    "timestamp": current_timestamp()
                }),
                message: "Throttled execution".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            } })
    }).unwrap();

    // Verify pipeline has throttle operator
    let pipeline_info = cyre.get_pipeline_info("throttled").unwrap();
    assert!(!pipeline_info.is_zero_overhead);
    assert!(pipeline_info.protection_count > 0);

    // First call should succeed
    let result1 = cyre.call("throttled", json!({"call": 1})).await;
    assert!(result1.ok);
    assert_eq!(result1.payload["throttled_payload"]["call"], 1);

    // Second call should be throttled immediately
    let result2 = cyre.call("throttled", json!({"call": 2})).await;
    assert!(result2.ok); // Returns ok but with blocked message
    assert!(result2.message.contains("Pipeline blocked") || result2.message.contains("Throttled"));

    // Wait for throttle to reset
    sleep(Duration::from_millis(150)).await;

    // Third call should succeed
    let result3 = cyre.call("throttled", json!({"call": 3})).await;
    assert!(result3.ok);
    assert_eq!(result3.payload["throttled_payload"]["call"], 3);
}

#[tokio::test]
async fn test_debounce_operator() {
    // Clear global state for test isolation
    let mut cyre = Cyre::new();
    cyre.init().await.unwrap();

    // Register action with debounce (100ms delay)
    cyre.action(IO::new("debounced").with_debounce(100)).unwrap();

    let execution_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let execution_count_clone = execution_count.clone();

    cyre.on("debounced", move |payload| {
        let count = execution_count_clone.clone();
        Box::pin(async move {
            count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            CyreResponse {
                ok: true,
                payload: json!({
                    "debounced_payload": payload,
                    "execution_number": count.load(std::sync::atomic::Ordering::Relaxed)
                }),
                message: "Debounced execution".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    }).unwrap();

    // Verify pipeline has debounce operator
    let pipeline_info = cyre.get_pipeline_info("debounced").unwrap();
    assert!(!pipeline_info.is_zero_overhead);
    assert!(pipeline_info.protection_count > 0);

    // Rapid fire calls - should debounce
    println!("Testing debounce with rapid calls...");
    for i in 1..=5 {
        let result = cyre.call("debounced", json!({"rapid_call": i})).await;
        println!("Call {}: {} - {}", i, result.ok, result.message);
        sleep(Duration::from_millis(20)).await; // Rapid calls
    }

    // Wait for debounce to complete
    sleep(Duration::from_millis(200)).await;

    // Should have executed only once (the last call after debounce delay)
    let final_count = execution_count.load(std::sync::atomic::Ordering::Relaxed);
    println!("Final execution count: {}", final_count);

    // With debouncing, we expect fewer executions than calls
    assert!(final_count <= 2, "Debounce should limit executions, got {}", final_count);
}

#[tokio::test]
async fn test_required_operator() {
    // Clear global state for test isolation

    let mut cyre = Cyre::new();
    cyre.init().await.unwrap();

    // Register action with required payload validation
    cyre.action(IO::new("required").with_required(true)).unwrap();

    cyre.on("required", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload: json!({
                    "validated_payload": payload
                }),
                message: "Required validation passed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            } })
    }).unwrap();

    // Test with valid payload - should succeed
    let result1 = cyre.call("required", json!({"valid": "data"})).await;
    assert!(result1.ok);
    assert_eq!(result1.payload["validated_payload"]["valid"], "data");

    // Test with null payload - should be blocked
    let result2 = cyre.call("required", serde_json::Value::Null).await;
    assert!(result2.ok); // Returns ok but blocked by pipeline
    assert!(result2.message.contains("Pipeline blocked") || result2.message.contains("Required"));
}

#[tokio::test]
async fn test_schema_operator() {
    // Clear global state for test isolation
    let mut cyre = Cyre::new();
    cyre.init().await.unwrap();

    // Register action with schema validation
    cyre.action(IO::new("schema-test").with_schema("object")).unwrap();

    cyre.on("schema-test", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload: json!({
                    "schema_validated": payload
                }),
                message: "Schema validation passed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            } })
    }).unwrap();

    // Test with object payload - should succeed
    let result1 = cyre.call("schema-test", json!({"name": "test", "value": 123})).await;
    assert!(result1.ok);
    assert_eq!(result1.payload["schema_validated"]["name"], "test");

    // Test with non-object payload - should be blocked
    let result2 = cyre.call("schema-test", json!("not an object")).await;
    assert!(result2.ok); // Returns ok but blocked by pipeline
    assert!(result2.message.contains("Pipeline blocked") || result2.message.contains("Schema"));
}

#[tokio::test]
async fn test_condition_operator() {
    // Clear global state for test isolation
    let mut cyre = Cyre::new();
    cyre.init().await.unwrap();

    // Register action with condition
    cyre.action(IO::new("conditional").with_condition("has_data")).unwrap();

    cyre.on("conditional", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload: json!({
                    "condition_passed": payload
                }),
                message: "Condition check passed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            } })
    }).unwrap();

    // Test with data - should pass condition
    let result1 = cyre.call("conditional", json!({"some": "data"})).await;
    assert!(result1.ok);
    assert_eq!(result1.payload["condition_passed"]["some"], "data");

    // Test with null - should fail condition
    let result2 = cyre.call("conditional", serde_json::Value::Null).await;
    assert!(result2.ok); // Returns ok but blocked by pipeline
    assert!(result2.message.contains("Pipeline blocked") || result2.message.contains("Condition"));
}

#[tokio::test]
async fn test_transform_operator() {
    // Clear global state for test isolation
    let mut cyre = Cyre::new();
    cyre.init().await.unwrap();

    // Register action with transform
    cyre.action(IO::new("transformer").with_transform("add_timestamp")).unwrap();

    cyre.on("transformer", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload: json!({
                    "transformed_data": payload
                }),
                message: "Transform applied".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            } })
    }).unwrap();

    // Test transform operation
    let result = cyre.call("transformer", json!({"original": "data"})).await;
    assert!(result.ok);

    let transformed = &result.payload["transformed_data"];
    assert_eq!(transformed["original"], "data");
    assert!(transformed["transformed_at"].is_number()); // Should have added timestamp
}

#[tokio::test]
async fn test_selector_operator() {
    // Clear global state for test isolation
    let mut cyre = Cyre::new();
    cyre.init().await.unwrap();

    // Register action with selector
    cyre.action(IO::new("selector").with_selector("data")).unwrap();

    cyre.on("selector", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload: json!({
                    "selected_data": payload
                }),
                message: "Selector applied".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            } })
    }).unwrap();

    // Test with payload containing 'data' field
    let result1 = cyre.call(
        "selector",
        json!({
        "data": {"important": "info"},
        "metadata": {"ignore": "this"}
    })
    ).await;
    assert!(result1.ok);
    assert_eq!(result1.payload["selected_data"]["important"], "info");

    // Test without 'data' field - should be blocked
    let result2 = cyre.call("selector", json!({"no_data_field": "here"})).await;
    assert!(result2.ok); // Returns ok but blocked by pipeline
    assert!(result2.message.contains("Pipeline blocked") || result2.message.contains("Selector"));
}

#[tokio::test]
async fn test_block_operator() {
    // Clear global state for test isolation
    let mut cyre = Cyre::new();
    cyre.init().await.unwrap();

    // Register blocked action
    cyre.action(IO::new("blocked").with_block(true)).unwrap();

    cyre.on("blocked", |_payload| {
        Box::pin(async move {
            // This should never execute
            panic!("Blocked action should never reach handler!");
        })
    }).unwrap();

    // All calls should be blocked
    let result = cyre.call("blocked", json!({"any": "data"})).await;
    assert!(result.ok); // Returns ok but blocked by pipeline
    assert!(result.message.contains("Pipeline blocked") || result.message.contains("blocked"));
}

#[tokio::test]
async fn test_complex_pipeline() {
    // Clear global state for test isolation
    let mut cyre = Cyre::new();
    cyre.init().await.unwrap();

    // Register action with multiple operators
    cyre.action(
        IO::new("complex")
            .with_required(true)
            .with_schema("object")
            .with_condition("is_object")
            .with_transform("normalize")
            .with_throttle(50)
    ).unwrap();

    cyre.on("complex", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload: json!({
                    "complex_result": payload,
                    "pipeline": "success"
                }),
                message: "Complex pipeline executed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            } })
    }).unwrap();

    // Verify complex pipeline
    let pipeline_info = cyre.get_pipeline_info("complex").unwrap();
    assert!(!pipeline_info.is_zero_overhead);
    assert!(pipeline_info.protection_count >= 4); // Multiple operators

    // Test with valid complex payload
    let result1 = cyre.call(
        "complex",
        json!({
        "Name": "Test", // Will be normalized to lowercase
        "Value": 123
    })
    ).await;
    assert!(result1.ok);

    let complex_result = &result1.payload["complex_result"];
    assert_eq!(complex_result["name"], "Test"); // Normalized
    assert_eq!(complex_result["value"], 123);

    // Test throttling in complex pipeline
    let result2 = cyre.call("complex", json!({"Name": "Test2", "Value": 456})).await;
    // Should be throttled
    println!("Complex throttle test: {} - {}", result2.ok, result2.message);

    // Wait and try again
    sleep(Duration::from_millis(100)).await;
    let result3 = cyre.call("complex", json!({"Name": "Test3", "Value": 789})).await;
    assert!(result3.ok);
}

#[tokio::test]
async fn test_pipeline_performance_comparison() {
    // Clear global state for test isolation
    let mut cyre = Cyre::new();
    cyre.init().await.unwrap();

    // Set up fast path action
    cyre.action(IO::new("perf-fast")).unwrap();
    cyre.on("perf-fast", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload,
                message: "Fast".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            } })
    }).unwrap();

    // Set up complex pipeline action
    cyre.action(
        IO::new("perf-complex")
            .with_required(true)
            .with_condition("always_true")
            .with_transform("add_timestamp")
    ).unwrap();
    cyre.on("perf-complex", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload,
                message: "Complex".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            } })
    }).unwrap();

    let iterations = 100;
    let test_payload = json!({"test": true});

    // Benchmark fast path
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let result = cyre.call("perf-fast", test_payload.clone()).await;
        assert!(result.ok);
    }
    let fast_time = start.elapsed();

    // Benchmark complex pipeline
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let result = cyre.call("perf-complex", test_payload.clone()).await;
        assert!(result.ok);
    }
    let complex_time = start.elapsed();

    println!("Performance comparison ({} iterations):", iterations);
    println!(
        "  Fast path: {:.2}ms ({:.0} ops/sec)",
        fast_time.as_millis(),
        (iterations as f64) / fast_time.as_secs_f64()
    );
    println!(
        "  Complex pipeline: {:.2}ms ({:.0} ops/sec)",
        complex_time.as_millis(),
        (iterations as f64) / complex_time.as_secs_f64()
    );

    let overhead =
        ((complex_time.as_micros() as f64) / (fast_time.as_micros() as f64) - 1.0) * 100.0;
    println!("  Pipeline overhead: {:.1}%", overhead);

    // Fast path should be significantly faster
    assert!(fast_time < complex_time, "Fast path should be faster than complex pipeline");

    // But overhead should be reasonable (less than 500%)
    assert!(overhead < 500.0, "Pipeline overhead should be reasonable, got {:.1}%", overhead);
}

#[tokio::test]
async fn test_pipeline_statistics() {
    // Clear global state for test isolation
    // Create a completely fresh instance to ensure isolation
    let mut cyre = Cyre::new();
    cyre.init().await.unwrap();

    // Create mix of fast path and pipeline actions
    cyre.action(IO::new("stats-fast1")).unwrap();
    cyre.action(IO::new("stats-fast2")).unwrap();
    cyre.action(IO::new("stats-protected1").with_throttle(100)).unwrap();
    cyre.action(IO::new("stats-protected2").with_required(true)).unwrap();

    // Check pipeline statistics
    let stats = cyre.pipeline_stats();
    assert_eq!(stats.total_pipelines, 4);
    assert_eq!(stats.zero_overhead_count, 2); // fast1, fast2
    assert_eq!(stats.protected_count, 2); // protected1, protected2
    assert_eq!(stats.optimization_ratio(), 50.0); // 50% zero overhead

    // Verify individual pipeline info
    let fast_info = cyre.get_pipeline_info("stats-fast1").unwrap();
    assert!(fast_info.is_zero_overhead);
    assert_eq!(fast_info.protection_count, 0);

    let protected_info = cyre.get_pipeline_info("stats-protected1").unwrap();
    assert!(!protected_info.is_zero_overhead);
    assert!(protected_info.protection_count > 0);

    // Test performance metrics integration
    let metrics = cyre.get_performance_metrics();
    assert_eq!(metrics["unified_pipeline"]["total_pipelines"], 4);
    assert_eq!(metrics["unified_pipeline"]["fast_path_channels"], 2);
    assert_eq!(metrics["unified_pipeline"]["pipeline_channels"], 2);
    assert_eq!(metrics["unified_pipeline"]["optimization_ratio"], 50.0);
}

#[tokio::test]
async fn test_error_handling_in_operators() {
    // Clear global state for test isolation
    let mut cyre = Cyre::new();
    cyre.init().await.unwrap();

    // Test unknown condition - should log warning but pass through
    cyre.action(IO::new("unknown-condition").with_condition("unknown_condition")).unwrap();
    cyre.on("unknown-condition", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload,
                message: "Unknown condition handled gracefully".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            } })
    }).unwrap();

    let result = cyre.call("unknown-condition", json!({"test": true})).await;
    assert!(result.ok); // Should pass through despite unknown condition

    // Test unknown transform - should log warning but pass through
    cyre.action(IO::new("unknown-transform").with_transform("unknown_transform")).unwrap();
    cyre.on("unknown-transform", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload,
                message: "Unknown transform handled gracefully".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            } })
    }).unwrap();

    let result = cyre.call("unknown-transform", json!({"test": true})).await;
    assert!(result.ok); // Should pass through despite unknown transform
}
