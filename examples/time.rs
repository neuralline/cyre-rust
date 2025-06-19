// Example usage of the updated TimeKeeper
// This shows how to use the essential TimeKeeper APIs

use cyre_rust::timekeeper::{ get_timekeeper, TimerRepeat, FormationBuilder };
use cyre_rust::utils::current_timestamp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🕒 TimeKeeper Essential API Demo");
    println!("================================");

    let timekeeper = get_timekeeper().await;

    // Example 1: Basic .keep() with all parameters (your required API)
    println!("\n📅 Example 1: Complete .keep() API");
    let breathing_timer = timekeeper.keep(
        1000, // Check every second interval
        || {
            Box::pin(async {
                println!("💨 Breathing system update at {}", current_timestamp());
                // Your breathing update logic here
            })
        },
        true, // repeat infinity
        "system-breathing", // id for tracking progress and cancellation
        Some(2000) // delay, start repetition after 2s delay
    ).await?;

    println!("✅ Breathing timer created: {}", breathing_timer);

    // Example 2: Using .wait() for delays
    println!("\n⏳ Example 2: Using .wait() for async delays");
    println!("Waiting 1 second...");
    timekeeper.wait(1000).await?;
    println!("✅ Wait completed!");

    // Example 3: Complex timer with FormationBuilder
    println!("\n🔧 Example 3: Complex timer with builder pattern");
    let complex_timer = FormationBuilder::new(500) // 500ms interval
        .repeat(TimerRepeat::Count(5)) // Execute 5 times
        .delay(1000) // Start after 1 second
        .id("complex-task")
        .schedule(|| {
            Box::pin(async {
                println!("🔄 Complex task execution at {}", current_timestamp());
            })
        }).await?;

    println!("✅ Complex timer created: {}", complex_timer);

    // Example 4: One-time delayed execution
    println!("\n⏰ Example 4: One-time delayed execution");
    let oneshot_timer = timekeeper.keep(
        0, // Not used for one-time
        || {
            Box::pin(async {
                println!("💥 One-time execution at {}", current_timestamp());
            })
        },
        TimerRepeat::Once, // Execute once
        "oneshot-task",
        Some(1500) // Delay 1.5 seconds
    ).await?;

    println!("✅ One-shot timer created: {}", oneshot_timer);

    // Let things run for a bit
    println!("\n🕐 Letting timers run for 8 seconds...");
    timekeeper.wait(8000).await?;

    // Example 5: Check status
    println!("\n📊 Example 5: TimeKeeper status");
    let status = timekeeper.status();
    println!("Status: {}", serde_json::to_string_pretty(&status)?);

    // Example 6: Forget specific timer
    println!("\n🗑️  Example 6: Forgetting specific timer");
    timekeeper.forget("system-breathing").await;
    println!("✅ Breathing timer cancelled");

    // Example 7: Hibernate (stop all timers)
    println!("\n🛌 Example 7: Hibernating TimeKeeper");
    timekeeper.hibernate().await;
    println!("✅ TimeKeeper hibernated - all timers stopped");

    // Check status after hibernation
    let hibernated_status = timekeeper.status();
    println!("Hibernated status: {}", serde_json::to_string_pretty(&hibernated_status)?);

    // Example 8: Reset (clear everything and wake up)
    println!("\n🔄 Example 8: Resetting TimeKeeper");
    timekeeper.reset().await;
    println!("✅ TimeKeeper reset - ready for new timers");

    // Final status
    let final_status = timekeeper.status();
    println!("Final status: {}", serde_json::to_string_pretty(&final_status)?);

    println!("\n🎉 TimeKeeper Demo Complete!");
    println!("=====================================");
    println!("✅ .keep() - ✅ .forget() - ✅ .wait()");
    println!("✅ .hibernate() - ✅ .reset() - ✅ .status()");

    Ok(())
}
