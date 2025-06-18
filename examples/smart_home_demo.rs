// examples/smart_home_demo.rs
// IoT Smart Home Demo - Showcasing Cyre's .action(), .on(), and .call() methods

use cyre_rust::prelude::*;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏠 CYRE SMART HOME DEMO");
    println!("=======================");
    println!("Experience the power of Cyre in IoT automation!");
    println!();

    // Initialize Cyre system
    let mut cyre = Cyre::new();

    // 🏠 Setup Smart Home System
    setup_smart_home_devices(&mut cyre).await;

    // 🎯 Run Interactive Demo Scenarios
    println!("🎮 Starting Interactive Demo Scenarios...");
    println!();

    // Scenario 1: Morning Routine
    morning_routine_demo(&cyre).await?;

    // Scenario 2: Security System
    security_system_demo(&cyre).await?;

    // Scenario 3: Energy Management
    energy_management_demo(&cyre).await?;

    // Scenario 4: Entertainment System
    entertainment_system_demo(&cyre).await?;

    // Scenario 5: Real-time Monitoring
    real_time_monitoring_demo(&cyre).await?;

    println!("🎉 Smart Home Demo Complete!");
    println!("✨ Cyre handled all IoT automation flawlessly!");

    Ok(())
}

//=============================================================================
// SMART HOME DEVICE SETUP
//=============================================================================

