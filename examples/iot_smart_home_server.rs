// examples/iot_smart_home_server.rs
// Advanced IoT Smart Home Management System using Cyre
// Demonstrates real-world server architecture with mock IoT device streams

use cyre_rust::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::time::Duration;
use tokio::time::{sleep, interval};
use tokio::sync::RwLock;
use rand::Rng;

//=============================================================================
// SMART HOME DEVICE TYPES
//=============================================================================

#[derive(Debug, Clone)]
pub struct DeviceState {
    pub device_id: String,
    pub device_type: String,
    pub location: String,
    pub status: String,
    pub last_updated: u64,
    pub battery_level: Option<u8>,
    pub firmware_version: String,
}

#[derive(Debug, Clone)]
pub struct SensorReading {
    pub sensor_id: String,
    pub sensor_type: String,
    pub value: f64,
    pub unit: String,
    pub location: String,
    pub timestamp: u64,
    pub quality: String, // "excellent", "good", "poor"
}

#[derive(Debug)]
pub struct SmartHomeSystem {
    cyre: Arc<RwLock<Cyre>>,
    devices: Arc<RwLock<HashMap<String, DeviceState>>>,
    active_automations: Arc<RwLock<Vec<String>>>,
    security_alerts: Arc<AtomicU64>,
    energy_consumption: Arc<RwLock<f64>>,
    system_uptime: u64,
}

//=============================================================================
// MAIN IOT SMART HOME SERVER
//=============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏠 CYRE IOT SMART HOME MANAGEMENT SYSTEM");
    println!("========================================");
    println!("🌐 Advanced server-based IoT automation");
    println!("🔄 Real-time event processing with Cyre");
    println!("🤖 Intelligent automation and monitoring");
    println!();

    // Initialize the smart home system
    let smart_home = SmartHomeSystem::new().await;
    
    // Setup all IoT device channels and handlers
    smart_home.setup_device_channels().await;
    
    // Start IoT device simulators (mock data generators)
    smart_home.start_device_simulators().await;
    
    // Start automation engine
    smart_home.start_automation_engine().await;
    
    // Start system monitoring
    smart_home.start_system_monitoring().await;
    
    // Run interactive command interface
    smart_home.run_command_interface().await;
    
    Ok(())
}

impl SmartHomeSystem {
    async fn new() -> Self {
        let mut cyre = Cyre::new();
        
        // Initialize TimeKeeper for advanced scheduling
        if let Err(e) = cyre.init_timekeeper().await {
            eprintln!("⚠️ TimeKeeper initialization failed: {}", e);
        }

        Self {
            cyre: Arc::new(RwLock::new(cyre)),
            devices: Arc::new(RwLock::new(HashMap::new())),
            active_automations: Arc::new(RwLock::new(Vec::new())),
            security_alerts: Arc::new(AtomicU64::new(0)),
            energy_consumption: Arc::new(RwLock::new(0.0)),
            system_uptime: current_timestamp(),
        }
    }

    //=========================================================================
    // DEVICE CHANNEL SETUP
    //=========================================================================

    async fn setup_device_channels(&self) {
        println!("🔧 Setting up IoT device channels...");
        let mut cyre = self.cyre.write().await;

        // Climate Control System
        cyre.action(IO::new("climate.thermostat")
            .with_name("Smart Thermostat")
            .with_priority(Priority::High)
            .with_throttle(5000)); // Prevent rapid temperature changes

        cyre.action(IO::new("climate.hvac")
            .with_name("HVAC System")
            .with_priority(Priority::Critical));

        // Security System
        cyre.action(IO::new("security.camera")
            .with_name("Security Camera")
            .with_change_detection());

        cyre.action(IO::new("security.motion")
            .with_name("Motion Sensor")
            .with_priority(Priority::Critical)
            .with_change_detection());

        cyre.action(IO::new("security.door")
            .with_name("Smart Door Lock")
            .with_priority(Priority::Critical));

        cyre.action(IO::new("security.alarm")
            .with_name("Security Alarm")
            .with_priority(Priority::Critical));

        // Energy Management
        cyre.action(IO::new("energy.monitor")
            .with_name("Energy Monitor")
            .with_throttle(2000));

        cyre.action(IO::new("energy.solar")
            .with_name("Solar Panel System")
            .with_throttle(10000));

        cyre.action(IO::new("energy.battery")
            .with_name("Home Battery")
            .with_priority(Priority::High));

        // Lighting System
        cyre.action(IO::new("lighting.smart")
            .with_name("Smart Lighting")
            .with_change_detection());

        cyre.action(IO::new("lighting.ambient")
            .with_name("Ambient Lighting")
            .with_debounce(1000));

        // Environmental Sensors
        cyre.action(IO::new("sensor.air_quality")
            .with_name("Air Quality Monitor")
            .with_throttle(30000));

        cyre.action(IO::new("sensor.water")
            .with_name("Water Quality Sensor")
            .with_throttle(60000));

        cyre.action(IO::new("sensor.noise")
            .with_name("Noise Level Monitor")
            .with_throttle(5000));

        // Appliance Control
        cyre.action(IO::new("appliance.washing_machine")
            .with_name("Smart Washing Machine")
            .with_priority(Priority::Medium));

        cyre.action(IO::new("appliance.refrigerator")
            .with_name("Smart Refrigerator")
            .with_throttle(10000));

        cyre.action(IO::new("appliance.oven")
            .with_name("Smart Oven")
            .with_priority(Priority::High));

        // System Management
        cyre.action(IO::new("system.backup")
            .with_name("System Backup")
            .with_priority(Priority::Low));

        cyre.action(IO::new("system.update")
            .with_name("Device Update Manager")
            .with_priority(Priority::Medium));

        cyre.action(IO::new("automation.trigger")
            .with_name("Automation Engine")
            .with_priority(Priority::High));

        // Setup all event handlers
        self.setup_event_handlers(&mut cyre).await;

        println!("✅ {} IoT device channels configured", 16);
    }

    //=========================================================================
    // EVENT HANDLERS
    //=========================================================================

