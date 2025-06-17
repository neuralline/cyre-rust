// examples/smart-home-server.rs
// IoT Smart Home Management Server using Cyre + Axum
// Clean, modern HTTP server with automatic JSON handling

use cyre_rust::prelude::*;
use axum::{
    extract::{ Path, Query, State },
    http::StatusCode,
    response::Json,
    routing::{ get, post },
    Router,
};
use serde::{ Deserialize, Serialize };
use serde_json::{ json, Value };
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/*
      🏠 SMART HOME IoT SERVER WITH CYRE + AXUM
      
      Modern HTTP server using Axum framework:
      - Clean route definitions with automatic JSON handling
      - Type-safe request/response handling
      - All device logic routed through Cyre channels
      - Real-time IoT device management
      - Intelligent automation and monitoring
*/

//=============================================================================
// REQUEST/RESPONSE TYPES
//=============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct TemperatureData {
    device_id: String,
    temperature: f64,
    humidity: f64,
    room: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LightControl {
    device_id: String,
    room: String,
    state: String, // "on" or "off"
    brightness: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MotionEvent {
    device_id: String,
    location: String,
    detected: bool,
    confidence: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DoorLockEvent {
    device_id: String,
    door: String,
    action: String, // "lock", "unlock", "status"
    user_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EnergyData {
    device_id: String,
    device_name: String,
    consumption_watts: f64,
    room: String,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
    message: String,
    timestamp: u64,
    processing_time_ms: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    page: Option<u32>,
    limit: Option<u32>,
    room: Option<String>,
}

//=============================================================================
// APPLICATION STATE
//=============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeviceState {
    device_id: String,
    device_type: String,
    room: String,
    last_update: u64,
    status: String,
    data: Value,
    is_online: bool,
}

#[derive(Debug)]
struct AppState {
    cyre: Arc<RwLock<Cyre>>,
    devices: Arc<RwLock<HashMap<String, DeviceState>>>,
    start_time: u64,
    total_events: Arc<std::sync::atomic::AtomicU64>,
}

impl AppState {
    async fn new() -> Self {
        let mut cyre = Cyre::new();

        // Initialize TimeKeeper for automation scheduling
        if let Err(e) = cyre.init_timekeeper().await {
            eprintln!("⚠️ TimeKeeper initialization failed: {}", e);
        }

        let state = Self {
            cyre: Arc::new(RwLock::new(cyre)),
            devices: Arc::new(RwLock::new(HashMap::new())),
            start_time: current_timestamp(),
            total_events: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        };

        // Setup Cyre channels for all device types
        state.setup_cyre_channels().await;

        state
    }

    async fn setup_cyre_channels(&self) {
        println!("🔧 Setting up Smart Home Cyre channels...");

        let mut cyre = self.cyre.write().await;

        // Temperature sensor channel
        cyre.action(
            IO::new("device.temperature").with_name("Temperature Sensor").with_throttle(1000)
        ); // Throttle rapid sensor readings

        cyre.on("device.temperature", {
            let devices = Arc::clone(&self.devices);
            move |payload| {
                let devices = Arc::clone(&devices);
                Box::pin(async move {
                    if let Ok(data) = serde_json::from_value::<TemperatureData>(payload.clone()) {
                        // Update device state
                        let mut devices_map = devices.write().await;
                        devices_map.insert(data.device_id.clone(), DeviceState {
                            device_id: data.device_id.clone(),
                            device_type: "temperature_sensor".to_string(),
                            room: data.room.clone(),
                            last_update: current_timestamp(),
                            status: "active".to_string(),
                            data: payload.clone(),
                            is_online: true,
                        });

                        // Smart automation: Alert if temperature too high
                        let alert_level = if data.temperature > 30.0 {
                            "high"
                        } else if data.temperature < 15.0 {
                            "low"
                        } else {
                            "normal"
                        };

                        CyreResponse {
                            ok: true,
                            payload: json!({
                                "device_id": data.device_id,
                                "temperature": data.temperature,
                                "humidity": data.humidity,
                                "room": data.room,
                                "alert_level": alert_level,
                                "comfort_index": calculate_comfort_index(data.temperature, data.humidity)
                            }),
                            message: format!(
                                "Temperature recorded: {}°C in {}",
                                data.temperature,
                                data.room
                            ),
                            error: None,
                            timestamp: current_timestamp(),
                            metadata: Some(json!({"device_type": "temperature_sensor"})),
                        }
                    } else {
                        CyreResponse {
                            ok: false,
                            payload: json!({"error": "Invalid temperature data"}),
                            message: "Failed to parse temperature data".to_string(),
                            error: Some("invalid_data".to_string()),
                            timestamp: current_timestamp(),
                            metadata: None,
                        }
                    }
                })
            }
        });

        // Light control channel
        cyre.action(
            IO::new("device.light").with_name("Smart Light Control").with_change_detection()
        ); // Ignore duplicate commands

        cyre.on("device.light", {
            let devices = Arc::clone(&self.devices);
            move |payload| {
                let devices = Arc::clone(&devices);
                Box::pin(async move {
                    if let Ok(data) = serde_json::from_value::<LightControl>(payload.clone()) {
                        let brightness = data.brightness.unwrap_or(100);
                        let power_usage = if data.state == "on" {
                            ((brightness as f64) / 100.0) * 10.0 // Watts
                        } else {
                            0.0
                        };

                        // Update device state
                        let mut devices_map = devices.write().await;
                        devices_map.insert(data.device_id.clone(), DeviceState {
                            device_id: data.device_id.clone(),
                            device_type: "smart_light".to_string(),
                            room: data.room.clone(),
                            last_update: current_timestamp(),
                            status: data.state.clone(),
                            data: payload.clone(),
                            is_online: true,
                        });

                        CyreResponse {
                            ok: true,
                            payload: json!({
                                "device_id": data.device_id,
                                "room": data.room,
                                "state": data.state,
                                "brightness": brightness,
                                "power_usage": power_usage
                            }),
                            message: format!(
                                "Light {} in {} - brightness {}%",
                                data.state,
                                data.room,
                                brightness
                            ),
                            error: None,
                            timestamp: current_timestamp(),
                            metadata: Some(json!({"device_type": "smart_light"})),
                        }
                    } else {
                        CyreResponse {
                            ok: false,
                            payload: json!({"error": "Invalid light control data"}),
                            message: "Failed to parse light control data".to_string(),
                            error: Some("invalid_data".to_string()),
                            timestamp: current_timestamp(),
                            metadata: None,
                        }
                    }
                })
            }
        });

        // Motion sensor channel
        cyre.action(IO::new("device.motion").with_name("Motion Detection").with_debounce(500)); // Debounce rapid motion events

        cyre.on("device.motion", {
            let devices = Arc::clone(&self.devices);
            move |payload| {
                let devices = Arc::clone(&devices);
                Box::pin(async move {
                    if let Ok(data) = serde_json::from_value::<MotionEvent>(payload.clone()) {
                        // Update device state
                        let mut devices_map = devices.write().await;
                        devices_map.insert(data.device_id.clone(), DeviceState {
                            device_id: data.device_id.clone(),
                            device_type: "motion_sensor".to_string(),
                            room: data.location.clone(),
                            last_update: current_timestamp(),
                            status: (
                                if data.detected {
                                    "motion_detected"
                                } else {
                                    "clear"
                                }
                            ).to_string(),
                            data: payload.clone(),
                            is_online: true,
                        });

                        let security_action = if data.detected {
                            "Motion detected - monitoring"
                        } else {
                            "Area clear"
                        };

                        CyreResponse {
                            ok: true,
                            payload: json!({
                                "device_id": data.device_id,
                                "location": data.location,
                                "motion_detected": data.detected,
                                "confidence": data.confidence.unwrap_or(0.95),
                                "security_action": security_action
                            }),
                            message: format!(
                                "Motion {} in {}",
                                if data.detected {
                                    "detected"
                                } else {
                                    "cleared"
                                },
                                data.location
                            ),
                            error: None,
                            timestamp: current_timestamp(),
                            metadata: Some(json!({"device_type": "motion_sensor"})),
                        }
                    } else {
                        CyreResponse {
                            ok: false,
                            payload: json!({"error": "Invalid motion data"}),
                            message: "Failed to parse motion data".to_string(),
                            error: Some("invalid_data".to_string()),
                            timestamp: current_timestamp(),
                            metadata: None,
                        }
                    }
                })
            }
        });

        // Door lock channel
        cyre.action(
            IO::new("device.door_lock").with_name("Smart Door Lock").with_priority(Priority::High)
        ); // Security is high priority

        cyre.on("device.door_lock", {
            let devices = Arc::clone(&self.devices);
            move |payload| {
                let devices = Arc::clone(&devices);
                Box::pin(async move {
                    if let Ok(data) = serde_json::from_value::<DoorLockEvent>(payload.clone()) {
                        // Update device state
                        let mut devices_map = devices.write().await;
                        devices_map.insert(data.device_id.clone(), DeviceState {
                            device_id: data.device_id.clone(),
                            device_type: "door_lock".to_string(),
                            room: data.door.clone(),
                            last_update: current_timestamp(),
                            status: data.action.clone(),
                            data: payload.clone(),
                            is_online: true,
                        });

                        let security_status = match data.action.as_str() {
                            "lock" => "SECURED",
                            "unlock" => "UNLOCKED",
                            _ => "UNKNOWN",
                        };

                        CyreResponse {
                            ok: true,
                            payload: json!({
                                "device_id": data.device_id,
                                "door": data.door,
                                "action": data.action,
                                "user_id": data.user_id,
                                "security_status": security_status
                            }),
                            message: format!("Door {} {}", data.door, data.action),
                            error: None,
                            timestamp: current_timestamp(),
                            metadata: Some(json!({"device_type": "door_lock"})),
                        }
                    } else {
                        CyreResponse {
                            ok: false,
                            payload: json!({"error": "Invalid door lock data"}),
                            message: "Failed to parse door lock data".to_string(),
                            error: Some("invalid_data".to_string()),
                            timestamp: current_timestamp(),
                            metadata: None,
                        }
                    }
                })
            }
        });

        // Energy monitoring channel
        cyre.action(IO::new("device.energy").with_name("Energy Monitor"));

        cyre.on("device.energy", {
            let devices = Arc::clone(&self.devices);
            move |payload| {
                let devices = Arc::clone(&devices);
                Box::pin(async move {
                    if let Ok(data) = serde_json::from_value::<EnergyData>(payload.clone()) {
                        // Update device state
                        let mut devices_map = devices.write().await;
                        devices_map.insert(data.device_id.clone(), DeviceState {
                            device_id: data.device_id.clone(),
                            device_type: "energy_monitor".to_string(),
                            room: data.room.clone(),
                            last_update: current_timestamp(),
                            status: "monitoring".to_string(),
                            data: payload.clone(),
                            is_online: true,
                        });

                        let hourly_cost = (data.consumption_watts * 0.12) / 1000.0; // $0.12 per kWh
                        let efficiency_rating = if data.consumption_watts < 100.0 {
                            "excellent"
                        } else if data.consumption_watts < 500.0 {
                            "good"
                        } else {
                            "high"
                        };

                        CyreResponse {
                            ok: true,
                            payload: json!({
                                "device_id": data.device_id,
                                "device_name": data.device_name,
                                "room": data.room,
                                "consumption_watts": data.consumption_watts,
                                "hourly_cost": hourly_cost,
                                "efficiency_rating": efficiency_rating
                            }),
                            message: format!(
                                "Energy monitoring: {} using {:.1}W",
                                data.device_name,
                                data.consumption_watts
                            ),
                            error: None,
                            timestamp: current_timestamp(),
                            metadata: Some(json!({"device_type": "energy_monitor"})),
                        }
                    } else {
                        CyreResponse {
                            ok: false,
                            payload: json!({"error": "Invalid energy data"}),
                            message: "Failed to parse energy data".to_string(),
                            error: Some("invalid_data".to_string()),
                            timestamp: current_timestamp(),
                            metadata: None,
                        }
                    }
                })
            }
        });

        println!("✅ Smart Home Cyre channels setup complete");
    }
}

//=============================================================================
// AXUM ROUTE HANDLERS
//=============================================================================

// Temperature sensor endpoint
async fn handle_temperature(
    State(state): State<Arc<AppState>>,
    Json(data): Json<TemperatureData>
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let start_time = std::time::Instant::now();
    state.total_events.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let cyre = state.cyre.read().await;
    let response = cyre.call("device.temperature", json!(data)).await;
    let processing_time = start_time.elapsed().as_secs_f64() * 1000.0;

    if response.ok {
        Ok(
            Json(ApiResponse {
                success: true,
                data: response.payload,
                message: response.message,
                timestamp: response.timestamp,
                processing_time_ms: Some(processing_time),
            })
        )
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// Light control endpoint
async fn handle_lights(
    State(state): State<Arc<AppState>>,
    Json(data): Json<LightControl>
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let start_time = std::time::Instant::now();
    state.total_events.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let cyre = state.cyre.read().await;
    let response = cyre.call("device.light", json!(data)).await;
    let processing_time = start_time.elapsed().as_secs_f64() * 1000.0;

    if response.ok {
        Ok(
            Json(ApiResponse {
                success: true,
                data: response.payload,
                message: response.message,
                timestamp: response.timestamp,
                processing_time_ms: Some(processing_time),
            })
        )
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// Motion detection endpoint
async fn handle_motion(
    State(state): State<Arc<AppState>>,
    Json(data): Json<MotionEvent>
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let start_time = std::time::Instant::now();
    state.total_events.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let cyre = state.cyre.read().await;
    let response = cyre.call("device.motion", json!(data)).await;
    let processing_time = start_time.elapsed().as_secs_f64() * 1000.0;

    if response.ok {
        Ok(
            Json(ApiResponse {
                success: true,
                data: response.payload,
                message: response.message,
                timestamp: response.timestamp,
                processing_time_ms: Some(processing_time),
            })
        )
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// Door lock endpoint
async fn handle_door_lock(
    State(state): State<Arc<AppState>>,
    Json(data): Json<DoorLockEvent>
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let start_time = std::time::Instant::now();
    state.total_events.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let cyre = state.cyre.read().await;
    let response = cyre.call("device.door_lock", json!(data)).await;
    let processing_time = start_time.elapsed().as_secs_f64() * 1000.0;

    if response.ok {
        Ok(
            Json(ApiResponse {
                success: true,
                data: response.payload,
                message: response.message,
                timestamp: response.timestamp,
                processing_time_ms: Some(processing_time),
            })
        )
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// Energy monitoring endpoint
async fn handle_energy(
    State(state): State<Arc<AppState>>,
    Json(data): Json<EnergyData>
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let start_time = std::time::Instant::now();
    state.total_events.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let cyre = state.cyre.read().await;
    let response = cyre.call("device.energy", json!(data)).await;
    let processing_time = start_time.elapsed().as_secs_f64() * 1000.0;

    if response.ok {
        Ok(
            Json(ApiResponse {
                success: true,
                data: response.payload,
                message: response.message,
                timestamp: response.timestamp,
                processing_time_ms: Some(processing_time),
            })
        )
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// Dashboard endpoint - system overview
async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryParams>
) -> Json<ApiResponse<Value>> {
    let devices = state.devices.read().await;
    let uptime = current_timestamp() - state.start_time;
    let total_events = state.total_events.load(std::sync::atomic::Ordering::SeqCst);

    // Filter devices by room if specified
    let filtered_devices: HashMap<_, _> = if let Some(room) = &params.room {
        devices
            .iter()
            .filter(|(_, device)| device.room == *room)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    } else {
        devices
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    };

    let total_devices = filtered_devices.len();
    let online_devices = filtered_devices
        .values()
        .filter(|d| d.is_online)
        .count();

    // Device summary by type
    let mut device_summary = HashMap::new();
    for device in filtered_devices.values() {
        *device_summary.entry(device.device_type.clone()).or_insert(0) += 1;
    }

    // Calculate total energy usage
    let total_energy: f64 = filtered_devices
        .values()
        .filter_map(|device| { device.data.get("consumption_watts").and_then(|v| v.as_f64()) })
        .sum();

    Json(ApiResponse {
        success: true,
        data: json!({
            "system_status": "OPERATIONAL",
            "uptime_seconds": uptime / 1000,
            "total_events_processed": total_events,
            "devices": {
                "total": total_devices,
                "online": online_devices,
                "offline": total_devices - online_devices,
                "by_type": device_summary,
                "details": filtered_devices
            },
            "energy": {
                "total_usage_watts": total_energy,
                "estimated_cost_per_hour": total_energy * 0.12 / 1000.0
            },
            "filter": {
                "room": params.room,
                "page": params.page.unwrap_or(1),
                "limit": params.limit.unwrap_or(10)
            }
        }),
        message: "Dashboard data retrieved successfully".to_string(),
        timestamp: current_timestamp(),
        processing_time_ms: None,
    })
}

// Get specific device info
async fn get_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>
) -> Result<Json<ApiResponse<DeviceState>>, (StatusCode, Json<ApiResponse<String>>)> {
    let devices = state.devices.read().await;

    if let Some(device) = devices.get(&device_id) {
        Ok(
            Json(ApiResponse {
                success: true,
                data: device.clone(),
                message: format!("Device {} found", device_id),
                timestamp: current_timestamp(),
                processing_time_ms: None,
            })
        )
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                data: "Device not found".to_string(),
                message: format!("Device {} not found", device_id),
                timestamp: current_timestamp(),
                processing_time_ms: None,
            }),
        ))
    }
}

//=============================================================================
// UTILITY FUNCTIONS
//=============================================================================

fn calculate_comfort_index(temperature: f64, humidity: f64) -> f64 {
    // Simple comfort index calculation
    let temp_comfort = if temperature >= 20.0 && temperature <= 24.0 { 1.0 } else { 0.5 };
    let humidity_comfort = if humidity >= 40.0 && humidity <= 60.0 { 1.0 } else { 0.5 };
    ((temp_comfort + humidity_comfort) / 2.0) * 100.0
}

//=============================================================================
// MAIN APPLICATION
//=============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏠 SMART HOME IoT SERVER WITH CYRE + AXUM");
    println!("==========================================");
    println!("🚀 Modern HTTP server with type-safe routing");
    println!("⚡ Intelligent automation powered by Cyre");
    println!("🔒 Real-time device management and monitoring");
    println!();

    // Initialize application state
    println!("🔧 Initializing Smart Home system...");
    let app_state = Arc::new(AppState::new().await);
    println!("✅ Smart Home system initialized");

    // Build Axum router with clean, typed routes
    let app = Router::new()
        // Device endpoints
        .route("/devices/temperature", post(handle_temperature))
        .route("/devices/lights", post(handle_lights))
        .route("/devices/motion", post(handle_motion))
        .route("/devices/door-lock", post(handle_door_lock))
        .route("/devices/energy", post(handle_energy))

        // Dashboard and device info
        .route("/dashboard", get(get_dashboard))
        .route("/devices/{device_id}", get(get_device))
        // Add basic CORS headers manually (since tower-http cors feature not enabled)
        .with_state(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;

    println!("🌐 Smart Home IoT Server running on http://localhost:3001");
    println!();
    println!("📡 DEVICE ENDPOINTS:");
    println!("   • POST /devices/temperature     - Temperature & humidity sensors");
    println!("   • POST /devices/lights          - Smart lighting control");
    println!("   • POST /devices/motion          - Motion detection & security");
    println!("   • POST /devices/door-lock       - Smart door lock management");
    println!("   • POST /devices/energy          - Energy usage monitoring");
    println!();
    println!("📊 DASHBOARD ENDPOINTS:");
    println!("   • GET  /dashboard               - System status & analytics");
    println!("   • GET  /dashboard?room=kitchen  - Filter by room");
    println!("   • GET  /devices/:device_id      - Individual device info");
    println!();
    println!("🏠 SMART HOME FEATURES:");
    println!("   ✅ Type-safe request/response handling");
    println!("   ✅ Automatic JSON serialization/deserialization");
    println!("   ✅ Intelligent climate control via Cyre");
    println!("   ✅ Automated lighting management with change detection");
    println!("   ✅ Advanced security monitoring with debouncing");
    println!("   ✅ Energy usage optimization and tracking");
    println!("   ✅ Real-time device status with throttling protection");
    println!();
    println!("🚀 Ready to receive IoT device data!");

    axum::serve(listener, app).await?;

    Ok(())
}
