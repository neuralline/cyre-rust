// examples/advanced_demo.rs
use cyre_rust::*;
use serde_json::json;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Cyre Rust - Advanced Features Demo");
    println!("🚀 Using latest Tokio 1.45+ with enhanced async features\n");
    
    let cyre = Cyre::new();
    
    // Helper function for timestamps
    let timestamp = || SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // =================================================================
    // Demo 1: High-Performance API Rate Limiting
    // =================================================================
    println!("📡 Demo 1: High-Performance API Rate Limiting");
    
    cyre.action(IO {
        id: "api-request".to_string(),
        name: Some("API Request Handler".to_string()),
        throttle: Some(200), // Max 5 requests per second
        detect_changes: Some(true), // Skip duplicate requests
        ..Default::default()
    });
    
    cyre.on("api-request", |payload| async move {
        println!("🌐 API request processed: {}", payload);
        
        // Simulate API processing time
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        CyreResponse {
            ok: true,
            payload: json!({
                "response": "data",
                "processed_at": timestamp,
                "request": payload
            }),
            message: "API request completed".to_string(),
            error: None,
            timestamp: 0,
            metadata: None,
        }
    });
    
    // Burst of API calls - only some should execute due to throttling
    println!("Making burst of API calls...");
    for i in 1..=10 {
        let result = cyre.call("api-request", Some(json!({
            "endpoint": "/users",
            "method": "GET",
            "request_id": i
        }))).await;
        
        println!("  Request {}: {}", i, if result.ok { "✅ Executed" } else { "🛡️ Throttled" });
        
        // Small delay to see throttling in action
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    // =================================================================
    // Demo 2: Smart Search with Debouncing
    // =================================================================
    println!("\n🔍 Demo 2: Smart Search with Debouncing");
    
    cyre.action(IO {
        id: "smart-search".to_string(),
        name: Some("Smart Search Handler".to_string()),
        debounce: Some(250), // 250ms debounce
        ..Default::default()
    });
    
    cyre.on("smart-search", |payload| async move {
        let query = payload["query"].as_str().unwrap_or("").to_string();
        println!("🔍 Executing search for: '{}'", query);
        
        // Simulate search processing
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        CyreResponse {
            ok: true,
            payload: json!({
                "query": query,
                "results": [
                    format!("Result 1 for {}", query),
                    format!("Result 2 for {}", query),
                    format!("Result 3 for {}", query)
                ],
                "count": 3
            }),
            message: "Search completed".to_string(),
            error: None,
            timestamp: 0,
            metadata: None,
        }
    });
    
    // Simulate user typing - rapid search calls
    println!("Simulating user typing...");
    let search_terms = ["h", "he", "hel", "hell", "hello"];
    
    for term in search_terms {
        let result = cyre.call("smart-search", Some(json!({
            "query": term,
            "timestamp": timestamp()
        }))).await;
        
        println!("  Search for '{}': {}", term, if result.ok { "✅ Executed" } else { "⏳ Debounced" });
        tokio::time::sleep(Duration::from_millis(80)).await; // Typing speed
    }
    
    // Wait for final debounced search
    println!("Waiting for final debounced search...");
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    // =================================================================
    // Demo 3: State Management with Change Detection
    // =================================================================
    println!("\n🔄 Demo 3: State Management with Change Detection");
    
    cyre.action(IO {
        id: "user-state".to_string(),
        name: Some("User State Manager".to_string()),
        detect_changes: Some(true),
        ..Default::default()
    });
    
    cyre.on("user-state", |payload| async move {
        println!("📝 User state updated: {}", payload);
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        CyreResponse {
            ok: true,
            payload: json!({
                "state": payload,
                "updated_at": timestamp,
                "version": 1
            }),
            message: "State updated successfully".to_string(),
            error: None,
            timestamp: 0,
            metadata: None,
        }
    });
    
    // Test state changes
    println!("Testing state change detection...");
    
    let state1 = json!({"user_id": 123, "status": "online"});
    let state2 = json!({"user_id": 123, "status": "online"}); // Same
    let state3 = json!({"user_id": 123, "status": "away"});   // Different
    
    let result1 = cyre.call("user-state", Some(state1)).await;
    let result2 = cyre.call("user-state", Some(state2)).await;
    let result3 = cyre.call("user-state", Some(state3)).await;
    
    println!("  State update 1: {}", if result1.ok { "✅ Applied" } else { "❌ Failed" });
    println!("  State update 2: {}", if result2.ok { "✅ Applied" } else { "🔄 Skipped (no change)" });
    println!("  State update 3: {}", if result3.ok { "✅ Applied" } else { "❌ Failed" });
    
    // =================================================================
    // Demo 4: Performance Metrics & Monitoring
    // =================================================================
    println!("\n📊 Demo 4: Performance Metrics & Monitoring");
    
    let metrics = cyre.get_metrics();
    println!("Current system metrics:");
    for (key, value) in &metrics {
        match key.as_str() {
            "execution_count" => println!("  🏃 Total Executions: {}", value),
            "total_duration" => println!("  ⏱️  Total Duration: {}ms", value),
            "error_count" => println!("  ❌ Error Count: {}", value),
            "avg_duration" => println!("  📈 Average Duration: {}ms", value),
            _ => println!("  {} = {}", key, value),
        }
    }
    
    // Calculate performance stats
    let total_executions = metrics.get("execution_count").unwrap_or(&0);
    let avg_duration = metrics.get("avg_duration").unwrap_or(&0);
    let error_rate = metrics.get("error_count").unwrap_or(&0) as f64 / (*total_executions as f64 + 1.0) * 100.0;
    
    println!("\n🎯 Performance Analysis:");
    println!("  🚀 Execution Rate: {} ops", total_executions);
    println!("  ⚡ Average Latency: {}ms", avg_duration);
    println!("  🛡️ Error Rate: {:.2}%", error_rate);
    println!("  🎖️ Reliability: {:.1}%", 100.0 - error_rate);
    
    if *avg_duration < 10 {
        println!("  ✨ Performance: Excellent (sub-10ms latency)");
    } else if *avg_duration < 50 {
        println!("  ⚡ Performance: Good (sub-50ms latency)");
    } else {
        println!("  ⚠️ Performance: Consider optimization");
    }
    
    println!("\n🎉 Advanced demo completed!");
    println!("🚀 Cyre Rust demonstrating production-ready features");
    
    Ok(())
}