    async fn setup_event_handlers(&self, cyre: &mut Cyre) {
        println!("🔧 Setting up event handlers...");

        let devices = Arc::clone(&self.devices);
        let security_alerts = Arc::clone(&self.security_alerts);
        let energy_consumption = Arc::clone(&self.energy_consumption);

        // Climate Control Handler
        let devices_clone = Arc::clone(&devices);
        cyre.on("climate.thermostat", move |payload| {
            let devices = Arc::clone(&devices_clone);
            Box::pin(async move {
                let target_temp = payload.get("temperature").and_then(|v| v.as_f64()).unwrap_or(22.0);
                let location = payload.get("location").and_then(|v| v.as_str()).unwrap_or("main");
                let mode = payload.get("mode").and_then(|v| v.as_str()).unwrap_or("auto");

                println!("🌡️ Thermostat: {}°C in {} ({})", target_temp, location, mode);

                // Update device state
                let mut devices_map = devices.write().await;
                devices_map.insert(format!("thermostat_{}", location), DeviceState {
                    device_id: format!("thermostat_{}", location),
                    device_type: "thermostat".to_string(),
                    location: location.to_string(),
                    status: format!("{}°C - {}", target_temp, mode),
                    last_updated: current_timestamp(),
                    battery_level: None,
                    firmware_version: "2.1.3".to_string(),
                });

                let efficiency = if target_temp >= 18.0 && target_temp <= 24.0 { "optimal" } else { "high_usage" };

                CyreResponse {
                    ok: true,
                    payload: json!({
                        "device_id": format!("thermostat_{}", location),
                        "target_temperature": target_temp,
                        "current_temperature": target_temp - 2.0,
                        "mode": mode,
                        "location": location,
                        "energy_efficiency": efficiency,
                        "estimated_cost_per_hour": target_temp * 0.05
                    }),
                    message: format!("Thermostat set to {}°C", target_temp),
                    error: None,
                    timestamp: current_timestamp(),
                    metadata: Some(json!({"device_type": "climate", "location": location})),
                }
            })
        });

        // Security Motion Handler
        let devices_clone = Arc::clone(&devices);
        let alerts_clone = Arc::clone(&security_alerts);
        cyre.on("security.motion", move |payload| {
            let devices = Arc::clone(&devices_clone);
            let alerts = Arc::clone(&alerts_clone);
            Box::pin(async move {
                let motion_detected = payload.get("detected").and_then(|v| v.as_bool()).unwrap_or(false);
                let location = payload.get("location").and_then(|v| v.as_str()).unwrap_or("unknown");
                let confidence = payload.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.95);

                if motion_detected {
                    println!("🚨 Motion detected in {} (confidence: {:.1}%)", location, confidence * 100.0);
                    alerts.fetch_add(1, Ordering::SeqCst);

                    // Update device state
                    let mut devices_map = devices.write().await;
                    devices_map.insert(format!("motion_{}", location), DeviceState {
                        device_id: format!("motion_{}", location),
                        device_type: "motion_sensor".to_string(),
                        location: location.to_string(),
                        status: "motion_detected".to_string(),
                        last_updated: current_timestamp(),
                        battery_level: Some(87),
                        firmware_version: "1.8.2".to_string(),
                    });
                }

                CyreResponse {
                    ok: true,
                    payload: json!({
                        "device_id": format!("motion_{}", location),
                        "motion_detected": motion_detected,
                        "location": location,
                        "confidence": confidence,
                        "alert_level": if motion_detected { "medium" } else { "none" },
                        "recommended_action": if motion_detected { "verify_identity" } else { "continue_monitoring" }
                    }),
                    message: if motion_detected { "Motion detected" } else { "Area clear" }.to_string(),
                    error: None,
                    timestamp: current_timestamp(),
                    metadata: Some(json!({"device_type": "security", "location": location})),
                }
            })
        });

        // Energy Monitor Handler
        let energy_clone = Arc::clone(&energy_consumption);
        let devices_clone = Arc::clone(&devices);
        cyre.on("energy.monitor", move |payload| {
            let energy = Arc::clone(&energy_clone);
            let devices = Arc::clone(&devices_clone);
            Box::pin(async move {
                let consumption = payload.get("consumption_kw").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let device_name = payload.get("device").and_then(|v| v.as_str()).unwrap_or("unknown");
                let cost_per_kwh = 0.15; // $0.15 per kWh

                println!("⚡ Energy: {} consuming {:.2} kW", device_name, consumption);

                // Update total energy consumption
                {
                    let mut total_energy = energy.write().await;
                    *total_energy += consumption;
                }

                // Update device state
                let mut devices_map = devices.write().await;
                devices_map.insert(format!("energy_{}", device_name), DeviceState {
                    device_id: format!("energy_{}", device_name),
                    device_type: "energy_monitor".to_string(),
                    location: "utility_room".to_string(),
                    status: format!("{:.2} kW", consumption),
                    last_updated: current_timestamp(),
                    battery_level: None,
                    firmware_version: "3.0.1".to_string(),
                });

                let efficiency_rating = if consumption < 1.0 { "excellent" } 
                    else if consumption < 3.0 { "good" } 
                    else { "review_needed" };

                CyreResponse {
                    ok: true,
                    payload: json!({
                        "device_id": format!("energy_{}", device_name),
                        "device": device_name,
                        "consumption_kw": consumption,
                        "hourly_cost": consumption * cost_per_kwh,
                        "daily_projection": consumption * cost_per_kwh * 24.0,
                        "efficiency_rating": efficiency_rating,
                        "carbon_footprint_kg": consumption * 0.5 // Rough estimate
                    }),
                    message: format!("Energy monitoring: {:.2} kW", consumption),
                    error: None,
                    timestamp: current_timestamp(),
                    metadata: Some(json!({"device_type": "energy"})),
                }
            })
        });

