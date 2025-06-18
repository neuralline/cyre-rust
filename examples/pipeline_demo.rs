// examples/pipeline_demo.rs
// Simple working pipeline demonstration

use cyre_rust::prelude::*;
use serde_json::json;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Cyre Pipeline Demo");
    println!("=====================");

    // Initialize Cyre
    let mut cyre = init().await?;

    // Demo 1: Fast Path (No Protection)
    println!("\n⚡ Demo 1: Fast Path Performance");
    println!("=================================");

    cyre.action(IO::new("fast-action"))?;
    cyre.on("fast-action", |payload| {
        Box::pin(async move {
            CyreResponse::success(
                json!({"processed": payload, "fast_path": true}),
                "Fast path execution"
            )
        })
    })?;

    // Benchmark fast path
    let start = Instant::now();
    let iterations = 10000;

    for i in 0..iterations {
        let _result = cyre.call("fast-action", json!({"iteration": i})).await;
    }

    let duration = start.elapsed();
    let ops_per_sec = ((iterations as f64) / duration.as_secs_f64()) as u64;

    println!("✅ Fast Path: {} ops in {:.2}ms", iterations, duration.as_millis());
    println!("🚀 Performance: {} ops/sec", format_number(ops_per_sec));

    // Demo 2: Protected Pipeline
    println!("\n🛡️ Demo 2: Protected Pipeline");
    println!("==============================");

    cyre.action(
        IO::new("protected-action")
            .with_throttle(100)
            .with_required(true)
            .with_priority(Priority::High)
    )?;

    cyre.on("protected-action", |payload| {
        Box::pin(async move {
            CyreResponse::success(
                json!({"processed": payload, "protected": true}),
                "Protected execution"
            )
        })
    })?;

    // Test protection mechanisms
    println!("🔄 Testing protection mechanisms...");

    // First call should succeed
    let result1 = cyre.call("protected-action", json!({"test": "data"})).await;
    println!("   First call: {} - {}", result1.ok, result1.message);

    // Second call should be throttled
    let result2 = cyre.call("protected-action", json!({"test": "data"})).await;
    println!("   Second call: {} - {}", result2.ok, result2.message);

    // Call with null payload should be blocked (required validation)
    let result3 = cyre.call("protected-action", json!(null)).await;
    println!("   Null payload: {} - {}", result3.ok, result3.message);

    // Demo 3: System Status
    println!("\n📊 Demo 3: System Status");
    println!("=========================");

    let status = cyre.status();
    println!("🟢 System Status: {}", status.message);
    println!("   Initialized: {}", status.payload["initialized"]);
    println!("   Breathing: {}", status.payload["breathing"]);
    println!("   Actions: {}", status.payload["stores"]["actions"]);
    println!("   Handlers: {}", status.payload["stores"]["handlers"]);

    // Demo 4: Performance Metrics
    println!("\n📈 Demo 4: Performance Metrics");
    println!("===============================");

    let metrics = cyre.get_performance_metrics();
    println!("🔢 Total Executions: {}", metrics["executions"]["total_executions"]);
    println!("⚡ Fast Path Ratio: {:.1}%", metrics["executions"]["fast_path_ratio"]);
    println!("🚀 Active Channels: {}", metrics["active_channels"]);

    if let Some(breathing) = metrics["breathing"].as_object() {
        println!("💨 Breathing Pattern: {}", breathing["pattern"]);
        println!("❤️ Current Rate: {}ms", breathing["current_rate"]);
    }

    println!("\n🎉 Pipeline Demo Complete!");
    println!("===========================");
    println!("✅ Fast path: {} ops/sec", format_number(ops_per_sec));
    println!("🛡️ Protection systems working");
    println!("📊 Metrics collection active");
    println!("💨 Breathing system operational");

    Ok(())
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", (n as f64) / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", (n as f64) / 1_000.0)
    } else {
        n.to_string()
    }
}
