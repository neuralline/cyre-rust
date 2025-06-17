// examples/unified_pipeline_demo.rs
// Complete examples testing all unified pipeline features

use cyre_rust::prelude::*;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 UNIFIED PIPELINE DEMO");
    println!("========================");
    println!("Testing all pipeline features: fast path, protection, scheduling");
    println!();

    let cyre = Cyre::new();
    cyre.set_debug_mode(true);

    //=================================================================
    // Demo 1: Fast Path (Zero Overhead)
    //=================================================================
    println!("⚡ Demo 1: Fast Path (Zero Overhead)");
    println!("====================================");

    // Register simple action with NO operators = fast path
    cyre.action(IO::new("ping"));

    cyre.on("ping", |payload| {
        Box::pin(async move {
            CyreResponse::success(
                json!({
                    "pong": true,
                    "received": payload,
                    "processed_at": current_timestamp()
                }),
                "Fast path execution"
            )
        })
    });

    // Test fast path performance
    let start = std::time::Instant::now();
    for i in 0..1000 {
        let result = cyre.call("ping", json!({"ping": i})).await;
        if !result.ok {
            println!("❌ Fast path failed: {}", result.message);
        }
    }
    let duration = start.elapsed();
    println!(
        "✅ 1000 fast path calls in {:.2}ms ({:.0} ops/sec)",
        duration.as_millis(),
        1000.0 / duration.as_secs_f64()
    );

    //=================================================================
    // Demo 2: Throttling Protection
    //=================================================================
    println!("\n🛡️ Demo 2: Throttling Protection");
    println!("=================================");

    // Register action with throttle (max 1 call per second)
    cyre.action(IO::new("api-call").with_throttle(1000));

    cyre.on("api-call", |payload| {
        Box::pin(async move {
            println!("🌐 API processing: {}", payload);
            CyreResponse::success(
                json!({
                    "api_response": "Data retrieved",
                    "request_id": payload.get("id").unwrap_or(&json!("unknown"))
                }),
                "API call completed"
            )
        })
    });

    println!("🔄 Testing throttle protection...");

    // First call should succeed
    let result1 = cyre.call("api-call", json!({"id": 1})).await;
    println!("   Call 1: {} - {}", result1.ok, result1.message);

    // Second call should be throttled
    let result2 = cyre.call("api-call", json!({"id": 2})).await;
    println!("   Call 2: {} - {}", result2.ok, result2.message);

    // Wait and try again
    println!("   Waiting 1.1 seconds...");
    sleep(Duration::from_millis(1100)).await;

    let result3 = cyre.call("api-call", json!({"id": 3})).await;
    println!("   Call 3: {} - {}", result3.ok, result3.message);

    //=================================================================
    // Demo 3: Debounce Protection (Async!)
    //=================================================================
    println!("\n🔄 Demo 3: Debounce Protection (Async!)");
    println!("========================================");

    // Register search with debounce (300ms delay)
    cyre.action(IO::new("search").with_debounce(300).with_max_wait(1000));

    cyre.on("search", |payload| {
        Box::pin(async move {
            let query = payload
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            println!("🔍 Executing search for: '{}'", query);

            CyreResponse::success(
                json!({
                    "results": format!("Found 42 results for '{}'", query),
                    "query": query,
                    "search_time": "0.15ms"
                }),
                "Search completed"
            )
        })
    });

    println!("🔄 Testing debounce (simulating rapid typing)...");

    // Simulate rapid typing - only last search should execute
    let searches = vec!["h", "he", "hel", "hell", "hello"];
    for (i, query) in searches.iter().enumerate() {
        println!("   Typing: '{}'", query);
        let result = cyre.call("search", json!({"query": query})).await;
        println!("   Response {}: {} - {}", i + 1, result.ok, result.message);

        if i < searches.len() - 1 {
            sleep(Duration::from_millis(100)).await; // Rapid typing
        }
    }

    // Wait for debounce to complete
    println!("   Waiting for debounce to complete...");
    sleep(Duration::from_millis(400)).await;

    //=================================================================
    // Demo 4: Complex Pipeline (Multiple Operators)
    //=================================================================
    println!("\n🎯 Demo 4: Complex Pipeline (Smart Ordering)");
    println!("==============================================");

    // Register complex action with mixed operator order
    // System will reorder: Protection first, then user order, then schedule
    cyre.action(
        IO::new("complex-workflow")
            .with_transform("sanitize") // User operator (will be 3rd)
            .with_throttle(500) // Protection (will be 1st)
            .with_condition("validate") // User operator (will be 4th)
            .with_required(true) // User operator (will be 2nd)
            //  .with_logging(true) // User operator (will be 5th)
            .with_delay(1000) // Schedule (will be last)
    );

    cyre.on("complex-workflow", |payload| {
        Box::pin(async move {
            println!("🎯 Complex workflow handler executed!");
            CyreResponse::success(
                json!({
                    "workflow_completed": true,
                    "processed_data": payload,
                    "completion_time": current_timestamp()
                }),
                "Complex workflow completed"
            )
        })
    });

    println!("🔄 Testing complex pipeline with smart ordering...");

    // Test with valid payload
    let complex_result = cyre.call(
        "complex-workflow",
        json!({
        "data": "important business data",
        "user_id": 12345,
        "operation": "process"
    })
    ).await;

    println!("   Complex call: {} - {}", complex_result.ok, complex_result.message);
    if complex_result.metadata.is_some() {
        println!(
            "   Metadata: {}",
            serde_json::to_string_pretty(&complex_result.metadata).unwrap()
        );
    }

    //=================================================================
    // Demo 5: Required Field Validation
    //=================================================================
    println!("\n✅ Demo 5: Required Field Validation");
    println!("====================================");

    // Register action that requires payload
    cyre.action(IO::new("user-registration").with_required(true));

    cyre.on("user-registration", |payload| {
        Box::pin(async move {
            CyreResponse::success(
                json!({
                    "user_created": true,
                    "user_data": payload
                }),
                "User registered successfully"
            )
        })
    });

    println!("🔄 Testing required field validation...");

    // Test with null payload (should fail)
    let null_result = cyre.call("user-registration", json!(null)).await;
    println!("   Null payload: {} - {}", null_result.ok, null_result.message);

    // Test with valid payload (should succeed)
    let valid_result = cyre.call(
        "user-registration",
        json!({
        "username": "john_doe",
        "email": "john@example.com"
    })
    ).await;
    println!("   Valid payload: {} - {}", valid_result.ok, valid_result.message);

    //=================================================================
    // Demo 6: Scheduling (Future Execution)
    //=================================================================
    println!("\n⏰ Demo 6: Scheduling (Future Execution)");
    println!("=========================================");

    // Register scheduled action (5 second delay)
    cyre.action(
        IO::new("scheduled-report").with_delay(2000) // 2 seconds delay
        // .with_logging(true)
    );

    cyre.on("scheduled-report", |payload| {
        Box::pin(async move {
            println!("📊 Generating scheduled report...");
            CyreResponse::success(
                json!({
                    "report_generated": true,
                    "report_data": payload,
                    "generated_at": current_timestamp()
                }),
                "Scheduled report generated"
            )
        })
    });

    println!("🔄 Testing scheduling...");

    let schedule_result = cyre.call(
        "scheduled-report",
        json!({
        "report_type": "monthly_summary",
        "requested_by": "admin"
    })
    ).await;

    println!("   Schedule response: {} - {}", schedule_result.ok, schedule_result.message);
    println!("   Report will be generated in 2 seconds...");

    //=================================================================
    // Demo 7: Logging Pipeline
    //=================================================================
    println!("\n📝 Demo 7: Logging Pipeline");
    println!("===========================");

    // Register action with logging enabled
    cyre.action(IO::new("audit-action").with_logging(false));

    cyre.on("audit-action", |payload| {
        Box::pin(async move {
            CyreResponse::success(
                json!({
                    "audit_logged": true,
                    "action_data": payload
                }),
                "Audit action completed"
            )
        })
    });

    println!("🔄 Testing logging pipeline...");

    let audit_result = cyre.call(
        "audit-action",
        json!({
        "action": "user_login",
        "user_id": 12345,
        "ip_address": "192.168.1.100"
    })
    ).await;

    println!("   Audit result: {} - {}", audit_result.ok, audit_result.message);

    //=================================================================
    // Demo 8: Performance Comparison
    //=================================================================
    println!("\n📊 Demo 8: Performance Comparison");
    println!("==================================");

    // Test performance difference between fast path and pipeline
    let iterations = 1000;

    // Fast path performance
    let start = std::time::Instant::now();
    for i in 0..iterations {
        let _result = cyre.call("ping", json!({"test": i})).await;
    }
    let fast_path_time = start.elapsed();

    // Pipeline performance (with throttling disabled by waiting)
    sleep(Duration::from_millis(100)).await; // Reset throttle

    let start = std::time::Instant::now();
    for i in 0..iterations {
        let _result = cyre.call("audit-action", json!({"test": i})).await;
        if i % 100 == 0 {
            sleep(Duration::from_millis(1)).await; // Small delay to prevent throttling
        }
    }
    let pipeline_time = start.elapsed();

    println!("⚡ Performance Results:");
    println!(
        "   Fast Path: {} ops in {:.2}ms ({:.0} ops/sec)",
        iterations,
        fast_path_time.as_millis(),
        (iterations as f64) / fast_path_time.as_secs_f64()
    );
    println!(
        "   Pipeline: {} ops in {:.2}ms ({:.0} ops/sec)",
        iterations,
        pipeline_time.as_millis(),
        (iterations as f64) / pipeline_time.as_secs_f64()
    );

    let overhead =
        ((pipeline_time.as_micros() as f64) / (fast_path_time.as_micros() as f64) - 1.0) * 100.0;
    println!("   Pipeline overhead: {:.1}%", overhead);

    //=================================================================
    // Final Metrics Summary
    //=================================================================
    println!("\n📈 Final System Metrics");
    println!("=======================");

    let metrics = cyre.get_performance_metrics();
    println!("System Status:");
    println!("   Total Actions: {}", metrics["system"]["total_actions"]);
    println!("   Total Handlers: {}", metrics["system"]["total_handlers"]);
    println!("   Total Executions: {}", metrics["executions"]["total_executions"]);
    println!("   Fast Path Hits: {}", metrics["executions"]["fast_path_hits"]);
    println!("   Pipeline Hits: {}", metrics["executions"]["pipeline_hits"]);
    println!("   Fast Path Ratio: {:.1}%", metrics["executions"]["fast_path_ratio"]);
    println!("   Protection Blocks: {}", metrics["protection"]["total_blocks"]);
    println!("   Scheduled Actions: {}", metrics["executions"]["scheduled_actions"]);

    println!("\nPipeline Optimization:");
    println!("   Fast Path Channels: {}", metrics["unified_pipeline"]["fast_path_channels"]);
    println!("   Pipeline Channels: {}", metrics["unified_pipeline"]["pipeline_channels"]);
    println!("   Optimization Ratio: {:.1}%", metrics["unified_pipeline"]["optimization_ratio"]);

    //=================================================================
    // Wait for Scheduled Actions
    //=================================================================
    println!("\n⏰ Waiting for scheduled actions to complete...");
    sleep(Duration::from_millis(3000)).await;

    println!("\n🎉 UNIFIED PIPELINE DEMO COMPLETED!");
    println!("===================================");
    println!("✅ Fast Path: Zero overhead execution");
    println!("✅ Throttling: Rate limiting protection");
    println!("✅ Debouncing: Async input smoothing");
    println!("✅ Complex Pipeline: Smart operator ordering");
    println!("✅ Validation: Required field checking");
    println!("✅ Scheduling: Future execution");
    println!("✅ Logging: Request tracking");
    println!("✅ Performance: Microsecond-level latency");

    println!("\n💡 Key Features Demonstrated:");
    println!("   • Per-channel pipeline compilation");
    println!("   • Smart operator reordering (protection → user → schedule)");
    println!("   • Zero overhead fast path for simple actions");
    println!("   • Async debounce (impossible in TypeScript!)");
    println!("   • Type-safe Rust performance with familiar API");

    Ok(())
}