        // Smart Lighting Handler
        let devices_clone = Arc::clone(&devices);
        cyre.on("lighting.smart", move |payload| {
            let devices = Arc::clone(&devices_clone);
            Box::pin(async move {
                let brightness = payload.get("brightness").and_then(|v| v.as_u64()).unwrap_or(100);
                let color = payload.get("color").and_then(|v| v.as_str()).unwrap_or("warm_white");
                let room = payload.get("room").and_then(|v| v.as_str()).unwrap_or("living_room");
                let state = payload.get("state").and_then(|v| v.as_str()).unwrap_or("on");

                println!("💡 Light: {} in {} - {} ({}%)", state, room, color, brightness);

                // Update device state
                let mut devices_map = devices.write().await;
                devices_map.insert(format!("light_{}", room), DeviceState {
                    device_id: format!("light_{}", room),
                    device_type: "smart_light".to_string(),
                    location: room.to_string(),
                    status: format!("{} - {}%", state, brightness),
                    last_updated: current_timestamp(),
                    battery_level: None,
                    firmware_version: "1.5.7".to_string(),
                });

                let power_usage = if state == "on" { brightness as f64 * 0.12 } else { 0.0 };

                CyreResponse {
                    ok: true,
                    payload: json!({
                        "device_id": format!("light_{}", room),
                        "room": room,
                        "state": state,
                        "brightness": brightness,
                        "color": color,
                        "power_usage_watts": power_usage,
                        "estimated_lifespan_hours": 50000 - (brightness as u64 * 10)
                    }),
                    message: format!("Light {} in {}", state, room),
                    error: None,
                    timestamp: current_timestamp(),
                    metadata: Some(json!({"device_type": "lighting", "room": room})),
                }
            })
        });

        // Automation Engine Handler
        let devices_clone = Arc::clone(&devices);
        cyre.on("automation.trigger", move |payload| {
            let devices = Arc::clone(&devices_clone);
            Box::pin(async move {
                let rule_name = payload.get("rule").and_then(|v| v.as_str()).unwrap_or("unknown");
                let trigger_type = payload.get("trigger").and_then(|v| v.as_str()).unwrap_or("manual");
                let actions = payload.get("actions").and_then(|v| v.as_array()).map(|arr| {
                    arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>()
                }).unwrap_or_default();

                println!("🤖 Automation: {} triggered by {}", rule_name, trigger_type);

                for action in &actions {
                    println!("   ⚙️ Executing: {}", action);
                }

                // Update automation device state
                let mut devices_map = devices.write().await;
                devices_map.insert("automation_engine".to_string(), DeviceState {
                    device_id: "automation_engine".to_string(),
                    device_type: "automation".to_string(),
                    location: "system".to_string(),
                    status: format!("executed_{}", rule_name),
                    last_updated: current_timestamp(),
                    battery_level: None,
                    firmware_version: "4.2.1".to_string(),
                });

                CyreResponse {
                    ok: true,
                    payload: json!({
                        "device_id": "automation_engine",
                        "rule_name": rule_name,
                        "trigger_type": trigger_type,
                        "actions_executed": actions.len(),
                        "actions": actions,
                        "execution_time_ms": 125,
                        "success_rate": "100%"
                    }),
                    message: format!("Automation rule '{}' executed", rule_name),
                    error: None,
                    timestamp: current_timestamp(),
                    metadata: Some(json!({"device_type": "automation"})),
                }
            })
        });

