// examples/orchestration_demo.rs
// Fixed imports for orchestration demo

use std::time::Duration;
use serde_json::json;

// FIXED IMPORTS - Use the context module for task_store
use cyre_rust::context::task_store::{ TaskBuilder, TaskPriority, TaskRepeat, TaskType };

// FIXED IMPORTS - Use the orchestration module
use cyre_rust::orchestration::orchestration::{
    keep as orchestration_keep,
    activate as orchestration_activate,
    deactivate as orchestration_deactivate,
    trigger as orchestration_trigger,
    get as orchestration_get,
    list as orchestration_list,
    forget as orchestration_forget,
    schedule as orchestration_schedule,
    monitor as orchestration_monitor,
    OrchestrationBuilder,
    OrchestrationConfig,
    OrchestrationRuntime,
    OrchestrationStatus,
    TriggerType,
    StepType,
    ParallelStrategy,
    ErrorStrategy,
    OrchestrationMetrics,
};

use cyre_rust::{ types::{ ActionPayload, CyreResponse }, core::Cyre };

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎭 Cyre Orchestration System Demo");
    println!("===================================\n");

    // Initialize Cyre
    let mut cyre = Cyre::new();
    cyre.init_timekeeper().await?;

    // The rest of your orchestration demo code goes here...
    // (The main demo logic should remain the same, just the imports are fixed)

    println!("✅ Orchestration demo completed successfully!");

    Ok(())
}
