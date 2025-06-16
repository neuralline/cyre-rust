// src/bin/iot-client-simulator.rs
// IoT Device Simulator - Generates realistic mock data for Smart Home Demo
// Simulates real IoT devices sending data over HTTP

use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use hyper::{Body, Client, Method, Request, Uri};
use hyper::client::HttpConnector;
use hyper::header::CONTENT_TYPE;
use rand::Rng;

/*

      🌐 IoT DEVICE SIMULATOR
      
      Realistic mock data generation for Smart Home demo:
      - Temperature sensors with daily cycles
      - Smart lights with usage patterns  
      - Motion sensors with realistic triggers
      - Door locks with user activity
      - Energy monitors with device-specific patterns
      - Configurable device behaviors and timing

*/

//=============================================================================
// DEVICE SIMULATORS
//=============================================================================

struct DeviceSimulator {
    client: Client<HttpConnector>,
    server_url: String,
    devices: Vec<SimulatedDevice>,
}

#[derive(Debug, Clone)]
struct SimulatedDevice {
    device_id: String,
    device_type: String,
    room: String,
    endpoint: String,
    update_interval: Duration,
    last_update: u64,
    state: DeviceState,
}

#[derive(Debug, Clone)]
enum DeviceState {
    TemperatureSensor {
        base_temp: f64,
        base_humidity: f64,
        temp_variation: f64,
        humidity_variation: f64,
    },
    SmartLight {
        is_on: bool,
        brightness: f64,
        auto_schedule: bool,
    },
    MotionSensor {
        sensitivity: f64,
        last_motion: u64,
        typical_activity_hours: Vec<u8>, // Hours when motion is common
    },
    DoorLock {
        is_locked: bool,
        users: Vec<String>,
        last_access: u64,
    },
    EnergyMonitor {
        device_name: String,
        base_usage: f64,
        usage_pattern: Vec<f64>, // Hourly usage multipliers
    },
}

impl DeviceSimulator {
    fn new(server_url: String) -> Self {
        Self {
            client: Client::new(),
            server_url,
            devices: Vec::new(),
        }
    }

    fn add_device(&mut self, device: SimulatedDevice) {
        self.devices.push(device);
    }

    async fn run_simulation(&mut self) {
        println!("🌐 Starting IoT Device Simulation...");
        println!("📡 Sending data to: {}", self.server_url);
        println!("🔧 Simulating {} devices", self.devices.len());
        println!();

        // Show device summary
        for device in &self.devices {
            println!("   📱 {} ({}) in {} - Updates every {:?}", 
                device.device_id, device.device_type, device.room, device.update_interval);
        }
        println!();

        let mut cycle_count = 0;
        loop {
            cycle_count += 1;
            println!("🔄 Simulation Cycle #{}", cycle_count);
            
            let current_time = current_timestamp();
            
            // Process devices one by one to avoid borrow checker issues
            let devices_len = self.devices.len();
            for i in 0..devices_len {
                let should_update = {
                    let device = &self.devices[i];
                    current_time - device.last_update >= device.update_interval.as_millis() as u64
                };
                
                if should_update {
                    // Generate data for this device
                    let (device_info, data) = {
                        let device = &self.devices[i];
                        let device_info = (device.device_id.clone(), device.device_type.clone(), device.endpoint.clone());
                        let data = self.generate_device_data_immutable(device, current_time);
                        (device_info, data)
                    };
                    
                    // Update device timestamp
                    self.devices[i].last_update = current_time;
                    
                    // Send the data
                    self.send_device_data_direct(device_info, data).await;
                }
            }
            
            sleep(Duration::from_secs(2)).await;
        }
    }