        println!("✅ Event handlers configured");
    }

    //=========================================================================
    // DEVICE SIMULATORS (MOCK DATA GENERATORS)
    //=========================================================================

    async fn start_device_simulators(&self) {
        println!("📡 Starting IoT device simulators...");

        let cyre = Arc::clone(&self.cyre);

        // Temperature sensors - every 30 seconds
        let cyre_temp = Arc::clone(&cyre);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            let locations = vec!["living_room", "bedroom", "kitchen", "garage", "basement"];
            
            loop {
                interval.tick().await;
                
                for location in &locations {
                    let base_temp = match *location {
                        "garage" => 15.0,
                        "basement" => 18.0,
                        _ => 21.0,
                    };
                    
                    // Generate temperature value first, then drop RNG
                    let temperature = {
                        let mut rng = rand::rng();
                        base_temp + rng.random_range(-3.0..3.0)
                    }; // RNG is dropped here
                    
                    let cyre_guard = cyre_temp.read().await;
                    let _ = cyre_guard.call("climate.thermostat", json!({
                        "temperature": temperature,
                        "location": location,
                        "mode": "auto"
                    })).await;
                }
            }
        });

        // Motion sensors - random intervals
        let cyre_motion = Arc::clone(&cyre);
        tokio::spawn(async move {
            let locations = vec!["front_door", "living_room", "kitchen", "bedroom", "garage"];
            
            loop {
                // Generate all random values first, then drop RNG
                let (delay, location_idx, motion_detected, confidence) = {
                    let mut rng = rand::rng();
                    (
                        rng.random_range(10..60), // 10-60 seconds
                        rng.random_range(0..locations.len()),
                        rng.random_bool(0.3), // 30% chance of motion
                        rng.random_range(0.7..1.0)
                    )
                }; // RNG is dropped here
                
                sleep(Duration::from_secs(delay)).await;
                
                let location = &locations[location_idx];
                
                let cyre_guard = cyre_motion.read().await;
                let _ = cyre_guard.call("security.motion", json!({
                    "detected": motion_detected,
                    "location": location,
                    "confidence": confidence
                })).await;
            }
        });

        // Energy monitoring - every 15 seconds
        let cyre_energy = Arc::clone(&cyre);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(15));
            let devices = vec![
                ("refrigerator", 0.15..0.25),
                ("washing_machine", 0.0..2.5),
                ("hvac_system", 1.5..4.0),
                ("lighting", 0.1..0.8),
                ("electronics", 0.2..0.6),
            ];
            
            loop {
                interval.tick().await;
                
                for (device, range) in &devices {
                    // Create new RNG for each iteration to avoid thread safety issues
                    let consumption = {
                        let mut rng = rand::rng();
                        rng.random_range(range.clone())
                    };
                    
                    let cyre_guard = cyre_energy.read().await;
                    let _ = cyre_guard.call("energy.monitor", json!({
                        "consumption_kw": consumption,
                        "device": device
                    })).await;
                }
            }
        });

        // Smart lighting - responsive to motion
        let cyre_lighting = Arc::clone(&cyre);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(45));
            let rooms = vec!["living_room", "kitchen", "bedroom", "office"];
            
            loop {
                interval.tick().await;
                
                for room in &rooms {
                    // Create new RNG for each iteration to avoid thread safety issues
                    let (brightness, state, color) = {
                        let mut rng = rand::rng();
                        let brightness = rng.random_range(20..100);
                        let state = if rng.random_bool(0.8) { "on" } else { "off" };
                        let colors = vec!["warm_white", "cool_white", "soft_blue", "amber"];
                        let color = &colors[rng.random_range(0..colors.len())];
                        (brightness, state, color.to_string())
                    };
                    
                    let cyre_guard = cyre_lighting.read().await;
                    let _ = cyre_guard.call("lighting.smart", json!({
                        "room": room,
                        "state": state,
                        "brightness": brightness,
                        "color": color
                    })).await;
                }
            }
        });

        println!("✅ IoT device simulators started");
    }

    //=========================================================================
    // AUTOMATION ENGINE
    //=========================================================================

    async fn start_automation_engine(&self) {
        println!("🤖 Starting automation engine...");

        let cyre = Arc::clone(&self.cyre);

        // Smart automation rules - every 2 minutes
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(120));
            let automation_rules = vec![
                ("energy_optimization", vec!["reduce_hvac_load", "optimize_lighting", "schedule_appliances"]),
                ("security_patrol", vec!["check_all_sensors", "verify_door_locks", "update_camera_positions"]),
                ("comfort_adjustment", vec!["balance_temperature", "adjust_humidity", "optimize_air_flow"]),
                ("evening_routine", vec!["dim_lights", "secure_home", "activate_night_mode"]),
                ("morning_routine", vec!["gradual_wake_lighting", "start_hvac", "security_summary"]),
            ];
            
            loop {
                interval.tick().await;
                
                // Generate rule index first, then drop RNG
                let rule_idx = {
                    let mut rng = rand::rng();
                    rng.random_range(0..automation_rules.len())
                }; // RNG is dropped here
                
                let (rule_name, actions) = &automation_rules[rule_idx];
                
                let cyre_guard = cyre.read().await;
                let _ = cyre_guard.call("automation.trigger", json!({
                    "rule": rule_name,
                    "trigger": "scheduled",
                    "actions": actions
                })).await;
            }
        });

        println!("✅ Automation engine started");
    }

    //=========================================================================
    // SYSTEM MONITORING
    //=========================================================================

    async fn start_system_monitoring(&self) {
        println!("📊 Starting system monitoring...");

        let cyre = Arc::clone(&self.cyre);
        let devices = Arc::clone(&self.devices);
        let security_alerts = Arc::clone(&self.security_alerts);
        let energy_consumption = Arc::clone(&self.energy_consumption);

        // System status monitoring - every minute
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let device_count = {
                    let devices_map = devices.read().await;
                    devices_map.len()
                };
                
                let total_alerts = security_alerts.load(Ordering::SeqCst);
                let total_energy = {
                    let energy = energy_consumption.read().await;
                    *energy
                };
                
                let cyre_guard = cyre.read().await;
                let metrics = cyre_guard.get_performance_metrics();
                
                println!("📈 System Status:");
                println!("   🏠 Active devices: {}", device_count);
                println!("   🚨 Security alerts: {}", total_alerts);
                println!("   ⚡ Total energy: {:.2} kWh", total_energy);
                println!("   🎯 Cyre executions: {}", metrics["total_executions"]);
                println!("   ⚡ Fast path ratio: {:.1}%", metrics["fast_path_ratio"]);
                println!();
            }
        });

        println!("✅ System monitoring started");
    }

    //=========================================================================
    // INTERACTIVE COMMAND INTERFACE
    //=========================================================================

    async fn run_command_interface(&self) {
        println!("🎮 SMART HOME COMMAND INTERFACE");
        println!("===============================");
        println!("Available commands:");
        println!("  • thermostat <temp> <location>  - Set temperature");
        println!("  • lights <on/off> <room>        - Control lights");
        println!("  • security status               - Check security");
        println!("  • energy report                 - Energy analysis");
        println!("  • devices                       - List all devices");
        println!("  • automation <rule>             - Trigger automation");
        println!("  • metrics                       - Cyre performance");
        println!("  • emergency                     - Emergency protocol");
        println!("  • quit                          - Exit system");
        println!();

        loop {
            println!("🏠 Enter command: ");
            
            // For demo purposes, we'll simulate some commands
            let demo_commands = vec![
                "thermostat 23 living_room",
                "lights on kitchen",
                "security status",
                "energy report",
                "devices",
                "automation evening_routine",
                "metrics",
                "emergency",
            ];

            for command in demo_commands {
                println!("🎮 Executing: {}", command);
                self.process_command(command).await;
                sleep(Duration::from_secs(3)).await;
                println!();
            }

            // Demo complete
            println!("🎉 Smart home demo commands completed!");
            println!("🔄 System continues running with automated simulators...");
            
            // Let simulators run for a while
            sleep(Duration::from_secs(30)).await;
            break;
        }
    }

    async fn process_command(&self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let cyre_guard = self.cyre.read().await;

        match parts[0] {
            "thermostat" => {
                if parts.len() >= 3 {
                    let temp: f64 = parts[1].parse().unwrap_or(22.0);
                    let location = parts[2];
                    
                    let result = cyre_guard.call("climate.thermostat", json!({
                        "temperature": temp,
                        "location": location,
                        "mode": "manual"
                    })).await;
                    
                    if result.ok {
                        println!("✅ Thermostat set to {}°C in {}", temp, location);
                    }
                }
            },
            "lights" => {
                if parts.len() >= 3 {
                    let state = parts[1];
                    let room = parts[2];
                    
                    let result = cyre_guard.call("lighting.smart", json!({
                        "state": state,
                        "room": room,
                        "brightness": if state == "on" { 100 } else { 0 }
                    })).await;
                    
                    if result.ok {
                        println!("✅ Lights {} in {}", state, room);
                    }
                }
            },
            "security" => {
                if parts.len() >= 2 && parts[1] == "status" {
                    let total_alerts = self.security_alerts.load(Ordering::SeqCst);
                    let devices_map = self.devices.read().await;
                    
                    let security_devices: Vec<_> = devices_map.values()
                        .filter(|d| d.device_type.contains("motion") || d.device_type.contains("camera"))
                        .collect();
                    
                    println!("🔒 Security Status Report:");
                    println!("   📊 Total alerts today: {}", total_alerts);
                    println!("   📹 Security devices: {}", security_devices.len());
                    
                    for device in security_devices.iter().take(3) {
                        println!("   🔍 {}: {} ({})", 
                            device.device_id, 
                            device.status, 
                            device.location
                        );
                    }
                }
            },
            "energy" => {
                if parts.len() >= 2 && parts[1] == "report" {
                    let total_energy = {
                        let energy = self.energy_consumption.read().await;
                        *energy
                    };
                    
                    let estimated_cost = total_energy * 0.15; // $0.15 per kWh
                    let carbon_footprint = total_energy * 0.5; // Rough kg CO2
                    
                    println!("⚡ Energy Report:");
                    println!("   📊 Total consumption: {:.2} kWh", total_energy);
                    println!("   💰 Estimated cost: ${:.2}", estimated_cost);
                    println!("   🌱 Carbon footprint: {:.1} kg CO2", carbon_footprint);
                    
                    if total_energy > 10.0 {
                        println!("   ⚠️ High energy usage detected");
                        
                        // Trigger energy optimization
                        let _ = cyre_guard.call("automation.trigger", json!({
                            "rule": "energy_optimization",
                            "trigger": "high_usage_detected",
                            "actions": ["reduce_hvac_load", "optimize_lighting"]
                        })).await;
                    } else {
                        println!("   ✅ Energy usage within normal range");
                    }
                }
            },
            "devices" => {
                let devices_map = self.devices.read().await;
                println!("🏠 Connected Devices ({}):", devices_map.len());
                
                for (_, device) in devices_map.iter().take(8) {
                    let battery_info = if let Some(battery) = device.battery_level {
                        format!(" ({}%)", battery)
                    } else {
                        String::new()
                    };
                    
                    println!("   📱 {} [{}]: {} in {}{}", 
                        device.device_id,
                        device.device_type,
                        device.status,
                        device.location,
                        battery_info
                    );
                }
                
                if devices_map.len() > 8 {
                    println!("   ... and {} more devices", devices_map.len() - 8);
                }
            },
            "automation" => {
                if parts.len() >= 2 {
                    let rule_name = parts[1];
                    
                    let actions = match rule_name {
                        "evening_routine" => vec!["dim_all_lights", "lock_doors", "arm_security", "reduce_temperature"],
                        "morning_routine" => vec!["gradual_lights", "coffee_maker_on", "news_briefing", "optimal_temperature"],
                        "away_mode" => vec!["all_lights_off", "security_active", "hvac_eco_mode", "appliances_standby"],
                        "movie_mode" => vec!["dim_living_room", "close_blinds", "sound_system_on", "ambient_lighting"],
                        _ => vec!["unknown_automation"]
                    };
                    
                    let result = cyre_guard.call("automation.trigger", json!({
                        "rule": rule_name,
                        "trigger": "manual_command",
                        "actions": actions
                    })).await;
                    
                    if result.ok {
                        println!("✅ Automation '{}' executed successfully", rule_name);
                    } else {
                        println!("❌ Failed to execute automation '{}'", rule_name);
                    }
                }
            },
            "metrics" => {
                let metrics = cyre_guard.get_performance_metrics();
                let uptime_seconds = current_timestamp() - self.system_uptime;
                let uptime_hours = uptime_seconds as f64 / 3600.0;
                
                println!("📈 Cyre Performance Metrics:");
                println!("   🎯 Total executions: {}", metrics["total_executions"]);
                println!("   ⚡ Fast path hits: {}", metrics["fast_path_hits"]);
                println!("   🛡️ Protection blocks: {}", metrics["protection_blocks"]);
                println!("   📊 Fast path ratio: {:.1}%", metrics["fast_path_ratio"]);
                println!("   🔗 Active channels: {}", metrics["active_channels"]);
                println!("   ⏰ System uptime: {:.1} hours", uptime_hours);
                
                let executions_per_minute = metrics.get("total_executions")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as f64 / (uptime_hours * 60.0);
                println!("   🚀 Execution rate: {:.1} events/min", executions_per_minute);
            },
            "emergency" => {
                println!("🚨 EMERGENCY PROTOCOL ACTIVATED");
                
                // Trigger multiple emergency actions simultaneously
                let emergency_actions = vec![
                    ("security.alarm", json!({"action": "activate", "level": "critical"})),
                    ("lighting.smart", json!({"state": "on", "brightness": 100, "room": "all"})),
                    ("automation.trigger", json!({
                        "rule": "emergency_response",
                        "trigger": "manual_emergency",
                        "actions": ["contact_authorities", "secure_all_zones", "emergency_lighting"]
                    }))
                ];
                
                for (channel, payload) in emergency_actions {
                    let _ = cyre_guard.call(channel, payload).await;
                }
                
                println!("✅ Emergency protocol executed");
                println!("   🚨 Security alarm activated");
                println!("   💡 Emergency lighting enabled");
                println!("   📞 Authorities contacted");
                println!("   🔒 All zones secured");
            },
            "quit" => {
                println!("👋 Shutting down smart home system...");
                return;
            },
            _ => {
                println!("❓ Unknown command: {}. Type 'help' for available commands.", parts[0]);
            }
        }
    }
}