async fn setup_smart_home_devices(cyre: &mut Cyre) {
    println!("🔧 Setting up Smart Home Devices with Cyre...");

    // =====================================================
    // LIGHTING SYSTEM
    // =====================================================

    // Smart Light Control
    cyre.action(
        IO::new("light.control").with_name("Smart Light Controller").with_priority(Priority::High)
    );

    cyre.on("light.control", |payload| {
        Box::pin(async move {
            let room = payload
                .get("room")
                .and_then(|v| v.as_str())
                .unwrap_or("living_room");
            let action = payload
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("toggle");
            let brightness = payload
                .get("brightness")
                .and_then(|v| v.as_u64())
                .unwrap_or(100);

            println!("💡 {} Light: {} (brightness: {}%)", room, action, brightness);

            CyreResponse {
                ok: true,
                payload: json!({
                    "device": "smart_light",
                    "room": room,
                    "action": action,
                    "brightness": brightness,
                    "status": "success",
                    "power_consumption": brightness as f64 * 0.1
                }),
                message: format!("Light {} in {}", action, room),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"device_type": "lighting"})),
            }
        })
    });

    // Scene Control
    cyre.action(
        IO::new("scene.activate").with_name("Scene Controller").with_priority(Priority::Normal)
    );

    cyre.on("scene.activate", |payload| {
        Box::pin(async move {
            let scene = payload
                .get("scene")
                .and_then(|v| v.as_str())
                .unwrap_or("relaxing");

            println!("🎭 Activating Scene: {}", scene);

            let scene_config = match scene {
                "morning" =>
                    json!({
                    "lights": {"brightness": 80, "color": "warm_white"},
                    "blinds": "open",
                    "music": "energetic_playlist"
                }),
                "evening" =>
                    json!({
                    "lights": {"brightness": 30, "color": "warm_orange"},
                    "blinds": "closed",
                    "music": "relaxing_playlist"
                }),
                "party" =>
                    json!({
                    "lights": {"brightness": 100, "color": "dynamic"},
                    "blinds": "closed", 
                    "music": "party_playlist",
                    "volume": 80
                }),
                _ =>
                    json!({
                    "lights": {"brightness": 50, "color": "soft_white"},
                    "blinds": "auto",
                    "music": "ambient"
                }),
            };

            CyreResponse {
                ok: true,
                payload: json!({
                    "scene": scene,
                    "config": scene_config,
                    "activated_at": current_timestamp()
                }),
                message: format!("Scene '{}' activated", scene),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    });

    // =====================================================
    // CLIMATE CONTROL
    // =====================================================

    // Thermostat Control
    cyre.action(IO::new("climate.control").with_name("Smart Thermostat").with_throttle(2000)); // Prevent rapid changes

    cyre.on("climate.control", |payload| {
        Box::pin(async move {
            let target_temp = payload
                .get("temperature")
                .and_then(|v| v.as_f64())
                .unwrap_or(22.0);
            let mode = payload
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("auto");

            println!("🌡️  Climate: {}°C, Mode: {}", target_temp, mode);

            // Simulate energy calculation
            let current_temp = 20.0; // Simulated current temperature
            let energy_needed = (target_temp - current_temp).abs() * 10.0;

            CyreResponse {
                ok: true,
                payload: json!({
                    "device": "thermostat",
                    "target_temperature": target_temp,
                    "current_temperature": current_temp,
                    "mode": mode,
                    "energy_needed": energy_needed,
                    "estimated_time": format!("{}min", energy_needed as u32)
                }),
                message: format!("Climate set to {}°C", target_temp),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"device_type": "climate"})),
            }
        })
    });

    // =====================================================
    // SECURITY SYSTEM
    // =====================================================

    // Motion Detection
    cyre.action(IO::new("security.motion").with_name("Motion Detector").with_change_detection()); // Only trigger on actual changes

    cyre.on("security.motion", |payload| {
        Box::pin(async move {
            let location = payload
                .get("location")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let detected = payload
                .get("detected")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if detected {
                println!("🚨 Motion detected in: {}", location);
            }

            CyreResponse {
                ok: true,
                payload: json!({
                    "device": "motion_sensor",
                    "location": location,
                    "motion_detected": detected,
                    "timestamp": current_timestamp(),
                    "alert_level": if detected { "medium" } else { "none" }
                }),
                message: if detected {
                    format!("Motion detected in {}", location)
                } else {
                    "No motion".to_string()
                },
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"device_type": "security"})),
            }
        })
    });

    // Door Lock Control
    cyre.action(IO::new("security.lock").with_name("Smart Lock").with_priority(Priority::Critical)); // Security is critical

    cyre.on("security.lock", |payload| {
        Box::pin(async move {
            let action = payload
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("lock");
            let door = payload
                .get("door")
                .and_then(|v| v.as_str())
                .unwrap_or("front_door");
            let user = payload
                .get("user")
                .and_then(|v| v.as_str())
                .unwrap_or("system");

            println!("🔐 Door {}: {} (by: {})", door, action, user);

            CyreResponse {
                ok: true,
                payload: json!({
                    "device": "smart_lock",
                    "door": door,
                    "action": action,
                    "user": user,
                    "security_log": {
                        "timestamp": current_timestamp(),
                        "action": action,
                        "user": user
                    }
                }),
                message: format!("Door {} {}", door, action),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"device_type": "security", "priority": "critical"})),
            }
        })
    });

    // =====================================================
    // ENERGY MANAGEMENT
    // =====================================================

    // Energy Monitoring
    cyre.action(IO::new("energy.monitor").with_name("Energy Monitor").with_debounce(1000)); // Smooth out rapid readings

    cyre.on("energy.monitor", |payload| {
        Box::pin(async move {
            let device = payload
                .get("device")
                .and_then(|v| v.as_str())
                .unwrap_or("house");
            let consumption = payload
                .get("consumption")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            // Calculate cost (mock rate)
            let rate_per_kwh = 0.15; // $0.15 per kWh
            let hourly_cost = consumption * rate_per_kwh;

            println!("⚡ Energy: {} using {:.2}kW (${:.2}/hr)", device, consumption, hourly_cost);

            CyreResponse {
                ok: true,
                payload: json!({
                    "device": device,
                    "consumption_kw": consumption,
                    "hourly_cost": hourly_cost,
                    "daily_projection": hourly_cost * 24.0,
                    "efficiency_rating": if consumption < 2.0 { "excellent" } else if consumption < 5.0 { "good" } else { "high" }
                }),
                message: format!("Energy monitoring for {}", device),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"device_type": "energy"})),
            }
        })
    });

    // =====================================================
    // ENTERTAINMENT SYSTEM
    // =====================================================

    // Smart TV Control
    cyre.action(IO::new("entertainment.tv").with_name("Smart TV").with_priority(Priority::Low)); // Entertainment is lower priority

    cyre.on("entertainment.tv", |payload| {
        Box::pin(async move {
            let action = payload
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("status");
            let channel = payload.get("channel").and_then(|v| v.as_str());
            let volume = payload
                .get("volume")
                .and_then(|v| v.as_u64())
                .unwrap_or(50);

            println!("📺 TV: {} (volume: {})", action, volume);
            if let Some(ch) = channel {
                println!("   📡 Channel: {}", ch);
            }

            CyreResponse {
                ok: true,
                payload: json!({
                    "device": "smart_tv",
                    "action": action,
                    "channel": channel,
                    "volume": volume,
                    "status": "active"
                }),
                message: format!("TV {}", action),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"device_type": "entertainment"})),
            }
        })
    });

    // =====================================================
    // AUTOMATION RULES
    // =====================================================

    // Smart Home Automation
    cyre.action(
        IO::new("automation.rule").with_name("Automation Engine").with_priority(Priority::High)
    );

    cyre.on("automation.rule", |payload| {
        Box::pin(async move {
            let rule = payload
                .get("rule")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let trigger = payload
                .get("trigger")
                .and_then(|v| v.as_str())
                .unwrap_or("manual");

            println!("🤖 Automation Rule: {} (trigger: {})", rule, trigger);

            let actions = match rule {
                "bedtime" =>
                    vec![
                        "Turn off all lights",
                        "Lock all doors",
                        "Set thermostat to 18°C",
                        "Activate security system"
                    ],
                "welcome_home" =>
                    vec![
                        "Turn on entrance lights",
                        "Disarm security system",
                        "Set comfortable temperature",
                        "Play welcome music"
                    ],
                "energy_save" =>
                    vec![
                        "Dim unnecessary lights",
                        "Optimize thermostat",
                        "Turn off idle devices",
                        "Close automated blinds"
                    ],
                _ => vec!["No actions defined"],
            };

            CyreResponse {
                ok: true,
                payload: json!({
                    "rule": rule,
                    "trigger": trigger,
                    "actions": actions,
                    "executed_at": current_timestamp()
                }),
                message: format!("Automation rule '{}' executed", rule),
                error: None,
                timestamp: current_timestamp(),
                metadata: Some(json!({"device_type": "automation"})),
            }
        })
    });

    println!("✅ Smart Home setup complete! {} devices configured", 8);
    println!();
}

