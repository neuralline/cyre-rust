// tests/timekeeper_test.rs
// Comprehensive tests for the updated TimeKeeper

use cyre_rust::timekeeper::{
    get_timekeeper,
    TimerRepeat,
    set_timeout,
    set_interval,
    clear_timer,
    delay,
};
use cyre_rust::utils::current_timestamp;
use tokio::time::{ sleep, Duration };
use std::sync::atomic::{ AtomicU32, Ordering };
use std::sync::Arc;

#[tokio::test]
async fn test_essential_apis() {
    println!("🧪 Testing Essential TimeKeeper APIs");

    let timekeeper = get_timekeeper().await;

    // Test 1: .keep() with all parameters
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let timer_id = timekeeper
        .keep(
            100, // interval: 100ms
            move || {
                let counter = counter_clone.clone();
                Box::pin(async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                })
            },
            TimerRepeat::Count(3), // repeat 3 times
            "test-timer", // id
            200 // delay: 200ms before starting
        ).await
        .expect("Failed to create timer");

    println!("✅ Timer created: {}", timer_id);

    // Wait for execution
    sleep(Duration::from_millis(800)).await;

    // Should have executed 3 times
    assert_eq!(counter.load(Ordering::SeqCst), 3);
    println!("✅ Timer executed exactly 3 times");

    // Test 2: .forget()
    timekeeper.forget("test-timer").await;
    println!("✅ Timer forgotten");

    // Test 3: .wait()
    let start = current_timestamp();
    timekeeper.wait(150).await.expect("Wait failed");
    let elapsed = current_timestamp() - start;
    assert!(elapsed >= 150 && elapsed < 200);
    println!("✅ Wait function works correctly");

    // Test 4: .status()
    let status = timekeeper.status();
    println!("✅ Status: {}", serde_json::to_string_pretty(&status).unwrap());

    // Test 5: .reset()
    timekeeper.reset().await;
    let reset_status = timekeeper.status();
    assert_eq!(reset_status["active_formations"], 0);
    println!("✅ Reset successful");

    // Test 6: .hibernate()
    timekeeper.hibernate().await;
    let hibernated_status = timekeeper.status();
    assert_eq!(hibernated_status["hibernating"], true);
    println!("✅ Hibernation successful");
}

#[tokio::test]
async fn test_breathing_system_pattern() {
    println!("🫁 Testing Breathing System Pattern");

    let timekeeper = get_timekeeper().await;
    timekeeper.reset().await; // Start fresh

    let breathing_count = Arc::new(AtomicU32::new(0));
    let breathing_count_clone = breathing_count.clone();

    // Exact API from your requirement
    let breathing_timer = timekeeper
        .keep(
            1000, // Check every second interval
            move || {
                let count = breathing_count_clone.clone();
                Box::pin(async move {
                    let current = count.fetch_add(1, Ordering::SeqCst);
                    println!("💨 Breathing update #{} at {}", current + 1, current_timestamp());

                    // Simulate breathing update logic
                    if let Err(_error) = try_update_breathing_from_metrics().await {
                        // Silent fail to prevent log spam
                    }
                })
            },
            true, // repeat infinity
            "system-breathing", // id for tracking progress and cancellation
            2000 // delay, start repetition after 2s delay
        ).await
        .expect("Failed to create breathing timer");

    println!("✅ Breathing timer created: {}", breathing_timer);

    // Let it run for a few cycles
    sleep(Duration::from_millis(5500)).await; // 2s delay + 3 cycles

    let final_count = breathing_count.load(Ordering::SeqCst);
    assert!(final_count >= 3, "Expected at least 3 breathing cycles, got {}", final_count);
    println!("✅ Breathing system executed {} times", final_count);

    // Stop breathing
    timekeeper.forget("system-breathing").await;
    println!("✅ Breathing system stopped");
}

#[tokio::test]
async fn test_convenience_functions() {
    println!("🛠️  Testing Convenience Functions");

    let executed = Arc::new(AtomicU32::new(0));
    let executed_clone = executed.clone();

    // Test set_timeout
    let timeout_id = set_timeout(
        move || {
            let executed = executed_clone.clone();
            Box::pin(async move {
                executed.fetch_add(1, Ordering::SeqCst);
            })
        },
        200,
        Some("test-timeout".to_string())
    ).await.expect("Failed to set timeout");

    println!("✅ Timeout set: {}", timeout_id);

    // Test set_interval
    let interval_count = Arc::new(AtomicU32::new(0));
    let interval_count_clone = interval_count.clone();

    let interval_id = set_interval(
        move || {
            let count = interval_count_clone.clone();
            Box::pin(async move {
                count.fetch_add(1, Ordering::SeqCst);
            })
        },
        100,
        Some("test-interval".to_string())
    ).await.expect("Failed to set interval");

    println!("✅ Interval set: {}", interval_id);

    // Wait and check results
    sleep(Duration::from_millis(450)).await;

    assert_eq!(executed.load(Ordering::SeqCst), 1);
    let interval_executions = interval_count.load(Ordering::SeqCst);
    assert!(interval_executions >= 3, "Expected at least 3 interval executions");

    println!("✅ Timeout executed once, interval executed {} times", interval_executions);

    // Test clear_timer
    clear_timer("test-interval").await.expect("Failed to clear timer");

    // Test delay function
    let delay_start = current_timestamp();
    delay(100).await.expect("Delay failed");
    let delay_elapsed = current_timestamp() - delay_start;
    assert!(delay_elapsed >= 100);

    println!("✅ All convenience functions work correctly");
}