//=============================================================================
// UTILITY FUNCTIONS
//=============================================================================

impl SmartHomeSystem {
    /// Generate a realistic sensor reading
    fn generate_sensor_reading(sensor_type: &str, location: &str) -> SensorReading {
        // Create RNG locally
        let mut rng = rand::rng();
        
        let (value, unit) = match sensor_type {
            "temperature" => (rng.random_range(18.0..26.0), "°C".to_string()),
            "humidity" => (rng.random_range(30.0..70.0), "%".to_string()),
            "air_quality" => (rng.random_range(50.0..150.0), "AQI".to_string()),
            "noise_level" => (rng.random_range(25.0..65.0), "dB".to_string()),
            "light_level" => (rng.random_range(0.0..1000.0), "lux".to_string()),
            _ => (0.0, "unknown".to_string()),
        };
        
        let quality = if value > 80.0 { "excellent" } 
            else if value > 50.0 { "good" } 
            else { "poor" };
        
        SensorReading {
            sensor_id: format!("{}_{}", sensor_type, location),
            sensor_type: sensor_type.to_string(),
            value,
            unit,
            location: location.to_string(),
            timestamp: current_timestamp(),
            quality: quality.to_string(),
        }
    }
    
    /// Simulate device failure and recovery
    async fn simulate_device_issues(&self) {
        // Create RNG locally
        let mut rng = rand::rng();
        
        // Occasionally simulate device connectivity issues
        if rng.random_bool(0.05) { // 5% chance
            println!("⚠️ Device connectivity issue detected");
            println!("   🔄 Attempting automatic recovery...");
            
            sleep(Duration::from_secs(2)).await;
            
            if rng.random_bool(0.8) { // 80% recovery success rate
                println!("   ✅ Device connectivity restored");
            } else {
                println!("   ❌ Manual intervention required");
            }
        }
    }
    
