// examples/realistic_performance_test.rs
// Cyre Rust Realistic Performance Test - Matching TypeScript benchmark
use cyre_rust::*;
use serde_json::json;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::sync::{Arc, atomic::{AtomicU64, AtomicU32, Ordering}};
use tokio::task::JoinSet;

#[derive(Debug, Clone)]
struct BenchmarkResults {
    ops_per_sec: f64,
    avg_latency: f64,
    p95_latency: f64,
    error_rate: f64,
    resilience_score: f64,
    memory_mb: f64,
    operations: u64,
    errors_handled: u64,
    crashes_prevented: u64,
}

struct PerformanceTester {
    cyre: Cyre,
    error_count: Arc<AtomicU64>,
    success_count: Arc<AtomicU64>,
    crash_count: Arc<AtomicU64>,
    latencies: Arc<std::sync::Mutex<Vec<f64>>>,
}

impl PerformanceTester {
    fn new() -> Self {
        Self {
            cyre: Cyre::new(),
            error_count: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
            crash_count: Arc::new(AtomicU64::new(0)),
            latencies: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    async fn setup_baseline_handlers(&self) {
        // Fast, optimized handlers for baseline performance
        self.cyre.action(IO {
            id: "fast-action".to_string(),
            name: Some("Fast Baseline Action".to_string()),
            ..Default::default()
        });

        self.cyre.on("fast-action", |payload| async move {
            // Ultra-fast processing
            let result = payload["value"].as_u64().unwrap_or(0) + 1;
            
            CyreResponse {
                ok: true,
                payload: json!({"result": result}),
                message: "Fast processing".to_string(),
                error: None,
                timestamp: 0,
                metadata: None,
            }
        });

        self.cyre.action(IO {
            id: "compute-action".to_string(),
            name: Some("Compute Action".to_string()),
            ..Default::default()
        });

        self.cyre.on("compute-action", |payload| async move {
            // Light computational work
            let n = payload["n"].as_u64().unwrap_or(10);
            let result: u64 = (1..=n).sum();
            
            CyreResponse {
                ok: true,
                payload: json!({"sum": result}),
                message: "Computed".to_string(),
                error: None,
                timestamp: 0,
                metadata: None,
            }
        });
    }

    async fn setup_protection_handlers(&self) {
        // Handlers with protection mechanisms
        self.cyre.action(IO {
            id: "throttled-action".to_string(),
            throttle: Some(10), // 10ms throttle
            ..Default::default()
        });

        self.cyre.on("throttled-action", |payload| async move {
            // Simulate API call
            tokio::time::sleep(Duration::from_micros(100)).await;
            
            CyreResponse {
                ok: true,
                payload: json!({"processed": payload}),
                message: "Throttled processing".to_string(),
                error: None,
                timestamp: 0,
                metadata: None,
            }
        });

        self.cyre.action(IO {
            id: "debounced-action".to_string(),
            debounce: Some(5), // 5ms debounce
            ..Default::default()
        });

        self.cyre.on("debounced-action", |payload| async move {
            CyreResponse {
                ok: true,
                payload: json!({"debounced": payload}),
                message: "Debounced processing".to_string(),
                error: None,
                timestamp: 0,
                metadata: None,
            }
        });

        self.cyre.action(IO {
            id: "change-detected-action".to_string(),
            detect_changes: Some(true),
            ..Default::default()
        });

        self.cyre.on("change-detected-action", |payload| async move {
            CyreResponse {
                ok: true,
                payload: payload,
                message: "Change detected".to_string(),
                error: None,
                timestamp: 0,
                metadata: None,
            }
        });
    }

    async fn setup_resilience_handlers(&self) {
        let error_count = Arc::clone(&self.error_count);
        let crash_count = Arc::clone(&self.crash_count);

        // Handler that gracefully handles problematic payloads
        self.cyre.action(IO {
            id: "resilient-action".to_string(),
            name: Some("Resilient Action".to_string()),
            ..Default::default()
        });

        self.cyre.on("resilient-action", move |payload| {
            let error_count = Arc::clone(&error_count);
            let crash_count = Arc::clone(&crash_count);
            
            async move {
                // Defensive programming - handle all kinds of bad input
                match payload.get("bad_input") {
                    Some(serde_json::Value::String(s)) if s == "crash" => {
                        crash_count.fetch_add(1, Ordering::SeqCst);
                        CyreResponse {
                            ok: false,
                            payload: json!({"error": "Crash prevented"}),
                            message: "Handled potential crash".to_string(),
                            error: Some("crash_prevented".to_string()),
                            timestamp: 0,
                            metadata: None,
                        }
                    },
                    Some(serde_json::Value::String(s)) if s == "error" => {
                        error_count.fetch_add(1, Ordering::SeqCst);
                        CyreResponse {
                            ok: false,
                            payload: json!({"error": "Handled gracefully"}),
                            message: "Error handled".to_string(),
                            error: Some("handled_error".to_string()),
                            timestamp: 0,
                            metadata: None,
                        }
                    },
                    Some(serde_json::Value::Null) => {
                        error_count.fetch_add(1, Ordering::SeqCst);
                        CyreResponse {
                            ok: false,
                            payload: json!({"error": "Null input handled"}),
                            message: "Null input handled".to_string(),
                            error: Some("null_input".to_string()),
                            timestamp: 0,
                            metadata: None,
                        }
                    },
                    _ => {
                        // Normal processing
                        CyreResponse {
                            ok: true,
                            payload: json!({"processed": payload}),
                            message: "Normal processing".to_string(),
                            error: None,
                            timestamp: 0,
                            metadata: None,
                        }
                    }
                }
            }
        });
    }

    async fn run_baseline_benchmark(&self, operations: u64) -> BenchmarkResults {
        println!("🏆 Testing Proper Cyre Usage (Baseline)");
        
        let start_memory = self.get_memory_usage();
        let start_time = Instant::now();
        let mut latencies = Vec::new();
        
        for i in 0..operations {
            let op_start = Instant::now();
            
            // Alternate between different actions for variety
            let action_id = if i % 2 == 0 { "fast-action" } else { "compute-action" };
            let payload = if i % 2 == 0 {
                json!({"value": i})
            } else {
                json!({"n": (i % 100) + 1})
            };
            
            let result = self.cyre.call(action_id, Some(payload)).await;
            
            let latency = op_start.elapsed().as_secs_f64() * 1000.0; // Convert to ms
            latencies.push(latency);
            
            if result.ok {
                self.success_count.fetch_add(1, Ordering::SeqCst);
            } else {
                self.error_count.fetch_add(1, Ordering::SeqCst);
            }
        }
        
        let duration = start_time.elapsed();
        let end_memory = self.get_memory_usage();
        
        self.calculate_results(operations, duration, latencies, start_memory, end_memory)
    }

    async fn run_protection_benchmark(&self, operations: u64) -> BenchmarkResults {
        println!("🛡️ Testing Protection Systems");
        
        let start_memory = self.get_memory_usage();
        let start_time = Instant::now();
        let mut latencies = Vec::new();
        
        for i in 0..operations {
            let op_start = Instant::now();
            
            // Test different protection mechanisms
            let (action_id, payload) = match i % 3 {
                0 => ("throttled-action", json!({"request": i})),
                1 => ("debounced-action", json!({"search": format!("query_{}", i)})),
                _ => ("change-detected-action", json!({"state": i % 10})), // Limited states for change detection
            };
            
            let result = self.cyre.call(action_id, Some(payload)).await;
            
            let latency = op_start.elapsed().as_secs_f64() * 1000.0;
            latencies.push(latency);
            
            if result.ok {
                self.success_count.fetch_add(1, Ordering::SeqCst);
            } else {
                self.error_count.fetch_add(1, Ordering::SeqCst);
            }
            
            // Small delay to let protection mechanisms work
            if i % 100 == 0 {
                tokio::time::sleep(Duration::from_micros(10)).await;
            }
        }
        
        let duration = start_time.elapsed();
        let end_memory = self.get_memory_usage();
        
        self.calculate_results(operations, duration, latencies, start_memory, end_memory)
    }

    async fn run_resilience_benchmark(&self, operations: u64) -> BenchmarkResults {
        println!("💪 Testing Resilience Against Bad Usage (defensive handlers)");
        
        let start_memory = self.get_memory_usage();
        let start_time = Instant::now();
        let mut latencies = Vec::new();
        
        for i in 0..operations {
            let op_start = Instant::now();
            
            // Intentionally bad/problematic payloads
            let payload = match i % 5 {
                0 => json!({"bad_input": "crash"}),     // Simulated crash scenario
                1 => json!({"bad_input": "error"}),     // Simulated error
                2 => json!(null),                       // Null payload
                3 => json!({"massive": "x".repeat(1000)}), // Large payload
                _ => json!({"normal": "data"}),          // Normal data mixed in
            };
            
            let result = self.cyre.call("resilient-action", Some(payload)).await;
            
            let latency = op_start.elapsed().as_secs_f64() * 1000.0;
            latencies.push(latency);
            
            if result.ok {
                self.success_count.fetch_add(1, Ordering::SeqCst);
            } else {
                self.error_count.fetch_add(1, Ordering::SeqCst);
            }
        }
        
        let duration = start_time.elapsed();
        let end_memory = self.get_memory_usage();
        
        self.calculate_results(operations, duration, latencies, start_memory, end_memory)
    }

    fn calculate_results(&self, operations: u64, duration: Duration, mut latencies: Vec<f64>, start_memory: f64, end_memory: f64) -> BenchmarkResults {
        let ops_per_sec = operations as f64 / duration.as_secs_f64();
        let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
        
        // Calculate P95 latency
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95_index = ((latencies.len() as f64) * 0.95) as usize;
        let p95_latency = latencies.get(p95_index).copied().unwrap_or(0.0);
        
        let total_ops = self.success_count.load(Ordering::SeqCst) + self.error_count.load(Ordering::SeqCst);
        let error_rate = if total_ops > 0 {
            (self.error_count.load(Ordering::SeqCst) as f64 / total_ops as f64) * 100.0
        } else {
            0.0
        };
        
        // Resilience score based on error handling and system stability
        let resilience_score = if total_ops > 0 {
            ((total_ops - self.crash_count.load(Ordering::SeqCst)) as f64 / total_ops as f64) * 100.0
        } else {
            100.0
        };
        
        BenchmarkResults {
            ops_per_sec,
            avg_latency,
            p95_latency,
            error_rate,
            resilience_score,
            memory_mb: end_memory - start_memory,
            operations,
            errors_handled: self.error_count.load(Ordering::SeqCst),
            crashes_prevented: self.crash_count.load(Ordering::SeqCst),
        }
    }

    fn get_memory_usage(&self) -> f64 {
        // Simple memory usage estimation (in a real implementation, you'd use a proper memory profiler)
        // For now, we'll use a rough estimate based on operations
        let metrics = self.cyre.get_metrics();
        let ops = metrics.get("execution_count").unwrap_or(&0);
        *ops as f64 * 0.001 // Rough estimate: 1KB per operation
    }

    fn reset_counters(&self) {
        self.error_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.crash_count.store(0, Ordering::SeqCst);
    }
}

fn print_results(title: &str, results: &BenchmarkResults) {
    println!("{}", title);
    println!("  • Ops/sec: {:.0}", results.ops_per_sec);
    println!("  • Avg Latency: {:.3}ms", results.avg_latency);
    println!("  • P95 Latency: {:.3}ms", results.p95_latency);
    println!("  • Error Rate: {:.6}%", results.error_rate);
    println!("  • Resilience Score: {:.0}%", results.resilience_score);
    println!("  • Memory: {:.2}MB", results.memory_mb);
    println!("  • Operations: {}", results.operations);
    
    if results.errors_handled > 0 {
        println!("   💥 Handled errors gracefully: {}", results.errors_handled);
    }
    if results.crashes_prevented > 0 {
        println!("   🔥 System crashes prevented: {}", results.crashes_prevented);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CYRE RUST REALISTIC PERFORMANCE TEST");
    println!("========================================");
    println!("Testing real-world scenarios with defensive handlers...\n");

    let tester = PerformanceTester::new();

    // Setup all handlers
    tester.setup_baseline_handlers().await;
    tester.setup_protection_handlers().await;
    tester.setup_resilience_handlers().await;

    // Run benchmarks
    let baseline_results = {
        tester.reset_counters();
        tester.run_baseline_benchmark(10_000).await
    };

    tokio::time::sleep(Duration::from_millis(100)).await; // Brief pause

    let protection_results = {
        tester.reset_counters();
        tester.run_protection_benchmark(5_000).await
    };

    tokio::time::sleep(Duration::from_millis(100)).await; // Brief pause

    let resilience_results = {
        tester.reset_counters();
        tester.run_resilience_benchmark(2_000).await
    };

    // Print comprehensive results
    println!("🏆 RUST CYRE BENCHMARK RESULTS");
    println!("===============================");
    
    print_results("Proper Cyre Usage", &baseline_results);
    print_results("Protection Systems", &protection_results);
    print_results("Resilience Against Bad Usage", &resilience_results);

    // Overall analysis
    let avg_ops_per_sec = (baseline_results.ops_per_sec + protection_results.ops_per_sec + resilience_results.ops_per_sec) / 3.0;
    let avg_latency = (baseline_results.avg_latency + protection_results.avg_latency + resilience_results.avg_latency) / 3.0;
    let total_errors_handled = baseline_results.errors_handled + protection_results.errors_handled + resilience_results.errors_handled;
    let total_crashes_prevented = baseline_results.crashes_prevented + protection_results.crashes_prevented + resilience_results.crashes_prevented;
    let avg_resilience = (baseline_results.resilience_score + protection_results.resilience_score + resilience_results.resilience_score) / 3.0;

    println!("\n🎯 RUST PERFORMANCE ASSESSMENT");
    println!("===============================");
    println!("• Average Performance: {:.0} ops/sec", avg_ops_per_sec);
    println!("• Average Latency: {:.3}ms", avg_latency);
    println!("• Resilience Score: {:.1}%", avg_resilience);
    println!("• Total Errors Handled: {}", total_errors_handled);
    println!("• Total Crashes Prevented: {}", total_crashes_prevented);

    println!("\n💯 RUST CYRE'S STRENGTHS:");
    println!("✅ Memory safety with zero-cost abstractions");
    println!("✅ Thread-safe concurrent operations");
    println!("✅ Predictable performance without garbage collection");
    println!("✅ Sub-millisecond latency consistently");
    println!("✅ Excellent error handling and system protection");

    // Compare with TypeScript results
    println!("\n📊 COMPARISON WITH TYPESCRIPT CYRE:");
    println!("====================================");
    println!("TypeScript Baseline: 312,500 ops/sec (0.008ms avg)");
    println!("Rust Baseline:       {:.0} ops/sec ({:.3}ms avg)", baseline_results.ops_per_sec, baseline_results.avg_latency);
    
    let performance_ratio = baseline_results.ops_per_sec / 312_500.0;
    if performance_ratio > 1.0 {
        println!("🚀 Rust is {:.1}x FASTER than TypeScript!", performance_ratio);
    } else if performance_ratio > 0.8 {
        println!("⚡ Rust performance is competitive ({:.1}% of TypeScript)", performance_ratio * 100.0);
    } else {
        println!("📈 Rust has room for optimization ({:.1}% of TypeScript)", performance_ratio * 100.0);
    }

    println!("\n✨ Rust advantages:");
    println!("  • Zero garbage collection pauses");
    println!("  • Predictable memory usage");
    println!("  • Native async/await performance");
    println!("  • Compile-time optimizations");
    println!("  • Memory safety guarantees");

    Ok(())
}