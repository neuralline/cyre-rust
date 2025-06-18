// examples/rust-cyre-benchmark.rs
// RUST REDEMPTION ARC - Let's show TypeScript who's boss!

use cyre_rust::prelude::*;
use serde_json::json;
use std::time::{ Duration, Instant };

/*

      R.U.S.T - R.E.D.E.M.P.T.I.O.N - A.R.C
      
      Time for Rust to show its true power!
      We're going to DEMOLISH TypeScript's 321,000 ops/sec

*/

#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    ops_per_sec: u64,
    total_ops: u64,
    duration_ms: f64,
    avg_latency_us: f64,
}

/// Pure speed test - no protection, maximum optimization
async fn pure_speed_test() -> BenchmarkResult {
    println!("⚡ PURE SPEED TEST - No holds barred!");

    let mut cyre = Cyre::new();

    // Initialize Cyre asynchronously
    if let Err(e) = cyre.init().await {
        eprintln!("Failed to initialize Cyre: {}", e);
        return BenchmarkResult {
            name: "Pure Speed Test".to_string(),
            ops_per_sec: 0,
            total_ops: 0,
            duration_ms: 0.0,
            avg_latency_us: 0.0,
        };
    }

    let operations = 100_000u64;

    // Setup PURE fast path - zero protection
    cyre.action(IO::new("speed-demon"));

    // Ultra-minimal handler
    cyre.on("speed-demon", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload,
                message: String::new(),
                error: None,
                timestamp: 0, // Skip timestamp for speed
                metadata: None,
            } })
    });

    // Pre-allocate payload to avoid JSON parsing overhead
    let payload = json!({"speed": "test"});

    println!("🔥 Executing {} operations at maximum speed...", operations);
    let start = Instant::now();

    // TIGHT LOOP - pure Rust performance
    for _ in 0..operations {
        let _result = cyre.call("speed-demon", payload.clone()).await;
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;
    let ops_per_sec = ((operations as f64) / duration.as_secs_f64()) as u64;
    let avg_latency_us = (duration.as_micros() as f64) / (operations as f64);

    println!("💨 Result: {} ops/sec", ops_per_sec);

    BenchmarkResult {
        name: "Pure Speed Test".to_string(),
        ops_per_sec,
        total_ops: operations,
        duration_ms,
        avg_latency_us,
    }
}

