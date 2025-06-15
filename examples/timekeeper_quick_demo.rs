// examples/timekeeper_quick_demo.rs
// Demonstrates quick scheduling methods with TimeKeeper

use cyre_rust::prelude::*;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🕒 TIMEKEEPER QUICK SCHEDULING DEMO");
    println!("===================================");
    println!("Demonstrating IO::delayed, interval, repeat, and complex methods");
    println!();

    let mut cyre = Cyre::new();
    cyre.init_timekeeper().await?;

    // =================================================================
    // Quick Scheduling Registration
    // =================================================================
    println!("🔧 Registering quick scheduled actions...");

    // 1. setTimeout equivalent - executes once after delay
    cyre.action(IO::delayed("delayed-task", 2000));

    // 2. setInterval equivalent - executes repeatedly forever
    cyre.action(IO::interval("monitor", 1500));

    // 3. Finite repetition - executes N times with interval
    cyre.action(IO::repeat("backup", 1000, 3));

    // 4. Complex scheduling - delay + interval + repeat count
    cyre.action(IO::complex("cleanup", 1000, 800, 4));

    println!("✅ All quick scheduling actions registered");
    println!();

    // =================================================================
    // Handler Registration
    // =================================================================
    println!("🔧 Registering handlers for scheduled actions...");

    // Handler for delayed task (setTimeout)
    cyre.on("delayed-task", |payload| {
        Box::pin(async move {
            println!("⏰ DELAYED TASK executed: {}", payload);
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "executed_at": current_timestamp(),
                    "type": "delayed",
                    "original_payload": payload
                }),
                message: "Delayed task completed".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"scheduling_type": "delayed"})),
            }
        })
    });

    // Handler for interval monitoring (setInterval)
    cyre.on("monitor", |payload| {
        Box::pin(async move {
            let check_id = payload.get("check_id").and_then(|v| v.as_u64()).unwrap_or(1);
            println!("🔄 MONITOR CHECK #{}: {}", check_id, payload);
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "check_id": check_id,
                    "status": "healthy",
                    "checked_at": current_timestamp(),
                    "type": "interval"
                }),
                message: format!("Monitor check #{} completed", check_id),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"scheduling_type": "interval"})),
            }
        })
    });

    // Handler for backup repeat
    cyre.on("backup", |payload| {
        Box::pin(async move {
            let iteration = payload.get("iteration").and_then(|v| v.as_u64()).unwrap_or(1);
            println!("💾 BACKUP iteration #{}: {}", iteration, payload);
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "iteration": iteration,
                    "backup_size": format!("{}MB", iteration * 10),
                    "backed_up_at": current_timestamp(),
                    "type": "repeat"
                }),
                message: format!("Backup iteration #{} completed", iteration),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"scheduling_type": "repeat"})),
            }
        })
    });

    // Handler for complex cleanup
    cyre.on("cleanup", |payload| {
        Box::pin(async move {
            let round = payload.get("round").and_then(|v| v.as_u64()).unwrap_or(1);
            println!("🧹 CLEANUP round #{}: {}", round, payload);
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "round": round,
                    "cleaned_files": round * 25,
                    "cleaned_at": current_timestamp(),
                    "type": "complex"
                }),
                message: format!("Cleanup round #{} completed", round),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"scheduling_type": "complex"})),
            }
        })
    });

    println!("✅ All handlers registered");
    println!();

    // =================================================================
    // Trigger Scheduled Executions
    // =================================================================
    println!("🚀 Triggering scheduled executions...");
    println!();

    // Trigger delayed task (will execute once after 2 seconds)
    println!("1️⃣ Triggering delayed task (2 second delay)...");
    let delayed_result = cyre.call("delayed-task", json!({
        "message": "This will execute after 2 seconds",
        "triggered_at": current_timestamp()
    })).await;
    
    if delayed_result.ok {
        println!("   ✅ Delayed task scheduled: {}", delayed_result.message);
        if let Some(formation_id) = delayed_result.payload.get("formation_id") {
            println!("   📅 Formation ID: {}", formation_id);
        }
    }

    sleep(Duration::from_millis(500)).await;

    // Trigger interval monitoring (will execute every 1.5 seconds)
    println!("2️⃣ Triggering interval monitoring (1.5 second intervals)...");
    let monitor_result = cyre.call("monitor", json!({
        "check_id": 1,
        "system": "database",
        "triggered_at": current_timestamp()
    })).await;
    
    if monitor_result.ok {
        println!("   ✅ Monitor scheduled: {}", monitor_result.message);
    }

    sleep(Duration::from_millis(500)).await;

    // Trigger backup repeat (will execute 3 times with 1 second intervals)
    println!("3️⃣ Triggering backup repeat (3 times, 1 second intervals)...");
    let backup_result = cyre.call("backup", json!({
        "iteration": 1,
        "target": "user_data",
        "triggered_at": current_timestamp()
    })).await;
    
    if backup_result.ok {
        println!("   ✅ Backup scheduled: {}", backup_result.message);
    }

    sleep(Duration::from_millis(500)).await;

    // Trigger complex cleanup (1 second delay, then 4 times with 0.8 second intervals)
    println!("4️⃣ Triggering complex cleanup (1s delay + 4 times @ 0.8s intervals)...");
    let cleanup_result = cyre.call("cleanup", json!({
        "round": 1,
        "area": "temp_files",
        "triggered_at": current_timestamp()
    })).await;
    
    if cleanup_result.ok {
        println!("   ✅ Cleanup scheduled: {}", cleanup_result.message);
    }

    // =================================================================
    // Wait and Observe Executions
    // =================================================================
    println!();
    println!("⏳ Watching scheduled executions for 10 seconds...");
    println!("   (You should see tasks executing based on their schedules)");
    println!();

    sleep(Duration::from_millis(10000)).await;

    // =================================================================
    // Show Performance Metrics
    // =================================================================
    println!("📊 PERFORMANCE SUMMARY");
    println!("======================");
    
    let metrics = cyre.get_performance_metrics();
    println!("🎯 Cyre Performance:");
    println!("   • Total executions: {}", metrics["total_executions"]);
    println!("   • TimeKeeper executions: {}", metrics.get("timekeeper_executions").unwrap_or(&json!(0)));
    println!("   • Fast path hits: {}", metrics["fast_path_hits"]);
    println!("   • Active channels: {}", metrics["active_channels"]);
    println!("   • TimeKeeper enabled: {}", metrics["timekeeper_enabled"]);

    println!();
    println!("🏆 Quick Scheduling Methods Demonstrated:");
    println!("   ✅ IO::delayed() - setTimeout equivalent");
    println!("   ✅ IO::interval() - setInterval equivalent"); 
    println!("   ✅ IO::repeat() - finite repetition");
    println!("   ✅ IO::complex() - delay + interval + repeat");
    
    println!();
    println!("💡 Usage Pattern:");
    println!("   cyre.action(IO::delayed(\"task\", 2000));");
    println!("   cyre.on(\"task\", |payload| {{ /* handler */ }});");
    println!("   cyre.call(\"task\", payload).await;");

    println!();
    println!("🎉 TimeKeeper Quick Scheduling Demo Complete!");

    Ok(())
}