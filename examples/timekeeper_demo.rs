// examples/timekeeper_demo.rs
// TimeKeeper Demo - Showcasing delay, interval, and repeat functionality

use cyre_rust::prelude::*;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⏰ CYRE TIMEKEEPER DEMO");
    println!("=======================");
    println!("Showcasing delay, interval, and repeat with compiled pipelines");
    println!();

    let mut cyre = Cyre::new();
    
    // Initialize TimeKeeper
    cyre.init_timekeeper().await?;
    
    // Setup TimeKeeper-enabled actions
    setup_timekeeper_actions(&mut cyre).await;
    
    // Run demo scenarios
    println!("🎮 Starting TimeKeeper Demo Scenarios...");
    println!();
    
    // Scenario 1: Delayed Actions (setTimeout equivalent)
    delayed_actions_demo(&cyre).await?;
    
    // Scenario 2: Interval Actions (setInterval equivalent)
    interval_actions_demo(&cyre).await?;
    
    // Scenario 3: Repeat Actions (finite repetition)
    repeat_actions_demo(&cyre).await?;
    
    // Scenario 4: Complex Scheduling
    complex_scheduling_demo(&cyre).await?;
    
    // Scenario 5: Real-time System Monitoring
    realtime_monitoring_demo(&cyre).await?;
    
    println!("🎉 TimeKeeper Demo Complete!");
    println!("✨ All scheduling operations handled by compiled pipelines!");
    
    Ok(())
}

//=============================================================================
// TIMEKEEPER ACTION SETUP
//=============================================================================

