// src/main.rs
// Cyre Rust - Demo application

use cyre_rust::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("🚀 CYRE RUST - DEMO");
  println!("===================");

  // Create and initialize Cyre instance
  let mut cyre = Cyre::new();
  cyre.init().await?;

  // =================================================================
  // Demo 1: Basic Fast Path Action
  // =================================================================
  println!("⚡ Demo 1: Fast Path Action");
  println!("===========================");

  // Register a simple fast path action
  cyre.action(IO::new("greet"))?;

  // Register handler
  cyre.on("greet", |payload| {
    Box::pin(async move {
      let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("World");

      CyreResponse {
        ok: true,
        payload: json!({
                    "greeting": format!("Hello, {}!", name),
                    "timestamp": current_timestamp()
                }),
        message: "Greeting generated".to_string(),
        error: None,
        timestamp: current_timestamp(),
        metadata: None,
      }
    })
  })?;

  // Call the action
  let result = cyre.call("greet", json!({"name": "Rust"})).await;
  println!("✅ Call result: {}", result.message);
  println!("📝 Greeting: {}", result.payload.get("greeting").unwrap());

  // =================================================================
  // Demo 2: Protected Action with Throttling
  // =================================================================
  println!("\n🛡️ Demo 2: Protected Action");
  println!("============================");

  // Register action with throttle protection
  cyre.action(IO::new("api-call").with_throttle(1000))?; // 1 second throttle

  cyre.on("api-call", |payload| {
    Box::pin(async move {
      println!("🌐 API processing: {}", payload);

      CyreResponse {
        ok: true,
        payload: json!({
                    "api_response": "Data retrieved successfully",
                    "request_id": payload.get("id").unwrap_or(&json!("unknown"))
                }),
        message: "API call completed".to_string(),
        error: None,
        timestamp: current_timestamp(),
        metadata: None,
      }
    })
  })?;

  // Test throttling
  println!("🔄 Testing throttle protection...");
  let call1 = cyre.call("api-call", json!({"id": 1, "data": "test"})).await;
  let call2 = cyre.call("api-call", json!({"id": 2, "data": "test"})).await;

  println!("   First call: {} ({})", call1.ok, call1.message);
  println!("   Second call: {} ({})", call2.ok, call2.message);

  // =================================================================
  // Demo 3: Priority System
  // =================================================================
  println!("\n🎯 Demo 3: Priority System");
  println!("===========================");

  // High priority action
  cyre.action(IO::new("urgent-task").with_priority(Priority::High))?;

  cyre.on("urgent-task", |payload| {
    Box::pin(async move { CyreResponse {
        ok: true,
        payload: json!({
                    "task": "urgent processing",
                    "priority": "high",
                    "data": payload
                }),
        message: "Urgent task completed".to_string(),
        error: None,
        timestamp: current_timestamp(),
        metadata: None,
      } })
  })?;

  let urgent_result = cyre.call("urgent-task", json!({"task": "important work"})).await;
  println!("🚨 Urgent task: {}", urgent_result.message);

  // =================================================================
  // Demo 4: Change Detection
  // =================================================================
  println!("\n🔄 Demo 4: Change Detection");
  println!("============================");

  cyre.action(IO::new("state-update").with_transform("add-two".to_string()));

  cyre.on("state-update", |payload| {
    Box::pin(async move {
      println!("📝 State change detected: {}", payload);

      CyreResponse {
        ok: true,
        payload: json!({
                    "state_updated": true,
                    "new_state": payload,
                    "timestamp": current_timestamp()
                }),
        message: "State updated".to_string(),
        error: None,
        timestamp: current_timestamp(),
        metadata: None,
      }
    })
  })?;

  println!("🔍 Testing change detection...");
  let same_data = json!({"status": "active", "count": 5});

  let update1 = cyre.call("state-update", same_data.clone()).await;
  let update2 = cyre.call("state-update", same_data.clone()).await; // Should be skipped
  let update3 = cyre.call("state-update", json!({"status": "active", "count": 6})).await; // Different

  println!("   Update 1: {} ({})", update1.ok, update1.message);
  println!("   Update 2: {} ({})", update2.ok, update2.message);
  println!("   Update 3: {} ({})", update3.ok, update3.message);

  // =================================================================
  // Demo 5: Performance Metrics
  // =================================================================
  println!("\n📊 Demo 5: Performance Metrics");
  println!("===============================");

  // Run a bunch of fast operations to generate metrics
  cyre.action(IO::new("benchmark"))?;
  cyre.on("benchmark", |payload| {
    Box::pin(async move { CyreResponse {
        ok: true,
        payload: json!({"benchmarked": payload}),
        message: "Benchmark completed".to_string(),
        error: None,
        timestamp: current_timestamp(),
        metadata: None,
      } })
  })?;

  println!("🏃 Running benchmark operations...");
  let start_time = std::time::Instant::now();

  for i in 0..1000 {
    let _result = cyre.call("benchmark", json!({"iteration": i})).await;
  }

  let duration = start_time.elapsed();
  let ops_per_sec = (1000.0 / duration.as_secs_f64()) as u64;

  println!("⚡ Completed 1000 operations in {:.2}ms", duration.as_millis());
  println!("🚀 Performance: {} ops/sec", ops_per_sec);

  // Get system metrics
  let metrics = cyre.get_performance_metrics();
  println!("\n📈 System Metrics:");
  if let Some(executions) = metrics.get("executions") {
    println!("   Total executions: {}", executions.get("total_executions").unwrap_or(&json!(0)));
    println!("   Fast path hits: {}", executions.get("fast_path_hits").unwrap_or(&json!(0)));
    println!(
      "   Fast path ratio: {:.1}%",
      executions.get("fast_path_ratio").unwrap_or(&json!(0.0))
    );
  }
  println!("   Active channels: {}", metrics.get("active_channels").unwrap_or(&json!(0)));

  // =================================================================
  // Demo 6: Error Handling
  // =================================================================
  println!("\n💪 Demo 6: Error Handling");
  println!("==========================");

  cyre.action(IO::new("error-test"))?;
  cyre.on("error-test", |payload| {
    Box::pin(async move {
      if
        payload
          .get("should_fail")
          .and_then(|v| v.as_bool())
          .unwrap_or(false)
      {
        CyreResponse {
          ok: false,
          payload: json!(null),
          message: "Intentional error for testing".to_string(),
          error: Some("test_error".to_string()),
          timestamp: current_timestamp(),
          metadata: None,
        }
      } else {
        CyreResponse {
          ok: true,
          payload: json!({"test": "success"}),
          message: "Error test passed".to_string(),
          error: None,
          timestamp: current_timestamp(),
          metadata: None,
        }
      }
    })
  })?;

  // Test success case
  let success_result = cyre.call("error-test", json!({"should_fail": false})).await;
  println!("✅ Success test: {} - {}", success_result.ok, success_result.message);

  // Test error case
  let error_result = cyre.call("error-test", json!({"should_fail": true})).await;
  println!("❌ Error test: {} - {}", error_result.ok, error_result.message);

  // =================================================================
  // Final System Status
  // =================================================================
  println!("\n📊 Final System Status");
  println!("======================");

  let status = cyre.status();
  println!("🟢 System initialized: {}", status.payload["initialized"]);
  if let Some(breathing) = status.payload.get("breathing") {
    println!("💨 Breathing system: {}", breathing);
  }
  if let Some(stores) = status.payload.get("stores") {
    println!("📋 Total actions: {}", stores.get("actions").unwrap_or(&json!(0)));
    println!("🔧 Total handlers: {}", stores.get("handlers").unwrap_or(&json!(0)));
  }

  println!("\n🎉 CYRE RUST DEMO COMPLETED!");
  println!("============================");
  println!("✅ All systems operational");
  println!("🚀 Performance optimizations active");
  println!("🔒 Memory safety guaranteed");

  Ok(())
}