    /// Generate system health report
    async fn generate_health_report(&self) -> HashMap<String, serde_json::Value> {
        let devices_map = self.devices.read().await;
        let total_alerts = self.security_alerts.load(Ordering::SeqCst);
        let total_energy = {
            let energy = self.energy_consumption.read().await;
            *energy
        };
        
        let cyre_guard = self.cyre.read().await;
        let metrics = cyre_guard.get_performance_metrics();
        
        let mut report = HashMap::new();
        report.insert("total_devices".to_string(), json!(devices_map.len()));
        report.insert("security_alerts".to_string(), json!(total_alerts));
        report.insert("energy_consumption".to_string(), json!(total_energy));
        report.insert("cyre_executions".to_string(), json!(metrics["total_executions"]));
        report.insert("system_uptime".to_string(), json!(current_timestamp() - self.system_uptime));
        report.insert("fast_path_ratio".to_string(), json!(metrics["fast_path_ratio"]));
        
        // Calculate system health score
        let fast_path_ratio = metrics.get("fast_path_ratio")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        let health_score = ((fast_path_ratio + 
                           (if total_alerts < 10 { 90.0 } else { 50.0 }) +
                           (if total_energy < 20.0 { 95.0 } else { 70.0 })) / 3.0) as u64;
        
        report.insert("health_score".to_string(), json!(health_score));
        report.insert("status".to_string(), json!(
            if health_score > 85 { "excellent" }
            else if health_score > 70 { "good" }
            else { "needs_attention" }
        ));
        
        report
    }
}

//=============================================================================
// ADVANCED FEATURES SHOWCASE
//=============================================================================

/// Demonstrate advanced Cyre features in IoT context
async fn demonstrate_advanced_features(smart_home: &SmartHomeSystem) {
    println!("🔬 ADVANCED CYRE FEATURES DEMONSTRATION");
    println!("======================================");
    
    let cyre_guard = smart_home.cyre.read().await;
    
    // Demonstrate high-frequency sensor data with throttling
    println!("📡 Testing high-frequency sensor data (throttling)...");
    for i in 1..=10 {
        let result = cyre_guard.call("energy.monitor", json!({
            "consumption_kw": 2.5 + (i as f64 * 0.1),
            "device": "test_sensor"
        })).await;
        
        if !result.ok {
            println!("   ⏸️ Reading {} throttled", i);
        }
        sleep(Duration::from_millis(200)).await; // Faster than throttle limit
    }
    
    // Demonstrate change detection
    println!("\n🔄 Testing change detection...");
    for state in vec!["on", "on", "off", "off", "on"] {
        let result = cyre_guard.call("lighting.smart", json!({
            "state": state,
            "room": "test_room",
            "brightness": 75
        })).await;
        
        if result.ok {
            println!("   ✅ Light state changed to: {}", state);
        } else {
            println!("   ⏸️ Duplicate state '{}' ignored", state);
        }
        sleep(Duration::from_millis(300)).await;
    }
    
    // Demonstrate priority handling
    println!("\n🎯 Testing priority handling...");
    let priority_tests = vec![
        ("security.alarm", "CRITICAL", json!({"action": "test", "level": "high"})),
        ("lighting.smart", "MEDIUM", json!({"state": "on", "room": "test"})),
        ("system.backup", "LOW", json!({"action": "test_backup"})),
    ];
    
    for (channel, priority, payload) in priority_tests {
        let result = cyre_guard.call(channel, payload).await;
        if result.ok {
            println!("   {} priority channel '{}' executed", priority, channel);
        }
    }
    
    println!("\n✅ Advanced features demonstration complete");
}

//=============================================================================
// MAIN DEMO RUNNER WITH COMPREHENSIVE SCENARIOS
//=============================================================================

/// Extended demo showcasing real-world IoT scenarios
pub async fn run_comprehensive_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌟 COMPREHENSIVE IOT SMART HOME DEMO");
    println!("===================================");
    
    let smart_home = SmartHomeSystem::new().await;
    smart_home.setup_device_channels().await;
    
    // Scenario 1: Morning routine automation
    println!("\n📅 SCENARIO 1: Automated Morning Routine");
    println!("----------------------------------------");
    
    let cyre_guard = smart_home.cyre.read().await;
    let _ = cyre_guard.call("automation.trigger", json!({
        "rule": "morning_routine",
        "trigger": "time_based",
        "actions": ["gradual_wake_lighting", "optimal_temperature", "coffee_preparation"]
    })).await;
    
    sleep(Duration::from_secs(3)).await;
    
    // Scenario 2: Security event response
    println!("\n🚨 SCENARIO 2: Security Event Response");
    println!("-------------------------------------");
    
    // Simulate motion detection
    let _ = cyre_guard.call("security.motion", json!({
        "detected": true,
        "location": "front_door",
        "confidence": 0.95
    })).await;
    
    sleep(Duration::from_secs(2)).await;
    
    // Automatic response
    let _ = cyre_guard.call("automation.trigger", json!({
        "rule": "security_response",
        "trigger": "motion_detected",
        "actions": ["activate_cameras", "increase_lighting", "send_notification"]
    })).await;
    
    // Scenario 3: Energy optimization
    println!("\n⚡ SCENARIO 3: Dynamic Energy Optimization");
    println!("------------------------------------------");
    
    // Simulate high energy usage
    let _ = cyre_guard.call("energy.monitor", json!({
        "consumption_kw": 8.5,
        "device": "hvac_system"
    })).await;
    
    sleep(Duration::from_secs(2)).await;
    
    // Trigger optimization
    let _ = cyre_guard.call("automation.trigger", json!({
        "rule": "energy_optimization",
        "trigger": "high_consumption",
        "actions": ["reduce_hvac_load", "optimize_lighting", "defer_non_critical"]
    })).await;
    
    // Performance summary
    let final_metrics = cyre_guard.get_performance_metrics();
    println!("\n📊 FINAL PERFORMANCE SUMMARY");
    println!("============================");
    println!("🎯 Total events processed: {}", final_metrics["total_executions"]);
    println!("⚡ Fast path efficiency: {:.1}%", final_metrics["fast_path_ratio"]);
    println!("🛡️ Protection activations: {}", final_metrics["protection_blocks"]);
    println!("🏆 System demonstrated: High-performance IoT event processing");
    
    drop(cyre_guard);
    
    // Show advanced features
    demonstrate_advanced_features(&smart_home).await;
    
    println!("\n🎉 COMPREHENSIVE DEMO COMPLETE!");
    println!("🏠 Smart home system ready for production deployment");
    
    Ok(())
}

