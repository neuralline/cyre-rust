// examples/validation_debug.rs
// Debug validation and protection issues

use cyre_rust::prelude::*;
use cyre_rust::{ validate_field, DataDefResult, compile_pipeline };
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 CYRE VALIDATION DEBUG");
    println!("========================");

    // =================================================================
    // Test 1: Direct Schema Validation (bypassing Cyre)
    // =================================================================
    println!("\n📋 Test 1: Direct Schema Validation");
    println!("===================================");

    let validation_tests = vec![
        // Valid cases
        ("id", json!("test-action"), "Valid ID"),
        ("throttle", json!(1000), "Valid throttle"),
        ("required", json!(true), "Required true"),
        ("required", json!("non-empty"), "Required non-empty"),

        // Invalid cases that should show errors + suggestions
        ("id", json!(""), "Empty ID"),
        ("id", json!(null), "Null ID"),
        ("throttle", json!(-1), "Negative throttle"),
        ("throttle", json!("invalid"), "Non-numeric throttle"),
        ("required", json!("invalid-value"), "Invalid required"),
        ("debounce", json!(0), "Zero debounce"),
        ("unknown_field", json!("test"), "Unknown field")
    ];

    for (field_name, value, description) in validation_tests {
        println!("\n🔍 Testing: {} ({})", description, field_name);
        println!("   Input: {}", value);

        let result: DataDefResult = validate_field(field_name, &value);

        println!("   ✅ Valid: {}", result.ok);

        if let Some(error) = &result.error {
            println!("   ❌ Error: {}", error);
        }

        if let Some(blocking) = result.blocking {
            println!("   🚫 Blocking: {}", blocking);
        }

        if let Some(talent) = &result.talent_name {
            println!("   🎯 Talent: {}", talent);
        }

        if let Some(suggestions) = &result.suggestions {
            println!("   💡 Suggestions:");
            for suggestion in suggestions {
                println!("      - {}", suggestion);
            }
        }
    }

    // =================================================================
    // Test 2: Pipeline Compilation Errors
    // =================================================================
    println!("\n\n⚙️ Test 2: Pipeline Compilation Errors");
    println!("======================================");

    // Test various invalid configurations
    let invalid_configs = vec![
        ("Empty ID", IO::new("")),
        (
            "Invalid throttle",
            {
                let mut config = IO::new("test");
                config.throttle = Some(0); // Should be > 0
                config
            },
        )
    ];

    for (description, mut config) in invalid_configs {
        println!("\n🔧 Testing: {}", description);
        println!("   Config: {:?}", config.id);

        let compile_result = compile_pipeline(&mut config);

        println!("   ✅ Compilation Success: {}", compile_result.ok);

        if !compile_result.errors.is_empty() {
            println!("   ❌ Compilation Errors:");
            for error in &compile_result.errors {
                println!("      - {}", error);
            }
        }

        if !compile_result.suggestions.is_empty() {
            println!("   💡 Compilation Suggestions:");
            for suggestion in &compile_result.suggestions {
                println!("      - {}", suggestion);
            }
        }
    }

    // =================================================================
    // Test 3: Protection Mechanisms in Isolation
    // =================================================================
    println!("\n\n🛡️ Test 3: Protection Mechanisms");
    println!("=================================");

    let mut cyre = Cyre::new();
    cyre.init().await?;

    // Create action with ONLY throttle to test isolation
    println!("\n🔍 Testing Throttle Protection Isolation");
    cyre.action(
        IO::new("throttle-test").with_throttle(2000) // 2 second throttle
    )?;

    cyre.on("throttle-test", |payload| {
        Box::pin(async move {
            println!("   🎯 HANDLER EXECUTED: {}", payload);
            CyreResponse {
                ok: true,
                payload: json!({"executed": true, "timestamp": current_timestamp()}),
                message: "Handler executed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    })?;

    // Test rapid fire to see throttling
    println!("   Rapid fire test (should see throttling):");
    for i in 1..=4 {
        let start = std::time::Instant::now();
        let result = cyre.call("throttle-test", json!({"call": i})).await;
        let duration = start.elapsed();

        println!(
            "   Call {}: success={}, message='{}', took={:.2}ms",
            i,
            result.ok,
            result.message,
            duration.as_millis()
        );

        // Small delay between calls
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // =================================================================
    // Test 4: Required Validation Debug
    // =================================================================
    println!("\n\n📋 Test 4: Required Validation Debug");
    println!("====================================");

    // Test different required configurations
    println!("\n🔍 Testing Required=true");
    cyre.action(IO::new("required-true").with_required(true))?;

    cyre.on("required-true", |payload| {
        Box::pin(async move {
            println!("   🎯 Required-true handler got: {}", payload);
            CyreResponse::success(json!({"received": payload}), "Required validation passed")
        })
    })?;

    let required_tests = vec![
        (json!({"data": "test"}), "Valid object"),
        (json!("string"), "Valid string"),
        (json!(42), "Valid number"),
        (json!(true), "Valid boolean"),
        (json!([1, 2, 3]), "Valid array"),
        (json!({}), "Empty object"),
        (json!(""), "Empty string"),
        (json!(null), "Null value")
    ];

    for (payload, description) in required_tests {
        let result = cyre.call("required-true", payload.clone()).await;
        println!("   {}: success={}, message='{}'", description, result.ok, result.message);
    }

    // Test non-empty required
    println!("\n🔍 Testing Required=non-empty");
    cyre.action(IO::new("required-non-empty").with_required_non_empty())?;

    cyre.on("required-non-empty", |payload| {
        Box::pin(async move {
            println!("   🎯 Required-non-empty handler got: {}", payload);
            CyreResponse::success(json!({"received": payload}), "Non-empty validation passed")
        })
    })?;

    for (payload, description) in vec![
        (json!({"data": "test"}), "Valid object"),
        (json!("string"), "Valid string"),
        (json!({}), "Empty object (should fail)"),
        (json!(""), "Empty string (should fail)"),
        (json!(null), "Null value (should fail)")
    ] {
        let result = cyre.call("required-non-empty", payload.clone()).await;
        println!("   {}: success={}, message='{}'", description, result.ok, result.message);
    }

    // =================================================================
    // Test 5: Manual Pipeline Execution
    // =================================================================
    println!("\n\n🔧 Test 5: Manual Pipeline Execution");
    println!("====================================");

    use cyre_rust::execute_pipeline;

    // Create a config and manually execute pipeline
    let mut test_config = IO::new("manual-test").with_required(true).with_throttle(1000);

    // Compile it first
    let compile_result = compile_pipeline(&mut test_config);
    println!(
        "Manual pipeline compilation: success={}, operators={}",
        compile_result.ok,
        compile_result.operators.len()
    );

    // Try executing with different payloads
    let test_payloads = vec![
        (json!({"valid": "data"}), "Valid payload"),
        (json!(null), "Null payload"),
        (json!({}), "Empty object")
    ];

    for (payload, description) in test_payloads {
        match execute_pipeline(&mut test_config, payload.clone()).await {
            Ok(result) => {
                match result {
                    crate::PipelineResult::Continue(processed) => {
                        println!("   {}: CONTINUE with {}", description, processed);
                    }
                    crate::PipelineResult::Block(reason) => {
                        println!("   {}: BLOCKED - {}", description, reason);
                    }
                    crate::PipelineResult::Schedule => {
                        println!("   {}: SCHEDULED", description);
                    }
                }
            }
            Err(error) => {
                println!("   {}: ERROR - {}", description, error);
            }
        }
    }

    println!("\n🎉 VALIDATION DEBUG COMPLETED!");
    println!("===============================");
    println!("Check the output above to see what's working and what's not.");
    println!("This will help identify why protection mechanisms aren't triggering.");

    Ok(())
}