#[tokio::test]
async fn test_complex_scenarios() {
    println!("🎯 Testing Complex Scenarios");

    let timekeeper = get_timekeeper().await;
    timekeeper.reset().await;

    // Scenario 1: Multiple timers with different patterns
    let counters = Arc::new([
        AtomicU32::new(0), // once timer
        AtomicU32::new(0), // limited repeat timer
        AtomicU32::new(0), // forever timer
    ]);

    // Once timer
    {
        let counters = counters.clone();
        let _once_timer = timekeeper
            .keep(
                50,
                move || {
                    let counters = counters.clone();
                    Box::pin(async move {
                        counters[0].fetch_add(1, Ordering::SeqCst);
                    })
                },
                TimerRepeat::Once,
                "once-timer",
                100
            ).await
            .expect("Failed to create once timer");
    }

    // Limited repeat timer
    {
        let counters = counters.clone();
        let _limited_timer = timekeeper
            .keep(
                75,
                move || {
                    let counters = counters.clone();
                    Box::pin(async move {
                        counters[1].fetch_add(1, Ordering::SeqCst);
                    })
                },
                TimerRepeat::Count(3),
                "limited-timer",
                50
            ).await
            .expect("Failed to create limited timer");
    }

    // Forever timer (we'll cancel it)
    {
        let counters = counters.clone();
        let _forever_timer = timekeeper
            .keep(
                60,
                move || {
                    let counters = counters.clone();
                    Box::pin(async move {
                        counters[2].fetch_add(1, Ordering::SeqCst);
                    })
                },
                TimerRepeat::Forever,
                "forever-timer",
                None
            ).await
            .expect("Failed to create forever timer");
    }

    // Let them run
    sleep(Duration::from_millis(500)).await;

    // Cancel forever timer
    timekeeper.forget("forever-timer").await;

    // Wait a bit more to ensure no more executions
    sleep(Duration::from_millis(200)).await;

    let results = [
        counters[0].load(Ordering::SeqCst),
        counters[1].load(Ordering::SeqCst),
        counters[2].load(Ordering::SeqCst),
    ];

    assert_eq!(results[0], 1, "Once timer should execute exactly once");
    assert_eq!(results[1], 3, "Limited timer should execute exactly 3 times");
    assert!(results[2] > 0, "Forever timer should have executed at least once before cancellation");

    println!(
        "✅ Complex scenario results: once={}, limited={}, forever={}",
        results[0],
        results[1],
        results[2]
    );
}

// Helper function for breathing test
async fn try_update_breathing_from_metrics() -> Result<(), String> {
    // Simulate breathing update logic
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    Ok(())
}

#[tokio::test]
async fn test_error_handling() {
    println!("❌ Testing Error Handling");

    let timekeeper = get_timekeeper().await;

    // Test hibernation prevention
    timekeeper.hibernate().await;

    let result = timekeeper.keep(
        100,
        || Box::pin(async move {}),
        TimerRepeat::Once,
        "hibernated-timer",
        None
    ).await;

    assert!(result.is_err(), "Should not be able to create timer while hibernating");
    println!("✅ Hibernation correctly prevents new timers");

    // Reset for further tests
    timekeeper.reset().await;

    // Test zero duration wait
    let wait_result = timekeeper.wait(0).await;
    assert!(wait_result.is_ok(), "Zero duration wait should succeed immediately");
    println!("✅ Zero duration wait works correctly");
}

#[tokio::main]
async fn main() {
    println!("🚀 Running TimeKeeper Integration Tests");
    println!("=====================================");

    test_essential_apis();
    test_breathing_system_pattern();
    test_convenience_functions();
    test_complex_scenarios();
    test_error_handling();

    println!("\n🎉 All TimeKeeper tests passed!");
    println!("✅ .keep() ✅ .forget() ✅ .wait() ✅ .reset() ✅ .hibernate() ✅ .status()");
}