//=============================================================================
// DEMO SCENARIOS
//=============================================================================

async fn morning_routine_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌅 SCENARIO 1: Morning Routine");
    println!("===============================");

    // Simulate morning automation sequence
    println!("☀️  Starting morning routine...");

    // 1. Activate morning scene
    let scene_result = cyre.call("scene.activate", json!({
        "scene": "morning"
    })).await;

    if scene_result.ok {
        println!("✨ {}", scene_result.message);
    }

    sleep(Duration::from_millis(500)).await;

    // 2. Adjust climate
    let climate_result = cyre.call(
        "climate.control",
        json!({
        "temperature": 21.0,
        "mode": "heat"
    })
    ).await;

    if climate_result.ok {
        println!("✨ {}", climate_result.message);
    }

    sleep(Duration::from_millis(500)).await;

    // 3. Turn on kitchen lights
    let light_result = cyre.call(
        "light.control",
        json!({
        "room": "kitchen",
        "action": "on",
        "brightness": 90
    })
    ).await;

    if light_result.ok {
        println!("✨ {}", light_result.message);
    }

    sleep(Duration::from_millis(500)).await;

    // 4. Execute morning automation rule
    let automation_result = cyre.call(
        "automation.rule",
        json!({
        "rule": "welcome_home",
        "trigger": "morning_routine"
    })
    ).await;

    if automation_result.ok {
        println!("✨ {}", automation_result.message);
        let actions = automation_result.payload.get("actions").unwrap();
        println!("   📋 Actions: {:?}", actions);
    }

    println!("🎯 Morning routine completed successfully!");
    println!();

    Ok(())
}

async fn security_system_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️  SCENARIO 2: Security System");
    println!("===============================");

    // Simulate security events
    println!("🚨 Simulating security events...");

    // 1. Motion detection
    let motion_result = cyre.call(
        "security.motion",
        json!({
        "location": "front_door",
        "detected": true
    })
    ).await;

    if motion_result.ok {
        println!("🔍 {}", motion_result.message);
    }

    sleep(Duration::from_millis(300)).await;

    // 2. Smart lock operation
    let lock_result = cyre.call(
        "security.lock",
        json!({
        "door": "front_door",
        "action": "unlock",
        "user": "john_doe"
    })
    ).await;

    if lock_result.ok {
        println!("🔓 {}", lock_result.message);
    }

    sleep(Duration::from_millis(300)).await;

    // 3. Another motion detection (should be detected due to change detection)
    let motion_result2 = cyre.call(
        "security.motion",
        json!({
        "location": "living_room", 
        "detected": true
    })
    ).await;

    if motion_result2.ok {
        println!("🔍 {}", motion_result2.message);
    }

    sleep(Duration::from_millis(300)).await;

    // 4. Lock door again
    let lock_result2 = cyre.call(
        "security.lock",
        json!({
        "door": "front_door",
        "action": "lock",
        "user": "automatic"
    })
    ).await;

    if lock_result2.ok {
        println!("🔐 {}", lock_result2.message);
    }

    println!("🎯 Security system demo completed!");
    println!();

    Ok(())
}

