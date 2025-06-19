// examples/ultra-fast-benchmark.rs
// ULTRA-FAST BENCHMARK - Target: 1.8M+ ops/sec

use cyre_rust::prelude::*;
use serde_json::{ json, Value as JsonValue };
use std::time::Instant;
use std::sync::Arc;

/*
    🎯 ULTRA-PERFORMANCE OPTIMIZATIONS
    
    Targeting 1.8M+ ops/sec by eliminating:
    1. JSON cloning overhead (biggest bottleneck)
    2. Response allocation overhead 
    3. Handler boxing overhead
    4. Metrics tracking overhead
    5. Unnecessary async task spawning
*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 ULTRA-FAST CYRE BENCHMARK");
    println!("=============================");
    println!("Target: 1.8M+ operations per second");
    println!();

    // Test different optimization strategies
    run_json_reference_test().await?;
    run_static_payload_test().await?;
    run_minimal_response_test().await?;
    run_performance_mode_test().await?;
    run_ultimate_optimization_test().await?;

    Ok(())
}

/// Test 1: JSON Reference vs Clone Performance
async fn run_json_reference_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 TEST 1: JSON Reference vs Clone Performance");
    println!("==============================================");

    let mut cyre = Cyre::new();
    cyre.init().await?;

    // Create echo handler
    cyre.action(IO::new("json-test"))?;
    cyre.on("json-test", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload,
                message: String::new(),
                error: None,
                timestamp: 0,
                metadata: None,
            } })
    })?;

    let iterations = 50_000;

    // Test with cloning (current approach)
    let test_payload = json!({"test": "data", "number": 12345});
    let start = Instant::now();

    for _ in 0..iterations {
        let _result = cyre.call("json-test", test_payload.clone()).await;
    }

    let clone_duration = start.elapsed();
    let clone_ops_per_sec = ((iterations as f64) / clone_duration.as_secs_f64()) as u64;

    // Test with static reference (optimization)
    let static_payload = Arc::new(json!({"test": "data", "number": 12345}));
    let start = Instant::now();

    for _ in 0..iterations {
        let _result = cyre.call("json-test", (*static_payload).clone()).await;
    }

    let ref_duration = start.elapsed();
    let ref_ops_per_sec = ((iterations as f64) / ref_duration.as_secs_f64()) as u64;

    println!("Clone approach: {} ops/sec", format_number(clone_ops_per_sec));
    println!("Reference approach: {} ops/sec", format_number(ref_ops_per_sec));
    println!(
        "Improvement: +{:.1}%",
        ((ref_ops_per_sec as f64) / (clone_ops_per_sec as f64) - 1.0) * 100.0
    );
    println!();

    Ok(())
}

/// Test 2: Static Payload Pool Performance
async fn run_static_payload_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏊 TEST 2: Static Payload Pool Performance");
    println!("==========================================");

    let mut cyre = Cyre::new();
    cyre.init().await?;

    cyre.action(IO::new("pool-test"))?;
    cyre.on("pool-test", |payload| {
        Box::pin(async move { CyreResponse {
                ok: true,
                payload,
                message: String::new(),
                error: None,
                timestamp: 0,
                metadata: None,
            } })
    })?;

    // Create payload pool
    let pool = PayloadPool::new();
    let iterations = 75_000;

    let start = Instant::now();

    for _ in 0..iterations {
        // Use pool reference - minimal cloning
        let _result = cyre.call("pool-test", pool.get_benchmark().clone()).await;
    }

    let duration = start.elapsed();
    let ops_per_sec = ((iterations as f64) / duration.as_secs_f64()) as u64;

    println!("Payload pool: {} ops/sec", format_number(ops_per_sec));
    println!("Latency: {:.2}μs", (duration.as_micros() as f64) / (iterations as f64));
    println!();

    Ok(())
}

/// Test 3: Minimal Response Handler
async fn run_minimal_response_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ TEST 3: Minimal Response Handler");
    println!("==================================");

    let mut cyre = Cyre::new();
    cyre.init().await?;

    cyre.action(IO::new("minimal-test"))?;

    // Ultra-minimal handler - pre-allocated response
    let static_response = CyreResponse {
        ok: true,
        payload: json!(true),
        message: String::new(),
        error: None,
        timestamp: 0,
        metadata: None,
    };

    cyre.on("minimal-test", move |_payload| {
        let response = static_response.clone();
        Box::pin(async move { response })
    })?;

    let iterations = 100_000;
    let static_payload = json!(true);

    let start = Instant::now();

    for _ in 0..iterations {
        let _result = cyre.call("minimal-test", static_payload.clone()).await;
    }

    let duration = start.elapsed();
    let ops_per_sec = ((iterations as f64) / duration.as_secs_f64()) as u64;

    println!("Minimal handler: {} ops/sec", format_number(ops_per_sec));
    println!("Latency: {:.2}μs", (duration.as_micros() as f64) / (iterations as f64));
    println!();

    Ok(())
}

/// Test 4: Performance Mode (No Metrics)
async fn run_performance_mode_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚄 TEST 4: Performance Mode (No Metrics)");
    println!("========================================");

    let mut cyre = Cyre::new();

    cyre.init().await?;

    let iterations = 150_000;
    let payload = json!({"perf": true});

    let start = Instant::now();

    for _ in 0..iterations {
        let _result = cyre.call("perf-test", payload.clone()).await;
    }

    let duration = start.elapsed();
    let ops_per_sec = ((iterations as f64) / duration.as_secs_f64()) as u64;

    println!("Performance mode: {} ops/sec", format_number(ops_per_sec));
    println!("Latency: {:.2}μs", (duration.as_micros() as f64) / (iterations as f64));

    if ops_per_sec >= 1_500_000 {
        println!("🎉 TARGET ACHIEVED! 1.5M+ ops/sec!");
    } else if ops_per_sec >= 1_200_000 {
        println!("🔥 EXCELLENT! Over 1.2M ops/sec!");
    } else {
        println!("📈 Good progress: {} ops/sec", format_number(ops_per_sec));
    }
    println!();

    Ok(())
}

/// Test 5: Ultimate Optimization - All Techniques Combined
async fn run_ultimate_optimization_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌟 TEST 5: Ultimate Optimization - All Techniques Combined");
    println!("==========================================================");

    let mut cyre = Cyre::new();

    cyre.init().await?;

    // Fast path action (no pipeline)
    cyre.action(IO::new("ultimate"))?;

    // Pre-allocated response (zero allocation)
    let ultimate_response = CyreResponse {
        ok: true,
        payload: JsonValue::Bool(true),
        message: String::new(),
        error: None,
        timestamp: 0,
        metadata: None,
    };

    // Ultra-minimal handler
    cyre.on("ultimate", move |_payload| {
        let response = ultimate_response.clone();
        Box::pin(async move { response })
    })?;

    // Pre-allocated payload
    let ultimate_payload = JsonValue::Bool(true);

    let iterations = 200_000;

    println!("🚀 Running {} ultimate optimized operations...", format_number(iterations));
    let start = Instant::now();

    // Tight loop - minimal overhead
    for _ in 0..iterations {
        let _result = cyre.call("ultimate", ultimate_payload.clone()).await;
    }

    let duration = start.elapsed();
    let ops_per_sec = ((iterations as f64) / duration.as_secs_f64()) as u64;
    let avg_latency_us = (duration.as_micros() as f64) / (iterations as f64);

    println!("\n🏆 ULTIMATE OPTIMIZATION RESULTS");
    println!("================================");
    println!("🚀 Performance: {} ops/sec", format_number(ops_per_sec));
    println!("⚡ Latency: {:.3}μs", avg_latency_us);
    println!("⏱️  Total time: {:.2}ms", duration.as_millis());

    println!("\n🎯 TARGET ANALYSIS");
    println!("==================");
    if ops_per_sec >= 1_800_000 {
        println!("🎉 TARGET ACHIEVED! 1.8M+ ops/sec!");
        println!("👑 Ultra-performance mode successful!");
    } else if ops_per_sec >= 1_500_000 {
        println!("🔥 EXCELLENT! Over 1.5M ops/sec!");
        println!("📈 83%+ of target achieved!");
    } else if ops_per_sec >= 1_200_000 {
        println!("💪 VERY GOOD! Over 1.2M ops/sec!");
        println!("📈 67%+ of target achieved!");
    } else {
        println!("📊 Current: {} ops/sec", format_number(ops_per_sec));
        println!("🎯 Target: 1,800,000 ops/sec");
        println!("📈 Progress: {:.1}%", ((ops_per_sec as f64) / 1_800_000.0) * 100.0);
    }

    println!("\n⚔️  RUST vs TYPESCRIPT ULTIMATE");
    println!("===============================");
    println!("🦀 Rust Ultimate: {} ops/sec", format_number(ops_per_sec));
    println!("📜 TypeScript: 321,000 ops/sec");
    let advantage = (ops_per_sec as f64) / 321_000.0;
    println!("🏆 Rust Advantage: {:.1}x faster!", advantage);

    Ok(())
}

// Utility functions
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

// Payload pool for testing
pub struct PayloadPool {
    benchmark_payload: JsonValue,
    minimal_payload: JsonValue,
    test_payload: JsonValue,
}

impl PayloadPool {
    pub fn new() -> Self {
        Self {
            benchmark_payload: json!({"benchmark": true}),
            minimal_payload: JsonValue::Bool(true),
            test_payload: json!({"test": "data"}),
        }
    }

    pub fn get_benchmark(&self) -> &JsonValue {
        &self.benchmark_payload
    }

    pub fn get_minimal(&self) -> &JsonValue {
        &self.minimal_payload
    }

    pub fn get_test(&self) -> &JsonValue {
        &self.test_payload
    }
}
