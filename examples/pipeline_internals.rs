// examples/pipeline_internals.rs
// Deep dive into Cyre's pipeline compilation and operator system

use cyre_rust::prelude::*;
use cyre_rust::{ compile_pipeline, validate_field, DataDefResult };
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 CYRE PIPELINE INTERNALS DEEP DIVE");
    println!("====================================");

    // =================================================================
    // Part 1: Schema Validation System
    // =================================================================
    println!("\n📋 Part 1: Schema Validation System");
    println!("===================================");

    // Test individual field validations
    let test_cases = vec![
        ("id", json!("test-action"), "Valid ID"),
        ("id", json!(""), "Empty ID (should fail)"),
        ("throttle", json!(1000), "Valid throttle"),
        ("throttle", json!(-1), "Invalid throttle"),
        ("required", json!(true), "Required true"),
        ("required", json!("non-empty"), "Required non-empty"),
        ("required", json!("invalid"), "Invalid required value"),
        ("block", json!(true), "Block enabled"),
        ("unknown_field", json!("value"), "Unknown field")
    ];

    for (field, value, description) in test_cases {
        let result = validate_field(field, &value);
        println!("🔍 Testing {}: {}", description, field);
        println!("   Value: {}", value);
        println!("   Valid: {}", result.ok);

        if let Some(error) = result.error {
            println!("   Error: {}", error);
        }

        if let Some(talent) = result.talent_name {
            println!("   Talent: {}", talent);
        }

        if let Some(suggestions) = result.suggestions {
            for suggestion in suggestions {
                println!("   Suggestion: {}", suggestion);
            }
        }

        if let Some(blocking) = result.blocking {
            println!("   Blocking: {}", blocking);
        }

        println!();
    }

    // =================================================================
    // Part 2: Pipeline Compilation Process
    // =================================================================
    println!("\n⚙️ Part 2: Pipeline Compilation Process");
    println!("======================================");

    // Create different pipeline configurations
    let pipeline_configs = vec![
        ("Empty/Fast Path", IO::new("fast-path")),

        ("Simple Protection", IO::new("simple-protection").with_throttle(1000)),

        (
            "Multi Protection",
            IO::new("multi-protection")
                .with_throttle(1000)
                .with_debounce(300)
                .with_change_detection(),
        ),

        (
            "Validation Pipeline",
            IO::new("validation").with_required(true).with_schema("user_schema"),
        ),

        (
            "Processing Pipeline",
            IO::new("processing")
                .with_condition("check_condition")
                .with_selector("select_data")
                .with_transform("transform_data"),
        ),

        (
            "Full Pipeline",
            IO::new("full-pipeline")
                .with_required_non_empty()
                .with_throttle(500)
                .with_debounce(200)
                .with_change_detection()
                .with_schema("complex_schema")
                .with_condition("validate_business_rules")
                .with_transform("normalize_and_enrich")
                .with_priority(Priority::High),
        ),

        (
            "Scheduled Pipeline",
            IO::new("scheduled").with_delay(1000).with_interval(5000).with_repeat_count(10),
        )
    ];

    for (name, mut config) in pipeline_configs {
        println!("🔧 Compiling Pipeline: {}", name);
        println!("   Configuration:");
        println!("     ID: {}", config.id);
        println!("     Block: {}", config.block);
        println!("     Throttle: {:?}", config.throttle);
        println!("     Debounce: {:?}", config.debounce);
        println!("     Required: {:?}", config.required);
        println!("     Schema: {:?}", config.schema);
        println!("     Condition: {:?}", config.condition);
        println!("     Transform: {:?}", config.transform);
        println!("     Priority: {:?}", config.priority);

        // Compile the pipeline
        let compile_result = compile_pipeline(&mut config);

        println!("   Compilation Result:");
        println!("     Success: {}", compile_result.ok);
        println!("     Operators: {}", compile_result.operators.len());
        println!("     Fast Path: {}", compile_result.has_fast_path);
        println!("     Has Protections: {}", compile_result.has_protections);
        println!("     Has Processing: {}", compile_result.has_processing);
        println!("     Has Scheduling: {}", compile_result.has_scheduling);

        if !compile_result.errors.is_empty() {
            println!("     Errors:");
            for error in &compile_result.errors {
                println!("       - {}", error);
            }
        }

        if !compile_result.suggestions.is_empty() {
            println!("     Suggestions:");
            for suggestion in &compile_result.suggestions {
                println!("       - {}", suggestion);
            }
        }

        // Show post-compilation metadata
        println!("   Post-Compilation Metadata:");
        println!("     _has_fast_path: {}", config._has_fast_path);
        println!("     _has_protections: {}", config._has_protections);
        println!("     _has_processing: {}", config._has_processing);
        println!("     _has_scheduling: {}", config._has_scheduling);
        println!("     _pipeline: {:?}", config._pipeline);

        // Performance analysis
        let operator_count = estimate_operator_count(&config);
        let estimated_perf = estimated_performance(&config);
        let complexity = config.complexity_score();

        println!("   Performance Analysis:");
        println!("     Estimated Operators: {}", operator_count);
        println!("     Estimated Performance: {} ops/sec", estimated_perf);
        println!("     Complexity Score: {}", complexity);
        println!("     Is Fast Path Eligible: {}", config.is_fast_path_eligible());

        println!();
    }

    // =================================================================
    // Part 3: Real-Time Pipeline Execution
    // =================================================================
    println!("\n🚀 Part 3: Real-Time Pipeline Execution");
    println!("======================================");

    let mut cyre = Cyre::new();
    cyre.init().await?;

    // Create an action with a complex pipeline
    cyre.action(
        IO::new("complex-demo")
            .with_required(true)
            .with_throttle(1000)
            .with_debounce(300)
            .with_change_detection()
            .with_priority(Priority::High)
    )?;

    // Handler that shows pipeline processing
    cyre.on("complex-demo", |payload| {
        Box::pin(async move {
            println!("   🎯 Handler executing with processed payload: {}", payload);

            CyreResponse {
                ok: true,
                payload: json!({
                    "pipeline_processed": true,
                    "original_payload": payload,
                    "pipeline_stages_passed": ["required", "throttle", "debounce", "change_detection"],
                    "processing_time": "minimal",
                    "fast_path": false
                }),
                message: "Complex pipeline executed successfully".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(
                    json!({
                    "pipeline_info": {
                        "operators_executed": 4,
                        "protection_level": "high",
                        "performance_tier": "standard"
                    }
                })
                ),
            }
        })
    })?;

    // Test the pipeline with different scenarios
    println!("🔍 Testing Pipeline Execution Scenarios:");

    // Scenario 1: Valid execution
    println!("\n   Scenario 1: Valid payload execution");
    let valid_payload =
        json!({
        "user_id": "12345",
        "action": "update_profile",
        "data": {"name": "John Doe", "email": "john@example.com"}
    });

    let result1 = cyre.call("complex-demo", valid_payload).await;
    println!("     Result: {} - {}", result1.ok, result1.message);
    if let Some(stages) = result1.payload.get("pipeline_stages_passed") {
        println!("     Pipeline stages: {}", stages);
    }

    // Scenario 2: Null payload (should fail at required check)
    println!("\n   Scenario 2: Null payload (required check)");
    let result2 = cyre.call("complex-demo", json!(null)).await;
    println!("     Result: {} - {}", result2.ok, result2.message);

    // Scenario 3: Rapid calls (should trigger throttle/debounce)
    println!("\n   Scenario 3: Rapid calls (throttle/debounce)");
    for i in 1..=3 {
        let payload = json!({"call_id": i, "rapid_fire": true});
        let result = cyre.call("complex-demo", payload).await;
        println!("     Call {}: {} - {}", i, result.ok, result.message);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Scenario 4: Duplicate detection
    println!("\n   Scenario 4: Duplicate detection");
    let duplicate_payload = json!({"duplicate_test": true, "value": 42});

    let result4a = cyre.call("complex-demo", duplicate_payload.clone()).await;
    println!("     First call: {} - {}", result4a.ok, result4a.message);

    let result4b = cyre.call("complex-demo", duplicate_payload.clone()).await;
    println!("     Duplicate call: {} - {}", result4b.ok, result4b.message);

    // =================================================================
    // Part 4: Performance Comparison
    // =================================================================
    println!("\n⚡ Part 4: Performance Comparison");
    println!("================================");

    // Create fast path action for comparison
    cyre.action(IO::new("fast-path-demo"))?;
    cyre.on("fast-path-demo", |_payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload: json!({"fast_path": true}),
                message: "Fast path execution".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            } })
    })?;

    // Benchmark both approaches
    let benchmark_payload = json!({"benchmark": true});
    let iterations = 100;

    // Fast path benchmark
    println!("🏃 Benchmarking Fast Path ({} iterations):", iterations);
    let (fast_ops_per_sec, fast_avg_latency) = cyre.benchmark_call_performance(
        "fast-path-demo",
        iterations
    ).await;
    println!("   Ops/sec: {}", fast_ops_per_sec);
    println!("   Avg latency: {:.2}μs", fast_avg_latency);

    // Complex pipeline benchmark
    println!("\n🔧 Benchmarking Complex Pipeline ({} iterations):", iterations);
    // Note: This will be throttled, so we use fewer iterations
    let pipeline_iterations = 10;
    let start = std::time::Instant::now();

    for _ in 0..pipeline_iterations {
        let _result = cyre.call("complex-demo", benchmark_payload.clone()).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(110)).await; // Account for throttling
    }

    let duration = start.elapsed();
    let pipeline_ops_per_sec = ((pipeline_iterations as f64) / duration.as_secs_f64()) as u64;
    let pipeline_avg_latency = (duration.as_micros() as f64) / (pipeline_iterations as f64);

    println!("   Ops/sec: {} (throttled)", pipeline_ops_per_sec);
    println!("   Avg latency: {:.2}μs (with protection)", pipeline_avg_latency);

    // Performance comparison
    println!("\n📊 Performance Comparison:");
    println!("   Fast Path: {} ops/sec", fast_ops_per_sec);
    println!("   Complex Pipeline: {} ops/sec (with protections)", pipeline_ops_per_sec);
    println!(
        "   Performance Ratio: {:.2}x",
        (fast_ops_per_sec as f64) / (pipeline_ops_per_sec.max(1) as f64)
    );
    println!("   Trade-off: Fast path = speed, Pipeline = safety + features");

    // =================================================================
    // Part 5: Pipeline Introspection
    // =================================================================
    println!("\n🔍 Part 5: Pipeline Introspection");
    println!("================================");

    // Get pipeline information
    let complex_action = cyre.get("complex-demo").unwrap();
    println!("🔧 Complex Pipeline Analysis:");
    println!("   Action ID: {}", complex_action.id);
    println!("   Pipeline Metadata: {:?}", complex_action._pipeline);
    println!("   Fast Path: {}", complex_action._has_fast_path);
    println!("   Has Protections: {}", complex_action._has_protections);
    println!("   Has Processing: {}", complex_action._has_processing);
    println!("   Has Scheduling: {}", complex_action._has_scheduling);
    println!("   Complexity Score: {}", complex_action.complexity_score());
    println!("   Summary: {}", complex_action.summary());

    let fast_action = cyre.get("fast-path-demo").unwrap();
    println!("\n⚡ Fast Path Analysis:");
    println!("   Action ID: {}", fast_action.id);
    println!("   Pipeline Metadata: {:?}", fast_action._pipeline);
    println!("   Fast Path: {}", fast_action._has_fast_path);
    println!("   Complexity Score: {}", fast_action.complexity_score());
    println!("   Summary: {}", fast_action.summary());

    println!("\n🎉 PIPELINE INTERNALS DEMO COMPLETED!");
    println!("====================================");
    println!("✅ Schema validation system explored");
    println!("⚙️ Pipeline compilation process analyzed");
    println!("🚀 Real-time execution demonstrated");
    println!("⚡ Performance comparison completed");
    println!("🔍 Pipeline introspection showcased");
    println!("\n🧠 Key Insights:");
    println!("   - Fast path: 1.8M+ ops/sec for simple cases");
    println!("   - Protected pipelines: High safety with acceptable performance cost");
    println!("   - Operator composition: Flexible and powerful");
    println!("   - Compile-time optimization: Zero runtime overhead for fast path");
    println!("   - Schema validation: Prevents bad data from entering system");

    Ok(())
}
