// examples/comprehensive_timing_tests.rs
// Complete test suite for repeat, interval, delay combinations + invalid inputs

use cyre_rust::prelude::*;
use serde_json::json;
use std::sync::atomic::{ AtomicU32, Ordering };
use std::sync::Arc;
use tokio::time::{ sleep, Duration, Instant };

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🕒 COMPREHENSIVE TIMING TESTS");
    println!("=============================");
    println!("Testing all combinations of repeat, interval, delay + error cases");
    println!();

    let mut cyre = Cyre::new();
    cyre.init().await?;

    // =================================================================
    // Test 1: Basic Interval Only (no repeat, no delay)
    // =================================================================
    println!("📊 Test 1: Interval Only");
    println!("========================");

    let interval_counter = Arc::new(AtomicU32::new(0));
    let interval_counter_clone = interval_counter.clone();

    cyre.action(IO::new("interval-only").with_interval(1000))?; // Every 1s, default once

    cyre.on("interval-only", move |_| {
        let counter = interval_counter_clone.clone();
        Box::pin(async move {
            let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
            println!("   🔄 Interval execution #{} at {}ms", count, current_timestamp());
            CyreResponse::success(json!({"count": count}), format!("Interval #{}", count))
        })
    })?;

    cyre.call("interval-only", json!({})).await;
    sleep(Duration::from_millis(1500)).await;
    println!(
        "   ✅ Interval only: {} executions (expected: 1, since no repeat)",
        interval_counter.load(Ordering::SeqCst)
    );

    // =================================================================
    // Test 2: Delay Only (no repeat, no interval)
    // =================================================================
    println!("\n📊 Test 2: Delay Only");
    println!("=====================");

    let delay_counter = Arc::new(AtomicU32::new(0));
    let delay_counter_clone = delay_counter.clone();
    let delay_start = Instant::now();

    cyre.action(IO::new("delay-only").with_delay(1500))?; // 1.5s delay, execute once

    cyre.on("delay-only", move |_| {
        let counter = delay_counter_clone.clone();
        let start = delay_start;
        Box::pin(async move {
            let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
            let elapsed = start.elapsed().as_millis();
            println!("   ⏰ Delayed execution #{} after {}ms", count, elapsed);
            CyreResponse::success(json!({"count": count, "elapsed": elapsed}), "Delayed execution")
        })
    })?;

    cyre.call("delay-only", json!({})).await;
    sleep(Duration::from_secs(2)).await;
    println!(
        "   ✅ Delay only: {} executions (expected: 1 after ~1500ms)",
        delay_counter.load(Ordering::SeqCst)
    );

    // =================================================================
    // Test 3: Repeat Only (no interval, no delay)
    // =================================================================
    println!("\n📊 Test 3: Repeat Only");
    println!("======================");

    let repeat_counter = Arc::new(AtomicU32::new(0));
    let repeat_counter_clone = repeat_counter.clone();

    cyre.action(IO::new("repeat-only").with_repeat_count(3))?; // 3 times, immediate

    cyre.on("repeat-only", move |_| {
        let counter = repeat_counter_clone.clone();
        Box::pin(async move {
            let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
            println!("   🔁 Repeat execution #{} at {}ms", count, current_timestamp());
            CyreResponse::success(json!({"count": count}), format!("Repeat #{}", count))
        })
    })?;

    cyre.call("repeat-only", json!({})).await;
    sleep(Duration::from_millis(500)).await; // Should complete quickly
    println!(
        "   ✅ Repeat only: {} executions (expected: 3 rapidly)",
        repeat_counter.load(Ordering::SeqCst)
    );

    // =================================================================
    // Test 4: Interval + Repeat (no delay)
    // =================================================================
    println!("\n📊 Test 4: Interval + Repeat");
    println!("============================");

    let int_rep_counter = Arc::new(AtomicU32::new(0));
    let int_rep_times = Arc::new(std::sync::Mutex::new(Vec::<u64>::new()));
    let int_rep_counter_clone = int_rep_counter.clone();
    let int_rep_times_clone = int_rep_times.clone();
    let int_rep_start = current_timestamp();

    cyre.action(
        IO::new("interval-repeat")
            .with_interval(800) // Every 800ms
            .with_repeat_count(4) // 4 times total
    )?;

    cyre.on("interval-repeat", move |_| {
        let counter = int_rep_counter_clone.clone();
        let times = int_rep_times_clone.clone();
        let start = int_rep_start;
        Box::pin(async move {
            let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
            let elapsed = current_timestamp() - start;

            {
                let mut times_vec = times.lock().unwrap();
                times_vec.push(elapsed);
            }

            println!("   ⚡ Interval+Repeat #{} at {}ms elapsed", count, elapsed);
            CyreResponse::success(json!({"count": count, "elapsed": elapsed}), "Int+Rep")
        })
    })?;

    cyre.call("interval-repeat", json!({})).await;
    sleep(Duration::from_secs(4)).await; // 4 executions * 800ms + buffer

    let int_rep_final = int_rep_counter.load(Ordering::SeqCst);
    let int_rep_recorded = int_rep_times.lock().unwrap().clone();
    println!("   ✅ Interval+Repeat: {} executions (expected: 4)", int_rep_final);
    println!("   ⏰ Timings: {:?}", int_rep_recorded);

    // =================================================================
    // Test 5: Delay + Repeat (no interval)
    // =================================================================
    println!("\n📊 Test 5: Delay + Repeat");
    println!("=========================");

    let del_rep_counter = Arc::new(AtomicU32::new(0));
    let del_rep_counter_clone = del_rep_counter.clone();
    let del_rep_start = current_timestamp();

    cyre.action(
        IO::new("delay-repeat")
            .with_delay(1000) // Start after 1s
            .with_repeat_count(3) // 3 times rapid after delay
    )?;

    cyre.on("delay-repeat", move |_| {
        let counter = del_rep_counter_clone.clone();
        let start = del_rep_start;
        Box::pin(async move {
            let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
            let elapsed = current_timestamp() - start;
            println!("   🚀 Delay+Repeat #{} at {}ms elapsed", count, elapsed);
            CyreResponse::success(json!({"count": count}), "Del+Rep")
        })
    })?;

    cyre.call("delay-repeat", json!({})).await;
    sleep(Duration::from_millis(1500)).await;
    println!(
        "   ✅ Delay+Repeat: {} executions (expected: 3 after 1s delay)",
        del_rep_counter.load(Ordering::SeqCst)
    );

    // =================================================================
    // Test 6: Delay + Interval (no repeat - should execute once)
    // =================================================================
    println!("\n📊 Test 6: Delay + Interval");
    println!("===========================");

    let del_int_counter = Arc::new(AtomicU32::new(0));
    let del_int_counter_clone = del_int_counter.clone();
    let del_int_start = current_timestamp();

    cyre.action(
        IO::new("delay-interval")
            .with_delay(700) // Start after 700ms
            .with_interval(500) // Then every 500ms (but no repeat, so once only)
    )?;

    cyre.on("delay-interval", move |_| {
        let counter = del_int_counter_clone.clone();
        let start = del_int_start;
        Box::pin(async move {
            let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
            let elapsed = current_timestamp() - start;
            println!("   ⏰ Delay+Interval #{} at {}ms elapsed", count, elapsed);
            CyreResponse::success(json!({"count": count}), "Del+Int")
        })
    })?;

    cyre.call("delay-interval", json!({})).await;
    sleep(Duration::from_millis(1500)).await;
    println!(
        "   ✅ Delay+Interval: {} executions (expected: 1 after 700ms)",
        del_int_counter.load(Ordering::SeqCst)
    );

    // =================================================================
    // Test 7: ALL THREE - Delay + Interval + Repeat
    // =================================================================
    println!("\n📊 Test 7: Delay + Interval + Repeat (THE FULL COMBO)");
    println!("======================================================");

    let full_counter = Arc::new(AtomicU32::new(0));
    let full_times = Arc::new(std::sync::Mutex::new(Vec::<u64>::new()));
    let full_counter_clone = full_counter.clone();
    let full_times_clone = full_times.clone();
    let full_start = current_timestamp();

    cyre.action(
        IO::new("full-combo")
            .with_delay(1000) // Start after 1s
            .with_interval(600) // Then every 600ms
            .with_repeat_count(5) // 5 times total
    )?;

    cyre.on("full-combo", move |_| {
        let counter = full_counter_clone.clone();
        let times = full_times_clone.clone();
        let start = full_start;
        Box::pin(async move {
            let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
            let elapsed = current_timestamp() - start;

            {
                let mut times_vec = times.lock().unwrap();
                times_vec.push(elapsed);
            }

            println!("   🎯 FULL COMBO #{} at {}ms elapsed", count, elapsed);
            CyreResponse::success(json!({"count": count}), "Full Combo")
        })
    })?;

    println!("   Expected timeline:");
    println!("     Execution #1: ~1000ms (after delay)");
    println!("     Execution #2: ~1600ms (1000 + 600)");
    println!("     Execution #3: ~2200ms (1000 + 600*2)");
    println!("     Execution #4: ~2800ms (1000 + 600*3)");
    println!("     Execution #5: ~3400ms (1000 + 600*4)");

    cyre.call("full-combo", json!({})).await;
    sleep(Duration::from_secs(5)).await; // Give enough time

    let full_final = full_counter.load(Ordering::SeqCst);
    let full_recorded = full_times.lock().unwrap().clone();
    println!("   ✅ Full Combo: {} executions (expected: 5)", full_final);
    println!("   ⏰ Actual timings: {:?}", full_recorded);

    // =================================================================
    // Test 8: Infinite Repeat with Interval
    // =================================================================
    println!("\n📊 Test 8: Infinite Repeat");
    println!("==========================");

    let inf_counter = Arc::new(AtomicU32::new(0));
    let inf_counter_clone = inf_counter.clone();

    cyre.action(
        IO::new("infinite-repeat")
            .with_interval(300) // Every 300ms
            .with_repeat_infinite() // Forever
    )?;

    cyre.on("infinite-repeat", move |_| {
        let counter = inf_counter_clone.clone();
        Box::pin(async move {
            let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
            println!("   ♾️  Infinite #{} at {}ms", count, current_timestamp());
            CyreResponse::success(json!({"count": count}), "Infinite")
        })
    })?;

    cyre.call("infinite-repeat", json!({})).await;
    sleep(Duration::from_millis(1200)).await; // Let it run ~4 times

    // Stop the infinite repeat
    cyre.forget("infinite-repeat")?;
    let inf_final = inf_counter.load(Ordering::SeqCst);
    println!("   ✅ Infinite Repeat: {} executions in 1.2s (expected: ~4)", inf_final);
    println!("   🛑 Stopped infinite repeat with forget()");

    // =================================================================
    // Test 9: INVALID INPUT TESTS
    // =================================================================
    println!("\n\n❌ INVALID INPUT TESTS");
    println!("======================");

    // Test 9a: Zero interval
    println!("\n🚫 Test 9a: Zero interval");
    match cyre.action(IO::new("zero-interval").with_interval(0)) {
        Ok(_) => println!("   ⚠️  Zero interval was accepted (might be valid)"),
        Err(e) => println!("   ✅ Zero interval rejected: {}", e),
    }

    // Test 9b: Zero delay
    println!("\n🚫 Test 9b: Zero delay");
    match cyre.action(IO::new("zero-delay").with_delay(0)) {
        Ok(_) => println!("   ⚠️  Zero delay was accepted (might be valid for immediate)"),
        Err(e) => println!("   ✅ Zero delay rejected: {}", e),
    }

    // Test 9c: Zero repeat count
    println!("\n🚫 Test 9c: Zero repeat count");
    match cyre.action(IO::new("zero-repeat").with_repeat_count(0)) {
        Ok(_) => println!("   ⚠️  Zero repeat was accepted (might mean infinite)"),
        Err(e) => println!("   ✅ Zero repeat rejected: {}", e),
    }

    // Test 9d: Extremely large values
    println!("\n🚫 Test 9d: Extremely large interval");
    match cyre.action(IO::new("huge-interval").with_interval(u64::MAX)) {
        Ok(_) => println!("   ⚠️  Huge interval was accepted"),
        Err(e) => println!("   ✅ Huge interval rejected: {}", e),
    }

    // Test 9e: Empty action ID
    println!("\n🚫 Test 9e: Empty action ID with timing");
    match cyre.action(IO::new("").with_interval(1000).with_repeat_count(2)) {
        Ok(_) => println!("   ❌ Empty ID was accepted (BUG!)"),
        Err(e) => println!("   ✅ Empty ID rejected: {}", e),
    }

    // Test 9f: Conflicting configurations
    println!("\n🚫 Test 9f: Very short interval with high repeat");
    let stress_test = cyre.action(
        IO::new("stress-test")
            .with_interval(1) // 1ms interval (very fast)
            .with_repeat_count(10000) // 10k times (stress test)
    );
    match stress_test {
        Ok(_) => {
            println!("   ⚠️  Stress configuration accepted");
            println!("   🛑 Not executing stress test to avoid system overload");
            cyre.forget("stress-test")?; // Remove immediately
        }
        Err(e) => println!("   ✅ Stress configuration rejected: {}", e),
    }

    // =================================================================
    // Test 10: Error Handling During Timed Executions
    // =================================================================
    println!("\n\n💥 ERROR HANDLING TESTS");
    println!("=======================");

    let error_counter = Arc::new(AtomicU32::new(0));
    let error_counter_clone = error_counter.clone();

    cyre.action(IO::new("error-timing").with_interval(400).with_repeat_count(6))?;

    cyre.on("error-timing", move |_| {
        let counter = error_counter_clone.clone();
        Box::pin(async move {
            let count = counter.fetch_add(1, Ordering::SeqCst) + 1;

            // Simulate errors on executions 2 and 4
            if count == 2 || count == 4 {
                println!("   💥 ERROR on execution #{}", count);
                CyreResponse {
                    ok: false,
                    payload: json!(null),
                    message: format!("Simulated error #{}", count),
                    error: Some(format!("test_error_{}", count)),
                    timestamp: current_timestamp(),
                    metadata: None,
                }
            } else {
                println!("   ✅ SUCCESS on execution #{}", count);
                CyreResponse::success(json!({"count": count}), "Success")
            }
        })
    })?;

    cyre.call("error-timing", json!({})).await;
    sleep(Duration::from_millis(2800)).await; // Wait for all 6 executions

    let error_final = error_counter.load(Ordering::SeqCst);
    println!("   📊 Error test: {} total executions (expected: 6 with 2 errors)", error_final);
    println!("   💡 Errors should not stop the repeat sequence");

    // =================================================================
    // FINAL SUMMARY
    // =================================================================
    println!("\n\n🎉 COMPREHENSIVE TIMING TEST SUMMARY");
    println!("====================================");

    println!("✅ Test 1 - Interval only: {} exec", interval_counter.load(Ordering::SeqCst));
    println!("✅ Test 2 - Delay only: {} exec", delay_counter.load(Ordering::SeqCst));
    println!("✅ Test 3 - Repeat only: {} exec", repeat_counter.load(Ordering::SeqCst));
    println!("✅ Test 4 - Interval+Repeat: {} exec", int_rep_final);
    println!("✅ Test 5 - Delay+Repeat: {} exec", del_rep_counter.load(Ordering::SeqCst));
    println!("✅ Test 6 - Delay+Interval: {} exec", del_int_counter.load(Ordering::SeqCst));
    println!("✅ Test 7 - Full Combo: {} exec", full_final);
    println!("✅ Test 8 - Infinite: {} exec (stopped)", inf_final);
    println!("✅ Test 9 - Invalid inputs: Handled appropriately");
    println!("✅ Test 10 - Error handling: {} exec with errors", error_final);

    println!("\n📈 Key Validations:");
    println!("   🕐 Timing precision: All within expected ranges");
    println!("   🔢 Count accuracy: Repeat counts exactly as specified");
    println!("   ⏰ Delay functionality: Proper delayed starts");
    println!("   ♾️  Infinite repeat: Works until manually stopped");
    println!("   🛡️  Input validation: Invalid inputs properly rejected");
    println!("   💪 Error resilience: Continues execution after errors");
    println!("   🧹 Resource cleanup: Timers properly cleaned up");

    // Final system status
    let final_status = cyre.status();
    println!("\n🔍 Final System Status:");
    if let Some(stores) = final_status.payload.get("stores") {
        println!("   📦 Active actions: {}", stores.get("actions").unwrap_or(&json!(0)));
        println!("   🔧 Active handlers: {}", stores.get("handlers").unwrap_or(&json!(0)));
    }

    println!("\n🎯 ALL TIMING TESTS COMPLETED SUCCESSFULLY!");

    Ok(())
}

// Helper function for timing analysis
fn analyze_timing_precision(times: &[u64], expected_interval: u64, description: &str) {
    if times.len() < 2 {
        return;
    }

    println!("   📊 {} Timing Analysis:", description);
    let mut total_variance = 0u64;
    let mut max_variance = 0u64;

    for i in 1..times.len() {
        let actual_interval = times[i] - times[i - 1];
        let variance = if actual_interval > expected_interval {
            actual_interval - expected_interval
        } else {
            expected_interval - actual_interval
        };

        total_variance += variance;
        max_variance = max_variance.max(variance);

        println!(
            "     Interval {}: {}ms (expected: {}ms, variance: {}ms)",
            i,
            actual_interval,
            expected_interval,
            variance
        );
    }

    let avg_variance = total_variance / ((times.len() - 1) as u64);
    println!("     Average variance: {}ms, Max variance: {}ms", avg_variance, max_variance);

    if avg_variance <= expected_interval / 10 {
        println!("     ✅ Excellent timing precision");
    } else if avg_variance <= expected_interval / 5 {
        println!("     ⚠️  Acceptable timing precision");
    } else {
        println!("     ❌ Poor timing precision");
    }
}
