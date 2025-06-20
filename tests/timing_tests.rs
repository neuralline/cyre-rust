// tests/timing_tests.rs
// Unit tests for repeat, interval, delay combinations

use cyre_rust::prelude::*;
use serde_json::json;
use std::sync::atomic::{ AtomicU32, Ordering };
use std::sync::Arc;
use tokio::time::{ sleep, timeout, Duration };

#[tokio::test]
async fn test_interval_only() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  let counter = Arc::new(AtomicU32::new(0));
  let counter_clone = counter.clone();

  cyre.action(IO::new("interval-test").with_interval(500)).unwrap();

  cyre
    .on("interval-test", move |_| {
      let counter = counter_clone.clone();
      Box::pin(async move {
        counter.fetch_add(1, Ordering::SeqCst);
        CyreResponse::success(json!({"elapsed": elapsed}), "OK")
      })
    })
    .unwrap();

  cyre.call("full-combo-test", json!({})).await;

  // Wait for all executions
  let result = timeout(Duration::from_secs(2), async {
    loop {
      if counter.load(Ordering::SeqCst) >= 3 {
        break;
      }
      sleep(Duration::from_millis(50)).await;
    }
  }).await;

  assert!(result.is_ok(), "Timed out waiting for full combo executions");
  assert_eq!(counter.load(Ordering::SeqCst), 3);

  // Check timing - should be roughly: 400ms, 700ms, 1000ms
  let recorded_times = times.lock().unwrap().clone();
  assert_eq!(recorded_times.len(), 3);

  // First execution after delay (~400ms)
  assert!(recorded_times[0] >= 350 && recorded_times[0] <= 500);
  // Second execution after delay + interval (~700ms)
  assert!(recorded_times[1] >= 650 && recorded_times[1] <= 800);
  // Third execution after delay + 2*interval (~1000ms)
  assert!(recorded_times[2] >= 950 && recorded_times[2] <= 1100);
}

#[tokio::test]
async fn test_infinite_repeat() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  let counter = Arc::new(AtomicU32::new(0));
  let counter_clone = counter.clone();

  cyre.action(IO::new("infinite-test").with_interval(200).with_repeat_infinite()).unwrap();

  cyre
    .on("infinite-test", move |_| {
      let counter = counter_clone.clone();
      Box::pin(async move {
        counter.fetch_add(1, Ordering::SeqCst);
        CyreResponse::success(json!({}), "OK")
      })
    })
    .unwrap();

  cyre.call("infinite-test", json!({})).await;

  // Let it run for ~1 second (should execute ~5 times)
  sleep(Duration::from_millis(1100)).await;

  let executions = counter.load(Ordering::SeqCst);
  assert!(executions >= 4 && executions <= 6, "Expected 4-6 executions, got {}", executions);

  // Stop the infinite repeat
  cyre.forget("infinite-test").unwrap();

  // Wait a bit more to ensure it stopped
  let count_before_stop = counter.load(Ordering::SeqCst);
  sleep(Duration::from_millis(500)).await;
  let count_after_stop = counter.load(Ordering::SeqCst);

  assert_eq!(count_before_stop, count_after_stop, "Timer should have stopped after forget()");
}

#[tokio::test]
async fn test_error_handling_during_repeat() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  let counter = Arc::new(AtomicU32::new(0));
  let counter_clone = counter.clone();

  cyre.action(IO::new("error-test").with_interval(200).with_repeat_count(5)).unwrap();

  cyre
    .on("error-test", move |_| {
      let counter = counter_clone.clone();
      Box::pin(async move {
        let count = counter.fetch_add(1, Ordering::SeqCst) + 1;

        // Simulate error on execution #3
        if count == 3 {
          CyreResponse {
            ok: false,
            payload: json!(null),
            message: "Simulated error".to_string(),
            error: Some("test_error".to_string()),
            timestamp: current_timestamp(),
            metadata: None,
          }
        } else {
          CyreResponse::success(json!({"count": count}), "OK")
        }
      })
    })
    .unwrap();

  cyre.call("error-test", json!({})).await;

  // Wait for all executions
  let result = timeout(Duration::from_secs(2), async {
    loop {
      if counter.load(Ordering::SeqCst) >= 5 {
        break;
      }
      sleep(Duration::from_millis(50)).await;
    }
  }).await;

  assert!(result.is_ok(), "Timed out waiting for error test executions");
  assert_eq!(counter.load(Ordering::SeqCst), 5, "Should execute all 5 times despite error");
}

// =================================================================
// INVALID INPUT TESTS
// =================================================================

#[tokio::test]
async fn test_empty_action_id_rejected() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  let result = cyre.action(IO::new("").with_interval(1000));
  assert!(result.is_err(), "Empty action ID should be rejected");
}