// Add to iot_simple_server.rs - HTTP endpoint for receiving client data
// This goes after the main() function in the simple server

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::header::{CONTENT_TYPE, ACCESS_CONTROL_ALLOW_ORIGIN};
use std::convert::Infallible;
use std::net::SocketAddr;

//=============================================================================
// HTTP SERVER FOR CLIENT COMMUNICATION
//=============================================================================

impl SmartHomeSystem {
    /// Start HTTP server to receive events from IoT clients
    async fn start_http_server(&self) {
        println!("🌐 Starting HTTP server for IoT client communication...");
        
        let cyre = Arc::clone(&self.cyre);
        
        let make_svc = make_service_fn(move |_conn| {
            let cyre = Arc::clone(&cyre);
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let cyre = Arc::clone(&cyre);
                    handle_http_request(req, cyre)
                }))
            }
        });

        let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
        let server = Server::bind(&addr).serve(make_svc);

        println!("✅ HTTP server listening on http://localhost:3000");
        println!("📡 Ready to receive IoT device events from clients");

        tokio::spawn(async move {
            if let Err(e) = server.await {
                eprintln!("❌ HTTP server error: {}", e);
            }
        });
    }
}

/// Handle incoming HTTP requests from IoT clients
async fn handle_http_request(
    req: Request<Body>,
    cyre: Arc<RwLock<Cyre>>
) -> Result<Response<Body>, Infallible> {
    
    // CORS headers for web clients
    let mut response = Response::builder()
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type");

    match (req.method(), req.uri().path()) {
        // Health check endpoint
        (&Method::GET, "/health") => {
            let body = json!({
                "status": "healthy",
                "service": "Cyre IoT Smart Home Server",
                "timestamp": current_timestamp()
            });

            response
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap()
        },

        // IoT device data endpoint
        (&Method::POST, "/api/iot/events") => {
            match process_iot_event(req, cyre).await {
                Ok(response_body) => {
                    response
                        .header(CONTENT_TYPE, "application/json")
                        .body(Body::from(response_body))
                        .unwrap()
                },
                Err(error_body) => {
                    response
                        .status(StatusCode::BAD_REQUEST)
                        .header(CONTENT_TYPE, "application/json")
                        .body(Body::from(error_body))
                        .unwrap()
                }
            }
        },

        // Temperature endpoint
        (&Method::POST, "/api/iot/temperature") => {
            match process_temperature_event(req, cyre).await {
                Ok(response_body) => {
                    response
                        .header(CONTENT_TYPE, "application/json")
                        .body(Body::from(response_body))
                        .unwrap()
                },
                Err(error_body) => {
                    response
                        .status(StatusCode::BAD_REQUEST)
                        .header(CONTENT_TYPE, "application/json")
                        .body(Body::from(error_body))
                        .unwrap()
                }
            }
        },

        // Motion sensor endpoint
        (&Method::POST, "/api/iot/motion") => {
            match process_motion_event(req, cyre).await {
                Ok(response_body) => {
                    response
                        .header(CONTENT_TYPE, "application/json")
                        .body(Body::from(response_body))
                        .unwrap()
                },
                Err(error_body) => {
                    response
                        .status(StatusCode::BAD_REQUEST)
                        .header(CONTENT_TYPE, "application/json")
                        .body(Body::from(error_body))
                        .unwrap()
                }
            }
        },

        // Energy monitoring endpoint
        (&Method::POST, "/api/iot/energy") => {
            match process_energy_event(req, cyre).await {
                Ok(response_body) => {
                    response
                        .header(CONTENT_TYPE, "application/json")
                        .body(Body::from(response_body))
                        .unwrap()
                },
                Err(error_body) => {
                    response
                        .status(StatusCode::BAD_REQUEST)
                        .header(CONTENT_TYPE, "application/json")
                        .body(Body::from(error_body))
                        .unwrap()
                }
            }
        },

        // Lighting control endpoint
        (&Method::POST, "/api/iot/lighting") => {
            match process_lighting_event(req, cyre).await {
                Ok(response_body) => {
                    response
                        .header(CONTENT_TYPE, "application/json")
                        .body(Body::from(response_body))
                        .unwrap()
                },
                Err(error_body) => {
                    response
                        .status(StatusCode::BAD_REQUEST)
                        .header(CONTENT_TYPE, "application/json")
                        .body(Body::from(error_body))
                        .unwrap()
                }
            }
        },

        // Automation trigger endpoint
        (&Method::POST, "/api/iot/automation") => {
            match process_automation_event(req, cyre).await {
                Ok(response_body) => {
                    response
                        .header(CONTENT_TYPE, "application/json")
                        .body(Body::from(response_body))
                        .unwrap()
                },
                Err(error_body) => {
                    response
                        .status(StatusCode::BAD_REQUEST)
                        .header(CONTENT_TYPE, "application/json")
                        .body(Body::from(error_body))
                        .unwrap()
                }
            }
        },

        // System status endpoint
        (&Method::GET, "/api/status") => {
            let cyre_guard = cyre.read().await;
            let metrics = cyre_guard.get_performance_metrics();
            
            let status = json!({
                "service": "Cyre IoT Smart Home Server",
                "status": "running",
                "cyre_metrics": {
                    "total_executions": metrics.get("total_executions"),
                    "fast_path_ratio": metrics.get("fast_path_ratio"),
                    "active_channels": metrics.get("active_channels")
                },
                "endpoints": [
                    "/health",
                    "/api/iot/events",
                    "/api/iot/temperature",
                    "/api/iot/motion", 
                    "/api/iot/energy",
                    "/api/iot/lighting",
                    "/api/iot/automation",
                    "/api/status"
                ],
                "timestamp": current_timestamp()
            });

            response
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(status.to_string()))
                .unwrap()
        },

        // CORS preflight
        (&Method::OPTIONS, _) => {
            response
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap()
        },

        // 404 for unknown endpoints
        _ => {
            let error = json!({
                "error": "endpoint_not_found",
                "message": format!("Unknown endpoint: {} {}", req.method(), req.uri().path()),
                "available_endpoints": [
                    "GET /health",
                    "GET /api/status", 
                    "POST /api/iot/events",
                    "POST /api/iot/temperature",
                    "POST /api/iot/motion",
                    "POST /api/iot/energy",
                    "POST /api/iot/lighting",
                    "POST /api/iot/automation"
                ]
            });

            response
                .status(StatusCode::NOT_FOUND)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(error.to_string()))
                .unwrap()
        }
    }
}

