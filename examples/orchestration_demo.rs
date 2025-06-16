// examples/orchestration_demo.rs
// Comprehensive demo of Rust's powerful orchestration system

use std::time::{ Duration, SystemTime };
use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;
use tokio::time::sleep;
use serde_json::json;

use cyre_rust::orchestration::task_store::{ self, TaskBuilder, TaskPriority, TaskRepeat, TaskType };
use cyre_rust::orchestration::orchestration::{
    self,
    OrchestrationBuilder,
    ParallelStrategy,
    LoopCount,
    ErrorStrategy,
    StepType,
};
use cyre_rust::types::{ ActionPayload, CyreResponse };

/*

      C.Y.R.E - R.U.S.T - O.R.C.H.E.S.T.R.A.T.I.O.N - D.E.M.O
      
      Showcasing Rust's power in orchestration:
      - Zero-cost abstractions
      - Compile-time safety
      - High-performance async execution
      - Rich type system
      - Memory safety without GC

*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 RUST CYRE ORCHESTRATION DEMO");
    println!("================================");
    println!("Leveraging Rust's power for high-performance task orchestration");
    println!();

    // =======================================================================
    // PART 1: BASIC TASK STORE DEMO
    // =======================================================================

    println!("📋 === TASK STORE DEMO (Rust Power) ===");

    // Create tasks using Rust's builder pattern with compile-time validation
    demo_task_builders().await?;

    // Show zero-cost filtering and statistics
    demo_task_filtering().await?;

    // =======================================================================
    // PART 2: ADVANCED ORCHESTRATION
    // =======================================================================

    println!("\n🎼 === ORCHESTRATION DEMO (Type Safety) ===");

    // Complex orchestration with type safety
    demo_complex_orchestration().await?;

    // Parallel execution with strategies
    demo_parallel_execution().await?;

    // Condition-based monitoring
    demo_condition_monitoring().await?;

    // Error handling and recovery
    demo_error_handling().await?;

    // Performance monitoring
    demo_performance_monitoring().await?;

    println!("\n✨ Rust Orchestration Demo Complete!");
    println!("🚀 Zero-cost abstractions with maximum performance!");

    Ok(())
}

//=============================================================================
// TASK STORE DEMONSTRATIONS
//=============================================================================

async fn demo_task_builders() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 Creating tasks with Rust's type-safe builders...");

    // Timeout task with compile-time validation
    let email_task = TaskBuilder::new()
        .id("welcome-email")
        .timeout_task(Duration::from_secs(2))
        .priority(TaskPriority::High)
        .metadata("type", "email")
        .metadata("recipient", "user@example.com")
        .execution_timeout(Duration::from_secs(30))
        .with_retry(3, crate::orchestration::task_store::BackoffStrategy::Exponential {
            base: Duration::from_millis(100),
            max: Duration::from_secs(5),
        })
        .build(|| async {
            println!("📧 Welcome email sent!");
            CyreResponse::ok(
                json!({
                "email_sent": true,
                "timestamp": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
            })
            )
        })?;

    let result = task_store::keep(email_task);
    println!("✅ Email task: {} (execution time: {:?})", result.message, result.execution_time);

    // Interval task with complex configuration
    let monitor_task = TaskBuilder::new()
        .id("system-monitor")
        .interval_task(Duration::from_secs(3), Some(TaskRepeat::Count(5)))
        .priority(TaskPriority::Critical)
        .metadata("category", "monitoring")
        .build(|| async {
            let cpu_usage = rand::random::<f64>() * 100.0;
            let memory_usage = rand::random::<f64>() * 100.0;

            println!("🔍 System Monitor - CPU: {:.1}%, Memory: {:.1}%", cpu_usage, memory_usage);

            CyreResponse::ok(
                json!({
                "cpu": cpu_usage,
                "memory": memory_usage,
                "healthy": cpu_usage < 80.0 && memory_usage < 85.0
            })
            )
        })?;

    let result = task_store::keep(monitor_task);
    println!("✅ Monitor task: {}", result.message);

    // Complex task showcasing all features
    let backup_task = TaskBuilder::new()
        .id("nightly-backup")
        .complex_task(
            Some(Duration::from_secs(1)), // Delay
            Duration::from_secs(4), // Interval
            Some(TaskRepeat::Forever) // Repeat
        )
        .priority(TaskPriority::Low)
        .metadata("schedule", "nightly")
        .metadata("retention", "30days")
        .build(|| async {
            println!("💾 Database backup started...");

            // Simulate backup process
            sleep(Duration::from_millis(100)).await;

            let backup_size = (rand::random::<u64>() % 1000) + 500; // MB

            CyreResponse::ok(
                json!({
                "backup_completed": true,
                "size_mb": backup_size,
                "timestamp": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
            })
            )
        })?;

    let result = task_store::keep(backup_task);
    println!("✅ Backup task: {}", result.message);

    // Activate all tasks
    println!("\n▶️ Activating tasks with zero-cost activation...");
    let _ = task_store::activate("welcome-email", true).await;
    let _ = task_store::activate("system-monitor", true).await;
    let _ = task_store::activate("nightly-backup", true).await;

    // Wait for some execution
    sleep(Duration::from_secs(2)).await;

    Ok(())
}

async fn demo_task_filtering() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Demonstrating zero-cost filtering...");

    // Zero-cost filtering by status
    let active_tasks = task_store::list(
        Some(crate::orchestration::task_store::TaskFilter {
            status: Some(crate::orchestration::task_store::TaskStatus::Active),
            ..Default::default()
        })
    );

    println!("🟢 Active tasks: {}", active_tasks.len());
    for task in &active_tasks {
        println!(
            "   • {} ({:?}) - {} executions",
            task.id,
            task.config.task_type,
            task.metrics.total_executions
        );
    }

    // High-priority tasks
    let high_priority_tasks = task_store::list(
        Some(crate::orchestration::task_store::TaskFilter {
            priority: Some(TaskPriority::High),
            ..Default::default()
        })
    );

    println!("\n🔴 High priority tasks: {}", high_priority_tasks.len());

    // System statistics with rich metrics
    let stats = task_store::stats();
    println!("\n📈 Task Store Statistics:");
    println!("   Total: {}", stats.total);
    println!("   Active: {}", stats.active_count);
    println!("   System Health: {:?}", stats.system_health);
    println!("   Average Active Time: {:?}", stats.average_active_time);
    println!("   By Type: {:?}", stats.by_type);
    println!(
        "   Performance: {:.2}% CPU, {:.2}MB memory",
        stats.performance_metrics.cpu_utilization,
        stats.performance_metrics.memory_efficiency
    );

    Ok(())
}

//=============================================================================
// ORCHESTRATION DEMONSTRATIONS
//=============================================================================

async fn demo_complex_orchestration() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 Creating complex type-safe orchestration...");

    // Build a sophisticated workflow using Rust's type system
    let config = OrchestrationBuilder::new()
        .id("system-health-workflow")
        .name("Comprehensive System Health Monitor")
        .description("Multi-step health monitoring with auto-scaling")
        .priority(TaskPriority::Critical)
        .timeout(Duration::from_secs(60))
        .max_concurrent_executions(3)
        .global_error_strategy(ErrorStrategy::Retry { max_attempts: 3 })

        // Time-based trigger
        .time_trigger("health-check", Duration::from_secs(5))

        // Health check action
        .action_step("check-system", vec!["health-monitor".to_string()], |_ctx| {
            json!({
                "check_type": "comprehensive",
                "timestamp": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
            }).into()
        })

        // Conditional scaling based on health
        .condition_step(
            "evaluate-health",
            |ctx|
                Box::pin(async move {
                    // Simulate health evaluation
                    let should_scale = rand::random::<bool>();
                    println!("🔍 Health evaluation: {}", if should_scale {
                        "Scale needed"
                    } else {
                        "System healthy"
                    });
                    should_scale
                }),
            Some(
                vec![
                    // Steps to execute if scaling is needed
                    create_scaling_step("scale-up"),
                    create_alert_step("scaling-alert")
                ]
            ),
            Some(
                vec![
                    // Steps if system is healthy
                    create_log_step("system-ok")
                ]
            )
        )

        // Final monitoring step
        .action_step("log-completion", vec!["logger".to_string()], |ctx| {
            json!({
                "workflow": "system-health",
                "steps_completed": ctx.step_history.len(),
                "execution_id": ctx.execution_id
            }).into()
        })

        .build()?;

    // Store and activate orchestration
    let result = orchestration::keep(config);
    println!("✅ Complex orchestration: {}", result.message);

    let activate_result = orchestration::activate("system-health-workflow").await;
    println!("▶️ Orchestration activated: {}", activate_result.message);

    Ok(())
}

async fn demo_parallel_execution() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚡ Demonstrating parallel execution strategies...");

    let config = OrchestrationBuilder::new()
        .id("parallel-reports")
        .name("Parallel Report Generation")
        .time_trigger("report-trigger", Duration::from_secs(8))

        // Parallel step with WaitAll strategy
        .parallel_step(
            "generate-reports",
            vec![
                create_report_step("sales-report"),
                create_report_step("performance-report"),
                create_report_step("security-report")
            ],
            ParallelStrategy::WaitAll
        )

        // Sequential cleanup after parallel execution
        .action_step("consolidate-reports", vec!["report-consolidator".to_string()], |ctx| {
            json!({
                "reports_generated": ctx.step_history.len(),
                "consolidation_time": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
            }).into()
        })

        .build()?;

    let result = orchestration::keep(config);
    println!("✅ Parallel orchestration: {}", result.message);

    let activate_result = orchestration::activate("parallel-reports").await;
    println!("▶️ Parallel execution activated: {}", activate_result.message);

    Ok(())
}

async fn demo_condition_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n👁️ Setting up condition-based monitoring...");

    // Static counter for demo purposes
    static mut ERROR_COUNT: u32 = 0;

    let config = OrchestrationBuilder::new()
        .id("error-threshold-monitor")
        .name("Error Threshold Monitor")
        .condition_trigger(
            "error-check",
            |_ctx|
                Box::pin(async move {
                    unsafe {
                        // Simulate random errors
                        if rand::random::<f64>() < 0.3 {
                            ERROR_COUNT += 1;
                            println!("❌ Error detected! Count: {}", ERROR_COUNT);
                        }

                        ERROR_COUNT > 2
                    }
                }),
            Duration::from_secs(2)
        )

        .action_step("critical-alert", vec!["alert-system".to_string()], |_ctx| {
            json!({
                "alert_type": "critical",
                "message": "Error threshold exceeded",
                "severity": "high"
            }).into()
        })

        .action_step("reset-counter", vec!["counter-reset".to_string()], |_ctx| {
            unsafe {
                ERROR_COUNT = 0;
            }
            json!({
                "action": "counter_reset",
                "timestamp": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
            }).into()
        })

        .build()?;

    let result = orchestration::keep(config);
    println!("✅ Condition monitor: {}", result.message);

    let activate_result = orchestration::activate("error-threshold-monitor").await;
    println!("👁️ Monitoring activated: {}", activate_result.message);

    Ok(())
}

async fn demo_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🛡️ Demonstrating advanced error handling...");

    let config = OrchestrationBuilder::new()
        .id("resilient-workflow")
        .name("Error-Resilient Workflow")
        .time_trigger("resilience-test", Duration::from_secs(6))

        // Step that might fail
        .action_step("risky-operation", vec!["risky-service".to_string()], |_ctx| {
            // 50% chance of failure for demo
            if rand::random::<bool>() {
                panic!("Simulated failure!");
            }
            json!({
                "operation": "risky",
                "success": true
            }).into()
        })

        // Recovery step
        .action_step("recovery-action", vec!["recovery-service".to_string()], |ctx| {
            json!({
                "recovery": "initiated",
                "previous_errors": ctx.step_history.iter()
                    .filter(|s| s.status == crate::orchestration::orchestration::StepStatus::Error)
                    .count()
            }).into()
        })

        .build()?;

    let result = orchestration::keep(config);
    println!("✅ Resilient workflow: {}", result.message);

    let activate_result = orchestration::activate("resilient-workflow").await;
    println!("🛡️ Error handling activated: {}", activate_result.message);

    Ok(())
}

async fn demo_performance_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Performance monitoring and statistics...");

    // Wait for some executions
    sleep(Duration::from_secs(10)).await;

    // Show orchestration statistics
    let orchestrations = orchestration::list(None);
    println!("\n🎼 Orchestration Status:");

    for orch in &orchestrations {
        println!(
            "   • {} ({:?})",
            orch.config.name.as_ref().unwrap_or(&orch.config.id),
            orch.status
        );
        println!("     Executions: {} ({}% success rate)", orch.metrics.total_executions, if
            orch.metrics.total_executions > 0
        {
            (orch.metrics.successful_executions * 100) / orch.metrics.total_executions
        } else {
            0
        });
        println!("     Avg time: {:?}", orch.metrics.average_execution_time);

        if !orch.metrics.step_metrics.is_empty() {
            println!("     Step performance:");
            for (step_id, metrics) in &orch.metrics.step_metrics {
                println!(
                    "       - {}: {:.1}% success, {:?} avg",
                    step_id,
                    metrics.error_rate * 100.0,
                    metrics.average_time
                );
            }
        }
    }

    // Show task store statistics
    let task_stats = task_store::stats();
    println!("\n📋 Task Store Performance:");
    println!("   Memory efficiency: {:.2}%", task_stats.performance_metrics.memory_efficiency);
    println!("   CPU utilization: {:.2}%", task_stats.performance_metrics.cpu_utilization);
    println!("   Peak concurrent: {}", task_stats.performance_metrics.peak_concurrent_tasks);

    // Manual trigger demonstration
    println!("\n🔥 Testing manual trigger...");
    let manual_result = orchestration::trigger("system-health-workflow", "manual-test", None).await;
    println!("Manual trigger result: {}", manual_result.message);

    Ok(())
}

//=============================================================================
// HELPER FUNCTIONS FOR STEP CREATION
//=============================================================================

fn create_scaling_step(step_id: &str) -> crate::orchestration::orchestration::OrchestrationStep {
    crate::orchestration::orchestration::OrchestrationStep {
        id: step_id.to_string(),
        step_type: StepType::Action {
            targets: vec!["auto-scaler".to_string()],
            payload: std::sync::Arc::new(|_ctx| {
                json!({
                    "action": "scale_up",
                    "resource": "cpu",
                    "target": "4_cores"
                }).into()
            }),
        },
        enabled: true,
        on_error: ErrorStrategy::Continue,
        retry_config: None,
        metadata: HashMap::new(),
    }
}

fn create_alert_step(step_id: &str) -> crate::orchestration::orchestration::OrchestrationStep {
    crate::orchestration::orchestration::OrchestrationStep {
        id: step_id.to_string(),
        step_type: StepType::Action {
            targets: vec!["alert-system".to_string()],
            payload: std::sync::Arc::new(|ctx| {
                json!({
                    "alert": "scaling_initiated",
                    "execution_id": ctx.execution_id,
                    "severity": "info"
                }).into()
            }),
        },
        enabled: true,
        on_error: ErrorStrategy::Continue,
        retry_config: None,
        metadata: HashMap::new(),
    }
}

fn create_log_step(step_id: &str) -> crate::orchestration::orchestration::OrchestrationStep {
    crate::orchestration::orchestration::OrchestrationStep {
        id: step_id.to_string(),
        step_type: StepType::Action {
            targets: vec!["logger".to_string()],
            payload: std::sync::Arc::new(|_ctx| {
                json!({
                    "log_level": "info",
                    "message": "System operating normally"
                }).into()
            }),
        },
        enabled: true,
        on_error: ErrorStrategy::Continue,
        retry_config: None,
        metadata: HashMap::new(),
    }
}

fn create_report_step(report_type: &str) -> crate::orchestration::orchestration::OrchestrationStep {
    let report_type = report_type.to_string();
    crate::orchestration::orchestration::OrchestrationStep {
        id: format!("generate-{}", report_type),
        step_type: StepType::Action {
            targets: vec![format!("{}-generator", report_type)],
            payload: std::sync::Arc::new(move |_ctx| {
                json!({
                    "report_type": report_type,
                    "records": rand::random::<u32>() % 1000 + 100,
                    "generated_at": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
                }).into()
            }),
        },
        enabled: true,
        on_error: ErrorStrategy::Retry { max_attempts: 2 },
        retry_config: None,
        metadata: {
            let mut map = HashMap::new();
            map.insert("category".to_string(), "reporting".to_string());
            map
        },
    }
}
