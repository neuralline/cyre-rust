// examples/simple_sensor_demo.rs
// Simple demonstration of the sensor logging system

use cyre_rust::context::sensor;
use serde_json::json;

/*

      C.Y.R.E - S.E.N.S.O.R - S.I.M.P.L.E - D.E.M.O
      
      Simple sensor usage:
      - Only error/critical log by default
      - Others need force_log=true
      - Uses your exact color theme
      - Format: [timestamp] LEVEL: action_id - message

*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Simple Sensor Demo");
    println!("====================\n");

    // =======================================================================
    // These will NOT log to terminal (only error/critical log by default)
    // =======================================================================

    println!("📝 Testing non-logging calls (won't appear):");
    sensor::success("user-action", "User logged in successfully", false);
    sensor::info("system", "System is running normally", false);
    sensor::warn("memory", "Memory usage at 75%", false);
    sensor::debug("db-query", "SELECT * FROM users executed", false);
    sensor::sys("system", "System already initialized", false);

    // =======================================================================
    // These WILL log to terminal (error/critical auto-log)
    // =======================================================================

    println!("\n🚨 These will log automatically:");

    // Error logging (auto-enabled)
    sensor::error(
        "payment-gateway",
        "Connection timeout",
        Some("payment::process_card"),
        Some(json!({
            "timeout_ms": 30000,
            "retry_count": 3
        }))
    );

    // Critical logging (auto-enabled)
    sensor::critical(
        "database",
        "Connection pool exhausted",
        Some("db::connection_manager"),
        Some(
            json!({
            "active_connections": 100,
            "max_connections": 100,
            "queue_length": 25
        })
        )
    );

    // =======================================================================
    // Force logging for non-error levels
    // =======================================================================

    println!("\n✅ Forced logging (force_log=true):");

    sensor::success("deployment", "Application deployed successfully", true);
    sensor::info("startup", "Cyre system initialized", true);
    sensor::warn("performance", "Response time above threshold", true);
    sensor::debug("cache", "Cache hit ratio: 95%", true);
    sensor::sys("system", "System health check completed", true);

    // =======================================================================
    // General log method examples
    // =======================================================================

    println!("\n🔧 Using general log method:");

    // Won't log (force_log=false, not error/critical)
    sensor::log("api", sensor::LogLevel::INFO, "API request processed", None::<&str>, false, None);

    // Will log (force_log=true)
    sensor::log(
        "background-job",
        sensor::LogLevel::SUCCESS,
        "Cleanup job completed",
        Some("scheduler::cleanup"),
        true,
        Some(json!({
            "files_deleted": 150,
            "space_freed_mb": 2048
        }))
    );

    // Will log (critical auto-logs)
    sensor::log(
        "security",
        sensor::LogLevel::CRITICAL,
        "Multiple failed login attempts detected",
        Some("auth::security_monitor"),
        false, // Even false here, critical still logs
        Some(
            json!({
            "failed_attempts": 5,
            "source_ip": "192.168.1.100",
            "timeframe_minutes": 2
        })
        )
    );

    println!("\n📊 Demo Summary:");
    println!("================");
    println!("🔴 ERROR/CRITICAL: Log automatically");
    println!("🟡 SUCCESS/INFO/WARN/DEBUG/SYS: Need force_log=true");
    println!("🎨 Colors match your exact theme");
    println!("📝 Format: [timestamp] LEVEL: action_id - message");

    Ok(())
}

//=============================================================================
// Integration with your existing Cyre modules
//=============================================================================

// Example of how to add to src/context/mod.rs:
/*

pub mod sensor;

// Re-export sensor functions
pub use sensor::{
    log as sensor_log,
    success as sensor_success,
    error as sensor_error,
    warn as sensor_warn,
    info as sensor_info,
    debug as sensor_debug,
    critical as sensor_critical,
    sys as sensor_sys,
    LogLevel,
    Sensor,
    SENSOR,
};

*/

// Example usage in your core modules:
/*

use crate::context::sensor;

impl Cyre {
    pub async fn call(&self, action_id: &str, payload: ActionPayload) -> CyreResponse {
        // Only errors will log automatically
        sensor::debug("core", "Action called", false); // Won't log
        
        let result = self.internal_call(action_id, payload).await;
        
        if result.ok {
            sensor::success("core", "Action completed", false); // Won't log unless forced
        } else {
            sensor::error("core", "Action failed", Some("core::call"), None); // Will log
        }
        
        result
    }
}

*/
