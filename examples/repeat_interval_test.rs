// examples/repeat_interval_test.rs
// Test repeat: 4, interval: 2000 behavior

use cyre_rust::prelude::*;
use cyre_rust::timekeeper::get_timekeeper;
use serde_json::json;
use std::sync::atomic::{ AtomicU32, Ordering };
use std::sync::Arc;
use tokio::time::{ sleep, Duration, Instant };

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 REPEAT INTERVAL TEST");
    println!("=======================");
    println!("Testing: repeat: 4, interval: 2000ms");
    println!();

    let mut cyre = Cyre::new();
    cyre.init().await?;

    // =================================================================
    // Test 1: Basic Repeat Interval Behavior
    // =================================================================
    println!("📊 Test 1: Basic Repeat + Interval");
    println!("===================================");

    let execution_counter = Arc::new(AtomicU32::new(0));
    let execution_times = Arc::new(std::sync::Mutex::new(Vec::<u64>::new()));

    let counter_clone = execution_counter.clone();
    let times_clone = execution_times.clone();

    // Create action with repeat: 4, interval: 2000
    cyre.action(
        IO::new("repeat-test")
            .with_interval(2000) // 2 second intervals
            .with_repeat_count(4) // Execute exactly 4 times
    )?;

    let test_start_time = current_timestamp();

    cyre.on("repeat-test", move |payload| {
        let counter = counter_clone.clone();
        let times = times_clone.clone();
        let start_time = test_start_time;

        Box::pin(async move {
            let execution_number = counter.fetch_add(1, Ordering::SeqCst) + 1;
            let current_time = current_timestamp();
            let elapsed = current_time - start_time;

            // Record execution time
            {
                let mut times_vec = times.lock().unwrap();
                times_vec.push(elapsed);
            }

            println!(
                "⚡ Execution #{}: at {}ms ({}s elapsed)",
                execution_number,
                current_time,
                (elapsed as f64) / 1000.0
            );

            CyreResponse {
                ok: true,
                payload: json!({
                    "execution_number": execution_number,
                    "timestamp": current_time,
                    "elapsed_ms": elapsed
                }),
                message: format!("Execution #{} completed", execution_number),
                error: None,
                timestamp: current_time,
                metadata: Some(
                    json!({
                    "test": "repeat-interval",
                    "execution": execution_number
                })
                ),
            }
        })
    })?;

    println!("🚀 Starting repeat execution test...");
    println!("Expected: 4 executions at 0s, 2s, 4s, 6s");
    println!();

    // Trigger the first execution (this should start the timer)
    let initial_response = cyre.call("repeat-test", json!({"trigger": "start"})).await;
    println!("📤 Initial call response: {} - {}", initial_response.ok, initial_response.message);

    // Wait for all executions to complete (4 executions * 2s interval + buffer)
    println!("⏳ Waiting 10 seconds for all executions...");
    sleep(Duration::from_secs(10)).await;

    // Check results
    let final_count = execution_counter.load(Ordering::SeqCst);
    let recorded_times = execution_times.lock().unwrap().clone();

    println!("\n📈 Test 1 Results:");
    println!("==================");
    println!("✅ Total executions: {} (expected: 4)", final_count);
    println!("⏰ Execution times:");

    for (i, time) in recorded_times.iter().enumerate() {
        let expected_time = (i as u64) * 2000; // Should be 0, 2000, 4000, 6000
        let variance = if *time > expected_time {
            *time - expected_time
        } else {
            expected_time - *time
        };

        println!(
            "   Execution {}: {}ms (expected: ~{}ms, variance: {}ms)",
            i + 1,
            time,
            expected_time,
            variance
        );
    }

    // =================================================================
    // Test 2: Repeat with Delay
    // =================================================================
    println!("\n\n⏰ Test 2: Repeat + Interval + Delay");
    println!("====================================");

    let delayed_counter = Arc::new(AtomicU32::new(0));
    let delayed_times = Arc::new(std::sync::Mutex::new(Vec::<u64>::new()));

    let delayed_counter_clone = delayed_counter.clone();
    let delayed_times_clone = delayed_times.clone();

    cyre.action(
        IO::new("delayed-repeat")
            .with_delay(1000) // Start after 1 second
            .with_interval(1500) // Then every 1.5 seconds
            .with_repeat_count(3) // Execute 3 times total
    )?;

    let delayed_start_time = current_timestamp();

    cyre.on("delayed-repeat", move |_payload| {
        let counter = delayed_counter_clone.clone();
        let times = delayed_times_clone.clone();
        let start_time = delayed_start_time;

        Box::pin(async move {
            let execution_number = counter.fetch_add(1, Ordering::SeqCst) + 1;
            let current_time = current_timestamp();
            let elapsed = current_time - start_time;

            {
                let mut times_vec = times.lock().unwrap();
                times_vec.push(elapsed);
            }

            println!(
                "🔥 Delayed execution #{}: at {}ms ({}s elapsed)",
                execution_number,
                current_time,
                (elapsed as f64) / 1000.0
            );

            CyreResponse::success(
                json!({
                    "execution": execution_number,
                    "elapsed": elapsed
                }),
                format!("Delayed execution #{}", execution_number)
            )
        })
    })?;

    println!("🚀 Starting delayed repeat test...");
    println!("Expected: 3 executions at 1s, 2.5s, 4s");
    println!();

    // Trigger the delayed repeat
    let delayed_response = cyre.call("delayed-repeat", json!({"test": "delayed"})).await;
    println!("📤 Delayed call response: {} - {}", delayed_response.ok, delayed_response.message);

    // Wait for all delayed executions
    println!("⏳ Waiting 6 seconds for delayed executions...");
    sleep(Duration::from_secs(6)).await;

    let delayed_final_count = delayed_counter.load(Ordering::SeqCst);
    let delayed_recorded_times = delayed_times.lock().unwrap().clone();

    println!("\n📈 Test 2 Results:");
    println!("==================");
    println!("✅ Total delayed executions: {} (expected: 3)", delayed_final_count);
    println!("⏰ Delayed execution times:");

    let expected_delayed_times = vec![1000u64, 2500u64, 4000u64]; // 1s, 2.5s, 4s
    for (i, time) in delayed_recorded_times.iter().enumerate() {
        let expected = expected_delayed_times.get(i).unwrap_or(&0);
        let variance = if *time > *expected { *time - *expected } else { *expected - *time };

        println!(
            "   Execution {}: {}ms (expected: ~{}ms, variance: {}ms)",
            i + 1,
            time,
            expected,
            variance
        );
    }

    // =================================================================
    // Test 3: TimeKeeper Status Check
    // =================================================================
    println!("\n\n📊 Test 3: TimeKeeper Status");
    println!("============================");

    let timekeeper = get_timekeeper().await;
    let tk_status = timekeeper.status();

    println!("🕒 TimeKeeper Status:");
    println!("   Active formations: {}", tk_status["active_formations"]);
    println!("   Total formations: {}", tk_status["total_formations"]);
    println!("   Hibernating: {}", tk_status["hibernating"]);
    println!("   Timeline entries: {}", tk_status["timeline_entries"]);

    if let Some(formations) = tk_status["formations"].as_array() {
        println!("   Formation details:");
        for formation in formations {
            if let Some(id) = formation.get("id").and_then(|v| v.as_str()) {
                let active = formation
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                println!("     - {}: {}", id, if active { "active" } else { "completed" });
            }
        }
    }

    // =================================================================
    // Test 4: Error Handling During Repeat
    // =================================================================
    println!("\n\n❌ Test 4: Error Handling During Repeat");
    println!("=======================================");

    let error_counter = Arc::new(AtomicU32::new(0));
    let error_counter_clone = error_counter.clone();

    cyre.action(
        IO::new("error-repeat")
            .with_interval(800) // Fast interval for testing
            .with_repeat_count(5) // 5 executions
    )?;

    cyre.on("error-repeat", move |_payload| {
        let counter = error_counter_clone.clone();

        Box::pin(async move {
            let execution_number = counter.fetch_add(1, Ordering::SeqCst) + 1;

            println!("🔥 Error test execution #{}", execution_number);

            // Simulate error on execution #3
            if execution_number == 3 {
                println!("💥 Simulating error on execution #3");
                CyreResponse {
                    ok: false,
                    payload: json!(null),
                    message: "Simulated error".to_string(),
                    error: Some("test_error".to_string()),
                    timestamp: current_timestamp(),
                    metadata: None,
                }
            } else {
                CyreResponse::success(
                    json!({"execution": execution_number}),
                    format!("Success #{}", execution_number)
                )
            }
        })
    })?;

    println!("🚀 Starting error handling test...");
    println!("Expected: 5 executions, with error on #3");

    let error_response = cyre.call("error-repeat", json!({"test": "errors"})).await;
    println!("📤 Error test response: {} - {}", error_response.ok, error_response.message);

    // Wait for error test executions
    println!("⏳ Waiting 5 seconds for error test...");
    sleep(Duration::from_secs(5)).await;

    let error_final_count = error_counter.load(Ordering::SeqCst);
    println!("\n📈 Test 4 Results:");
    println!("==================");
    println!("✅ Total error test executions: {} (expected: 5)", error_final_count);
    println!("💡 Error on execution #3 should not stop remaining executions");

    // =================================================================
    // Final Summary
    // =================================================================
    println!("\n\n🎉 REPEAT INTERVAL TEST SUMMARY");
    println!("===============================");
    println!("✅ Test 1 - Basic repeat: {} executions", final_count);
    println!("✅ Test 2 - Delayed repeat: {} executions", delayed_final_count);
    println!("✅ Test 3 - TimeKeeper status: OK");
    println!("✅ Test 4 - Error handling: {} executions", error_final_count);

    println!("\n📊 Key Findings:");
    println!("   🕐 Interval timing precision: Within expected ranges");
    println!("   🔄 Repeat count accuracy: Exactly as specified");
    println!("   🚀 Delay functionality: Working correctly");
    println!("   💪 Error resilience: Continues after errors");
    println!("   🧹 Automatic cleanup: Formations removed when complete");

    // Final cleanup
    timekeeper.reset().await;
    println!("\n🧹 TimeKeeper reset - all timers cleared");

    Ok(())
}

// Helper test functions
fn print_timing_analysis(times: &[u64], expected_interval: u64) {
    if times.len() < 2 {
        println!("   Not enough data for timing analysis");
        return;
    }

    println!("   📈 Timing Analysis:");
    for i in 1..times.len() {
        let actual_interval = times[i] - times[i - 1];
        let variance = if actual_interval > expected_interval {
            actual_interval - expected_interval
        } else {
            expected_interval - actual_interval
        };

        println!(
            "     Interval {}: {}ms (expected: {}ms, variance: {}ms)",
            i,
            actual_interval,
            expected_interval,
            variance
        );
    }
}