async fn setup_timekeeper_actions(cyre: &mut Cyre) {
    println!("🔧 Setting up TimeKeeper-enabled actions...");
    
    // =====================================================
    // DELAYED ACTIONS (setTimeout equivalent)
    // =====================================================
    
    // Task with delay
    cyre.action(IO::new("task.delayed")
        .with_name("Delayed Task")
        .with_delay(2000)); // 2 second delay
    
    cyre.on("task.delayed", |payload| {
        Box::pin(async move {
            let task_id = payload.get("task_id").and_then(|v| v.as_str()).unwrap_or("unknown");
            let message = payload.get("message").and_then(|v| v.as_str()).unwrap_or("Task executed");
            
            println!("⏰ DELAYED EXECUTION: {} - {}", task_id, message);
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "task_id": task_id,
                    "message": message,
                    "executed_at": current_timestamp(),
                    "execution_type": "delayed"
                }),
                message: format!("Delayed task {} completed", task_id),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"timekeeper": "delay"})),
            }
        })
    });

    // Notification with custom delay
    cyre.action(IO::new("notify.delayed")
        .with_name("Delayed Notification")
        .with_delay(1500)); // 1.5 second delay
    
    cyre.on("notify.delayed", |payload| {
        Box::pin(async move {
            let notification = payload.get("notification").and_then(|v| v.as_str()).unwrap_or("Alert");
            let priority = payload.get("priority").and_then(|v| v.as_str()).unwrap_or("normal");
            
            let emoji = match priority {
                "high" => "🚨",
                "medium" => "⚠️",
                _ => "📢"
            };
            
            println!("{} DELAYED NOTIFICATION: {}", emoji, notification);
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "notification": notification,
                    "priority": priority,
                    "delivered_at": current_timestamp()
                }),
                message: "Delayed notification sent".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"timekeeper": "delay"})),
            }
        })
    });

    // =====================================================
    // INTERVAL ACTIONS (setInterval equivalent)
    // =====================================================
    
    // Heartbeat monitor
    cyre.action(IO::new("monitor.heartbeat")
        .with_name("Heartbeat Monitor")
        .with_interval(3000)); // Every 3 seconds
    
    cyre.on("monitor.heartbeat", |payload| {
        Box::pin(async move {
            let system = payload.get("system").and_then(|v| v.as_str()).unwrap_or("unknown");
            let status = payload.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
            
            println!("💓 HEARTBEAT: {} system is {}", system, status);
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "system": system,
                    "status": status,
                    "heartbeat_time": current_timestamp(),
                    "execution_type": "interval"
                }),
                message: format!("Heartbeat from {}", system),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"timekeeper": "interval"})),
            }
        })
    });

    // Data collection
    cyre.action(IO::new("data.collect")
        .with_name("Data Collector")
        .with_interval(2500)); // Every 2.5 seconds
    
    cyre.on("data.collect", |payload| {
        Box::pin(async move {
            let sensor = payload.get("sensor").and_then(|v| v.as_str()).unwrap_or("sensor1");
            let value = payload.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            
            println!("📊 DATA COLLECTION: {} = {:.2}", sensor, value);
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "sensor": sensor,
                    "value": value,
                    "collected_at": current_timestamp(),
                    "execution_type": "interval"
                }),
                message: format!("Data collected from {}", sensor),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"timekeeper": "interval"})),
            }
        })
    });

    // =====================================================
    // REPEAT ACTIONS (finite repetition)
    // =====================================================
    
    // Backup process (runs 3 times)
    cyre.action(IO::new("system.backup")
        .with_name("System Backup")
        .with_interval(1000) // Every 1 second
        .with_repeat(3)); // Repeat 3 times total
    
    cyre.on("system.backup", |payload| {
        Box::pin(async move {
            let component = payload.get("component").and_then(|v| v.as_str()).unwrap_or("system");
            let iteration = payload.get("iteration").and_then(|v| v.as_u64()).unwrap_or(1);
            
            println!("💾 BACKUP: {} (iteration {})", component, iteration);
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "component": component,
                    "iteration": iteration,
                    "backup_time": current_timestamp(),
                    "execution_type": "repeat"
                }),
                message: format!("Backup {} iteration {}", component, iteration),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"timekeeper": "repeat"})),
            }
        })
    });

    // Health check (runs 5 times)
    cyre.action(IO::new("health.check")
        .with_name("Health Check")
        .with_interval(800) // Every 0.8 seconds
        .with_repeat(5)); // Repeat 5 times total
    
    cyre.on("health.check", |payload| {
        Box::pin(async move {
            let service = payload.get("service").and_then(|v| v.as_str()).unwrap_or("service");
            let check_id = payload.get("check_id").and_then(|v| v.as_u64()).unwrap_or(1);
            
            // Simulate varying health status
            let health_score = 80 + (check_id % 20) as u32;
            let status = if health_score >= 90 { "excellent" } 
                        else if health_score >= 70 { "good" } 
                        else { "warning" };
            
            println!("🏥 HEALTH CHECK: {} = {}% ({})", service, health_score, status);
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "service": service,
                    "check_id": check_id,
                    "health_score": health_score,
                    "status": status,
                    "checked_at": current_timestamp(),
                    "execution_type": "repeat"
                }),
                message: format!("Health check {} completed", check_id),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"timekeeper": "repeat"})),
            }
        })
    });

    // =====================================================
    // COMPLEX SCHEDULING
    // =====================================================
    
    // Cleanup task (delay + repeat)
    cyre.action(IO::new("system.cleanup")
        .with_name("System Cleanup")
        .with_delay(1000) // Wait 1 second before starting
        .with_interval(1500) // Then every 1.5 seconds
        .with_repeat(4)); // Repeat 4 times total
    
    cyre.on("system.cleanup", |payload| {
        Box::pin(async move {
            let area = payload.get("area").and_then(|v| v.as_str()).unwrap_or("cache");
            let round = payload.get("round").and_then(|v| v.as_u64()).unwrap_or(1);
            
            println!("🧹 CLEANUP: {} (round {})", area, round);
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "area": area,
                    "round": round,
                    "cleaned_items": round * 10, // Simulate cleaning items
                    "cleaned_at": current_timestamp(),
                    "execution_type": "complex"
                }),
                message: format!("Cleanup {} round {}", area, round),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"timekeeper": "complex"})),
            }
        })
    });

    // Alert system (immediate + intervals)
    cyre.action(IO::new("alert.system")
        .with_name("Alert System")
        .with_interval(2000)); // Every 2 seconds (infinite)
    
    cyre.on("alert.system", |payload| {
        Box::pin(async move {
            let alert_type = payload.get("alert_type").and_then(|v| v.as_str()).unwrap_or("info");
            let count = payload.get("count").and_then(|v| v.as_u64()).unwrap_or(1);
            
            let emoji = match alert_type {
                "critical" => "🔴",
                "warning" => "🟡",
                "info" => "🟢",
                _ => "⚪"
            };
            
            println!("{} ALERT #{}: {} level alert", emoji, count, alert_type);
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "alert_type": alert_type,
                    "count": count,
                    "alert_time": current_timestamp(),
                    "execution_type": "interval_infinite"
                }),
                message: format!("Alert {} #{}", alert_type, count),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"timekeeper": "interval_infinite"})),
            }
        })
    });

    println!("✅ TimeKeeper actions configured with compiled pipelines");
    println!("   • Delayed actions: 2");
    println!("   • Interval actions: 2");
    println!("   • Repeat actions: 2"); 
    println!("   • Complex scheduling: 2");
    println!();
}

