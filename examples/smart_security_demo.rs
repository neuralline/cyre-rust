// examples/smart_security_demo.rs
// Smart Home Security System Demo
// Real-world application of Cyre for home automation and security

use cyre_rust::prelude::*;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏠 CYRE SMART HOME SECURITY SYSTEM");
    println!("===================================");
    println!("Real-world home security automation with Cyre");
    println!("Features: Motion detection, door sensors, cameras, automation");
    println!();

    // Initialize Cyre system
    let mut cyre = Cyre::new();
    let init_result = cyre.init().await?;

    println!("✅ Security System initialized successfully");
    println!("   Instance ID: {}", init_result.payload["instance_id"]);
    println!("   System Status: {}", init_result.payload["breathing"]);
    println!();

    // Setup security system
    setup_security_system(&mut cyre).await?;

    // Run security scenarios
    println!("🚀 Starting Security Scenarios...");
    println!();

    // Scenario 1: Normal Day - Family Activities
    normal_day_scenario(&cyre).await?;

    // Scenario 2: Unauthorized Access Detection
    unauthorized_access_scenario(&cyre).await?;

    // Scenario 3: Fire/Smoke Detection
    fire_detection_scenario(&cyre).await?;

    // Scenario 4: Power Outage & Backup Systems
    power_outage_scenario(&cyre).await?;

    // Scenario 5: Vacation Mode
    vacation_mode_scenario(&cyre).await?;

    // Scenario 6: System Health Monitoring
    system_health_scenario(&cyre).await?;

    println!("🎉 Security System Demo Complete!");
    println!("✨ Cyre handled all security events with intelligent automation!");

    Ok(())
}

//=============================================================================
// SECURITY SYSTEM SETUP
//=============================================================================

