// src/main.rs
use cyre_rust::*;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Cyre Rust - Starting Demo (Latest Version)");
    
    // Create Cyre instance
    let cyre = Cyre::new();
    
    // 1. Register action
    let result = cyre.action(IO {
        id: "user-login".to_string(),
        name: Some("User Login".to_string()),
        payload: Some(json!({"status": "idle"})),
        throttle: Some(1000), // 1 second throttle
        ..Default::default()
    });
    
    if result.ok {
        println!("✅ Action registered: {}", result.message);
    }
    
    // 2. Subscribe to channel
    let sub_result = cyre.on("user-login", |payload| async move {
        println!("🔥 User login handler triggered with: {}", payload);
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        CyreResponse {
            ok: true,
            payload: json!({
                "success": true,
                "timestamp": timestamp,
                "user_data": payload
            }),
            message: "Login processed successfully".to_string(),
            error: None,
            timestamp: 0,
            metadata: None,
        }
    });
    
    if sub_result.ok {
        println!("✅ Subscribed: {}", sub_result.message);
    }
    
    // 3. Call the action
    println!("\n📡 Calling user-login...");
    let call_result = cyre.call("user-login", Some(json!({
        "userId": 123,
        "email": "user@example.com"
    }))).await;
    
    if call_result.ok {
        println!("✅ Call successful: {}", call_result.message);
        println!("📦 Response payload: {}", call_result.payload);
    } else {
        println!("❌ Call failed: {}", call_result.message);
    }
    
    // 4. Test throttle protection
    println!("\n🛡️ Testing throttle protection...");
    let throttled_result = cyre.call("user-login", Some(json!({
        "userId": 456,
        "email": "another@example.com"
    }))).await;
    
    if !throttled_result.ok {
        println!("✅ Throttle working: {}", throttled_result.message);
    } else {
        println!("⚠️ Throttle might not be working as expected");
    }
    
    // 5. Test debounce functionality
    println!("\n🎯 Testing debounce protection...");
    
    // Register debounced action
    cyre.action(IO {
        id: "search-input".to_string(),
        name: Some("Search Input".to_string()),
        debounce: Some(300), // 300ms debounce
        ..Default::default()
    });
    
    cyre.on("search-input", |payload| async move {
        println!("🔍 Search executed with: {}", payload);
        CyreResponse {
            ok: true,
            payload: json!({"results": ["result1", "result2"]}),
            message: "Search completed".to_string(),
            error: None,
            timestamp: 0,
            metadata: None,
        }
    });
    
    // Rapid calls - should be debounced
    println!("Making rapid search calls...");
    let search1 = cyre.call("search-input", Some(json!({"term": "a"}))).await;
    let search2 = cyre.call("search-input", Some(json!({"term": "ab"}))).await;
    let search3 = cyre.call("search-input", Some(json!({"term": "abc"}))).await;
    
    println!("Search call 1 result: ok={}", search1.ok);
    println!("Search call 2 result: ok={}", search2.ok);
    println!("Search call 3 result: ok={}", search3.ok);
    
    // Wait for debounce to complete
    println!("Waiting for debounce timer...");
    tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
    
    // 6. Test change detection
    println!("\n🔄 Testing change detection...");
    
    cyre.action(IO {
        id: "state-update".to_string(),
        detect_changes: Some(true),
        ..Default::default()
    });
    
    cyre.on("state-update", |payload| async move {
        println!("📝 State changed to: {}", payload);
        CyreResponse {
            ok: true,
            payload: payload,
            message: "State updated".to_string(),
            error: None,
            timestamp: 0,
            metadata: None,
        }
    });
    
    // Test change detection
    let change1 = cyre.call("state-update", Some(json!({"value": 1}))).await;
    let change2 = cyre.call("state-update", Some(json!({"value": 1}))).await; // Same - should skip
    let change3 = cyre.call("state-update", Some(json!({"value": 2}))).await; // Different - should execute
    
    println!("Change call 1 result: ok={}", change1.ok);
    println!("Change call 2 result: ok={} (should be false - no change)", change2.ok);
    println!("Change call 3 result: ok={}", change3.ok);
    
    // 7. Show metrics
    println!("\n📊 Performance Metrics:");
    let metrics = cyre.get_metrics();
    for (key, value) in metrics {
        println!("  {}: {}", key, value);
    }
    
    println!("\n🎉 Cyre Rust demo completed!");
    println!("🚀 Using Tokio {} with latest async features", "1.45+");
    
    Ok(())
}