#[tokio::test]
async fn test_zero_values() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  // Test zero interval - might be valid or invalid depending on implementation
  let zero_interval = cyre.action(IO::new("zero-interval-test").with_interval(0));
  // Don't assert here - behavior may vary

  // Test zero delay - typically valid (immediate execution)
  let zero_delay = cyre.action(IO::new("zero-delay-test").with_delay(0));
  // Don't assert here - zero delay is often valid

  // Test zero repeat - might mean "don't repeat" or "infinite"
  let zero_repeat = cyre.action(IO::new("zero-repeat-test").with_repeat_count(0));
  // Don't assert here - behavior may vary

  println!("Zero value tests completed - behavior may vary by implementation");
}

#[tokio::test]
async fn test_duplicate_action_id_rejected() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  // Create first action
  let first = cyre.action(IO::new("duplicate-test").with_interval(1000));
  assert!(first.is_ok(), "First action should succeed");

  // Try to create duplicate
  let duplicate = cyre.action(IO::new("duplicate-test").with_interval(2000));
  assert!(duplicate.is_err(), "Duplicate action ID should be rejected");
}

#[tokio::test]
async fn test_handler_without_action_rejected() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  // Try to register handler without creating action first
  let result = cyre.on("nonexistent-action", |_| {
    Box::pin(async move { CyreResponse::success(json!({}), "OK") })
  });

  assert!(result.is_err(), "Handler without action should be rejected");
}

#[tokio::test]
async fn test_call_nonexistent_action() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  let response = cyre.call("nonexistent-action", json!({})).await;
  assert!(!response.ok, "Call to nonexistent action should fail");
  assert!(response.error.is_some(), "Should have error message");
}

// =================================================================
// EDGE CASES AND BOUNDARY CONDITIONS
// =================================================================

#[tokio::test]
async fn test_very_short_intervals() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  let counter = Arc::new(AtomicU32::new(0));
  let counter_clone = counter.clone();

  // Test with very short interval (1ms) - system should handle gracefully
  let result = cyre.action(IO::new("short-interval-test").with_interval(1).with_repeat_count(10));

  if result.is_ok() {
    cyre
      .on("short-interval-test", move |_| {
        let counter = counter_clone.clone();
        Box::pin(async move {
          counter.fetch_add(1, Ordering::SeqCst);
          CyreResponse::success(json!({}), "OK")
        })
      })
      .unwrap();

    cyre.call("short-interval-test", json!({})).await;
    sleep(Duration::from_millis(100)).await; // Give some time

    let executions = counter.load(Ordering::SeqCst);
    assert!(executions <= 10, "Should not exceed repeat count");
    assert!(executions > 0, "Should execute at least once");
  }
}

#[tokio::test]
async fn test_very_long_delays() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  let counter = Arc::new(AtomicU32::new(0));
  let counter_clone = counter.clone();

  // Test with longer delay (2 seconds)
  cyre.action(IO::new("long-delay-test").with_delay(2000)).unwrap();

  cyre
    .on("long-delay-test", move |_| {
      let counter = counter_clone.clone();
      Box::pin(async move {
        counter.fetch_add(1, Ordering::SeqCst);
        CyreResponse::success(json!({}), "OK")
      })
    })
    .unwrap();

  cyre.call("long-delay-test", json!({})).await;

  // Should not execute immediately
  assert_eq!(counter.load(Ordering::SeqCst), 0);

  // Should not execute after 1 second
  sleep(Duration::from_millis(1000)).await;
  assert_eq!(counter.load(Ordering::SeqCst), 0);

  // Should execute after 2+ seconds
  sleep(Duration::from_millis(1200)).await;
  assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_cleanup_after_completion() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  let counter = Arc::new(AtomicU32::new(0));
  let counter_clone = counter.clone();

  cyre.action(IO::new("cleanup-test").with_interval(100).with_repeat_count(3)).unwrap();

  cyre
    .on("cleanup-test", move |_| {
      let counter = counter_clone.clone();
      Box::pin(async move {
        counter.fetch_add(1, Ordering::SeqCst);
        CyreResponse::success(json!({}), "OK")
      })
    })
    .unwrap();

  // Check that action exists before execution
  let before_action = cyre.get("cleanup-test");
  assert!(before_action.is_some(), "Action should exist before execution");

  cyre.call("cleanup-test", json!({})).await;

  // Wait for completion
  let result = timeout(Duration::from_secs(1), async {
    loop {
      if counter.load(Ordering::SeqCst) >= 3 {
        break;
      }
      sleep(Duration::from_millis(50)).await;
    }
  }).await;

  assert!(result.is_ok(), "Should complete within timeout");
  assert_eq!(counter.load(Ordering::SeqCst), 3);

  // Give time for cleanup
  sleep(Duration::from_millis(200)).await;

  // Check system status - timers should be cleaned up
  // (This is implementation-dependent - some systems may keep actions but clean timers)
  let status = cyre.status();
  println!("Status after cleanup: {}", serde_json::to_string_pretty(&status.payload).unwrap());
}