    async fn send_device_data_direct(&self, device_info: (String, String, String), data: serde_json::Value) {
        let (device_id, device_type, endpoint) = device_info;
        
        let url_str = format!("{}{}", self.server_url, endpoint);
        let uri: Uri = match url_str.parse() {
            Ok(uri) => uri,
            Err(e) => {
                println!("   ❌ Invalid URL: {}", e);
                return;
            }
        };
        
        println!("📤 Sending data from {} ({})", device_id, device_type);
        
        let body = Body::from(data.to_string());
        
        let request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(body);

        let request = match request {
            Ok(req) => req,
            Err(e) => {
                println!("   ❌ Failed to build request: {}", e);
                return;
            }
        };
        
        match self.client.request(request).await {
            Ok(response) => {
                if response.status().is_success() {
                    match hyper::body::to_bytes(response.into_body()).await {
                        Ok(body_bytes) => {
                            match String::from_utf8(body_bytes.to_vec()) {
                                Ok(response_text) => {
                                    if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                                        if let Some(success) = response_json["success"].as_bool() {
                                            if success {
                                                println!("   ✅ Success: {}", response_json["message"].as_str().unwrap_or("OK"));
                                                if let Some(data) = response_json["data"].as_object() {
                                                    if let Some(actions) = data.get("actions_taken") {
                                                        if let Some(actions_array) = actions.as_array() {
                                                            if !actions_array.is_empty() {
                                                                println!("   🤖 Automated actions: {:?}", actions_array);
                                                            }
                                                        }
                                                    }
                                                    if let Some(alerts) = data.get("alerts") {
                                                        if let Some(alerts_array) = alerts.as_array() {
                                                            if !alerts_array.is_empty() {
                                                                println!("   🚨 Alerts triggered: {:?}", alerts_array);
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                println!("   ❌ Server error: {}", response_json["message"].as_str().unwrap_or("Unknown"));
                                            }
                                        }
                                    }
                                },
                                Err(e) => println!("   ❌ Response decode error: {}", e),
                            }
                        },
                        Err(e) => println!("   ❌ Failed to read response body: {}", e),
                    }
                } else {
                    println!("   ❌ HTTP Error: {}", response.status());
                }
            },
            Err(e) => {
                println!("   ❌ Connection error: {}", e);
            }
        }
    }



    fn generate_device_data_immutable(&self, device: &SimulatedDevice, current_time: u64) -> serde_json::Value {
        // Clone the device state to avoid borrow issues
        let mut device_state = device.state.clone();
        
        match &mut device_state {
            DeviceState::TemperatureSensor { base_temp, base_humidity, temp_variation, humidity_variation } => {
                // Simulate daily temperature cycle
                let hour_of_day = ((current_time / 1000) % 86400) / 3600; // Hour of day (0-23)
                let daily_temp_adjustment = 3.0 * ((hour_of_day as f64 - 14.0) / 24.0 * 2.0 * std::f64::consts::PI).sin();
                
                let mut rng = rand::rng();
                let temperature = *base_temp + daily_temp_adjustment + rng.random_range(-*temp_variation..*temp_variation);
                let humidity = (*base_humidity + rng.random_range(-*humidity_variation..*humidity_variation)).clamp(20.0, 95.0);

                println!("   🌡️  Temperature: {:.1}°C, Humidity: {:.1}% in {}", temperature, humidity, device.room);

                json!({
                    "device_id": device.device_id,
                    "device_type": "temperature_sensor",
                    "room": device.room,
                    "temperature": temperature,
                    "humidity": humidity,
                    "battery_level": rng.random_range(75.0..100.0),
                    "timestamp": current_time
                })
            },

            DeviceState::SmartLight { is_on, brightness, auto_schedule } => {
                let mut rng = rand::rng();
                let hour_of_day = ((current_time / 1000) % 86400) / 3600;
                
                // Auto-schedule logic
                if *auto_schedule {
                    let should_be_on = hour_of_day >= 18 || hour_of_day <= 7; // Evening and early morning
                    if should_be_on != *is_on && rng.random_bool(0.3) { // 30% chance to toggle when it should
                        *is_on = should_be_on;
                        *brightness = if *is_on { rng.random_range(60.0..100.0) } else { 0.0 };
                    }
                } else {
                    // Random state changes
                    if rng.random_bool(0.1) { // 10% chance to change state
                        *is_on = !*is_on;
                        *brightness = if *is_on { rng.random_range(30.0..100.0) } else { 0.0 };
                    }
                }

                let action = if *is_on { 
                    if *brightness < 30.0 { "dim" } else { "turn_on" }
                } else { 
                    "turn_off" 
                };

                println!("   💡 Light {}: {} ({}%) in {}", 
                    device.device_id, if *is_on { "ON" } else { "OFF" }, *brightness as u8, device.room);

                json!({
                    "device_id": device.device_id,
                    "device_type": "smart_light",
                    "room": device.room,
                    "action": action,
                    "brightness": *brightness,
                    "is_on": *is_on,
                    "timestamp": current_time
                })
            },

            DeviceState::MotionSensor { sensitivity, last_motion, typical_activity_hours } => {
                let mut rng = rand::rng();
                let hour_of_day = ((current_time / 1000) % 86400) / 3600;
                
                // Higher motion probability during typical activity hours
                let base_probability = if typical_activity_hours.contains(&(hour_of_day as u8)) {
                    0.25 // 25% chance during active hours
                } else {
                    0.05 // 5% chance during quiet hours
                };
                
                let motion_detected = rng.random_bool(base_probability);
                let confidence = if motion_detected {
                    rng.random_range(0.7..1.0)
                } else {
                    rng.random_range(0.0..0.3)
                };

                if motion_detected {
                    *last_motion = current_time;
                    println!("   👤 Motion detected in {} (confidence: {:.2})", device.room, confidence);
                } else {
                    println!("   👁️  No motion in {} (monitoring)", device.room);
                }

                json!({
                    "device_id": device.device_id,
                    "device_type": "motion_sensor",
                    "room": device.room,
                    "motion_detected": motion_detected,
                    "confidence": confidence,
                    "sensitivity": *sensitivity,
                    "timestamp": current_time
                })
            },

            DeviceState::DoorLock { is_locked, users, last_access } => {
                let mut rng = rand::rng();
                let hour_of_day = ((current_time / 1000) % 86400) / 3600;
                
                // More activity during typical entry/exit times
                let activity_probability = if (hour_of_day >= 7 && hour_of_day <= 9) || (hour_of_day >= 17 && hour_of_day <= 19) {
                    0.15 // 15% chance during rush hours
                } else {
                    0.03 // 3% chance otherwise
                };

                if rng.random_bool(activity_probability) {
                    let action = if *is_locked { "unlock" } else { "lock" };
                    *is_locked = !*is_locked;
                    *last_access = current_time;
                    
                    let user_id = if !users.is_empty() && rng.random_bool(0.8) {
                        Some(users[rng.random_range(0..users.len())].clone())
                    } else {
                        None
                    };

                    println!("   🔒 Door {} - {} {}", 
                        device.device_id, 
                        if *is_locked { "LOCKED" } else { "UNLOCKED" },
                        user_id.as_ref().map(|u| format!("by {}", u)).unwrap_or("(manual)".to_string())
                    );

                    json!({
                        "device_id": device.device_id,
                        "device_type": "door_lock",
                        "door_name": device.room,
                        "action": action,
                        "is_locked": *is_locked,
                        "user_id": user_id,
                        "timestamp": current_time
                    })
                } else {
                    // Status check - no action
                    println!("   🔒 Door {} status: {}", device.device_id, if *is_locked { "LOCKED" } else { "UNLOCKED" });
                    
                    json!({
                        "device_id": device.device_id,
                        "device_type": "door_lock",
                        "door_name": device.room,
                        "action": "status_check",
                        "is_locked": *is_locked,
                        "timestamp": current_time
                    })
                }
            },

            DeviceState::EnergyMonitor { device_name, base_usage, usage_pattern } => {
                let hour_of_day = ((current_time / 1000) % 86400) / 3600;
                let hourly_multiplier = usage_pattern.get(hour_of_day as usize).unwrap_or(&1.0);
                
                let mut rng = rand::rng();
                let variation = rng.random_range(0.8..1.2); // ±20% variation
                let power_usage = (*base_usage * hourly_multiplier * variation).max(0.0);

                println!("   ⚡ {} consuming {:.1}W", device_name, power_usage);

                json!({
                    "device_id": device.device_id,
                    "device_type": "energy_monitor",
                    "device_name": device_name,
                    "power_usage": power_usage,
                    "room": device.room,
                    "timestamp": current_time
                })
            }
        }
    }
}

//=============================================================================
// DEVICE CONFIGURATIONS
//=============================================================================

fn create_realistic_devices() -> Vec<SimulatedDevice> {
    vec![
        // Temperature sensors in different rooms
        SimulatedDevice {
            device_id: "temp_sensor_living_room".to_string(),
            device_type: "temperature_sensor".to_string(),
            room: "living_room".to_string(),
            endpoint: "/devices/temperature".to_string(),
            update_interval: Duration::from_secs(30),
            last_update: 0,
            state: DeviceState::TemperatureSensor {
                base_temp: 22.0,
                base_humidity: 45.0,
                temp_variation: 2.0,
                humidity_variation: 5.0,
            },
        },
        SimulatedDevice {
            device_id: "temp_sensor_bedroom".to_string(),
            device_type: "temperature_sensor".to_string(),
            room: "bedroom".to_string(),
            endpoint: "/devices/temperature".to_string(),
            update_interval: Duration::from_secs(45),
            last_update: 0,
            state: DeviceState::TemperatureSensor {
                base_temp: 20.0,
                base_humidity: 50.0,
                temp_variation: 1.5,
                humidity_variation: 3.0,
            },
        },
        SimulatedDevice {
            device_id: "temp_sensor_kitchen".to_string(),
            device_type: "temperature_sensor".to_string(),
            room: "kitchen".to_string(),
            endpoint: "/devices/temperature".to_string(),
            update_interval: Duration::from_secs(20),
            last_update: 0,
            state: DeviceState::TemperatureSensor {
                base_temp: 24.0,
                base_humidity: 55.0,
                temp_variation: 3.0,
                humidity_variation: 8.0,
            },
        },

        // Smart lights
        SimulatedDevice {
            device_id: "light_living_room_main".to_string(),
            device_type: "smart_light".to_string(),
            room: "living_room".to_string(),
            endpoint: "/devices/lights".to_string(),
            update_interval: Duration::from_secs(60),
            last_update: 0,
            state: DeviceState::SmartLight {
                is_on: false,
                brightness: 0.0,
                auto_schedule: true,
            },
        },
        SimulatedDevice {
            device_id: "light_bedroom_bedside".to_string(),
            device_type: "smart_light".to_string(),
            room: "bedroom".to_string(),
            endpoint: "/devices/lights".to_string(),
            update_interval: Duration::from_secs(90),
            last_update: 0,
            state: DeviceState::SmartLight {
                is_on: false,
                brightness: 0.0,
                auto_schedule: true,
            },
        },
        SimulatedDevice {
            device_id: "light_kitchen_under_cabinet".to_string(),
            device_type: "smart_light".to_string(),
            room: "kitchen".to_string(),
            endpoint: "/devices/lights".to_string(),
            update_interval: Duration::from_secs(75),
            last_update: 0,
            state: DeviceState::SmartLight {
                is_on: false,
                brightness: 0.0,
                auto_schedule: false,
            },
        },

        // Motion sensors
        SimulatedDevice {
            device_id: "motion_front_entrance".to_string(),
            device_type: "motion_sensor".to_string(),
            room: "entrance".to_string(),
            endpoint: "/devices/motion".to_string(),
            update_interval: Duration::from_secs(15),
            last_update: 0,
            state: DeviceState::MotionSensor {
                sensitivity: 0.8,
                last_motion: 0,
                typical_activity_hours: vec![7, 8, 9, 17, 18, 19, 20, 21, 22],
            },
        },
        SimulatedDevice {
            device_id: "motion_living_room".to_string(),
            device_type: "motion_sensor".to_string(),
            room: "living_room".to_string(),
            endpoint: "/devices/motion".to_string(),
            update_interval: Duration::from_secs(20),
            last_update: 0,
            state: DeviceState::MotionSensor {
                sensitivity: 0.7,
                last_motion: 0,
                typical_activity_hours: vec![6, 7, 8, 18, 19, 20, 21, 22, 23],
            },
        },
        SimulatedDevice {
            device_id: "motion_garage".to_string(),
            device_type: "motion_sensor".to_string(),
            room: "garage".to_string(),
            endpoint: "/devices/motion".to_string(),
            update_interval: Duration::from_secs(25),
            last_update: 0,
            state: DeviceState::MotionSensor {
                sensitivity: 0.9,
                last_motion: 0,
                typical_activity_hours: vec![7, 8, 17, 18, 19], // Limited garage activity
            },
        },

        // Door locks
        SimulatedDevice {
            device_id: "lock_front_door".to_string(),
            device_type: "door_lock".to_string(),
            room: "front_door".to_string(),
            endpoint: "/devices/door-lock".to_string(),
            update_interval: Duration::from_secs(120),
            last_update: 0,
            state: DeviceState::DoorLock {
                is_locked: true,
                users: vec!["alice".to_string(), "bob".to_string(), "charlie".to_string()],
                last_access: 0,
            },
        },
        SimulatedDevice {
            device_id: "lock_back_door".to_string(),
            device_type: "door_lock".to_string(),
            room: "back_door".to_string(),
            endpoint: "/devices/door-lock".to_string(),
            update_interval: Duration::from_secs(180),
            last_update: 0,
            state: DeviceState::DoorLock {
                is_locked: true,
                users: vec!["alice".to_string(), "bob".to_string()],
                last_access: 0,
            },
        },

        // Energy monitors
        SimulatedDevice {
            device_id: "energy_refrigerator".to_string(),
            device_type: "energy_monitor".to_string(),
            room: "kitchen".to_string(),
            endpoint: "/devices/energy".to_string(),
            update_interval: Duration::from_secs(60),
            last_update: 0,
            state: DeviceState::EnergyMonitor {
                device_name: "Refrigerator".to_string(),
                base_usage: 150.0,
                usage_pattern: vec![1.0; 24], // Constant usage
            },
        },
        SimulatedDevice {
            device_id: "energy_washing_machine".to_string(),
            device_type: "energy_monitor".to_string(),
            room: "laundry".to_string(),
            endpoint: "/devices/energy".to_string(),
            update_interval: Duration::from_secs(30),
            last_update: 0,
            state: DeviceState::EnergyMonitor {
                device_name: "Washing Machine".to_string(),
                base_usage: 500.0,
                // Usage pattern: mostly off, but high usage in mornings and evenings
                usage_pattern: vec![0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.8, 1.5, 0.3, 0.2, 0.2, 0.2, 
                                  0.2, 0.2, 0.2, 0.2, 0.2, 0.2, 1.2, 0.8, 0.3, 0.2, 0.1, 0.1],
            },
        },
        SimulatedDevice {
            device_id: "energy_air_conditioner".to_string(),
            device_type: "energy_monitor".to_string(),
            room: "living_room".to_string(),
            endpoint: "/devices/energy".to_string(),
            update_interval: Duration::from_secs(45),
            last_update: 0,
            state: DeviceState::EnergyMonitor {
                device_name: "Air Conditioner".to_string(),
                base_usage: 1200.0,
                // Usage pattern: higher during day and evening
                usage_pattern: vec![0.2, 0.1, 0.1, 0.1, 0.1, 0.3, 0.5, 0.7, 0.8, 0.9, 1.0, 1.2,
                                  1.3, 1.4, 1.5, 1.4, 1.2, 1.0, 0.9, 0.8, 0.7, 0.5, 0.4, 0.3],
            },
        },
    ]
}

//=============================================================================
// MAIN SIMULATOR
//=============================================================================

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🌐 IoT DEVICE SIMULATOR");
    println!("=======================");
    println!("🏠 Generating realistic Smart Home device data");
    println!("📡 Sending data to Smart Home Server");
    println!();

    let server_url = "http://localhost:3001".to_string();
    println!("🔗 Target server: {}", server_url);
    
    // Test server connection using hyper
    let client = hyper::Client::new();
    let dashboard_uri: hyper::Uri = match format!("{}/dashboard", server_url).parse() {
        Ok(uri) => uri,
        Err(e) => {
            println!("❌ Invalid server URL: {}", e);
            return Ok(());
        }
    };
    
    match client.get(dashboard_uri).await {
        Ok(response) => {
            if response.status().is_success() {
                println!("✅ Server connection successful");
            } else {
                println!("⚠️ Server responded with status: {}", response.status());
            }
        },
        Err(e) => {
            println!("❌ Cannot connect to server: {}", e);
            println!("💡 Make sure the Smart Home Server is running on port 3001");
            println!("   Run: cargo run --bin smart-home-server");
            return Ok(());
        }
    }

    // Initialize simulator with realistic devices
    let mut simulator = DeviceSimulator::new(server_url);
    
    // Add all realistic devices
    let devices = create_realistic_devices();
    for device in devices {
        simulator.add_device(device);
    }
    
    println!();
    println!("🚀 Starting device simulation...");
    println!("⏱️  Updates will be sent at regular intervals");
    println!("🔄 Press Ctrl+C to stop simulation");
    println!();
    
    // Run the simulation
    simulator.run_simulation().await;

    Ok(())
}