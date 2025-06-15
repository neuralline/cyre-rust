// examples/simple_usage.rs
// Simple usage example demonstrating modular Cyre

use cyre_rust::prelude::*;
use cyre_rust::talent::{Talent, TalentRegistry, functions::*};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 SIMPLE CYRE USAGE EXAMPLE");
    println!("==============================");
    println!("Demonstrating modular architecture and clean API");
    println!();

    // Initialize Cyre
    let mut cyre = Cyre::new();
    println!("✅ Cyre initialized");

    // Example 1: Basic action and handler
    println!("\n📋 Example 1: Basic Action and Handler");
    println!("--------------------------------------");
    
    cyre.action(IO::new("greet"));
    cyre.on("greet", |payload| {
        Box::pin(async move {
            let name = payload.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("World");
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "message": format!("Hello, {}!", name),
                    "timestamp": current_timestamp()
                }),
                message: "Greeting generated".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    });

    let response = cyre.call("greet", json!({"name": "Alice"})).await;
    println!("Response: {}", response.payload.get("message").unwrap());

    // Example 2: Protected action with throttling
    println!("\n🛡️ Example 2: Protected Action (Throttling)");
    println!("--------------------------------------------");
    
    let protected_config = IO::new("api-call")
        .with_throttle(1000) // 1 second throttle
        .with_priority(Priority::High);
    
    cyre.action(protected_config);
    cyre.on("api-call", |payload| {
        Box::pin(async move {
            // Simulate API processing
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "api_response": "Data fetched successfully",
                    "request_id": payload.get("id")
                }),
                message: "API call completed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    });

    // First call should succeed
    let response1 = cyre.call("api-call", json!({"id": 1})).await;
    println!("First call: {} ({})", response1.ok, response1.message);

    // Second immediate call should be blocked
    let response2 = cyre.call("api-call", json!({"id": 2})).await;
    println!("Second call: {} ({})", response2.ok, response2.message);

    // Example 3: Data processing with talents
    println!("\n🧠 Example 3: Data Processing with Talents");
    println!("------------------------------------------");
    
    // Create talent registry
    let talent_registry = TalentRegistry::new();
    
    // Register some talents
    let validation_talent = Talent::schema(
        "validate-user",
        "User Validation",
        "Validates user data",
        require_fields(&["name", "email"])
    );
    
    let transform_talent = Talent::transform(
        "add-metadata",
        "Add Metadata",
        "Adds processing metadata",
        add_timestamp("processed_at")
    );
    
    let cleanup_talent = Talent::transform(
        "cleanup-data",
        "Data Cleanup",
        "Removes sensitive fields",
        remove_field("password")
    );
    
    talent_registry.register_talent(validation_talent)?;
    talent_registry.register_talent(transform_talent)?;
    talent_registry.register_talent(cleanup_talent)?;
    
    println!("Registered {} talents", talent_registry.talent_count());
    
    // Test talent execution
    let user_data = json!({
        "name": "Bob Smith",
        "email": "bob@example.com",
        "password": "secret123",
        "age": 30
    });
    
    let config = IO::new("process-user");
    let talent_ids = vec![
        "validate-user".to_string(),
        "add-metadata".to_string(),
        "cleanup-data".to_string()
    ];
    
    let talent_result = talent_registry.execute_talents(&talent_ids, &config, user_data);
    
    if talent_result.success {
        println!("✅ Talent processing succeeded");
        println!("Final payload: {}", serde_json::to_string_pretty(&talent_result.payload)?);
    } else {
        println!("❌ Talent processing failed: {}", talent_result.error.unwrap_or_default());
    }

    // Example 4: Performance demonstration
    println!("\n⚡ Example 4: Performance Demonstration");
    println!("---------------------------------------");
    
    // Setup fast path action
    cyre.action(IO::new("fast-action"));
    cyre.on("fast-action", |payload| {
        Box::pin(async move {
            CyreResponse {
                ok: true,
                payload: json!({
                    "processed": payload.get("value"),
                    "fast_path": true
                }),
                message: "Fast processing".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    });

    // Perform multiple fast calls
    let start_time = std::time::Instant::now();
    let iterations = 1000;
    
    for i in 0..iterations {
        let _response = cyre.call("fast-action", json!({"value": i})).await;
    }
    
    let duration = start_time.elapsed();
    let ops_per_sec = (iterations as f64 / duration.as_secs_f64()) as u64;
    
    println!("Executed {} operations in {:.2}ms", iterations, duration.as_millis());
    println!("Performance: {} ops/sec", ops_per_sec);

    // Show metrics
    println!("\n📊 System Metrics");
    println!("------------------");
    let metrics = cyre.get_performance_metrics();
    println!("Total executions: {}", metrics["total_executions"]);
    println!("Fast path hits: {}", metrics["fast_path_hits"]);
    println!("Fast path ratio: {:.1}%", metrics["fast_path_ratio"]);
    println!("Active channels: {}", metrics["active_channels"]);
    
    let talent_metrics = talent_registry.get_metrics();
    println!("Talent executions: {}", talent_metrics["execution"]["talent_executions"]);
    println!("Talent success rate: {:.1}%", talent_metrics["execution"]["success_rate"]);

    println!("\n🎉 Example completed successfully!");
    println!("🏗️ Modular architecture benefits:");
    println!("  • Clean separation of concerns");
    println!("  • Easy to test individual components");  
    println!("  • Maintainable and extensible");
    println!("  • Performance optimizations preserved");
    println!("  • Type-safe with Rust's guarantees");

    Ok(())
}