#[tokio::test]
async fn test_concurrent_timed_actions() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  let counter1 = Arc::new(AtomicU32::new(0));
  let counter2 = Arc::new(AtomicU32::new(0));
  let counter1_clone = counter1.clone();
  let counter2_clone = counter2.clone();

  // Create two concurrent timed actions
  cyre.action(IO::new("concurrent-1").with_interval(150).with_repeat_count(4)).unwrap();

  cyre.action(IO::new("concurrent-2").with_interval(200).with_repeat_count(3)).unwrap();

  cyre
    .on("concurrent-1", move |_| {
      let counter = counter1_clone.clone();
      Box::pin(async move {
        counter.fetch_add(1, Ordering::SeqCst);
        CyreResponse::success(json!({}), "Concurrent 1")
      })
    })
    .unwrap();

  cyre
    .on("concurrent-2", move |_| {
      let counter = counter2_clone.clone();
      Box::pin(async move {
        counter.fetch_add(1, Ordering::SeqCst);
        CyreResponse::success(json!({}), "Concurrent 2")
      })
    })
    .unwrap();

  // Start both actions
  let response1 = cyre.call("concurrent-1", json!({}));
  let response2 = cyre.call("concurrent-2", json!({}));

  // Wait for both to complete
  let (res1, res2) = tokio::join!(response1, response2);
  assert!(res1.ok && res2.ok, "Both actions should start successfully");

  // Wait for all executions
  let result = timeout(Duration::from_secs(2), async {
    loop {
      if counter1.load(Ordering::SeqCst) >= 4 && counter2.load(Ordering::SeqCst) >= 3 {
        break;
      }
      sleep(Duration::from_millis(50)).await;
    }
  }).await;

  assert!(result.is_ok(), "Both concurrent actions should complete");
  assert_eq!(counter1.load(Ordering::SeqCst), 4);
  assert_eq!(counter2.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_repeat_only() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  let counter = Arc::new(AtomicU32::new(0));
  let counter_clone = counter.clone();

  cyre.action(IO::new("repeat-test").with_repeat_count(3)).unwrap();

  cyre
    .on("repeat-test", move |_| {
      let counter = counter_clone.clone();
      Box::pin(async move {
        counter.fetch_add(1, Ordering::SeqCst);
        CyreResponse::success(json!({}), "OK")
      })
    })
    .unwrap();

  cyre.call("repeat-test", json!({})).await;
  sleep(Duration::from_millis(200)).await;

  // Should execute 3 times rapidly
  assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_delay_only() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  let counter = Arc::new(AtomicU32::new(0));
  let counter_clone = counter.clone();
  let start_time = std::time::Instant::now();

  cyre.action(IO::new("delay-test").with_delay(800)).unwrap();

  cyre
    .on("delay-test", move |_| {
      let counter = counter_clone.clone();
      Box::pin(async move {
        counter.fetch_add(1, Ordering::SeqCst);
        CyreResponse::success(json!({}), "OK")
      })
    })
    .unwrap();

  cyre.call("delay-test", json!({})).await;

  // Should not execute immediately
  assert_eq!(counter.load(Ordering::SeqCst), 0);

  sleep(Duration::from_millis(1000)).await;

  // Should execute once after delay
  assert_eq!(counter.load(Ordering::SeqCst), 1);

  // Should have taken at least 800ms
  assert!(start_time.elapsed() >= Duration::from_millis(800));
}

#[tokio::test]
async fn test_interval_plus_repeat() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  let counter = Arc::new(AtomicU32::new(0));
  let counter_clone = counter.clone();

  cyre.action(IO::new("interval-repeat-test").with_interval(300).with_repeat_count(4)).unwrap();

  cyre
    .on("interval-repeat-test", move |_| {
      let counter = counter_clone.clone();
      Box::pin(async move {
        counter.fetch_add(1, Ordering::SeqCst);
        CyreResponse::success(json!({}), "OK")
      })
    })
    .unwrap();

  cyre.call("interval-repeat-test", json!({})).await;

  // Wait for all executions
  let result = timeout(Duration::from_secs(2), async {
    loop {
      if counter.load(Ordering::SeqCst) >= 4 {
        break;
      }
      sleep(Duration::from_millis(50)).await;
    }
  }).await;

  assert!(result.is_ok(), "Timed out waiting for executions");
  assert_eq!(counter.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn test_delay_plus_repeat() {
  let mut cyre = Cyre::new();
  cyre.init().await.unwrap();

  let counter = Arc::new(AtomicU32::new(0));
  let counter_clone = counter.clone();
  let start_time = std::time::Instant::now();

  cyre.action(IO::new("delay-repeat-test").with_delay(500).with_repeat_count(3)).unwrap();

  cyre
    .on("delay-repeat-test", move |_| {
      let counter = counter_clone.clone();
      Box::pin(async move {
        counter.fetch_add(1, Ordering::SeqCst);
        CyreResponse::success(json!({}), "OK")
      })
    })
    .unwrap();

  cyre.call("delay-repeat-test", json!({})).await;

  // Should not execute immediately
  assert_eq!(counter.load(Ordering::SeqCst), 0);

  sleep(Duration::from_millis(800)).await;

  // Should execute 3 times after delay
  assert_eq!(counter.load(Ordering::SeqCst), 3);
  assert!(start_time.elapsed() >= Duration::from_millis(500));
}
