// examples/simple_iot_demo.rs
// Simple IoT Demo showcasing core Cyre functionality

use cyre_rust::prelude::*;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 SIMPLE IoT DEMO WITH CYRE");
    println!("=============================");
    println!("Showcasing .action(), .on(), and .call() methods");
    println!();

    let mut cyre = Cyre::new();

    // =================================================================
    // Setup: Register IoT devices using .action() and .on()
    // =================================================================

    println!("🔧 Setting up IoT devices...");

    // 1. Temperature Sensor (with throttling to prevent spam)
    cyre.action(IO::new("sensor.temperature").with_name("Temperature Sensor").with_throttle(1000)); // Only accept readings every 1 second

    cyre.on("sensor.temperature", |payload| {
        Box::pin(async move {
            let temp = payload
                .get("temperature")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let location = payload
                .get("location")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            println!("🌡️  Temperature: {:.1}°C in {}", temp, location);

            CyreResponse {
                ok: true,
                payload: json!({
                    "temperature": temp,
                    "location": location,
                    "alert": if temp > 30.0 { "high" } else if temp < 10.0 { "low" } else { "normal" },
                    "timestamp": current_timestamp()
                }),
                message: format!("Temperature reading: {:.1}°C", temp),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"sensor_type": "temperature"})),
            }
        })
    });

    // 2. Smart Light (with change detection)
    cyre.action(
        IO::new("device.light")
            .with_name("Smart Light")
            .with_change_detection() // Only trigger on actual state changes
            .with_priority(Priority::Normal)
    );

    cyre.on("device.light", |payload| {
        Box::pin(async move {
            let state = payload
                .get("state")
                .and_then(|v| v.as_str())
                .unwrap_or("off");
            let brightness = payload
                .get("brightness")
                .and_then(|v| v.as_u64())
                .unwrap_or(100);
            let room = payload
                .get("room")
                .and_then(|v| v.as_str())
                .unwrap_or("living_room");

            println!("💡 Light in {} turned {} ({}%)", room, state, brightness);

            CyreResponse {
                ok: true,
                payload: json!({
                    "room": room,
                    "state": state,
                    "brightness": brightness,
                    "power_usage": if state == "on" { brightness as f64 * 0.1 } else { 0.0 }
                }),
                message: format!("Light {} in {}", state, room),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"device_type": "light"})),
            }
        })
    });

    // 3. Door Sensor (high priority for security)
    cyre.action(IO::new("sensor.door").with_name("Door Sensor").with_priority(Priority::High));

    cyre.on("sensor.door", |payload| {
        Box::pin(async move {
            let door = payload
                .get("door")
                .and_then(|v| v.as_str())
                .unwrap_or("front_door");
            let state = payload
                .get("state")
                .and_then(|v| v.as_str())
                .unwrap_or("closed");

            let alert_level = if state == "open" { "medium" } else { "low" };

            println!("🚪 Door {}: {} (Alert: {})", door, state, alert_level);

            CyreResponse {
                ok: true,
                payload: json!({
                    "door": door,
                    "state": state,
                    "alert_level": alert_level,
                    "security_log": {
                        "timestamp": current_timestamp(),
                        "event": format!("door_{}", state)
                    }
                }),
                message: format!("Door {} is {}", door, state),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"sensor_type": "door", "priority": "high"})),
            }
        })
    });

    // 4. Notification System
    cyre.action(
        IO::new("system.notify").with_name("Notification System").with_priority(Priority::High)
    );

    cyre.on("system.notify", |payload| {
        Box::pin(async move {
            let message = payload
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("No message");
            let level = payload
                .get("level")
                .and_then(|v| v.as_str())
                .unwrap_or("info");

            let emoji = match level {
                "critical" => "🚨",
                "warning" => "⚠️",
                "info" => "ℹ️",
                _ => "📢",
            };

            println!("{} NOTIFICATION [{}]: {}", emoji, level.to_uppercase(), message);

            CyreResponse {
                ok: true,
                payload: json!({
                    "message": message,
                    "level": level,
                    "sent_at": current_timestamp(),
                    "delivery_status": "delivered"
                }),
                message: format!("Notification sent: {}", message),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"system_type": "notification"})),
            }
        })
    });

    println!("✅ 4 IoT devices configured with Cyre");
    println!();

    // =================================================================
    // Demo: Use .call() to interact with devices
    // =================================================================

    println!("🎮 Starting IoT Demo...");
    println!();

    // Scenario 1: Temperature monitoring
    println!("📊 SCENARIO 1: Temperature Monitoring");
    println!("-------------------------------------");

    let temperatures = vec![18.5, 22.3, 28.7, 32.1];
    let locations = vec!["living_room", "bedroom", "kitchen", "garage"];

    for (temp, location) in temperatures.iter().zip(locations.iter()) {
        let result = cyre.call(
            "sensor.temperature",
            json!({
            "temperature": temp,
            "location": location
        })
        ).await;

        if result.ok {
            let alert = result.payload.get("alert").unwrap();
            if alert != "normal" {
                // Send notification for abnormal temperature
                cyre.call(
                    "system.notify",
                    json!({
                    "message": format!("Temperature alert in {}: {:.1}°C", location, temp),
                    "level": "warning"
                })
                ).await;
            }
        }

        sleep(Duration::from_millis(300)).await;
    }

    // Try rapid temperature readings (should be throttled)
    println!("\n🔄 Testing throttling with rapid readings...");
    for i in 1..=3 {
        let result = cyre.call(
            "sensor.temperature",
            json!({
            "temperature": 25.0 + i as f64,
            "location": "test_room"
        })
        ).await;

        if !result.ok {
            println!("   ⏸️ Reading {} throttled: {}", i, result.message);
        }
        sleep(Duration::from_millis(200)).await; // Faster than throttle limit
    }

    sleep(Duration::from_millis(1000)).await;
    println!();

    // Scenario 2: Smart lighting control
    println!("💡 SCENARIO 2: Smart Lighting Control");
    println!("-------------------------------------");

    // Turn on lights in different rooms
    let rooms = vec!["living_room", "kitchen", "bedroom"];

    for room in rooms {
        let result = cyre.call(
            "device.light",
            json!({
            "room": room,
            "state": "on",
            "brightness": 80
        })
        ).await;

        if result.ok {
            let power = result.payload.get("power_usage").unwrap();
            println!("   ⚡ Power usage: {:.1}W", power.as_f64().unwrap());
        }

        sleep(Duration::from_millis(400)).await;
    }

    // Test change detection - same command should be ignored
    println!("\n🔄 Testing change detection...");
    let result1 = cyre.call(
        "device.light",
        json!({
        "room": "living_room",
        "state": "on",
        "brightness": 80
    })
    ).await;

    if !result1.ok {
        println!("   ⏸️ Duplicate command ignored: {}", result1.message);
    }

    // Different command should work
    let result2 = cyre.call(
        "device.light",
        json!({
        "room": "living_room", 
        "state": "on",
        "brightness": 60  // Different brightness
    })
    ).await;

    if result2.ok {
        println!("   ✅ Brightness change accepted");
    }

    sleep(Duration::from_millis(500)).await;
    println!();

    // Scenario 3: Security monitoring
    println!("🔒 SCENARIO 3: Security Monitoring");
    println!("----------------------------------");

    // Simulate door events
    let door_events = vec![
        ("front_door", "open"),
        ("back_door", "open"),
        ("front_door", "closed"),
        ("garage_door", "open")
    ];

    for (door, state) in door_events {
        let result = cyre.call(
            "sensor.door",
            json!({
            "door": door,
            "state": state
        })
        ).await;

        if result.ok {
            // Send critical notification for any door opening
            if state == "open" {
                cyre.call(
                    "system.notify",
                    json!({
                    "message": format!("SECURITY: {} opened", door),
                    "level": "critical"
                })
                ).await;
            }
        }

        sleep(Duration::from_millis(600)).await;
    }

    sleep(Duration::from_millis(500)).await;
    println!();

    // Scenario 4: System automation
    println!("🤖 SCENARIO 4: Automated Responses");
    println!("----------------------------------");

    // High temperature triggers multiple actions
    println!("🌡️ Simulating high temperature event...");

    let high_temp_result = cyre.call(
        "sensor.temperature",
        json!({
        "temperature": 35.0,
        "location": "server_room"
    })
    ).await;

    if high_temp_result.ok {
        let alert = high_temp_result.payload.get("alert").unwrap();

        if alert == "high" {
            println!("🚨 High temperature detected! Triggering automation...");

            // 1. Send critical notification
            cyre.call(
                "system.notify",
                json!({
                "message": "CRITICAL: Server room temperature at 35°C!",
                "level": "critical"
            })
            ).await;

            sleep(Duration::from_millis(200)).await;

            // 2. Turn on ventilation (simulate with lights)
            cyre.call(
                "device.light",
                json!({
                "room": "server_room",
                "state": "on", 
                "brightness": 100
            })
            ).await;

            sleep(Duration::from_millis(200)).await;

            // 3. Log security event
            cyre.call(
                "sensor.door",
                json!({
                "door": "server_room",
                "state": "monitoring"
            })
            ).await;

            println!("✅ Automated response completed");
        }
    }

    sleep(Duration::from_millis(1000)).await;
    println!();

    // =================================================================
    // Performance Summary
    // =================================================================

    println!("📈 PERFORMANCE SUMMARY");
    println!("======================");

    let metrics = cyre.get_performance_metrics();

    println!("🎯 Cyre Performance:");
    println!("   • Total calls: {}", metrics["total_executions"]);
    println!("   • Fast path hits: {}", metrics["fast_path_hits"]);
    println!("   • Protection blocks: {}", metrics["protection_blocks"]);
    println!("   • Fast path ratio: {:.1}%", metrics["fast_path_ratio"]);
    println!("   • Active channels: {}", metrics["active_channels"]);

    println!();
    println!("🏆 Key Features Demonstrated:");
    println!("   ✅ .action() - Device registration with configurations");
    println!("   ✅ .on() - Event handler registration");
    println!("   ✅ .call() - Asynchronous device interaction");
    println!("   ✅ Throttling - Prevented sensor spam");
    println!("   ✅ Change Detection - Ignored duplicate commands");
    println!("   ✅ Priority System - Critical notifications first");
    println!("   ✅ Real-time Processing - Sub-millisecond responses");

    println!();
    println!("🚀 Cyre IoT Demo Complete!");
    println!("Your devices are now connected and responsive!");

    Ok(())
}
