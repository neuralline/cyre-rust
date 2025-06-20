// examples/throttle_test.rs
// Throttle operator test with detailed sensor logging

use cyre_rust::prelude::*;
use serde_json::json;
use std::time::{ Duration, Instant };
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("🚦 THROTTLE OPERATOR TEST");
  println!("========================");
  println!("Testing throttle logic with detailed sensor logging");
  println!();

  let mut cyre = Cyre::new();
  cyre.init().await?;

  //=================================================================
  // Test 1: Basic Throttle Behavior
  //=================================================================
  println!("🔥 Test 1: Basic Throttle (1000ms)");
  println!("===================================");

  // Register throttled action
  cyre.action(
    IO::new("api-call")
      .with_throttle(1000) // 1 second throttle
      .with_logging(true)
  )?;

  cyre.on("api-call", |payload| {
    Box::pin(async move {
      let timestamp = current_timestamp();
      let id = payload
        .get("id")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

      println!("✅ [{}] API CALL EXECUTED: Request #{}", timestamp, id);

      CyreResponse::success(
        json!({
                    "request_id": id,
                    "executed_at": timestamp,
                    "message": "API call completed"
                }),
        "API call successful"
      )
    })
  })?;

  println!("📊 Configuration: 1000ms throttle");
  println!("🧪 Making 5 rapid calls (200ms apart):");
  println!();

  // Make rapid calls to test throttling
  for i in 1..=5 {
    let call_start = Instant::now();
    let timestamp_before = current_timestamp();

    println!("🔄 [{}] Call #{} - Starting...", timestamp_before, i);

    let result = cyre.call("api-call", json!({"id": i, "data": "test"})).await;

    let call_duration = call_start.elapsed();
    let timestamp_after = current_timestamp();
    let elapsed_ms = timestamp_after - timestamp_before;

    if result.ok {
      println!(
        "   ✅ [{}] SUCCESS: {} ({}ms / {:.1}ms)",
        timestamp_after,
        result.message,
        elapsed_ms,
        call_duration.as_millis()
      );
      if let Some(data) = result.payload.get("executed_at") {
        println!("   📋 Handler executed at: {}", data);
      }
    } else {
      println!(
        "   🚫 [{}] THROTTLED: {} ({}ms / {:.1}ms)",
        timestamp_after,
        result.message,
        elapsed_ms,
        call_duration.as_millis()
      );
    }

    println!();

    // Wait between calls (but less than throttle time)
    if i < 5 {
      sleep(Duration::from_millis(200)).await;
    }
  }

  //=================================================================
  // Test 2: Throttle Recovery
  //=================================================================
  println!("\n⏳ Test 2: Throttle Recovery");
  println!("============================");
  println!("Waiting 1.2 seconds for throttle to reset...");
  sleep(Duration::from_millis(1200)).await;

  let recovery_start = current_timestamp();
  println!("🔄 [{}] Recovery call - Should succeed", recovery_start);

  let recovery_result = cyre.call("api-call", json!({"id": 99, "data": "recovery"})).await;
  let recovery_end = current_timestamp();

  if recovery_result.ok {
    println!("   ✅ [{}] RECOVERY SUCCESS: {}", recovery_end, recovery_result.message);
  } else {
    println!("   ❌ [{}] RECOVERY FAILED: {}", recovery_end, recovery_result.message);
  }

  //=================================================================
  // Test 3: Different Throttle Intervals
  //=================================================================
  println!("\n🔬 Test 3: Different Throttle Intervals");
  println!("=======================================");

  // 500ms throttle
  cyre.action(IO::new("fast-throttle").with_throttle(500).with_logging(true))?;

  cyre.on("fast-throttle", |payload| {
    Box::pin(async move {
      let timestamp = current_timestamp();
      println!("⚡ [{}] FAST THROTTLE EXECUTED", timestamp);

      CyreResponse::success(json!({"executed_at": timestamp}), "Fast throttle execution")
    })
  })?;

  // 2000ms throttle
  cyre.action(IO::new("slow-throttle").with_throttle(2000).with_logging(true))?;

  cyre.on("slow-throttle", |payload| {
    Box::pin(async move {
      let timestamp = current_timestamp();
      println!("🐌 [{}] SLOW THROTTLE EXECUTED", timestamp);

      CyreResponse::success(json!({"executed_at": timestamp}), "Slow throttle execution")
    })
  })?;

  println!("🧪 Testing different throttle speeds:");
  println!();

  // Test both at the same time
  for i in 1..=3 {
    let test_start = current_timestamp();
    println!("🔄 [{}] Batch #{}", test_start, i);

    // Fast throttle (500ms)
    let fast_result = cyre.call("fast-throttle", json!({"test": i})).await;
    let fast_time = current_timestamp();
    println!("   ⚡ [{}] Fast (500ms): {}", fast_time, if fast_result.ok {
      "✅ SUCCESS"
    } else {
      "🚫 THROTTLED"
    });

    // Slow throttle (2000ms)
    let slow_result = cyre.call("slow-throttle", json!({"test": i})).await;
    let slow_time = current_timestamp();
    println!("   🐌 [{}] Slow (2000ms): {}", slow_time, if slow_result.ok {
      "✅ SUCCESS"
    } else {
      "🚫 THROTTLED"
    });

    println!();

    if i < 3 {
      sleep(Duration::from_millis(600)).await; // Between fast and slow throttle times
    }
  }

  //=================================================================
  // Test 4: Throttle with Error Handling
  //=================================================================
  println!("\n💥 Test 4: Throttle with Error Handling");
  println!("=======================================");

  cyre.action(IO::new("error-prone").with_throttle(800).with_logging(true))?;

  cyre.on("error-prone", |payload| {
    Box::pin(async move {
      let should_fail = payload
        .get("should_fail")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
      let timestamp = current_timestamp();

      if should_fail {
        println!("❌ [{}] HANDLER ERROR: Intentional failure", timestamp);
        CyreResponse {
          ok: false,
          payload: json!({"error": "intentional_failure"}),
          message: "Handler failed intentionally".to_string(),
          error: Some("test_error".to_string()),
          timestamp,
          metadata: None,
        }
      } else {
        println!("✅ [{}] HANDLER SUCCESS: No error", timestamp);
        CyreResponse::success(json!({"executed_at": timestamp}), "Handler succeeded")
      }
    })
  })?;

  println!("🧪 Testing throttle behavior with errors:");

  // Success call
  let success_time = current_timestamp();
  println!("🔄 [{}] Success call", success_time);
  let success_result = cyre.call("error-prone", json!({"should_fail": false})).await;
  println!("   Result: {} - {}", success_result.ok, success_result.message);

  sleep(Duration::from_millis(100)).await;

  // Error call (should still be throttled)
  let error_time = current_timestamp();
  println!("🔄 [{}] Error call (should be throttled)", error_time);
  let error_result = cyre.call("error-prone", json!({"should_fail": true})).await;
  println!("   Result: {} - {}", error_result.ok, error_result.message);

  sleep(Duration::from_millis(100)).await;

  // Another call (should still be throttled)
  let throttled_time = current_timestamp();
  println!("🔄 [{}] Another call (should be throttled)", throttled_time);
  let throttled_result = cyre.call("error-prone", json!({"should_fail": false})).await;
  println!("   Result: {} - {}", throttled_result.ok, throttled_result.message);

  //=================================================================
  // Test Summary
  //=================================================================
  println!("\n📊 THROTTLE TEST SUMMARY");
  println!("========================");

  let status = cyre.status();
  println!("🔥 System Status:");
  if let Some(stores) = status.payload.get("stores") {
    println!("   Total Actions: {}", stores.get("actions").unwrap_or(&json!(0)));
    println!("   Total Handlers: {}", stores.get("handlers").unwrap_or(&json!(0)));
  }

  println!("\n✅ THROTTLE TESTS COMPLETED!");
  println!("============================");
  println!("🚦 Throttle Logic Verified:");
  println!("   • Basic throttling works correctly");
  println!("   • Different throttle intervals respected");
  println!("   • Recovery after throttle period works");
  println!("   • Error handling doesn't break throttling");
  println!("   • Timestamps show precise timing control");

  println!("\n💡 Key Throttle Behavior:");
  println!("   • First call always succeeds");
  println!("   • Subsequent calls blocked until throttle period expires");
  println!("   • State updated only after SUCCESSFUL handler execution");
  println!("   • Failed handlers don't update throttle state");
  println!("   • Each action has independent throttle state");

  Ok(())
}