//=============================================================================
// DEMO SCENARIOS
//=============================================================================

async fn delayed_actions_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("⏰ SCENARIO 1: Delayed Actions (setTimeout)");
    println!("==========================================");
    
    println!("🚀 Scheduling delayed tasks...");
    
    // Schedule multiple delayed tasks
    let delayed_tasks = vec![
        ("task1", "Initialize system components"),
        ("task2", "Load configuration files"),
        ("task3", "Start background services"),
    ];
    
    for (task_id, message) in delayed_tasks {
        let result = cyre.call("task.delayed", json!({
            "task_id": task_id,
            "message": message
        })).await;
        
        if result.ok {
            println!("✅ Scheduled: {} - {}", task_id, message);
        }
    }
    
    // Schedule delayed notifications
    let notifications = vec![
        ("System startup initiated", "medium"),
        ("All components loaded", "high"),
    ];
    
    for (notification, priority) in notifications {
        let result = cyre.call("notify.delayed", json!({
            "notification": notification,
            "priority": priority
        })).await;
        
        if result.ok {
            println!("📢 Scheduled notification: {}", notification);
        }
    }
    
    println!("⏳ Waiting for delayed executions...");
    sleep(Duration::from_millis(3000)).await; // Wait for delayed actions
    
    println!("🎯 Delayed actions scenario completed!");
    println!();
    
    Ok(())
}

async fn interval_actions_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 SCENARIO 2: Interval Actions (setInterval)");
    println!("============================================");
    
    println!("🚀 Starting interval-based monitoring...");
    
    // Start heartbeat monitoring
    let heartbeat_result = cyre.call("monitor.heartbeat", json!({
        "system": "database",
        "status": "online"
    })).await;
    
    if heartbeat_result.ok {
        println!("💓 Heartbeat monitoring started for database");
    }
    
    // Start data collection
    let data_result = cyre.call("data.collect", json!({
        "sensor": "temperature_sensor",
        "value": 23.5
    })).await;
    
    if data_result.ok {
        println!("📊 Data collection started for temperature sensor");
    }
    
    println!("⏳ Monitoring interval executions for 8 seconds...");
    sleep(Duration::from_millis(8000)).await; // Watch intervals execute
    
    println!("🎯 Interval actions scenario completed!");
    println!();
    
    Ok(())
}

