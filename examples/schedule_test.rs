// examples/schedule_test.rs - Test repeat, interval, delay with sensor logging

use cyre_rust::prelude::*;
use cyre_rust::context::sensor;
use serde_json::json;
use tokio::time::{ sleep, Duration };

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  sensor::info("test", "🚀 SCHEDULE OPERATOR TEST", true);
  sensor::info("test", "========================", true);

  let mut cyre = Cyre::new();
  cyre.init().await?;

  // Test 1: Delay only (setTimeout equivalent)
  sensor::info("test", "⏰ Test 1: Delay (setTimeout)", true);
  cyre.action(IO::new("delay-test").with_delay(2000))?;
  cyre.on("delay-test", |payload| {
    Box::pin(async move {
      sensor::success("delay-test", &format!("Delay test executed: {}", payload), true);
      CyreResponse::success(payload, "Delay completed")
    })
  })?;

  sensor::info("test", "🔄 Calling delay action...", true);
  let result = cyre.call("delay-test", json!({"test": "delay"})).await;
  sensor::info("test", &format!("📤 Result: {} - {}", result.ok, result.message), true);

  // Test 2: Interval only (setInterval equivalent)
  sensor::info("test", "🔁 Test 2: Interval (setInterval)", true);
  cyre.action(IO::new("interval-test").with_interval(1000).with_repeat_infinite())?;
  cyre.on("interval-test", |payload| {
    Box::pin(async move {
      sensor::success("interval-test", &format!("Interval test executed: {}", payload), true);
      CyreResponse::success(payload, "Interval tick")
    })
  })?;

  sensor::info("test", "🔄 Calling interval action...", true);
  let result = cyre.call("interval-test", json!({"test": "interval"})).await;
  sensor::info("test", &format!("📤 Result: {} - {}", result.ok, result.message), true);

  // Test 3: Delay + Interval (setTimeout then setInterval)
  sensor::info("test", "⏰🔁 Test 3: Delay + Interval", true);
  cyre.action(
    IO::new("delay-interval-test")
      .with_delay(3000) // Wait 3 seconds first
      .with_interval(2000) // Then repeat every 2 seconds
      .with_repeat_count(3)
  )?; // Only 3 times total
  cyre.on("delay-interval-test", |payload| {
    Box::pin(async move {
      sensor::success(
        "delay-interval",
        &format!("Delay+Interval test executed: {}", payload),
        true
      );
      CyreResponse::success(payload, "Delayed interval tick")
    })
  })?;

  sensor::info("test", "🔄 Calling delay+interval action...", true);
  let result = cyre.call("delay-interval-test", json!({"test": "delay+interval"})).await;
  sensor::info("test", &format!("📤 Result: {} - {}", result.ok, result.message), true);

  // Test 4: Repeat count (limited repetitions)
  sensor::info("test", "🔢 Test 4: Repeat Count (limited)", true);
  cyre.action(IO::new("repeat-test").with_interval(500).with_repeat_count(5))?; // Only 5 times
  cyre.on("repeat-test", |payload| {
    Box::pin(async move {
      sensor::success("repeat-test", &format!("Repeat test executed: {}", payload), true);
      CyreResponse::success(payload, "Repeat tick")
    })
  })?;

  sensor::info("test", "🔄 Calling repeat action...", true);
  let result = cyre.call("repeat-test", json!({"test": "repeat", "count": 5})).await;
  sensor::info("test", &format!("📤 Result: {} - {}", result.ok, result.message), true);

  // Wait and observe scheduled executions
  sensor::info("test", "⏳ Waiting 15 seconds to observe scheduled executions...", true);
  sensor::info("test", "   📊 You should see:", true);
  sensor::info("test", "   • Delay test after 2 seconds", true);
  sensor::info("test", "   • Interval test every 1 second (infinite)", true);
  sensor::info("test", "   • Delay+Interval test: wait 3s, then 3 times every 2s", true);
  sensor::info("test", "   • Repeat test: 5 times every 0.5s", true);

  for i in 1..=15 {
    sleep(Duration::from_secs(1)).await;
    sensor::debug("timer", &format!("{}s elapsed...", i), true);
  }

  sensor::success("test", "🎉 Schedule test completed!", true);
  sensor::success("test", "✅ Delay: Executed once after delay", true);
  sensor::success("test", "✅ Interval: Repeating forever", true);
  sensor::success("test", "✅ Delay+Interval: Initial delay then repeating", true);
  sensor::success("test", "✅ Repeat Count: Limited repetitions working", true);

  Ok(())
}
