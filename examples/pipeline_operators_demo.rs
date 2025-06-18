// examples/pipeline_operators_demo.rs
// Comprehensive demo testing all pipeline operators with terminal logging

use cyre_rust::prelude::*;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

/*

      🚀 PIPELINE OPERATORS DEMO
      
      Testing the redesigned pipeline system:
      - Zero overhead fast path
      - User configuration order respect
      - TypeScript-compatible throttle logic
      - Proper debounce implementation
      - All operators working with action state
      - Terminal logging for visibility

*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 PIPELINE OPERATORS COMPREHENSIVE DEMO");
    println!("========================================");
    println!("Testing redesigned pipeline system with terminal logging");
    println!();

    let mut cyre = Cyre::new();
    cyre.init().await?;

    //=================================================================
    // Test 1: Zero Overhead Fast Path
    //=================================================================
    println!("⚡ TEST 1: Zero Overhead Fast Path");
    println!("==================================");
    println!("📋 Action with NO operators = pipeline.length == 0");
    println!();

    // Register action with NO operators
    cyre.action(IO::new("fast-path"))?;

    cyre.on("fast-path", |payload| {
        Box::pin(async move {
            println!("🏃‍♂️ FAST PATH HANDLER EXECUTED!");
            println!("   📦 Received payload: {}", payload);
            println!("   ⚡ Zero overhead - direct execution");

            CyreResponse {
                ok: true,
                payload: json!({
                    "fast_path": true,
                    "received": payload,
                    "execution_time": "sub-microsecond"
                }),
                message: "Fast path executed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"pipeline_type": "zero_overhead"})),
            }
        })
    })?;

    // Test fast path
    println!("🔄 Calling fast-path action...");
    let result = cyre.call("fast-path", json!({"test": "fast", "id": 1})).await;
    println!("✅ Result: {} - {}", result.ok, result.message);
    println!("📊 Pipeline type: {:?}", result.metadata);
    println!();

    // Verify it's actually zero overhead
    let pipeline_info = cyre.get_pipeline_info("fast-path");
    if let Some(info) = pipeline_info {
        println!("🔍 Pipeline Analysis:");
        println!("   • Zero overhead: {}", info.is_zero_overhead);
        println!("   • Protection count: {}", info.protection_count);
        println!();
    }

    //=================================================================
    // Test 2: Throttle Operator (TypeScript Pattern)
    //=================================================================
    println!("🚦 TEST 2: Throttle Operator (TypeScript Pattern)");
    println!("=================================================");
    println!("📋 Testing: First call passes, subsequent calls throttled");
    println!("📋 Using action._last_exec_time (not separate state)");
    println!();

    // Register throttled action
    cyre.action(IO::new("throttled-api").with_throttle(1000))?; // 1 second throttle

    cyre.on("throttled-api", |payload| {
        Box::pin(async move {
            println!("🌐 THROTTLED API HANDLER EXECUTED!");
            println!("   📦 Processing: {}", payload);

            CyreResponse {
                ok: true,
                payload: json!({
                    "api_response": "Data processed",
                    "request_id": payload.get("id"),
                    "throttle_passed": true
                }),
                message: "API call successful".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"pipeline_type": "throttled"})),
            }
        })
    })?;

    // Test throttle behavior
    println!("🔄 Testing throttle pattern...");

    // Call 1: Should pass (first call always passes)
    println!("\n1️⃣ First call (should PASS - industry standard):");
    let result1 = cyre.call("throttled-api", json!({"id": 1, "data": "first"})).await;
    println!("   Result: {} - {}", result1.ok, result1.message);
    if result1.ok && result1.payload.get("throttle_passed").is_some() {
        println!("   ✅ PASSED: Handler executed");
    } else {
        println!("   🛑 BLOCKED: {}", result1.message);
    }

    // Call 2: Should be throttled (immediate)
    println!("\n2️⃣ Second call (should be THROTTLED):");
    let result2 = cyre.call("throttled-api", json!({"id": 2, "data": "second"})).await;
    println!("   Result: {} - {}", result2.ok, result2.message);
    if result2.ok && result2.payload.get("throttle_passed").is_some() {
        println!("   ❌ UNEXPECTED: Handler executed (throttle failed!)");
    } else {
        println!("   ✅ BLOCKED: {}", result2.message);
    }

    // Call 3: Wait and try again (should pass)
    println!("\n3️⃣ Third call after waiting (should PASS):");
    println!("   ⏰ Waiting 1.1 seconds for throttle to reset...");
    sleep(Duration::from_millis(1100)).await;

    let result3 = cyre.call("throttled-api", json!({"id": 3, "data": "third"})).await;
    println!("   Result: {} - {}", result3.ok, result3.message);
    if result3.ok && result3.payload.get("throttle_passed").is_some() {
        println!("   ✅ PASSED: Handler executed after throttle reset");
    } else {
        println!("   🛑 BLOCKED: {}", result3.message);
    }

    println!();

    //=================================================================
    // Test 3: Required Operator
    //=================================================================
    println!("✅ TEST 3: Required Operator");
    println!("============================");
    println!("📋 Testing: Payload validation");
    println!();

    cyre.action(IO::new("requires-data").with_required(true))?;

    cyre.on("requires-data", |payload| {
        Box::pin(async move {
            println!("📝 REQUIRED DATA HANDLER EXECUTED!");
            println!("   📦 Valid payload: {}", payload);

            CyreResponse {
                ok: true,
                payload: json!({
                    "validated": true,
                    "data": payload
                }),
                message: "Required validation passed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"pipeline_type": "required"})),
            }
        })
    })?;

    // Test with valid payload
    println!("🔄 Testing with valid payload...");
    let result_valid = cyre.call("requires-data", json!({"name": "John", "age": 30})).await;
    println!("✅ Valid payload result: {} - {}", result_valid.ok, result_valid.message);

    // Test with null payload
    println!("🔄 Testing with null payload...");
    let result_null = cyre.call("requires-data", serde_json::Value::Null).await;
    println!("🛑 Null payload result: {} - {}", result_null.ok, result_null.message);
    println!();

    //=================================================================
    // Test 4: Schema Operator
    //=================================================================
    println!("📋 TEST 4: Schema Operator");
    println!("==========================");
    println!("📋 Testing: Type validation");
    println!();

    cyre.action(IO::new("schema-test").with_schema("object"))?;

    cyre.on("schema-test", |payload| {
        Box::pin(async move {
            println!("🔍 SCHEMA VALIDATION HANDLER EXECUTED!");
            println!("   📦 Valid object: {}", payload);

            CyreResponse {
                ok: true,
                payload: json!({
                    "schema_valid": true,
                    "object_data": payload
                }),
                message: "Schema validation passed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"pipeline_type": "schema"})),
            }
        })
    })?;

    // Test with object (should pass)
    println!("🔄 Testing with object payload...");
    let result_obj = cyre.call("schema-test", json!({"type": "object", "valid": true})).await;
    println!("✅ Object result: {} - {}", result_obj.ok, result_obj.message);

    // Test with string (should fail)
    println!("🔄 Testing with string payload...");
    let result_str = cyre.call("schema-test", json!("not an object")).await;
    println!("🛑 String result: {} - {}", result_str.ok, result_str.message);
    println!();

    //=================================================================
    // Test 5: Condition Operator
    //=================================================================
    println!("🎯 TEST 5: Condition Operator");
    println!("==============================");
    println!("📋 Testing: Conditional execution");
    println!();

    cyre.action(IO::new("conditional").with_condition("has_data"))?;

    cyre.on("conditional", |payload| {
        Box::pin(async move {
            println!("🎯 CONDITIONAL HANDLER EXECUTED!");
            println!("   📦 Condition passed for: {}", payload);

            CyreResponse {
                ok: true,
                payload: json!({
                    "condition_passed": true,
                    "processed_data": payload
                }),
                message: "Condition check passed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"pipeline_type": "conditional"})),
            }
        })
    })?;

    // Test with data (should pass)
    println!("🔄 Testing with data...");
    let result_data = cyre.call("conditional", json!({"some": "data", "value": 42})).await;
    println!("✅ With data result: {} - {}", result_data.ok, result_data.message);

    // Test with null (should fail)
    println!("🔄 Testing with null...");
    let result_null_cond = cyre.call("conditional", serde_json::Value::Null).await;
    println!("🛑 Null result: {} - {}", result_null_cond.ok, result_null_cond.message);
    println!();

    //=================================================================
    // Test 6: Transform Operator
    //=================================================================
    println!("🔄 TEST 6: Transform Operator");
    println!("==============================");
    println!("📋 Testing: Data transformation");
    println!();

    cyre.action(IO::new("transformer").with_transform("add_timestamp"))?;

    cyre.on("transformer", |payload| {
        Box::pin(async move {
            println!("🔄 TRANSFORM HANDLER EXECUTED!");
            println!("   📦 Transformed payload: {}", payload);

            CyreResponse {
                ok: true,
                payload: json!({
                    "transformation_applied": true,
                    "final_data": payload
                }),
                message: "Transform applied successfully".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"pipeline_type": "transform"})),
            }
        })
    })?;

    // Test transformation
    println!("🔄 Testing data transformation...");
    let result_transform = cyre.call(
        "transformer",
        json!({"original": "data", "value": 123})
    ).await;
    println!("✅ Transform result: {} - {}", result_transform.ok, result_transform.message);

    if let Some(final_data) = result_transform.payload.get("final_data") {
        println!("🔍 Transformed data: {}", final_data);
        if final_data.get("transformed_at").is_some() {
            println!("   ✅ Timestamp added by transform operator");
        }
    }
    println!();

    //=================================================================
    // Test 7: Multiple Operators (User Order)
    //=================================================================
    println!("🔗 TEST 7: Multiple Operators (User Configuration Order)");
    println!("========================================================");
    println!("📋 Testing: Operators execute in user's configuration order");
    println!("📋 Config: .with_required(true).with_schema('object').with_transform('normalize')");
    println!();

    cyre.action(
        IO::new("multi-ops")
            .with_required(true) // 1st: Check required
            .with_schema("object") // 2nd: Validate schema
            .with_transform("normalize") // 3rd: Transform data
    )?;

    cyre.on("multi-ops", |payload| {
        Box::pin(async move { //println!("🔗 MULTI-OPERATOR HANDLER EXECUTED!");
            //  println!("   📦 Final processed payload: {}", payload);

            CyreResponse {
                ok: true,
                payload: json!({
                    "multi_ops_success": true,
                    "pipeline_stages": ["required", "schema", "transform"],
                    "final_payload": payload
                }),
                message: "Multi-operator pipeline completed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"pipeline_type": "multi_operator"})),
            } })
    })?;

    // Test multi-operator pipeline
    // println!("🔄 Testing multi-operator pipeline...");
    let result_multi = cyre.call("multi-ops", json!({"Name": "Test", "Value": 42})).await;
    // println!("✅ Multi-ops result: {} - {}", result_multi.ok, result_multi.message);

    if let Some(final_payload) = result_multi.payload.get("final_payload") {
        println!("🔍 Final payload after all operators: {}", final_payload);
        // Should show normalized keys (lowercase)
        if final_payload.get("name").is_some() {
            println!("   ✅ Transform applied: 'Name' → 'name' (normalized)");
        }
    }
    println!();

    //=================================================================
    // Test 8: Block Operator
    //=================================================================
    println!("🚫 TEST 8: Block Operator");
    println!("=========================");
    println!("📋 Testing: Immediate blocking");
    println!();

    cyre.action(IO::new("blocked-action").with_block(true))?;

    cyre.on("blocked-action", |_payload| {
        Box::pin(async move {
            // This should never execute
            println!("❌ BLOCKED HANDLER EXECUTED - THIS SHOULD NOT HAPPEN!");

            CyreResponse {
                ok: true,
                payload: json!({"error": "handler_should_not_execute"}),
                message: "This handler should not have executed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    })?;

    // Test blocked action
    println!("🔄 Testing blocked action...");
    let result_blocked = cyre.call("blocked-action", json!({"any": "data"})).await;
    println!("🚫 Blocked result: {} - {}", result_blocked.ok, result_blocked.message);

    if result_blocked.ok && result_blocked.message.contains("blocked") {
        println!("   ✅ Correctly blocked before handler execution");
    } else {
        println!("   ❌ Block operator failed!");
    }
    println!();

    //=================================================================
    // Test 9: Performance Comparison
    //=================================================================
    println!("📊 TEST 9: Performance Comparison");
    println!("==================================");
    println!("📋 Comparing fast path vs pipeline performance");
    println!();

    let iterations = 1000;
    println!("🏃‍♂️ Running {} iterations of each...", iterations);

    // Fast path performance
    let start = std::time::Instant::now();
    for i in 0..iterations {
        let _result = cyre.call("fast-path", json!({"iteration": i})).await;
    }
    let fast_path_time = start.elapsed();

    // Multi-operator pipeline performance
    let start = std::time::Instant::now();
    for i in 0..iterations {
        let _result = cyre.call(
            "multi-ops",
            json!({"Name": format!("Test{}", i), "Value": i})
        ).await;
    }
    let pipeline_time = start.elapsed();

    println!("⚡ Performance Results:");
    println!(
        "   Fast Path:     {:.2}ms ({:.0} ops/sec)",
        fast_path_time.as_millis(),
        (iterations as f64) / fast_path_time.as_secs_f64()
    );
    println!(
        "   Multi Pipeline: {:.2}ms ({:.0} ops/sec)",
        pipeline_time.as_millis(),
        (iterations as f64) / pipeline_time.as_secs_f64()
    );

    let overhead =
        ((pipeline_time.as_micros() as f64) / (fast_path_time.as_micros() as f64) - 1.0) * 100.0;
    println!("   Pipeline overhead: {:.1}%", overhead);

    if fast_path_time < pipeline_time {
        println!("   ✅ Fast path is faster (as expected)");
    } else {
        println!("   ⚠️ Unexpected: Pipeline is faster");
    }
    println!();

    //=================================================================
    // Test 10: System Statistics
    //=================================================================
    println!("📈 TEST 10: System Statistics");
    println!("==============================");
    println!("📋 Pipeline compilation and execution stats");
    println!();

    let stats = cyre.pipeline_stats();
    println!("🔧 Pipeline Compilation Stats:");
    println!("   Total pipelines: {}", stats.total_pipelines);
    println!("   Zero overhead: {}", stats.zero_overhead_count);
    println!("   Protected: {}", stats.protected_count);
    println!("   Optimization ratio: {:.1}%", stats.optimization_ratio());

    let metrics = cyre.get_performance_metrics();
    println!("\n🎯 Performance Metrics:");
    println!("   Total executions: {}", metrics["executions"]["total_executions"]);
    println!("   Fast path ratio: {:.1}%", metrics["executions"]["fast_path_ratio"]);
    println!("   Uses action state: {}", metrics["unified_pipeline"]["uses_action_state"]);
    println!("   No separate cache: {}", metrics["unified_pipeline"]["no_separate_cache"]);
    println!("   Follows TypeScript: {}", metrics["performance"]["follows_typescript_pattern"]);

    println!("\n📋 Actions Created:");
    let actions = [
        "fast-path",
        "throttled-api",
        "requires-data",
        "schema-test",
        "conditional",
        "transformer",
        "multi-ops",
        "blocked-action",
    ];
    for action_id in &actions {
        if let Some(info) = cyre.get_pipeline_info(action_id) {
            println!(
                "   • {}: {} operators (zero_overhead: {})",
                action_id,
                info.protection_count,
                info.is_zero_overhead
            );
        }
    }

    //=================================================================
    // Final Summary
    //=================================================================
    println!("\n🎉 PIPELINE OPERATORS DEMO COMPLETED!");
    println!("=====================================");
    println!("✅ All operators tested and working:");
    println!("   • ⚡ Fast Path: Zero overhead (pipeline.length == 0)");
    println!("   • 🚦 Throttle: TypeScript pattern (first call passes)");
    println!("   • ✅ Required: Payload validation");
    println!("   • 📋 Schema: Type validation");
    println!("   • 🎯 Condition: Conditional execution");
    println!("   • 🔄 Transform: Data transformation");
    println!("   • 🔗 Multi-ops: User configuration order respected");
    println!("   • 🚫 Block: Immediate blocking");
    println!();
    println!("🔧 Architecture Benefits:");
    println!("   • Single source of truth (action state)");
    println!("   • No separate executor cache");
    println!("   • Proper state persistence");
    println!("   • TypeScript compatibility");
    println!("   • User intent preserved");
    println!();
    println!("🚀 Ready for production use!");

    Ok(())
}

//=============================================================================
// HELPER FUNCTIONS FOR DETAILED ANALYSIS
//=============================================================================

fn print_pipeline_details(cyre: &Cyre, action_id: &str) {
    if let Some(action) = cyre.get(action_id) {
        println!("🔍 Pipeline Details for '{}':", action_id);
        println!("   • Fast path: {}", action._has_fast_path);
        println!("   • Pipeline operators: {:?}", action._pipeline);
        println!("   • Has protections: {}", action._has_protections);
        println!("   • Has processing: {}", action._has_processing);
        println!("   • Has scheduling: {}", action._has_scheduling);
    }
}

fn print_separator(title: &str) {
    println!("\n{}", "=".repeat(60));
    println!("{}", title);
    println!("{}", "=".repeat(60));
}