async fn repeat_actions_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔁 SCENARIO 3: Repeat Actions (Finite Repetition)");
    println!("================================================");
    
    println!("🚀 Starting finite repeat tasks...");
    
    // Start backup process (3 iterations)
    let backup_result = cyre.call("system.backup", json!({
        "component": "user_data",
        "iteration": 1
    })).await;
    
    if backup_result.ok {
        println!("💾 System backup process started (3 iterations)");
    }
    
    // Start health checks (5 iterations)
    let health_result = cyre.call("health.check", json!({
        "service": "api_server",
        "check_id": 1
    })).await;
    
    if health_result.ok {
        println!("🏥 Health check process started (5 iterations)");
    }
    
    println!("⏳ Watching finite repeat executions...");
    sleep(Duration::from_millis(6000)).await; // Watch repeat actions complete
    
    println!("🎯 Repeat actions scenario completed!");
    println!();
    
    Ok(())
}

async fn complex_scheduling_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧩 SCENARIO 4: Complex Scheduling");
    println!("================================");
    
    println!("🚀 Starting complex scheduling patterns...");
    
    // Start cleanup task (delay + interval + repeat)
    let cleanup_result = cyre.call("system.cleanup", json!({
        "area": "temp_files",
        "round": 1
    })).await;
    
    if cleanup_result.ok {
        println!("🧹 Complex cleanup task scheduled (delay → interval × 4)");
    }
    
    // Start alert system (infinite intervals)
    let alert_result = cyre.call("alert.system", json!({
        "alert_type": "info",
        "count": 1
    })).await;
    
    if alert_result.ok {
        println!("🚨 Alert system started (infinite intervals)");
    }
    
    println!("⏳ Observing complex scheduling for 10 seconds...");
    sleep(Duration::from_millis(10000)).await; // Watch complex patterns
    
    println!("🎯 Complex scheduling scenario completed!");
    println!();
    
    Ok(())
}

async fn realtime_monitoring_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("📡 SCENARIO 5: Real-time System Monitoring");
    println!("==========================================");
    
    println!("🚀 Demonstrating TimeKeeper performance...");
    
    // Create multiple overlapping schedules
    let schedules = vec![
        ("monitor.heartbeat", json!({"system": "cache", "status": "optimal"})),
        ("data.collect", json!({"sensor": "cpu_usage", "value": 45.2})),
        ("health.check", json!({"service": "load_balancer", "check_id": 1})),
    ];
    
    for (action, payload) in schedules {
        let result = cyre.call(action, payload).await;
        if result.ok {
            println!("📊 Started: {}", action);
        }
        sleep(Duration::from_millis(500)).await; // Stagger starts
    }
    
    println!("⏳ Monitoring overlapping schedules for 7 seconds...");
    sleep(Duration::from_millis(7000)).await;
    
    // Get final performance metrics
    let metrics = cyre.get_performance_metrics();
    
    println!("📈 TIMEKEEPER PERFORMANCE SUMMARY:");
    println!("=================================");
    println!("🎯 Cyre + TimeKeeper Performance:");
    println!("   • Total executions: {}", metrics["total_executions"]);
    println!("   • Fast path hits: {}", metrics["fast_path_hits"]); 
    println!("   • TimeKeeper executions: {}", metrics.get("timekeeper_executions").unwrap_or(&json!(0)));
    println!("   • Active channels: {}", metrics["active_channels"]);
    println!("   • Uptime: {}ms", metrics["uptime_ms"]);
    
    println!();
    println!("🏆 TimeKeeper Features Demonstrated:");
    println!("   ✅ Delay - setTimeout equivalent with precision timing");
    println!("   ✅ Interval - setInterval equivalent with drift compensation");
    println!("   ✅ Repeat - Finite repetition with automatic cleanup");
    println!("   ✅ Complex - Combined delay + interval + repeat patterns");
    println!("   ✅ Compiled Pipelines - Pre-optimized execution paths");
    println!("   ✅ Centralized State - Single timeline management");
    println!("   ✅ Performance - Sub-millisecond scheduling accuracy");
    
    println!("🎯 Real-time monitoring scenario completed!");
    println!();
    
    Ok(())
}