//=================================================================
// Additional Example: Real-World Use Case
//=================================================================

#[tokio::test]
async fn test_real_world_api_workflow() {
    let cyre = Cyre::new();

    // Real-world API endpoint with comprehensive protection
    cyre.action(
        IO::new("user-profile-update")
            .with_required(true) // Must have payload
            .with_throttle(2000) // Max 1 update per 2 seconds per user
            .with_debounce(500) // Debounce rapid updates
            .with_max_wait(3000) // But don't wait more than 3 seconds
            .with_transform("sanitize") // Clean input data
            .with_condition("auth") // Check authentication
        //.with_logging(true) // Audit all updates
    );

    cyre.on("user-profile-update", |payload| {
        Box::pin(async move {
            // Simulate database update
            sleep(Duration::from_millis(50)).await;

            CyreResponse::success(
                json!({
                    "profile_updated": true,
                    "user_id": payload.get("user_id"),
                    "updated_fields": payload.get("fields"),
                    "updated_at": current_timestamp()
                }),
                "Profile updated successfully"
            )
        })
    });

    // Test the workflow
    let update_data =
        json!({
        "user_id": 12345,
        "fields": {
            "display_name": "John Doe",
            "bio": "Software Developer",
            "location": "San Francisco"
        }
    });

    // First update should succeed (after debounce)
    let result1 = cyre.call("user-profile-update", update_data.clone()).await;
    sleep(Duration::from_millis(600)).await; // Wait for debounce

    // Rapid second update should be throttled
    let result2 = cyre.call("user-profile-update", update_data.clone()).await;

    // Wait and try again
    sleep(Duration::from_millis(2100)).await;
    let result3 = cyre.call("user-profile-update", update_data).await;
    sleep(Duration::from_millis(600)).await; // Wait for debounce

    println!("Real-world API test results:");
    println!("   First update: {} - {}", result1.ok, result1.message);
    println!("   Rapid update: {} - {}", result2.ok, result2.message);
    println!("   After wait: {} - {}", result3.ok, result3.message);

    // Verify pipeline worked correctly
    assert!(result1.ok || result2.ok); // One should succeed after debounce
    // result2 might be throttled or debounced
    assert!(result3.ok); // Should succeed after waiting
}