async fn setup_security_system(cyre: &mut Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Setting up Smart Home Security System...");

    // =====================================================
    // MOTION DETECTION
    // =====================================================

    // Motion sensor with intelligent processing
    cyre.action(
        IO::new("sensor.motion")
            .with_name("Motion Detection")
            .with_priority(Priority::High)
            .with_throttle(2000) // Prevent false alarms - 2 second cooldown
            .with_required(true)
            .with_condition("valid_motion_data")
            .with_transform("analyze_motion_pattern")
            .with_logging(true)
    )?;

    cyre.on("sensor.motion", |payload| {
        Box::pin(async move {
            let zone = payload
                .get("zone")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let confidence = payload
                .get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let timestamp = payload
                .get("timestamp")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let motion_type = payload
                .get("motion_type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            println!(
                "👁️  Motion Detected: Zone {} - {} (confidence: {:.1}%)",
                zone,
                motion_type,
                confidence * 100.0
            );

            // Determine threat level based on zone and time
            let threat_level = if zone == "front_door" || zone == "back_door" {
                "HIGH"
            } else if zone == "living_room" && is_night_time() {
                "MEDIUM"
            } else {
                "LOW"
            };

            let should_alert = confidence > 0.7 && threat_level != "LOW";

            CyreResponse::success(
                json!({
                    "zone": zone,
                    "motion_type": motion_type,
                    "confidence": confidence,
                    "threat_level": threat_level,
                    "should_alert": should_alert,
                    "timestamp": timestamp,
                    "processed_at": current_timestamp()
                }),
                format!("Motion processed in zone {}", zone)
            )
        })
    })?;

    // =====================================================
    // DOOR/WINDOW SENSORS
    // =====================================================

    // Door sensor with access control
    cyre.action(
        IO::new("sensor.door")
            .with_name("Door Access Control")
            .with_priority(Priority::Critical)
            .with_throttle(1000) // Prevent rapid door events
            .with_required(true)
            .with_condition("valid_door_event")
            .with_transform("validate_access")
            .with_logging(true)
    )?;

    cyre.on("sensor.door", |payload| {
        Box::pin(async move {
            let door_id = payload
                .get("door_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let action = payload
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let user_id = payload.get("user_id").and_then(|v| v.as_str());
            let timestamp = payload
                .get("timestamp")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            println!("🚪 Door Event: {} - {} by {}", door_id, action, user_id.unwrap_or("unknown"));

            // Check if this is authorized access
            let is_authorized = match user_id {
                Some(id) => is_authorized_user(id),
                None => false,
            };

            let security_status = if is_authorized {
                "AUTHORIZED"
            } else if action == "opened" {
                "UNAUTHORIZED_ACCESS"
            } else {
                "NORMAL"
            };

            CyreResponse::success(
                json!({
                    "door_id": door_id,
                    "action": action,
                    "user_id": user_id,
                    "is_authorized": is_authorized,
                    "security_status": security_status,
                    "timestamp": timestamp,
                    "processed_at": current_timestamp()
                }),
                format!("Door {} {} - {}", door_id, action, security_status)
            )
        })
    })?;

    // =====================================================
    // SMOKE/FIRE DETECTION
    // =====================================================

    // Smoke detector with emergency response
    cyre.action(
        IO::new("sensor.smoke")
            .with_name("Smoke/Fire Detection")
            .with_priority(Priority::Critical)
            .with_throttle(500) // Allow rapid fire detection
            .with_required(true)
            .with_condition("valid_smoke_reading")
            .with_transform("assess_fire_risk")
            .with_logging(true)
    )?;

    cyre.on("sensor.smoke", |payload| {
        Box::pin(async move {
            let sensor_id = payload
                .get("sensor_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let smoke_level = payload
                .get("smoke_level")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let temperature = payload
                .get("temperature")
                .and_then(|v| v.as_f64())
                .unwrap_or(20.0);
            let timestamp = payload
                .get("timestamp")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            println!(
                "🔥 Smoke Alert: Sensor {} - Level: {:.1}, Temp: {:.1}°C",
                sensor_id,
                smoke_level,
                temperature
            );

            // Determine fire risk level
            let fire_risk = if temperature > 80.0 && smoke_level > 0.7 {
                "CRITICAL"
            } else if temperature > 60.0 || smoke_level > 0.5 {
                "HIGH"
            } else if smoke_level > 0.2 {
                "MEDIUM"
            } else {
                "LOW"
            };

            let should_evacuate = fire_risk == "CRITICAL";
            let should_call_fire_dept = fire_risk == "HIGH" || fire_risk == "CRITICAL";

            CyreResponse::success(
                json!({
                    "sensor_id": sensor_id,
                    "smoke_level": smoke_level,
                    "temperature": temperature,
                    "fire_risk": fire_risk,
                    "should_evacuate": should_evacuate,
                    "should_call_fire_dept": should_call_fire_dept,
                    "timestamp": timestamp,
                    "processed_at": current_timestamp()
                }),
                format!("Fire risk assessment: {}", fire_risk)
            )
        })
    })?;

    // =====================================================
    // CAMERA SURVEILLANCE
    // =====================================================

    // Camera with AI analysis
    cyre.action(
        IO::new("camera.analyze")
            .with_name("AI Camera Analysis")
            .with_priority(Priority::Normal)
            .with_throttle(5000) // Process every 5 seconds max
            .with_required(true)
            .with_condition("valid_camera_feed")
            .with_transform("ai_object_detection")
            .with_logging(true)
    )?;

    cyre.on("camera.analyze", |payload| {
        Box::pin(async move {
            let camera_id = payload
                .get("camera_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let empty_vec = Vec::new();
            let objects_detected = payload
                .get("objects")
                .and_then(|v| v.as_array())
                .unwrap_or(&empty_vec);
            let confidence = payload
                .get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let timestamp = payload
                .get("timestamp")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            println!(
                "📹 Camera Analysis: {} - {} objects detected",
                camera_id,
                objects_detected.len()
            );

            // Check for suspicious objects or people
            let suspicious_objects: Vec<String> = objects_detected
                .iter()
                .filter_map(|obj| {
                    let obj_type = obj.get("type")?.as_str()?;
                    if obj_type == "person" || obj_type == "vehicle" {
                        Some(obj_type.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            let threat_level = if
                suspicious_objects.contains(&"person".to_string()) &&
                is_night_time()
            {
                "HIGH"
            } else if !suspicious_objects.is_empty() {
                "MEDIUM"
            } else {
                "LOW"
            };

            CyreResponse::success(
                json!({
                    "camera_id": camera_id,
                    "objects_detected": objects_detected.len(),
                    "suspicious_objects": suspicious_objects,
                    "threat_level": threat_level,
                    "confidence": confidence,
                    "timestamp": timestamp,
                    "processed_at": current_timestamp()
                }),
                format!(
                    "Camera analysis complete - {} suspicious objects",
                    suspicious_objects.len()
                )
            )
        })
    })?;

    // =====================================================
    // AUTOMATION CONTROLLER
    // =====================================================

    // Smart automation based on security events
    cyre.action(
        IO::new("automation.trigger")
            .with_name("Security Automation")
            .with_priority(Priority::High)
            .with_throttle(1000) // Prevent automation loops
            .with_required(true)
            .with_condition("valid_automation_event")
            .with_transform("determine_automation_action")
            .with_logging(true)
    )?;

    cyre.on("automation.trigger", |payload| {
        Box::pin(async move {
            let event_type = payload
                .get("event_type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let severity = payload
                .get("severity")
                .and_then(|v| v.as_str())
                .unwrap_or("low");
            let zone = payload
                .get("zone")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let timestamp = payload
                .get("timestamp")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            println!("🤖 Automation Triggered: {} - {} in {}", event_type, severity, zone);

            // Determine automation actions based on event
            let mut actions = Vec::new();

            match (event_type, severity) {
                ("motion", "HIGH") => {
                    actions.push("turn_on_lights".to_string());
                    actions.push("start_recording".to_string());
                    actions.push("send_alert".to_string());
                }
                ("door", "UNAUTHORIZED_ACCESS") => {
                    actions.push("lock_all_doors".to_string());
                    actions.push("activate_alarm".to_string());
                    actions.push("call_police".to_string());
                }
                ("smoke", "CRITICAL") => {
                    actions.push("activate_sprinklers".to_string());
                    actions.push("unlock_exit_doors".to_string());
                    actions.push("call_fire_department".to_string());
                }
                ("camera", "HIGH") => {
                    actions.push("turn_on_floodlights".to_string());
                    actions.push("send_alert".to_string());
                }
                _ => {
                    actions.push("log_event".to_string());
                }
            }

            CyreResponse::success(
                json!({
                    "event_type": event_type,
                    "severity": severity,
                    "zone": zone,
                    "actions": actions,
                    "timestamp": timestamp,
                    "processed_at": current_timestamp()
                }),
                format!("Automation triggered with {} actions", actions.len())
            )
        })
    })?;

    // =====================================================
    // SYSTEM HEALTH MONITORING
    // =====================================================

    // Monitor system health and battery levels
    cyre.action(
        IO::new("system.health")
            .with_name("System Health Monitor")
            .with_priority(Priority::Normal)
            .with_throttle(30000) // Check every 30 seconds
            .with_required(true)
            .with_condition("valid_health_data")
            .with_transform("assess_system_health")
            .with_logging(true)
    )?;

    cyre.on("system.health", |payload| {
        Box::pin(async move {
            let empty_map = serde_json::Map::new();
            let battery_levels = payload
                .get("batteries")
                .and_then(|v| v.as_object())
                .unwrap_or(&empty_map);
            let sensor_status = payload
                .get("sensors")
                .and_then(|v| v.as_object())
                .unwrap_or(&empty_map);
            let network_status = payload
                .get("network")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let timestamp = payload
                .get("timestamp")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            println!(
                "💚 System Health: Network {}, {} sensors active",
                network_status,
                sensor_status.len()
            );

            // Check for low batteries or offline sensors
            let mut alerts = Vec::new();
            let mut low_battery_count = 0;

            for (sensor_id, battery_level) in battery_levels {
                if let Some(level) = battery_level.as_f64() {
                    if level < 0.2 {
                        alerts.push(format!("Low battery: {}", sensor_id));
                        low_battery_count += 1;
                    }
                }
            }

            let system_status = if network_status == "offline" {
                "CRITICAL"
            } else if low_battery_count > 2 {
                "WARNING"
            } else {
                "HEALTHY"
            };

            CyreResponse::success(
                json!({
                    "network_status": network_status,
                    "active_sensors": sensor_status.len(),
                    "low_battery_count": low_battery_count,
                    "alerts": alerts,
                    "system_status": system_status,
                    "timestamp": timestamp,
                    "processed_at": current_timestamp()
                }),
                format!("System health: {}", system_status)
            )
        })
    })?;

    println!("✅ Security system setup complete!");
    println!();

    Ok(())
}

//=============================================================================
// SECURITY SCENARIOS
//=============================================================================

async fn normal_day_scenario(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏠 SCENARIO 1: Normal Day - Family Activities");
    println!("=============================================");

    // Simulate normal family activities
    let events = vec![
        (
            "sensor.door",
            json!({"door_id": "front_door", "action": "opened", "user_id": "dad", "timestamp": current_timestamp()}),
        ),
        (
            "sensor.motion",
            json!({"zone": "kitchen", "motion_type": "person", "confidence": 0.9, "timestamp": current_timestamp()}),
        ),
        (
            "camera.analyze",
            json!({"camera_id": "front_yard", "objects": [{"type": "person", "confidence": 0.8}], "confidence": 0.8, "timestamp": current_timestamp()}),
        ),
        (
            "sensor.door",
            json!({"door_id": "back_door", "action": "opened", "user_id": "mom", "timestamp": current_timestamp()}),
        )
    ];

    for (action, payload) in events {
        let result = cyre.call(action, payload).await;
        if result.ok {
            println!("   ✅ {}: {}", action, result.message);
        } else {
            println!("   ❌ {}: Failed - {}", action, result.message);
        }
        sleep(Duration::from_millis(500)).await;
    }

    println!();

    Ok(())
}

async fn unauthorized_access_scenario(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚨 SCENARIO 2: Unauthorized Access Detection");
    println!("=============================================");

    // Simulate unauthorized access
    let events = vec![
        (
            "sensor.motion",
            json!({"zone": "back_yard", "motion_type": "person", "confidence": 0.95, "timestamp": current_timestamp()}),
        ),
        (
            "camera.analyze",
            json!({"camera_id": "back_yard", "objects": [{"type": "person", "confidence": 0.9}], "confidence": 0.9, "timestamp": current_timestamp()}),
        ),
        (
            "sensor.door",
            json!({"door_id": "back_door", "action": "opened", "user_id": null, "timestamp": current_timestamp()}),
        ),
        (
            "automation.trigger",
            json!({"event_type": "door", "severity": "UNAUTHORIZED_ACCESS", "zone": "back_door", "timestamp": current_timestamp()}),
        )
    ];

    for (action, payload) in events {
        let result = cyre.call(action, payload).await;
        if result.ok {
            println!("   ✅ {}: {}", action, result.message);
        } else {
            println!("   ❌ {}: Failed - {}", action, result.message);
        }
        sleep(Duration::from_millis(500)).await;
    }

    println!();

    Ok(())
}

async fn fire_detection_scenario(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 SCENARIO 3: Fire/Smoke Detection");
    println!("====================================");

    // Simulate fire detection
    let events = vec![
        (
            "sensor.smoke",
            json!({"sensor_id": "kitchen", "smoke_level": 0.3, "temperature": 45.0, "timestamp": current_timestamp()}),
        ),
        (
            "sensor.smoke",
            json!({"sensor_id": "kitchen", "smoke_level": 0.8, "temperature": 85.0, "timestamp": current_timestamp()}),
        ),
        (
            "automation.trigger",
            json!({"event_type": "smoke", "severity": "CRITICAL", "zone": "kitchen", "timestamp": current_timestamp()}),
        )
    ];

    for (action, payload) in events {
        let result = cyre.call(action, payload).await;
        if result.ok {
            println!("   ✅ {}: {}", action, result.message);
        } else {
            println!("   ❌ {}: Failed - {}", action, result.message);
        }
        sleep(Duration::from_millis(500)).await;
    }

    println!();

    Ok(())
}

async fn power_outage_scenario(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ SCENARIO 4: Power Outage & Backup Systems");
    println!("=============================================");

    // Simulate power outage
    let events = vec![
        (
            "system.health",
            json!({"network": "offline", "sensors": {"motion_1": "online", "door_1": "offline"}, "batteries": {"motion_1": 0.8, "door_1": 0.1}, "timestamp": current_timestamp()}),
        ),
        (
            "automation.trigger",
            json!({"event_type": "power_outage", "severity": "CRITICAL", "zone": "system", "timestamp": current_timestamp()}),
        )
    ];

    for (action, payload) in events {
        let result = cyre.call(action, payload).await;
        if result.ok {
            println!("   ✅ {}: {}", action, result.message);
        } else {
            println!("   ❌ {}: Failed - {}", action, result.message);
        }
        sleep(Duration::from_millis(500)).await;
    }

    println!();

    Ok(())
}

async fn vacation_mode_scenario(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏖️  SCENARIO 5: Vacation Mode");
    println!("=============================");

    // Simulate vacation mode with different sensitivity
    let events = vec![
        (
            "sensor.motion",
            json!({"zone": "living_room", "motion_type": "person", "confidence": 0.7, "timestamp": current_timestamp()}),
        ),
        (
            "camera.analyze",
            json!({"camera_id": "front_yard", "objects": [{"type": "vehicle", "confidence": 0.8}], "confidence": 0.8, "timestamp": current_timestamp()}),
        ),
        (
            "automation.trigger",
            json!({"event_type": "motion", "severity": "HIGH", "zone": "living_room", "timestamp": current_timestamp()}),
        )
    ];

    for (action, payload) in events {
        let result = cyre.call(action, payload).await;
        if result.ok {
            println!("   ✅ {}: {}", action, result.message);
        } else {
            println!("   ❌ {}: Failed - {}", action, result.message);
        }
        sleep(Duration::from_millis(500)).await;
    }

    println!();

    Ok(())
}

async fn system_health_scenario(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
    println!("💚 SCENARIO 6: System Health Monitoring");
    println!("=======================================");

    // Simulate system health monitoring
    let events = vec![
        (
            "system.health",
            json!({"network": "online", "sensors": {"motion_1": "online", "door_1": "online", "smoke_1": "online"}, "batteries": {"motion_1": 0.9, "door_1": 0.8, "smoke_1": 0.15}, "timestamp": current_timestamp()}),
        ),
        (
            "system.health",
            json!({"network": "online", "sensors": {"motion_1": "online", "door_1": "offline", "smoke_1": "online"}, "batteries": {"motion_1": 0.8, "door_1": 0.05, "smoke_1": 0.1}, "timestamp": current_timestamp()}),
        )
    ];

    for (action, payload) in events {
        let result = cyre.call(action, payload).await;
        if result.ok {
            println!("   ✅ {}: {}", action, result.message);
        } else {
            println!("   ❌ {}: Failed - {}", action, result.message);
        }
        sleep(Duration::from_millis(500)).await;
    }

    println!();

    Ok(())
}

//=============================================================================
// UTILITY FUNCTIONS
//=============================================================================

fn current_timestamp() -> u64 {
    std::time::SystemTime
        ::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn is_night_time() -> bool {
    // Simulate night time detection (6 PM to 6 AM)
    let hour = (current_timestamp() / 3600000) % 24;
    hour >= 18 || hour < 6
}

fn is_authorized_user(user_id: &str) -> bool {
    // Simulate user authorization check
    let authorized_users = vec!["dad", "mom", "kid1", "kid2"];
    authorized_users.contains(&user_id)
}