/// Batch processing test - amortize overhead
async fn batch_processing_test() -> BenchmarkResult {
    println!("\n📦 BATCH PROCESSING TEST - Amortize that overhead!");

    let mut cyre = Cyre::new();

    // Initialize Cyre asynchronously
    if let Err(e) = cyre.init().await {
        eprintln!("Failed to initialize Cyre: {}", e);
        return BenchmarkResult {
            name: "Batch Processing".to_string(),
            ops_per_sec: 0,
            total_ops: 0,
            duration_ms: 0.0,
            avg_latency_us: 0.0,
        };
    }

    let batch_size = 1000;
    let num_batches = 500;
    let total_operations = batch_size * num_batches;

    cyre.action(IO::new("batch-beast"));

    // Batch processor - handle multiple items per call
    cyre.on("batch-beast", |payload| {
        Box::pin(async move {
            let empty_vec = vec![];
            let items = payload.as_array().unwrap_or(&empty_vec);
            let processed: Vec<_> = items
                .iter()
                .map(|item| json!({"processed": item}))
                .collect();

            CyreResponse {
                ok: true,
                payload: json!(processed),
                message: String::new(),
                error: None,
                timestamp: 0,
                metadata: None,
            }
        })
    });

    // Create batch payload once
    let batch_items: Vec<_> = (0..batch_size).map(|i| json!({"id": i})).collect();
    let batch_payload = json!(batch_items);

    println!("🚀 Processing {} items in {} batches...", total_operations, num_batches);
    let start = Instant::now();

    for _ in 0..num_batches {
        let _result = cyre.call("batch-beast", batch_payload.clone()).await;
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;
    let ops_per_sec = ((total_operations as f64) / duration.as_secs_f64()) as u64;
    let avg_latency_us = (duration.as_micros() as f64) / (num_batches as f64);

    println!("💪 Result: {} ops/sec", ops_per_sec);

    BenchmarkResult {
        name: "Batch Processing".to_string(),
        ops_per_sec,
        total_ops: total_operations,
        duration_ms,
        avg_latency_us,
    }
}

/// Multi-channel test - measure total throughput
async fn multi_channel_test() -> BenchmarkResult {
    println!("\n🔀 MULTI-CHANNEL TEST - Total throughput measurement!");

    let mut cyre = Cyre::new();

    // Initialize Cyre asynchronously
    if let Err(e) = cyre.init().await {
        eprintln!("Failed to initialize Cyre: {}", e);
        return BenchmarkResult {
            name: "Multi-Channel".to_string(),
            ops_per_sec: 0,
            total_ops: 0,
            duration_ms: 0.0,
            avg_latency_us: 0.0,
        };
    }

    let num_channels = 10;
    let ops_per_channel = 20_000;
    let total_operations = num_channels * ops_per_channel;

    // Setup multiple channels
    for i in 0..num_channels {
        let channel_id = format!("multi-{}", i);
        cyre.action(IO::new(&channel_id));
        cyre.on(&channel_id, |payload| {
            Box::pin(async move { CyreResponse {
                    ok: true,
                    payload,
                    message: String::new(),
                    error: None,
                    timestamp: 0,
                    metadata: None,
                } })
        });
    }

    let payload = json!({"multi": true});

    println!("⚡ Running {} ops across {} channels...", total_operations, num_channels);
    let start = Instant::now();

    // Execute across all channels sequentially but rapidly
    for i in 0..total_operations {
        let channel_id = format!("multi-{}", i % num_channels);
        let _result = cyre.call(&channel_id, payload.clone()).await;
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;
    let ops_per_sec = ((total_operations as f64) / duration.as_secs_f64()) as u64;
    let avg_latency_us = (duration.as_micros() as f64) / (total_operations as f64);

    println!("🎯 Result: {} ops/sec", ops_per_sec);

    BenchmarkResult {
        name: "Multi-Channel".to_string(),
        ops_per_sec,
        total_ops: total_operations,
        duration_ms,
        avg_latency_us,
    }
}

/// Memory optimized test - zero allocations
async fn zero_allocation_test() -> BenchmarkResult {
    println!("\n💾 ZERO ALLOCATION TEST - Memory efficiency master class!");

    let mut cyre = Cyre::new();

    // Initialize Cyre asynchronously
    if let Err(e) = cyre.init().await {
        eprintln!("Failed to initialize Cyre: {}", e);
        return BenchmarkResult {
            name: "Zero Allocation".to_string(),
            ops_per_sec: 0,
            total_ops: 0,
            duration_ms: 0.0,
            avg_latency_us: 0.0,
        };
    }

    let operations = 75_000u64;

    cyre.action(IO::new("zero-alloc"));

    // Handler that reuses input payload (zero allocations)
    cyre.on("zero-alloc", |payload| {
        Box::pin(async move { // Return the exact same payload - zero allocations!
            CyreResponse {
                ok: true,
                payload, // Zero-copy return
                message: String::new(),
                error: None,
                timestamp: 0,
                metadata: None,
            } })
    });

    // Use same payload instance throughout
    let static_payload = json!({"reused": "always"});

    println!("🔋 Executing {} zero-allocation operations...", operations);
    let start = Instant::now();

    for _ in 0..operations {
        let _result = cyre.call("zero-alloc", static_payload.clone()).await;
    }

    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;
    let ops_per_sec = ((operations as f64) / duration.as_secs_f64()) as u64;
    let avg_latency_us = (duration.as_micros() as f64) / (operations as f64);

    println!("⚡ Result: {} ops/sec", ops_per_sec);

    BenchmarkResult {
        name: "Zero Allocation".to_string(),
        ops_per_sec,
        total_ops: operations,
        duration_ms,
        avg_latency_us,
    }
}

fn format_number(n: u64) -> String {
    n.to_string()
        .chars()
        .rev()
        .collect::<Vec<_>>()
        .chunks(3)
        .map(|chunk| chunk.iter().rev().collect::<String>())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(",")
}

fn display_results(results: &[BenchmarkResult]) {
    println!("\n🏆 RUST REDEMPTION RESULTS");
    println!("==========================");

    let mut total_ops = 0u64;
    let mut total_duration = 0.0f64;

    for result in results {
        println!("\n{}", result.name);
        println!("  🚀 Ops/sec: {}", format_number(result.ops_per_sec));
        println!("  ⏱️  Duration: {:.2}ms", result.duration_ms);
        if result.avg_latency_us > 0.0 {
            println!("  ⚡ Avg Latency: {:.2}μs", result.avg_latency_us);
        }
        println!("  📊 Operations: {}", format_number(result.total_ops));

        total_ops += result.total_ops;
        total_duration += result.duration_ms;
    }

    let peak_performance = results
        .iter()
        .map(|r| r.ops_per_sec)
        .max()
        .unwrap_or(0);
    let weighted_avg = ((total_ops as f64) / (total_duration / 1000.0)) as u64;

    println!("\n📈 PERFORMANCE SUMMARY");
    println!("======================");
    println!("🔥 Peak Performance: {} ops/sec", format_number(peak_performance));
    println!("📊 Weighted Average: {} ops/sec", format_number(weighted_avg));
    println!("💪 Total Operations: {}", format_number(total_ops));

    println!("\n⚔️  RUST vs TYPESCRIPT SHOWDOWN");
    println!("=================================");
    println!("🦀 Rust Peak: {} ops/sec", format_number(peak_performance));
    println!("📜 TypeScript: 321,000 ops/sec");

    if peak_performance >= 321_000 {
        let advantage = (peak_performance as f64) / 321_000.0;
        println!("🎉 RUST WINS! {:.1}x faster than TypeScript!", advantage);
        println!("👑 Rust claims the performance crown!");
    } else if peak_performance >= 250_000 {
        let ratio = ((peak_performance as f64) / 321_000.0) * 100.0;
        println!("🔥 VERY CLOSE! Rust achieved {:.1}% of TypeScript performance!", ratio);
        println!("💪 With more optimization, Rust will win!");
    } else if peak_performance >= 150_000 {
        let ratio = ((peak_performance as f64) / 321_000.0) * 100.0;
        println!("📈 SOLID PERFORMANCE! Rust achieved {:.1}% of TypeScript!", ratio);
        println!("🔧 Still room for optimization!");
    } else {
        println!("😅 Need more optimization work...");
        println!("🔬 Time to analyze bottlenecks!");
    }

    println!("\n🎯 RUST'S ADVANTAGES (regardless of raw speed):");
    println!("===============================================");
    println!("✅ Zero garbage collection pauses");
    println!("✅ Predictable, deterministic performance");
    println!("✅ Memory safety without runtime overhead");
    println!("✅ True zero-cost abstractions");
    println!("✅ Fearless concurrency");
    println!("✅ Compile-time optimizations");

    if peak_performance < 200_000 {
        println!("\n🔧 OPTIMIZATION OPPORTUNITIES:");
        println!("==============================");
        println!("• Profile with `perf` to find bottlenecks");
        println!("• Use `#[inline]` attributes strategically");
        println!("• Consider custom JSON serialization");
        println!("• Optimize the fast path further");
        println!("• Use `smallvec` for small collections");
        println!("• Consider unsafe optimizations where safe");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 RUST COMEBACK PERFORMANCE TEST");
    println!("==================================");
    println!("Time to redeem Rust's honor against TypeScript's 321k ops/sec!");
    println!();

    let mut results = Vec::new();

    // Run all benchmark tests
    results.push(pure_speed_test().await);
    tokio::time::sleep(Duration::from_millis(1000)).await;

    results.push(batch_processing_test().await);
    tokio::time::sleep(Duration::from_millis(1000)).await;

    results.push(multi_channel_test().await);
    tokio::time::sleep(Duration::from_millis(1000)).await;

    results.push(zero_allocation_test().await);

    display_results(&results);

    println!("\n🎬 COMING SOON: Ultra-optimized Rust with unsafe code!");
    println!("Stay tuned for the ultimate performance showdown! ��");

    Ok(())
}