async fn energy_management_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ SCENARIO 3: Energy Management");
    println!("===============================");

    // Monitor different devices
    let devices = vec![
        ("living_room_lights", 0.5),
        ("kitchen_appliances", 2.3),
        ("hvac_system", 3.1),
        ("entertainment_center", 0.8)
    ];

    println!("📊 Monitoring energy consumption...");

    for (device, consumption) in devices {
        let energy_result = cyre.call(
            "energy.monitor",
            json!({
            "device": device,
            "consumption": consumption
        })
        ).await;

        if energy_result.ok {
            let efficiency = energy_result.payload.get("efficiency_rating").unwrap();
            let cost = energy_result.payload.get("hourly_cost").unwrap();
            println!(
                "   📈 {}: {} efficiency (${:.2}/hr)",
                device,
                efficiency,
                cost.as_f64().unwrap()
            );
        }

        sleep(Duration::from_millis(200)).await;
    }

    // Trigger energy saving automation
    sleep(Duration::from_millis(500)).await;

    let automation_result = cyre.call(
        "automation.rule",
        json!({
        "rule": "energy_save",
        "trigger": "high_consumption_detected"
    })
    ).await;

    if automation_result.ok {
        println!("🤖 {}", automation_result.message);
    }

    println!("🎯 Energy management demo completed!");
    println!();

    Ok(())
}

async fn entertainment_system_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎬 SCENARIO 4: Entertainment System");
    println!("===================================");

    // Activate party scene first
    let scene_result = cyre.call("scene.activate", json!({
        "scene": "party"
    })).await;

    if scene_result.ok {
        println!("🎉 {}", scene_result.message);
    }

    sleep(Duration::from_millis(500)).await;

    // Control TV
    let tv_result = cyre.call(
        "entertainment.tv",
        json!({
        "action": "on",
        "channel": "Netflix",
        "volume": 75
    })
    ).await;

    if tv_result.ok {
        println!("📺 {}", tv_result.message);
    }

    sleep(Duration::from_millis(300)).await;

    // Adjust living room lights for movie mode
    let light_result = cyre.call(
        "light.control",
        json!({
        "room": "living_room",
        "action": "dim",
        "brightness": 20
    })
    ).await;

    if light_result.ok {
        println!("💡 {}", light_result.message);
    }

    println!("🎯 Entertainment system ready!");
    println!();

    Ok(())
}

async fn real_time_monitoring_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("📡 SCENARIO 5: Real-time Monitoring");
    println!("===================================");

    println!("🔄 Simulating real-time IoT data streams...");

    // Simulate multiple rapid sensor readings
    for i in 1..=5 {
        println!("   📊 Data burst #{}", i);

        // Temperature reading (with throttling)
        let temp_result = cyre.call(
            "climate.control",
            json!({
            "temperature": 20.0 + i as f64 * 0.5,
            "mode": "auto"
        })
        ).await;

        if temp_result.ok && i == 1 {
            println!("      🌡️ Temperature updated");
        } else if !temp_result.ok {
            println!("      ⏸️ Temperature update throttled");
        }

        // Energy reading (with debouncing)
        let energy_result = cyre.call(
            "energy.monitor",
            json!({
            "device": "sensor_array",
            "consumption": 1.0 + (i as f64 * 0.1)
        })
        ).await;

        if energy_result.ok {
            println!("      ⚡ Energy reading processed");
        }

        // Motion detection (with change detection)
        let motion_result = cyre.call(
            "security.motion",
            json!({
            "location": "sensor_grid",
            "detected": i % 2 == 1 // Alternating true/false
        })
        ).await;

        if motion_result.ok {
            println!("      🔍 Motion status: {}", if i % 2 == 1 { "detected" } else { "clear" });
        }

        sleep(Duration::from_millis(300)).await;
    }

    // Get system performance metrics
    let metrics = cyre.get_performance_metrics();

    println!("📈 Cyre Performance Summary:");
    println!("   • Total executions: {}", metrics["total_executions"]);
    println!("   • Fast path hits: {}", metrics["fast_path_hits"]);
    println!("   • Protection blocks: {}", metrics["protection_blocks"]);
    println!("   • Fast path ratio: {:.1}%", metrics["fast_path_ratio"]);

    println!("🎯 Real-time monitoring demo completed!");
    println!();

    Ok(())
}
