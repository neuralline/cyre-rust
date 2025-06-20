// debug_transform.rs - Debug the transform operator issue

use cyre_rust::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("🔍 DEBUG: Transform Operator Pipeline Issue");
  println!("==========================================");

  // Create and initialize Cyre
  let mut cyre = Cyre::new();
  cyre.init().await?;

  // Create action with ONLY transform - no other fields
  let config = IO::new("debug-transform").with_transform("add-two".to_string());

  // DEBUG: Print the IO config BEFORE compilation
  println!("\n📋 IO Config BEFORE compilation:");
  println!("   ID: {}", config.id);
  println!("   Block field: {}", config.block); // Should be false
  println!("   Transform field: {:?}", config.transform); // Should be Some("add-two")
  println!("   Throttle field: {:?}", config.throttle); // Should be None
  println!("   Debounce field: {:?}", config.debounce); // Should be None

  // Check what fields actually have values using serde serialization
  if let Ok(json_value) = serde_json::to_value(&config) {
    if let Some(obj) = json_value.as_object() {
      println!("\n🔍 FIELDS WITH NON-NULL VALUES:");
      for (key, value) in obj {
        if
          !value.is_null() &&
          !key.starts_with('_') &&
          key != "additional_properties" &&
          !(value.is_string() && value.as_str().unwrap_or("").is_empty()) &&
          !(value.is_array() && value.as_array().unwrap().is_empty()) &&
          !(value.is_object() && value.as_object().unwrap().is_empty())
        {
          println!("   {}: {}", key, value);
        }
      }
    }
  }

  // Register the action and see what happens during compilation
  println!("\n⚙️ REGISTERING ACTION (this triggers compilation)...");
  match cyre.action(config) {
    Ok(_) => println!("✅ Action registered successfully"),
    Err(e) => {
      println!("❌ Action registration failed: {}", e);
      return Ok(());
    }
  }

  // Get the compiled action back from storage to see what pipeline was created
  if let Some(compiled_action) = cyre.get("debug-transform") {
    println!("\n📋 COMPILED ACTION INFO:");
    println!("   _has_fast_path: {}", compiled_action._has_fast_path);
    println!("   _has_protections: {}", compiled_action._has_protections);
    println!("   _has_scheduling: {}", compiled_action._has_scheduling);
    println!("   _pipeline: {:?}", compiled_action._pipeline);

    // This is the smoking gun - what's in the pipeline?
    if compiled_action._pipeline.is_empty() {
      println!("   🚀 FAST PATH: No pipeline operators");
    } else {
      println!("   🔧 PIPELINE PATH: {} operators", compiled_action._pipeline.len());
      for (i, op) in compiled_action._pipeline.iter().enumerate() {
        println!("     {}. {:?}", i + 1, op);
      }
    }
  } else {
    println!("❌ Could not retrieve compiled action from storage");
    return Ok(());
  }

  // Register handler
  cyre.on("debug-transform", |payload| {
    Box::pin(async move {
      println!("📝 Handler received payload: {}", payload);
      CyreResponse::success(
        json!({"handler_processed": true, "original": payload}),
        "Handler completed"
      )
    })
  })?;

  // Test the action
  println!("\n🧪 TESTING ACTION EXECUTION:");
  let test_payload = json!({"value": 5, "name": "test"});
  println!("   Input payload: {}", test_payload);

  let response = cyre.call("debug-transform", test_payload).await;

  println!("\n🎯 EXECUTION RESULT:");
  println!("   Success: {}", response.ok);
  println!("   Message: {}", response.message);
  println!("   Payload: {}", response.payload);
  if let Some(error) = response.error {
    println!("   Error: {}", error);
  }

  Ok(())
}