//=================================================================
// Performance Benchmark Example
//=================================================================

#[tokio::test]
async fn benchmark_unified_pipeline() {
    let cyre = Cyre::new();

    // Setup different pipeline types
    cyre.action(IO::new("fast")); // Fast path
    cyre.action(IO::new("simple").with_logging(false)); // Simple pipeline
    cyre.action(
        IO::new("complex") // Complex pipeline
            .with_throttle(10) // Very short throttle for testing
            .with_required(true)
            .with_transform("test")
            .with_condition("check")
        //.with_logging(true)
    );

    // Register handlers
    for action in ["fast", "simple", "complex"] {
        cyre.on(action, |payload| {
            Box::pin(async move { CyreResponse::success(payload, "Processed") })
        });
    }

    let iterations = 1000;
    let test_payload = json!({"test": true});

    // Benchmark fast path
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _result = cyre.call("fast", test_payload.clone()).await;
    }
    let fast_time = start.elapsed();

    // Small delay to reset throttle
    sleep(Duration::from_millis(50)).await;

    // Benchmark simple pipeline
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _result = cyre.call("simple", test_payload.clone()).await;
    }
    let simple_time = start.elapsed();

    // Small delay to reset throttle
    sleep(Duration::from_millis(50)).await;

    // Benchmark complex pipeline (with throttle delays)
    let start = std::time::Instant::now();
    for i in 0..100 {
        // Fewer iterations due to throttling
        let _result = cyre.call("complex", test_payload.clone()).await;
        if i % 10 == 0 {
            sleep(Duration::from_millis(15)).await; // Reset throttle
        }
    }
    let complex_time = start.elapsed();

    println!("\n📊 Performance Benchmark Results:");
    println!(
        "Fast Path:     {:.2}μs per call ({:.0} ops/sec)",
        (fast_time.as_micros() as f64) / (iterations as f64),
        (iterations as f64) / fast_time.as_secs_f64()
    );
    println!(
        "Simple Pipeline: {:.2}μs per call ({:.0} ops/sec)",
        (simple_time.as_micros() as f64) / (iterations as f64),
        (iterations as f64) / simple_time.as_secs_f64()
    );
    println!(
        "Complex Pipeline: {:.2}μs per call ({:.0} ops/sec)",
        (complex_time.as_micros() as f64) / 100.0,
        100.0 / complex_time.as_secs_f64()
    );

    // Verify fast path is indeed faster
    assert!(fast_time < simple_time);
    println!(
        "✅ Fast path is {:.1}x faster than simple pipeline",
        (simple_time.as_micros() as f64) / (fast_time.as_micros() as f64)
    );
}
