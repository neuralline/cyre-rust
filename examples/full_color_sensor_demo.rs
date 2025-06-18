// examples/full_color_sensor_demo.rs
// Demo showing FULL COLOR backgrounds and different timestamp styles

use cyre_rust::context::sensor;
use serde_json::json;

/*

      C.Y.R.E - S.E.N.S.O.R - F.U.L.L - C.O.L.O.R - D.E.M.O
      
      Now the ENTIRE log line gets color/background treatment:
      - [2025-06-18T14:30:45.123Z] LEVEL: action_id - message (ALL colored)
      - Using proper ISO 8601 timestamp format
      - SYS level: full background magenta with white text

*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Full Color Sensor Demo - ENTIRE LINE COLORED");
    println!("===============================================\n");

    // =======================================================================
    // Show full color effect - ENTIRE line gets background/color
    // =======================================================================

    println!("🌈 Full Color Effect - Entire Line Colored:");
    println!("(Each line will have full background color)\n");

    // SYS level - ENTIRE line with magenta background
    sensor::sys("breathing", "Quantum Breathing System starting", true);
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // CRITICAL - ENTIRE line with red background
    sensor::critical("system", "Critical failure detected", Some("system::monitor"), None);
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // ERROR - ENTIRE line with bright red text
    sensor::error("database", "Connection failed", Some("db::pool"), None);
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // SUCCESS - ENTIRE line with bright green
    sensor::success("deploy", "Application deployed successfully", true);
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // WARN - ENTIRE line with bright yellow
    sensor::warn("memory", "Memory usage at 85%", true);
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // INFO - ENTIRE line with cyan
    sensor::info("system", "System ready for requests", true);
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // DEBUG - ENTIRE line with dim cyan
    sensor::debug("cache", "Cache warmed with 1000 entries", true);

    // =======================================================================
    // Show metadata still works with full color
    // =======================================================================

    println!("\n📊 With Metadata (metadata shows normal, line stays colored):");

    sensor::error(
        "api",
        "Request timeout",
        Some("api::handler"),
        Some(
            json!({
            "endpoint": "/api/users",
            "timeout_ms": 5000,
            "retry_count": 3,
            "user_id": "12345"
        })
        )
    );

    // =======================================================================
    // Breathing system simulation with full color
    // =======================================================================

    println!("\n🫁 Breathing System with Full Color:");

    sensor::sys("breathing", "System initialization starting", true);
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    sensor::sys("breathing", "Metrics collection enabled", true);
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    sensor::sys("breathing", "Stress calculation: 0.15 (low)", true);
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    sensor::sys("breathing", "Next breath in 2000ms", true);
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    sensor::sys("breathing", "Breathing cycle complete", true);

    println!("\n✅ Now your logs have FULL COLOR backgrounds!");
    println!("   - Entire line gets the color treatment");
    println!("   - SYS level: Full magenta background");
    println!("   - CRITICAL: Full red background");
    println!("   - Message, timestamp, action_id all colored together");

    // =======================================================================
    // Show different timestamp format examples
    // =======================================================================

    println!("\n⏰ Available Timestamp Formats:");
    println!("   (Change the format_timestamp() method to use different styles)");
    println!();
    println!("   1. ISO 8601 (current):     [2025-06-18T14:30:45.123Z]");
    println!("   2. Unix + Z:               [1750224675.155Z]");
    println!("   3. Time only:              [14:30:45.123]");
    println!("   4. Compact millis:         [1750224675155]");
    println!("   5. Short format:           [675.155]");
    println!();
    println!("   ✅ Currently using ISO 8601 format!");
    println!("   To change timestamp style, modify the format_timestamp() method");
    println!("   to call a different timestamp function (all included in code).");

    Ok(())
}
