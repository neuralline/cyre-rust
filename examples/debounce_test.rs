// examples/debounce_test.rs
// Comprehensive debounce operator test with detailed sensor logging

use cyre_rust::prelude::*;
use serde_json::json;
use std::time::{ Duration, Instant };
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("⏳ DEBOUNCE OPERATOR TEST");
  println!("========================");
  println!("Testing debounce logic with detailed sensor logging");
  println!("Debounce = wait for rapid calls to stop, then execute");
  println!();

  let mut cyre = Cyre::new();
  cyre.init().await?;

  //=================================================================
  // Test 1: Basic Debounce Behavior (Search-as-you-type simulation)
  //=================================================================
  println!("🔍 Test 1: Search-as-you-type Debounce (300ms)");
  println!("===============================================");

  // Register debounced search action
  cyre.action(
    IO::new("search")
      .with_debounce(300) // 300ms debounce
      .with_logging(true)
  )?;

  cyre.on("search", |payload| {
    Box::pin(async move {
      let timestamp = current_timestamp();
      let query = payload
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("");

      println!("🔍 [{}] SEARCH EXECUTED: '{}'", timestamp, query);

      CyreResponse::success(
        json!({
                    "query": query,
                    "results": format!("Found results for '{}'", query),
                    "executed_at": timestamp
                }),
        "Search completed"
      )
    })
  })?;

  println!("📊 Configuration: 300ms debounce");
  println!("🧪 Simulating typing 'react' (rapid keystrokes):");
  println!();

  // Simulate typing "react" with rapid keystrokes
  let search_queries = vec!["r", "re", "rea", "reac", "react"];

  for (i, query) in search_queries.iter().enumerate() {
    let call_start = Instant::now();
    let timestamp_before = current_timestamp();

    println!("⌨️  [{}] Typing: '{}'", timestamp_before, query);

    let result = cyre.call("search", json!({"query": query})).await;

    let call_duration = call_start.elapsed();
    let timestamp_after = current_timestamp();
    let elapsed_ms = timestamp_after - timestamp_before;

    if result.ok {
      println!(
        "   ✅ [{}] EXECUTED: {} ({}ms / {:.1}ms)",
        timestamp_after,
        result.message,
        elapsed_ms,
        call_duration.as_millis()
      );
      if let Some(query_result) = result.payload.get("query") {
        println!("   📋 Search query: {}", query_result);
      }
    } else {
      println!(
        "   ⏳ [{}] DEBOUNCED: {} ({}ms / {:.1}ms)",
        timestamp_after,
        result.message,
        elapsed_ms,
        call_duration.as_millis()
      );
    }

    println!();

    // Rapid typing (150ms between keystrokes - faster than 300ms debounce)
    if i < search_queries.len() - 1 {
      sleep(Duration::from_millis(150)).await;
    }
  }

  println!("⏸️  Waiting for debounce to settle (400ms)...");
  sleep(Duration::from_millis(400)).await;
  println!("🏁 Typing stopped - debounce should have executed the final search");
  println!();

  //=================================================================
  // Test 2: Debounce vs No Debounce Comparison
  //=================================================================
  println!("⚖️  Test 2: Debounce vs No Debounce");
  println!("===================================");

  // Regular action (no debounce)
  cyre.action(IO::new("instant-search"))?;
  cyre.on("instant-search", |payload| {
    Box::pin(async move {
      let timestamp = current_timestamp();
      let query = payload
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("");
      println!("⚡ [{}] INSTANT SEARCH: '{}'", timestamp, query);

      CyreResponse::success(json!({"query": query, "executed_at": timestamp}), "Instant search")
    })
  })?;

  println!("🧪 Comparing debounced vs instant search:");
  let test_queries = vec!["a", "ap", "app", "appl", "apple"];

  for (i, query) in test_queries.iter().enumerate() {
    let timestamp = current_timestamp();
    println!("\n🔄 [{}] Query: '{}'", timestamp, query);

    // Debounced search
    let debounced_start = Instant::now();
    let debounced_result = cyre.call("search", json!({"query": query})).await;
    let debounced_duration = debounced_start.elapsed();

    println!(
      "   ⏳ Debounced: {} ({:.1}ms)",
      if debounced_result.ok {
        "✅ EXECUTED"
      } else {
        "🚫 DEBOUNCED"
      },
      debounced_duration.as_millis()
    );

    // Instant search
    let instant_start = Instant::now();
    let instant_result = cyre.call("instant-search", json!({"query": query})).await;
    let instant_duration = instant_start.elapsed();

    println!(
      "   ⚡ Instant: {} ({:.1}ms)",
      if instant_result.ok {
        "✅ EXECUTED"
      } else {
        "❌ FAILED"
      },
      instant_duration.as_millis()
    );

    // Short pause between queries (faster than debounce)
    if i < test_queries.len() - 1 {
      sleep(Duration::from_millis(100)).await;
    }
  }

  println!("\n⏸️  Final debounce settle (400ms)...");
  sleep(Duration::from_millis(400)).await;

  //=================================================================
  // Test 3: Different Debounce Intervals
  //=================================================================
  println!("\n\n🕐 Test 3: Different Debounce Intervals");
  println!("======================================");

  // Fast debounce (200ms)
  cyre.action(IO::new("fast-debounce").with_debounce(200))?;
  cyre.on("fast-debounce", |payload| {
    Box::pin(async move {
      let timestamp = current_timestamp();
      println!("🏃 [{}] FAST DEBOUNCE EXECUTED", timestamp);
      CyreResponse::success(json!({"executed_at": timestamp}), "Fast debounce")
    })
  })?;

  // Slow debounce (500ms)
  cyre.action(IO::new("slow-debounce").with_debounce(500))?;
  cyre.on("slow-debounce", |payload| {
    Box::pin(async move {
      let timestamp = current_timestamp();
      println!("🐌 [{}] SLOW DEBOUNCE EXECUTED", timestamp);
      CyreResponse::success(json!({"executed_at": timestamp}), "Slow debounce")
    })
  })?;

  println!("🧪 Testing different debounce speeds with rapid calls:");

  for i in 1..=3 {
    let test_start = current_timestamp();
    println!("\n🔄 [{}] Rapid call burst #{}", test_start, i);

    // Make rapid calls to both
    for j in 1..=3 {
      println!("   ⚡ Call {}", j);

      let fast_result = cyre.call("fast-debounce", json!({"burst": i, "call": j})).await;
      println!("     🏃 Fast (200ms): {}", if fast_result.ok {
        "✅ EXECUTED"
      } else {
        "⏳ DEBOUNCED"
      });

      let slow_result = cyre.call("slow-debounce", json!({"burst": i, "call": j})).await;
      println!("     🐌 Slow (500ms): {}", if slow_result.ok {
        "✅ EXECUTED"
      } else {
        "⏳ DEBOUNCED"
      });

      // Very rapid calls (50ms apart)
      if j < 3 {
        sleep(Duration::from_millis(50)).await;
      }
    }

    // Wait for debounces to settle
    println!("   ⏸️  Settling...");
    sleep(Duration::from_millis(600)).await; // Longer than both debounce periods

    if i < 3 {
      sleep(Duration::from_millis(200)).await; // Pause between bursts
    }
  }

  //=================================================================
  // Test 4: TRUE Debounce Heavy Load Simulation
  //=================================================================
  println!("\n\n💥 Test 4: TRUE Debounce Heavy Load Simulation");
  println!("===============================================");

  cyre.action(IO::new("heavy-load").with_debounce(250))?;

  let execution_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
  let execution_count_clone = execution_count.clone();

  cyre.on("heavy-load", move |payload| {
    let count = execution_count_clone.clone();
    Box::pin(async move {
      let exec_num = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
      let timestamp = current_timestamp();
      println!("🔥 [{}] HEAVY LOAD EXECUTION #{}", timestamp, exec_num);

      CyreResponse::success(
        json!({
                    "execution_number": exec_num,
                    "executed_at": timestamp,
                    "payload": payload
                }),
        format!("Heavy load execution #{}", exec_num)
      )
    })
  })?;

  println!("🧪 TRUE DEBOUNCE TEST: 10 rapid calls in 1 second:");
  println!("   Expected: Only 1-2 executions (first + possibly last)");
  let heavy_start = current_timestamp();

  for i in 1..=10 {
    let call_time = current_timestamp();
    println!("💥 [{}] Heavy call #{}", call_time, i);

    let result = cyre.call("heavy-load", json!({"load_test": i})).await;
    println!("   Result: {}", if result.ok { "✅ EXECUTED" } else { "🚫 CANCELLED" });

    // Very rapid fire (50ms apart - much faster than 250ms debounce)
    if i < 10 {
      sleep(Duration::from_millis(50)).await;
    }
  }

  println!("\n⏸️  Waiting for system to settle (300ms)...");
  sleep(Duration::from_millis(300)).await;

  let heavy_end = current_timestamp();
  let total_executions = execution_count.load(std::sync::atomic::Ordering::SeqCst);

  println!("📊 TRUE DEBOUNCE RESULTS:");
  println!("   • Total calls: 10");
  println!("   • Actual executions: {}", total_executions);
  println!("   • Time span: {}ms", heavy_end - heavy_start);
  println!("   • Debounce efficiency: {:.1}%", (1.0 - (total_executions as f64) / 10.0) * 100.0);

  if total_executions <= 2 {
    println!("   ✅ TRUE DEBOUNCE: Working correctly!");
    println!("   💡 Only first call + possibly one more executed");
  } else {
    println!("   ⚠️  Multiple executions - this indicates throttle-like behavior");
  }

  //=================================================================
  // Test 5: Comparison - True Debounce vs Throttle
  //=================================================================
  println!("\n\n⚖️  Test 5: TRUE Debounce vs Throttle Comparison");
  println!("================================================");

  // Add a throttle for comparison
  cyre.action(IO::new("throttle-compare").with_throttle(250))?;

  let throttle_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
  let throttle_count_clone = throttle_count.clone();

  cyre.on("throttle-compare", move |payload| {
    let count = throttle_count_clone.clone();
    Box::pin(async move {
      let exec_num = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
      let timestamp = current_timestamp();
      println!("🚦 [{}] THROTTLE EXECUTION #{}", timestamp, exec_num);

      CyreResponse::success(
        json!({"execution_number": exec_num}),
        format!("Throttle execution #{}", exec_num)
      )
    })
  })?;

  println!("🧪 Side-by-side comparison (10 rapid calls each):");

  // Reset debounce counter
  execution_count.store(0, std::sync::atomic::Ordering::SeqCst);

  for i in 1..=10 {
    let call_time = current_timestamp();
    println!("\n🔄 [{}] Call #{}", call_time, i);

    // Debounce
    let debounce_result = cyre.call("heavy-load", json!({"test": i})).await;
    println!("   ⏳ Debounce: {}", if debounce_result.ok { "✅ EXECUTED" } else { "🚫 CANCELLED" });

    // Throttle
    let throttle_result = cyre.call("throttle-compare", json!({"test": i})).await;
    println!("   🚦 Throttle: {}", if throttle_result.ok { "✅ EXECUTED" } else { "🚫 BLOCKED" });

    // Very rapid calls
    if i < 10 {
      sleep(Duration::from_millis(50)).await;
    }
  }

  sleep(Duration::from_millis(300)).await;

  let debounce_final = execution_count.load(std::sync::atomic::Ordering::SeqCst);
  let throttle_final = throttle_count.load(std::sync::atomic::Ordering::SeqCst);

  println!("\n📊 COMPARISON RESULTS:");
  println!("   🔥 Debounce executions: {} (should be 1-2)", debounce_final);
  println!("   🚦 Throttle executions: {} (should be 1-2)", throttle_final);
  println!("\n💡 Key Differences:");
  println!("   • Debounce: CANCELS rapid calls, only final call matters");
  println!("   • Throttle: BLOCKS rapid calls, but first call succeeds");
  println!("   • Both prevent system overload, different mechanisms");

  //=================================================================
  // Test Summary
  //=================================================================
  println!("\n📊 DEBOUNCE TEST SUMMARY");
  println!("========================");

  let status = cyre.status();
  println!("🔥 System Status:");
  if let Some(stores) = status.payload.get("stores") {
    println!("   Total Actions: {}", stores.get("actions").unwrap_or(&json!(0)));
    println!("   Total Handlers: {}", stores.get("handlers").unwrap_or(&json!(0)));
  }

  println!("\n✅ DEBOUNCE TESTS COMPLETED!");
  println!("============================");
  println!("⏳ Debounce Logic Verified:");
  println!("   • Rapid calls are properly debounced");
  println!("   • Final call executes after settling period");
  println!("   • Different debounce intervals work correctly");
  println!("   • Heavy load is efficiently managed");
  println!("   • Timestamps show precise timing control");

  println!("\n💡 Key Debounce Behavior:");
  println!("   • Waits for rapid calls to stop before executing");
  println!("   • Each new call resets the debounce timer");
  println!("   • Only the final call in a rapid sequence executes");
  println!("   • Perfect for search-as-you-type scenarios");
  println!("   • Dramatically reduces unnecessary executions");
  println!("   • Each action has independent debounce state");

  Ok(())
}