//=============================================================================
// EVENT PROCESSORS - HTTP -> CYRE
//=============================================================================

/// Process generic IoT events
async fn process_iot_event(
    req: Request<Body>,
    cyre: Arc<RwLock<Cyre>>
) -> Result<String, String> {
    let body_bytes = hyper::body::to_bytes(req.into_body())
        .await
        .map_err(|e| format!("Failed to read request body: {}", e))?;
    
    let event: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let event_type = event.get("event_type")
        .and_then(|v| v.as_str())
        .ok_or("Missing event_type field")?;

    let cyre_guard = cyre.read().await;
    
    let result = match event_type {
        "temperature" => cyre_guard.call("climate.thermostat", event).await,
        "motion" => cyre_guard.call("security.motion", event).await,
        "energy" => cyre_guard.call("energy.monitor", event).await,
        "lighting" => cyre_guard.call("lighting.smart", event).await,
        "automation" => cyre_guard.call("automation.trigger", event).await,
        _ => return Err(format!("Unknown event type: {}", event_type))
    };

    println!("📡 HTTP → Cyre: {} event processed (success: {})", event_type, result.ok);

    let response = json!({
        "success": result.ok,
        "message": result.message,
        "event_type": event_type,
        "processed_at": current_timestamp(),
        "cyre_response": result.payload
    });

    Ok(response.to_string())
}

/// Process temperature events specifically
async fn process_temperature_event(
    req: Request<Body>,
    cyre: Arc<RwLock<Cyre>>
) -> Result<String, String> {
    let body_bytes = hyper::body::to_bytes(req.into_body())
        .await
        .map_err(|e| format!("Failed to read request body: {}", e))?;
    
    let temp_data: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let cyre_guard = cyre.read().await;
    let result = cyre_guard.call("climate.thermostat", temp_data).await;

    println!("🌡️ HTTP → Cyre: Temperature event processed");

    let response = json!({
        "success": result.ok,
        "message": result.message,
        "event_type": "temperature",
        "processed_at": current_timestamp()
    });

    Ok(response.to_string())
}

/// Process motion sensor events
async fn process_motion_event(
    req: Request<Body>,
    cyre: Arc<RwLock<Cyre>>
) -> Result<String, String> {
    let body_bytes = hyper::body::to_bytes(req.into_body())
        .await
        .map_err(|e| format!("Failed to read request body: {}", e))?;
    
    let motion_data: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let cyre_guard = cyre.read().await;
    let result = cyre_guard.call("security.motion", motion_data).await;

    println!("👤 HTTP → Cyre: Motion event processed");

    let response = json!({
        "success": result.ok,
        "message": result.message,
        "event_type": "motion",
        "processed_at": current_timestamp()
    });

    Ok(response.to_string())
}

/// Process energy monitoring events
async fn process_energy_event(
    req: Request<Body>,
    cyre: Arc<RwLock<Cyre>>
) -> Result<String, String> {
    let body_bytes = hyper::body::to_bytes(req.into_body())
        .await
        .map_err(|e| format!("Failed to read request body: {}", e))?;
    
    let energy_data: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let cyre_guard = cyre.read().await;
    let result = cyre_guard.call("energy.monitor", energy_data).await;

    println!("⚡ HTTP → Cyre: Energy event processed");

    let response = json!({
        "success": result.ok,
        "message": result.message,
        "event_type": "energy",
        "processed_at": current_timestamp()
    });

    Ok(response.to_string())
}

/// Process lighting control events
async fn process_lighting_event(
    req: Request<Body>,
    cyre: Arc<RwLock<Cyre>>
) -> Result<String, String> {
    let body_bytes = hyper::body::to_bytes(req.into_body())
        .await
        .map_err(|e| format!("Failed to read request body: {}", e))?;
    
    let lighting_data: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let cyre_guard = cyre.read().await;
    let result = cyre_guard.call("lighting.smart", lighting_data).await;

    println!("💡 HTTP → Cyre: Lighting event processed");

    let response = json!({
        "success": result.ok,
        "message": result.message,
        "event_type": "lighting",
        "processed_at": current_timestamp()
    });

    Ok(response.to_string())
}

/// Process automation trigger events
async fn process_automation_event(
    req: Request<Body>,
    cyre: Arc<RwLock<Cyre>>
) -> Result<String, String> {
    let body_bytes = hyper::body::to_bytes(req.into_body())
        .await
        .map_err(|e| format!("Failed to read request body: {}", e))?;
    
    let automation_data: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let cyre_guard = cyre.read().await;
    let result = cyre_guard.call("automation.trigger", automation_data).await;

    println!("🤖 HTTP → Cyre: Automation event processed");

    let response = json!({
        "success": result.ok,
        "message": result.message,
        "event_type": "automation",
        "processed_at": current_timestamp()
    });

    Ok(response.to_string())
}