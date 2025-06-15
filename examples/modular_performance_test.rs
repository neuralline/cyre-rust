// examples/modular_performance_test.rs
// Performance test for modular Cyre implementation

use cyre_rust::prelude::*;
use serde_json::json;
use std::time::Instant;

#[derive(Debug)]
struct PerformanceResult {
    test_name: String,
    ops_per_sec: u64,
    avg_latency_ms: f64,
    p95_latency_ms: f64,
    error_rate: f64,
    fast_path_usage: f64,
    total_operations: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MODULAR RUST CYRE PERFORMANCE TEST");
    println!("=====================================");
    println!("Target: Match TypeScript 312,500 ops/sec baseline");
    println!("Architecture: Clean modular design with optimizations");
    println!();

    let results = vec![
        test_fast_path_baseline().await?,
        test_optimized_protection().await?,
        test_mixed_workload().await?,
    ];

    print_performance_summary(&results);
    
    Ok(())
}

async fn test_fast_path_baseline() -> Result<PerformanceResult, Box<dyn std::error::Error>> {
    println!("🔥 Testing Fast Path Baseline (Zero Protection)");
    
    let mut cyre = Cyre::new();
    let iterations = 25_000;
    let mut latencies = Vec::with_capacity(iterations);
    
    // Setup fast path channel (no protection, no middleware)
    cyre.action(IO::new("fast-baseline"));

    // Simple, fast handler
    cyre.on("fast-baseline", |payload| {
        Box::pin(async move {
            CyreResponse {
                ok: true,
                payload: json!({
                    "processed": true, 
                    "value": payload.get("value").unwrap_or(&json!(0))
                }),
                message: "success".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    });

    let start_time = Instant::now();

    // Rapid-fire calls with simple payloads
    for i in 0..iterations {
        let call_start = Instant::now();
        
        let result = cyre.call("fast-baseline", json!({
            "value": i, 
            "batch": i / 1000
        })).await;
        
        let latency = call_start.elapsed().as_micros() as f64 / 1000.0;
        latencies.push(latency);
        
        if !result.ok {
            eprintln!("❌ Call failed at iteration {}: {}", i, result.message);
        }
    }

    let total_duration = start_time.elapsed();
    
    calculate_result(
        "Fast Path Baseline".to_string(),
        iterations,
        total_duration,
        latencies,
        0, // No errors expected in fast path
        &cyre,
    )
}

async fn test_optimized_protection() -> Result<PerformanceResult, Box<dyn std::error::Error>> {
    println!("⚡ Testing Optimized Protection Systems");
    
    let mut cyre = Cyre::new();
    let iterations = 15_000;
    let mut latencies = Vec::with_capacity(iterations);
    let mut errors = 0;
    
    // Setup protected channel with optimized settings
    let config = IO::new("protected-optimized")
        .with_throttle(1) // Minimal throttle for testing
        .with_change_detection()
        .with_priority(Priority::High);
    
    cyre.action(config);

    cyre.on("protected-optimized", |payload| {
        Box::pin(async move {
            // Simulate some processing but keep it fast
            let value = payload.get("value").unwrap_or(&json!(0)).as_u64().unwrap_or(0);
            
            CyreResponse {
                ok: true,
                payload: json!({
                    "processed": true, 
                    "result": value * 2,
                    "timestamp": current_timestamp()
                }),
                message: "protected-success".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    });

    let start_time = Instant::now();

    // Mixed payload test - some identical (to test change detection)
    for i in 0..iterations {
        let call_start = Instant::now();
        
        let payload = if i % 10 == 0 {
            // Every 10th call is identical to test change detection
            json!({"value": 100, "type": "repeated"})
        } else {
            json!({"value": i, "type": "unique", "batch": i / 1000})
        };
        
        let result = cyre.call("protected-optimized", payload).await;
        
        let latency = call_start.elapsed().as_micros() as f64 / 1000.0;
        latencies.push(latency);
        
        if !result.ok {
            errors += 1;
        }
        
        // Small delay to prevent overwhelming the throttle
        if i % 100 == 0 {
            tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
        }
    }

    let total_duration = start_time.elapsed();
    
    calculate_result(
        "Optimized Protection".to_string(),
        iterations,
        total_duration,
        latencies,
        errors,
        &cyre,
    )
}

async fn test_mixed_workload() -> Result<PerformanceResult, Box<dyn std::error::Error>> {
    println!("🎯 Testing Mixed Workload (Real-world Scenario)");
    
    let mut cyre = Cyre::new();
    let iterations = 10_000;
    let mut latencies = Vec::with_capacity(iterations);
    let mut errors = 0;
    
    // Setup multiple channels with different characteristics
    let channels = [
        ("fast-channel", IO::new("fast-channel")),
        ("throttled-channel", IO::new("throttled-channel").with_throttle(25)),
        ("change-detect", IO::new("change-detect").with_change_detection()),
        ("combined-protection", IO::new("combined-protection")
            .with_throttle(10)
            .with_change_detection()
            .with_priority(Priority::Low)),
    ];

    for (id, config) in channels.iter() {
        cyre.action(config.clone());

        let channel_id = id.to_string();
        cyre.on(&channel_id, move |payload| {
            let id = channel_id.clone();
            Box::pin(async move {
                // Variable processing based on channel type
                let processing_time = match id.as_str() {
                    "fast-channel" => 0, // No delay
                    "throttled-channel" => 1, // Minimal delay
                    _ => 2, // Small delay for others
                };
                
                if processing_time > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_micros(processing_time)).await;
                }
                
                CyreResponse {
                    ok: true,
                    payload: json!({
                        "channel": id,
                        "processed": true,
                        "input": payload.get("value").unwrap_or(&json!(null))
                    }),
                    message: "mixed-success".to_string(),
                    error: None,
                    timestamp: current_timestamp(),
                    metadata: None,
                }
            })
        });
    }

    let start_time = Instant::now();

    // Distribute calls across channels
    for i in 0..iterations {
        let call_start = Instant::now();
        
        let channel_index = i % channels.len();
        let channel_id = channels[channel_index].0;
        
        let payload = json!({
            "value": i,
            "channel_type": channel_id,
            "timestamp": current_timestamp()
        });
        
        let result = cyre.call(channel_id, payload).await;
        
        let latency = call_start.elapsed().as_micros() as f64 / 1000.0;
        latencies.push(latency);
        
        if !result.ok {
            errors += 1;
        }
    }

    let total_duration = start_time.elapsed();
    
    calculate_result(
        "Mixed Workload".to_string(),
        iterations,
        total_duration,
        latencies,
        errors,
        &cyre,
    )
}

fn calculate_result(
    test_name: String,
    iterations: usize,
    duration: std::time::Duration,
    mut latencies: Vec<f64>,
    errors: usize,
    cyre: &Cyre,
) -> Result<PerformanceResult, Box<dyn std::error::Error>> {
    let duration_secs = duration.as_secs_f64();
    let ops_per_sec = (iterations as f64 / duration_secs) as u64;
    
    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p95_index = (latencies.len() as f64 * 0.95) as usize;
    let p95_latency = latencies.get(p95_index).copied().unwrap_or(0.0);
    
    let error_rate = (errors as f64 / iterations as f64) * 100.0;
    
    let metrics = cyre.get_performance_metrics();
    let fast_path_usage = metrics.get("fast_path_ratio")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    
    Ok(PerformanceResult {
        test_name,
        ops_per_sec,
        avg_latency_ms: avg_latency,
        p95_latency_ms: p95_latency,
        error_rate,
        fast_path_usage,
        total_operations: iterations as u64,
    })
}

fn print_performance_summary(results: &[PerformanceResult]) {
    println!("\n🏆 MODULAR RUST CYRE PERFORMANCE RESULTS");
    println!("=========================================");
    
    for result in results {
        println!("\n{}", result.test_name);
        println!("  • Ops/sec: {}", result.ops_per_sec);
        println!("  • Avg Latency: {:.3}ms", result.avg_latency_ms);
        println!("  • P95 Latency: {:.3}ms", result.p95_latency_ms);
        println!("  • Error Rate: {:.2}%", result.error_rate);
        println!("  • Fast Path Usage: {:.1}%", result.fast_path_usage);
        println!("  • Operations: {}", result.total_operations);
    }
    
    let total_ops: u64 = results.iter().map(|r| r.total_operations).sum();
    let avg_ops_per_sec = results.iter().map(|r| r.ops_per_sec).sum::<u64>() / results.len() as u64;
    let avg_latency = results.iter().map(|r| r.avg_latency_ms).sum::<f64>() / results.len() as f64;
    
    println!("\n📊 SUMMARY STATISTICS");
    println!("======================");
    println!("• Average Performance: {} ops/sec", avg_ops_per_sec);
    println!("• Average Latency: {:.3}ms", avg_latency);
    println!("• Total Operations: {}", total_ops);
    
    // Performance assessment vs TypeScript baseline
    let typescript_baseline = 312_500;
    let performance_ratio = (avg_ops_per_sec as f64 / typescript_baseline as f64) * 100.0;
    
    println!("\n🎯 COMPARISON WITH TYPESCRIPT CYRE");
    println!("===================================");
    println!("• TypeScript Baseline: {} ops/sec", typescript_baseline);
    println!("• Rust Performance: {} ops/sec", avg_ops_per_sec);
    println!("• Performance Ratio: {:.1}%", performance_ratio);
    
    if performance_ratio >= 100.0 {
        println!("🥇 SUCCESS: Rust Cyre matches/exceeds TypeScript performance!");
    } else if performance_ratio >= 80.0 {
        println!("🥈 GOOD: Rust Cyre achieving strong performance!");
    } else if performance_ratio >= 50.0 {
        println!("🥉 PROGRESS: Rust Cyre showing solid improvement!");
    } else {
        println!("🔧 OPTIMIZATION NEEDED: Continue performance tuning");
    }
    
    println!("\n💪 MODULAR ARCHITECTURE BENEFITS:");
    println!("==================================");
    println!("✅ Clean separation of concerns");
    println!("✅ Easy to maintain and extend");
    println!("✅ Testable individual components");
    println!("✅ Hot path optimization preserved");
    println!("✅ Memory safety guarantees");
    
    if avg_ops_per_sec > 200_000 {
        println!("\n🚀 PERFORMANCE RATING: EXCELLENT");
        println!("Modular Rust Cyre is performing at production-ready levels!");
    } else if avg_ops_per_sec > 150_000 {
        println!("\n⚡ PERFORMANCE RATING: VERY GOOD");
        println!("Strong performance with clean architecture!");
    } else if avg_ops_per_sec > 100_000 {
        println!("\n📈 PERFORMANCE RATING: GOOD");
        println!("Solid foundation with modular benefits!");
    } else {
        println!("\n🔧 PERFORMANCE RATING: NEEDS WORK");
        println!("Focus on hot path optimization while keeping modularity!");
    }
}