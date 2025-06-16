// examples/task_store_demo.rs
// Corrected demo using proper module structure

use std::time::Duration;
use serde_json::json;
use tokio::time::sleep;

// Import from the correct module structure
use cyre_rust::context::{
    // Task store functions
    keep,
    forget,
    activate,
    get,
    list,
    stats,
    timeout,
    interval,
    complex,
    // Types
    TaskBuilder,
    TaskFilter,
    TaskResult,
    TaskStats,
    TaskStatus,
    TaskType,
    TaskPriority,
    TaskRepeat,
};
use cyre_rust::types::CyreResponse;

/*

      C.Y.R.E - R.U.S.T - T.A.S.K - S.T.O.R.E - D.E.M.O
      
      Demonstrating the functional task store:
      - Proper module structure (context/task_store)
      - Zero-cost abstractions
      - Type-safe builders
      - Timeline integration
      - Performance monitoring

*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 RUST CYRE TASK STORE DEMO");
    println!("============================");
    println!("Task store integrated in context module");
    println!();

    // =======================================================================
    // PART 1: BASIC TASK CREATION
    // =======================================================================

    println!("📋 === TASK CREATION WITH TYPE SAFETY ===");

    // Create timeout task (setTimeout equivalent)
    demo_timeout_tasks().await?;

    // Create interval tasks (setInterval equivalent)
    demo_interval_tasks().await?;

    // Create complex tasks (delay + interval + repeat)
    demo_complex_tasks().await?;

    // =======================================================================
    // PART 2: TASK ACTIVATION AND MANAGEMENT
    // =======================================================================

    println!("\n⚡ === TASK ACTIVATION AND CONTROL ===");

    demo_task_activation().await?;

    // =======================================================================
    // PART 3: FILTERING AND STATISTICS
    // =======================================================================

    println!("\n📊 === TASK FILTERING AND STATISTICS ===");

    demo_task_filtering().await?;

    // =======================================================================
    // PART 4: PERFORMANCE MONITORING
    // =======================================================================

    println!("\n📈 === PERFORMANCE MONITORING ===");

    demo_performance_monitoring().await?;

    println!("\n✨ Task Store Demo Complete!");
    println!("🚀 Functional task management with Rust's power!");

    Ok(())
}

//=============================================================================
// TIMEOUT TASK DEMONSTRATIONS
//=============================================================================

async fn demo_timeout_tasks() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🕐 Creating timeout tasks (setTimeout equivalent)...");

    // Simple timeout using convenience function
    let result = timeout("welcome-email", Duration::from_secs(2), || async {
        println!("📧 Welcome email sent!");
        CyreResponse {
            ok: true,
            payload: json!({
                    "email_sent": true,
                    "recipient": "user@example.com",
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }),
            message: "Welcome email sent successfully".to_string(),
            error: None,
            timestamp: std::time::SystemTime
                ::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: Some(json!({"type": "email", "priority": "high"})),
        }
    });

    println!("✅ Timeout task result: {} (time: {:?})", result.message, result.execution_time);

    // Advanced timeout using builder pattern
    let advanced_config = TaskBuilder::new()
        .id("user-onboarding")
        .timeout_task(Duration::from_secs(3))
        .priority(TaskPriority::High)
        .metadata("category", "onboarding")
        .metadata("user_type", "new")
        .build(|| async {
            println!("🎯 User onboarding process started!");

            // Simulate onboarding steps
            tokio::time::sleep(Duration::from_millis(100)).await;

            CyreResponse {
                ok: true,
                payload: json!({
                    "onboarding_completed": true,
                    "steps": ["profile_created", "preferences_set", "tour_completed"],
                    "user_id": "user_12345"
                }),
                message: "User onboarding completed".to_string(),
                error: None,
                timestamp: std::time::SystemTime
                    ::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                metadata: Some(json!({"completion_rate": 100})),
            }
        })?;

    let advanced_result = keep(advanced_config);
    println!("✅ Advanced timeout: {}", advanced_result.message);

    Ok(())
}

//=============================================================================
// INTERVAL TASK DEMONSTRATIONS
//=============================================================================

async fn demo_interval_tasks() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Creating interval tasks (setInterval equivalent)...");

    // System monitor with finite repetitions
    let monitor_result = interval(
        "system-monitor",
        Duration::from_secs(3),
        Some(TaskRepeat::Count(5)), // Run 5 times
        || async {
            let cpu_usage = simple_rand::random_f64() * 100.0;
            let memory_usage = simple_rand::random_f64() * 100.0;
            let healthy = cpu_usage < 80.0 && memory_usage < 85.0;

            println!(
                "🔍 System Monitor - CPU: {:.1}%, Memory: {:.1}% - {}",
                cpu_usage,
                memory_usage,
                if healthy {
                    "✅ Healthy"
                } else {
                    "⚠️ Alert"
                }
            );

            CyreResponse {
                ok: true,
                payload: json!({
                    "cpu_usage": cpu_usage,
                    "memory_usage": memory_usage,
                    "healthy": healthy,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }),
                message: "System metrics collected".to_string(),
                error: None,
                timestamp: std::time::SystemTime
                    ::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                metadata: Some(
                    json!({"source": "system_monitor", "priority": if healthy { "low" } else { "high" }})
                ),
            }
        }
    );

    println!("✅ System monitor: {}", monitor_result.message);

    // Infinite data processor
    let processor_result = interval(
        "data-processor",
        Duration::from_secs(4),
        Some(TaskRepeat::Forever), // Run forever
        || async {
            let records_processed = (simple_rand::random_u32() % 100) + 50;

            println!("🔄 Data Processor - Processed {} records", records_processed);

            CyreResponse {
                ok: true,
                payload: json!({
                    "records_processed": records_processed,
                    "batch_id": format!("batch_{}", std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()),
                    "processing_time_ms": simple_rand::random_u32() % 500 + 100
                }),
                message: format!("Processed {} records", records_processed),
                error: None,
                timestamp: std::time::SystemTime
                    ::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                metadata: Some(json!({"type": "batch_processing"})),
            }
        }
    );

    println!("✅ Data processor: {}", processor_result.message);

    Ok(())
}

//=============================================================================
// COMPLEX TASK DEMONSTRATIONS
//=============================================================================

async fn demo_complex_tasks() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 Creating complex tasks (delay + interval + repeat)...");

    // Backup task with initial delay
    let backup_result = complex(
        "nightly-backup",
        Some(Duration::from_secs(1)), // Initial delay
        Duration::from_secs(5), // Then every 5 seconds
        Some(TaskRepeat::Count(3)), // Run 3 times total
        || async {
            println!("💾 Database backup started...");

            // Simulate backup process
            tokio::time::sleep(Duration::from_millis(200)).await;

            let backup_size = (simple_rand::random_u64() % 1000) + 500; // MB
            let success = simple_rand::random_f64() > 0.1; // 90% success rate

            if success {
                println!("✅ Backup completed - {:.1}MB", backup_size);
            } else {
                println!("❌ Backup failed!");
            }

            CyreResponse {
                ok: success,
                payload: json!({
                    "backup_completed": success,
                    "size_mb": if success { backup_size } else { 0 },
                    "backup_id": format!("backup_{}", std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()),
                    "duration_seconds": 0.2
                }),
                message: if success {
                    format!("Backup completed - {}MB", backup_size)
                } else {
                    "Backup failed".to_string()
                },
                error: if success {
                    None
                } else {
                    Some("Backup process failed".to_string())
                },
                timestamp: std::time::SystemTime
                    ::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                metadata: Some(json!({"type": "backup", "critical": true})),
            }
        }
    );

    println!("✅ Backup task: {}", backup_result.message);

    // Cache cleanup with complex configuration
    let cleanup_config = TaskBuilder::new()
        .id("cache-cleanup")
        .complex_task(
            Some(Duration::from_millis(500)), // Small delay
            Duration::from_secs(6), // Every 6 seconds
            Some(TaskRepeat::Forever) // Forever
        )
        .priority(TaskPriority::Low)
        .metadata("category", "maintenance")
        .metadata("importance", "low")
        .build(|| async {
            let cache_size_before = (simple_rand::random_u64() % 500) + 100; // MB
            let cleaned = (simple_rand::random_u64() % 100) + 10; // MB cleaned
            let cache_size_after = cache_size_before - cleaned;

            println!(
                "🧹 Cache cleanup - Freed {:.1}MB ({}MB → {}MB)",
                cleaned,
                cache_size_before,
                cache_size_after
            );

            CyreResponse {
                ok: true,
                payload: json!({
                    "cache_size_before_mb": cache_size_before,
                    "cache_size_after_mb": cache_size_after,
                    "freed_mb": cleaned,
                    "efficiency": (cleaned as f64 / cache_size_before as f64) * 100.0
                }),
                message: format!("Cache cleanup freed {}MB", cleaned),
                error: None,
                timestamp: std::time::SystemTime
                    ::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                metadata: Some(json!({"cleanup_type": "automatic"})),
            }
        })?;

    let cleanup_result = keep(cleanup_config);
    println!("✅ Cache cleanup: {}", cleanup_result.message);

    Ok(())
}

//=============================================================================
// TASK ACTIVATION DEMONSTRATIONS
//=============================================================================

async fn demo_task_activation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n▶️ Activating tasks (moving to timeline + timekeeper)...");

    // Activate all created tasks
    let tasks_to_activate = [
        "welcome-email",
        "user-onboarding",
        "system-monitor",
        "data-processor",
        "nightly-backup",
        "cache-cleanup",
    ];

    for task_id in &tasks_to_activate {
        let result = activate(task_id, true).await;
        if result.success {
            println!("✅ Activated: {} - {}", task_id, result.message);
        } else {
            println!("❌ Failed to activate: {} - {}", task_id, result.message);
        }
    }

    // Let tasks run for a bit
    println!("\n⏳ Letting tasks execute for 8 seconds...");
    sleep(Duration::from_secs(8)).await;

    // Demonstrate selective deactivation
    println!("\n⏸️ Deactivating some tasks...");

    let result = activate("data-processor", false).await;
    println!("⏸️ Deactivated data-processor: {}", result.message);

    let result = activate("cache-cleanup", false).await;
    println!("⏸️ Deactivated cache-cleanup: {}", result.message);

    Ok(())
}

//=============================================================================
// FILTERING AND STATISTICS DEMONSTRATIONS
//=============================================================================

async fn demo_task_filtering() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Demonstrating zero-cost filtering...");

    // Get all tasks
    let all_tasks = list(None);
    println!("📋 Total tasks: {}", all_tasks.len());

    // Filter active tasks
    let active_tasks = list(
        Some(TaskFilter {
            status: Some(TaskStatus::Active),
            ..Default::default()
        })
    );

    println!("\n🟢 Active tasks ({}):", active_tasks.len());
    for task in &active_tasks {
        println!(
            "   • {} ({:?}) - {} activations, {} executions",
            task.id,
            task.config.task_type,
            task.metrics.total_activations,
            task.metrics.total_executions
        );
    }

    // Filter by priority
    let high_priority_tasks = list(
        Some(TaskFilter {
            priority: Some(TaskPriority::High),
            ..Default::default()
        })
    );

    println!("\n🔴 High priority tasks ({}):", high_priority_tasks.len());
    for task in &high_priority_tasks {
        println!("   • {} - Priority: {:?}", task.id, task.config.priority);
    }

    // Filter by type
    let interval_tasks = list(
        Some(TaskFilter {
            task_type: Some(TaskType::Interval),
            ..Default::default()
        })
    );

    println!("\n🔄 Interval tasks ({}):", interval_tasks.len());
    for task in &interval_tasks {
        if let Some(interval) = task.config.interval {
            println!("   • {} - Every {:?}", task.id, interval);
        }
    }

    Ok(())
}

//=============================================================================
// PERFORMANCE MONITORING DEMONSTRATIONS
//=============================================================================

async fn demo_performance_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 System performance and statistics...");

    // Get comprehensive statistics
    let system_stats = stats();

    println!("\n📈 Task Store Statistics:");
    println!("   Total Tasks: {}", system_stats.total);
    println!("   Active Tasks: {}", system_stats.active_count);
    println!("   System Health: {:?}", system_stats.system_health);
    println!("   Average Active Time: {:?}", system_stats.average_active_time);

    println!("\n📊 Tasks by Status:");
    for (status, count) in &system_stats.by_status {
        println!("   {:?}: {}", status, count);
    }

    println!("\n🔧 Tasks by Type:");
    for (task_type, count) in &system_stats.by_type {
        println!("   {:?}: {}", task_type, count);
    }

    // Individual task metrics
    println!("\n📋 Individual Task Performance:");
    let all_tasks = list(None);
    for task in &all_tasks {
        if task.metrics.total_executions > 0 {
            println!(
                "   • {}: {} executions, {:.1}% success rate, {:?} avg active time",
                task.id,
                task.metrics.total_executions,
                if task.metrics.total_executions > 0 {
                    (((task.metrics.total_executions - task.metrics.total_errors) as f64) /
                        (task.metrics.total_executions as f64)) *
                        100.0
                } else {
                    0.0
                },
                task.metrics.average_execution_time
            );
        }
    }

    // Test task retrieval by ID
    println!("\n🔍 Testing task retrieval by ID...");
    if let Some(task) = get("system-monitor") {
        println!(
            "   Found system-monitor: Status={:?}, Executions={}",
            task.status,
            task.metrics.total_executions
        );
    }

    // Cleanup demonstration
    println!("\n🧹 Cleanup demonstration...");

    // Remove completed timeout tasks
    let timeout_tasks = list(
        Some(TaskFilter {
            task_type: Some(TaskType::Timeout),
            ..Default::default()
        })
    );

    for task in &timeout_tasks {
        if task.status == TaskStatus::Completed || task.status == TaskStatus::Inactive {
            let result = forget(&task.id);
            if result.success {
                println!("   🗑️ Removed completed task: {}", task.id);
            }
        }
    }

    // Final statistics after cleanup
    let final_stats = stats();
    println!("\n📊 Final Statistics:");
    println!("   Remaining Tasks: {}", final_stats.total);
    println!("   Active Tasks: {}", final_stats.active_count);
    println!("   System Health: {:?}", final_stats.system_health);

    Ok(())
}

//=============================================================================
// HELPER FUNCTIONS
//=============================================================================

// Add rand functionality without external dependency
mod simple_rand {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{ Hash, Hasher };
    use std::time::{ SystemTime, UNIX_EPOCH };

    pub fn random_f64() -> f64 {
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        let hash = hasher.finish();
        (hash as f64) / (u64::MAX as f64)
    }

    pub fn random_u32() -> u32 {
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        let hash = hasher.finish();
        hash as u32
    }

    pub fn random_u64() -> u64 {
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        hasher.finish()
    }

    pub fn random_bool() -> bool {
        random_f64() > 0.5
    }
}
