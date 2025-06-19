// examples/trading_bot_demo.rs
// Real-time Trading Bot with Advanced Risk Management
// Demonstrates Cyre's pipeline processing, protection mechanisms, and complex business logic

use cyre_rust::prelude::*;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("📈 CYRE TRADING BOT DEMO");
  println!("=========================");
  println!("Advanced algorithmic trading with real-time risk management");
  println!("Showcasing Cyre's pipeline processing and protection mechanisms");
  println!();

  // Initialize Cyre system
  let mut cyre = Cyre::new();
  let init_result = cyre.init().await?;

  println!("✅ Trading Bot initialized successfully");
  println!("   Instance ID: {}", init_result.payload["instance_id"]);
  println!("   Performance Mode: {}", init_result.payload["performance_mode"]);
  println!();

  // Setup trading system
  setup_trading_system(&mut cyre).await?;

  // Run trading scenarios
  println!("🚀 Starting Trading Scenarios...");
  println!();

  // Scenario 1: Market Data Processing
  market_data_processing_demo(&cyre).await?;

  // Scenario 2: Risk Management
  risk_management_demo(&cyre).await?;

  // Scenario 3: Order Execution
  order_execution_demo(&cyre).await?;

  // Scenario 4: Portfolio Management
  portfolio_management_demo(&cyre).await?;

  // Scenario 5: Real-time Monitoring
  real_time_monitoring_demo(&cyre).await?;

  // Scenario 6: Stress Testing
  stress_testing_demo(&cyre).await?;

  println!("🎉 Trading Bot Demo Complete!");
  println!("✨ Cyre handled all trading operations with advanced risk management!");

  Ok(())
}

//=============================================================================
// TRADING SYSTEM SETUP
//=============================================================================

async fn setup_trading_system(cyre: &mut Cyre) -> Result<(), Box<dyn std::error::Error>> {
  println!("🔧 Setting up Advanced Trading System...");

  // =====================================================
  // MARKET DATA PROCESSING
  // =====================================================

  // Real-time market data ingestion with validation and transformation
  cyre.action(
    IO::new("market.data.ingest")
      .with_name("Market Data Ingestion")
      .with_priority(Priority::High)
      .with_throttle(100) // Prevent overwhelming
      .with_required(true) // Must have valid data
      .with_condition("valid_market_data") // Business validation
      .with_transform("normalize_market_data") // Data transformation
      .with_logging(true) // Audit trail
  )?;

  cyre.on("market.data.ingest", |payload| {
    Box::pin(async move {
      let symbol = payload
        .get("symbol")
        .and_then(|v| v.as_str())
        .unwrap_or("UNKNOWN");
      let price = payload
        .get("price")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
      let volume = payload
        .get("volume")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
      let timestamp = payload
        .get("timestamp")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

      println!("📊 Market Data: {} @ ${:.2} (vol: {})", symbol, price, volume);

      // Simulate processing time
      sleep(Duration::from_millis(5)).await;

      CyreResponse::success(
        json!({
                    "symbol": symbol,
                    "price": price,
                    "volume": volume,
                    "timestamp": timestamp,
                    "processed_at": current_timestamp(),
                    "data_quality": "high",
                    "latency_ms": 5
                }),
        format!("Market data processed for {}", symbol)
      )
    })
  })?;

  // =====================================================
  // RISK MANAGEMENT SYSTEM
  // =====================================================

  // Position risk assessment with multiple protection layers
  cyre.action(
    IO::new("risk.assess")
      .with_name("Risk Assessment Engine")
      .with_priority(Priority::Critical)
      .with_throttle(50) // Critical operation - limit frequency
      .with_required(true)
      .with_condition("valid_position_data")
      .with_transform("calculate_risk_metrics")
      .with_logging(true)
  )?;

  cyre.on("risk.assess", |payload| {
    Box::pin(async move {
      let symbol = payload
        .get("symbol")
        .and_then(|v| v.as_str())
        .unwrap_or("UNKNOWN");
      let position_size = payload
        .get("position_size")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
      let current_price = payload
        .get("current_price")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
      let account_value = payload
        .get("account_value")
        .and_then(|v| v.as_f64())
        .unwrap_or(100000.0);

      // Calculate risk metrics
      let position_value = position_size * current_price;
      let risk_percentage = (position_value / account_value) * 100.0;
      let var_95 = position_value * 0.02; // 2% VaR at 95% confidence
      let max_loss = position_value * 0.1; // 10% max loss

      let risk_level = if risk_percentage > 5.0 {
        "HIGH"
      } else if risk_percentage > 2.0 {
        "MEDIUM"
      } else {
        "LOW"
      };

      println!(
        "⚠️  Risk Assessment: {} - {}% of portfolio (Risk: {})",
        symbol,
        risk_percentage.round(),
        risk_level
      );

      CyreResponse::success(
        json!({
                    "symbol": symbol,
                    "position_size": position_size,
                    "position_value": position_value,
                    "risk_percentage": risk_percentage,
                    "var_95": var_95,
                    "max_loss": max_loss,
                    "risk_level": risk_level,
                    "assessment_timestamp": current_timestamp()
                }),
        format!("Risk assessment completed for {}", symbol)
      )
    })
  })?;

  // =====================================================
  // ORDER EXECUTION SYSTEM
  // =====================================================

  // Smart order routing with protection mechanisms
  cyre.action(
    IO::new("order.execute")
      .with_name("Order Execution Engine")
      .with_priority(Priority::Critical)
      .with_throttle(10) // Very strict throttling for orders
      .with_required(true)
      .with_condition("valid_order")
      .with_transform("validate_order_parameters")
      .with_logging(true)
  )?;

  cyre.on("order.execute", |payload| {
    Box::pin(async move {
      let symbol = payload
        .get("symbol")
        .and_then(|v| v.as_str())
        .unwrap_or("UNKNOWN");
      let side = payload
        .get("side")
        .and_then(|v| v.as_str())
        .unwrap_or("BUY");
      let quantity = payload
        .get("quantity")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
      let price = payload
        .get("price")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
      let order_type = payload
        .get("order_type")
        .and_then(|v| v.as_str())
        .unwrap_or("MARKET");

      println!("📋 Order: {} {} {} @ ${:.2}", side, quantity, symbol, price);

      // Simulate order execution
      sleep(Duration::from_millis(20)).await;

      let execution_price = if order_type == "MARKET" {
        price * (1.0 + (rand::random::<f64>() - 0.5) * 0.001) // Slight price improvement
      } else {
        price
      };

      let order_id = format!("ORD_{}", current_timestamp());

      CyreResponse::success(
        json!({
                    "order_id": order_id,
                    "symbol": symbol,
                    "side": side,
                    "quantity": quantity,
                    "requested_price": price,
                    "execution_price": execution_price,
                    "order_type": order_type,
                    "status": "FILLED",
                    "execution_time_ms": 20,
                    "timestamp": current_timestamp()
                }),
        format!("Order executed: {} {} {}", side, quantity, symbol)
      )
    })
  })?;

  // =====================================================
  // PORTFOLIO MANAGEMENT
  // =====================================================

  // Portfolio rebalancing with intelligent allocation
  cyre.action(
    IO::new("portfolio.rebalance")
      .with_name("Portfolio Rebalancing")
      .with_priority(Priority::Normal)
      .with_throttle(300) // Rebalance every 5 minutes max
      .with_required(true)
      .with_condition("rebalance_needed")
      .with_transform("calculate_optimal_allocation")
      .with_logging(true)
  )?;

  cyre.on("portfolio.rebalance", |payload| {
    Box::pin(async move {
      let empty_map = serde_json::Map::new();
      let current_allocation = payload
        .get("current_allocation")
        .and_then(|v| v.as_object())
        .unwrap_or(&empty_map);
      let target_allocation = payload
        .get("target_allocation")
        .and_then(|v| v.as_object())
        .unwrap_or(&empty_map);
      let total_value = payload
        .get("total_value")
        .and_then(|v| v.as_f64())
        .unwrap_or(100000.0);

      println!("⚖️  Portfolio Rebalancing: ${:.2} total value", total_value);

      // Calculate rebalancing trades
      let mut rebalancing_trades = Vec::new();
      for (symbol, target_pct) in target_allocation {
        let target_value = total_value * target_pct.as_f64().unwrap_or(0.0);
        let current_value = current_allocation
          .get(symbol)
          .and_then(|v| v.as_f64())
          .unwrap_or(0.0);
        let difference = target_value - current_value;

        if difference.abs() > total_value * 0.01 {
          // 1% threshold
          rebalancing_trades.push(
            json!({
                        "symbol": symbol,
                        "action": if difference > 0.0 { "BUY" } else { "SELL" },
                        "quantity": (difference.abs() / 100.0).round(), // Assuming $100 per share
                        "value": difference.abs()
                    })
          );
        }
      }

      CyreResponse::success(
        json!({
                    "total_value": total_value,
                    "rebalancing_trades": rebalancing_trades,
                    "trade_count": rebalancing_trades.len(),
                    "rebalance_timestamp": current_timestamp()
                }),
        format!("Portfolio rebalancing calculated with {} trades", rebalancing_trades.len())
      )
    })
  })?;

  // =====================================================
  // REAL-TIME MONITORING
  // =====================================================

  // Performance monitoring with alerting
  cyre.action(
    IO::new("monitor.performance")
      .with_name("Performance Monitor")
      .with_priority(Priority::Normal)
      .with_throttle(1000) // Monitor every 10 seconds
      .with_required(true)
      .with_condition("performance_data_available")
      .with_transform("calculate_performance_metrics")
      .with_logging(true)
  )?;

  cyre.on("monitor.performance", |payload| {
    Box::pin(async move {
      let pnl = payload
        .get("pnl")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
      let win_rate = payload
        .get("win_rate")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
      let sharpe_ratio = payload
        .get("sharpe_ratio")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
      let max_drawdown = payload
        .get("max_drawdown")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

      println!(
        "📊 Performance: P&L: ${:.2}, Win Rate: {:.1}%, Sharpe: {:.2}",
        pnl,
        win_rate * 100.0,
        sharpe_ratio
      );

      // Generate alerts for poor performance
      let mut alerts = Vec::new();
      if pnl < -1000.0 {
        alerts.push("High losses detected".to_string());
      }
      if win_rate < 0.4 {
        alerts.push("Low win rate warning".to_string());
      }
      if max_drawdown > 0.1 {
        alerts.push("Maximum drawdown exceeded".to_string());
      }

      CyreResponse::success(
        json!({
                    "pnl": pnl,
                    "win_rate": win_rate,
                    "sharpe_ratio": sharpe_ratio,
                    "max_drawdown": max_drawdown,
                    "alerts": alerts,
                    "monitor_timestamp": current_timestamp()
                }),
        format!("Performance monitored - {} alerts generated", alerts.len())
      )
    })
  })?;

  println!("✅ Trading system setup complete!");
  println!();

  Ok(())
}

//=============================================================================
// TRADING SCENARIOS
//=============================================================================

async fn market_data_processing_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
  println!("📈 SCENARIO 1: Market Data Processing");
  println!("=====================================");

  // Simulate high-frequency market data
  let symbols = vec!["AAPL", "GOOGL", "TSLA", "MSFT", "AMZN"];
  let mut iteration = 0;

  for _ in 0..5 {
    for symbol in &symbols {
      let price = 100.0 + (rand::random::<f64>() - 0.5) * 20.0;
      let volume = ((rand::random::<f64>() * 10000.0) as u64) + 1000;

      let result = cyre.call(
        "market.data.ingest",
        json!({
                    "symbol": symbol,
                    "price": price,
                    "volume": volume,
                    "timestamp": current_timestamp(),
                    "source": "real_time_feed"
                })
      ).await;

      if result.ok {
        println!("   ✅ {}: ${:.2} (vol: {})", symbol, price, volume);
      } else {
        println!("   ❌ {}: Failed - {}", symbol, result.message);
      }

      iteration += 1;
      if iteration % 10 == 0 {
        sleep(Duration::from_millis(100)).await; // Brief pause
      }
    }
  }

  println!("   📊 Processed {} market data points", iteration);
  println!();

  Ok(())
}

async fn risk_management_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
  println!("⚠️  SCENARIO 2: Risk Management");
  println!("===============================");

  // Test different position scenarios
  let scenarios = vec![
    ("AAPL", 100.0, 150.0, 100000.0), // Normal position
    ("TSLA", 500.0, 200.0, 50000.0), // High risk position
    ("GOOGL", 50.0, 120.0, 200000.0) // Low risk position
  ];

  for (symbol, position_size, current_price, account_value) in scenarios {
    let result = cyre.call(
      "risk.assess",
      json!({
                "symbol": symbol,
                "position_size": position_size,
                "current_price": current_price,
                "account_value": account_value,
                "assessment_type": "real_time"
            })
    ).await;

    if result.ok {
      let risk_level = result.payload["risk_level"].as_str().unwrap_or("UNKNOWN");
      let risk_pct = result.payload["risk_percentage"].as_f64().unwrap_or(0.0);
      println!("   ✅ {}: {}% risk ({})", symbol, risk_pct.round(), risk_level);
    } else {
      println!("   ❌ {}: Risk assessment failed - {}", symbol, result.message);
    }
  }

  println!();

  Ok(())
}

async fn order_execution_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
  println!("📋 SCENARIO 3: Order Execution");
  println!("==============================");

  // Test different order types
  let orders = vec![
    ("AAPL", "BUY", 10.0, 150.0, "MARKET"),
    ("TSLA", "SELL", 5.0, 200.0, "LIMIT"),
    ("GOOGL", "BUY", 20.0, 120.0, "MARKET")
  ];

  for (symbol, side, quantity, price, order_type) in orders {
    let result = cyre.call(
      "order.execute",
      json!({
                "symbol": symbol,
                "side": side,
                "quantity": quantity,
                "price": price,
                "order_type": order_type,
                "time_in_force": "DAY"
            })
    ).await;

    if result.ok {
      let order_id = result.payload["order_id"].as_str().unwrap_or("UNKNOWN");
      let execution_price = result.payload["execution_price"].as_f64().unwrap_or(0.0);
      println!(
        "   ✅ {}: {} {} @ ${:.2} (ID: {})",
        side,
        quantity,
        symbol,
        execution_price,
        order_id
      );
    } else {
      println!("   ❌ {}: Order failed - {}", symbol, result.message);
    }
  }

  println!();

  Ok(())
}

async fn portfolio_management_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
  println!("⚖️  SCENARIO 4: Portfolio Management");
  println!("====================================");

  // Simulate portfolio rebalancing
  let result = cyre.call(
    "portfolio.rebalance",
    json!({
            "current_allocation": {
                "AAPL": 0.3,
                "GOOGL": 0.25,
                "TSLA": 0.2,
                "MSFT": 0.15,
                "CASH": 0.1
            },
            "target_allocation": {
                "AAPL": 0.25,
                "GOOGL": 0.3,
                "TSLA": 0.15,
                "MSFT": 0.2,
                "CASH": 0.1
            },
            "total_value": 100000.0
        })
  ).await;

  if result.ok {
    let trade_count = result.payload["trade_count"].as_u64().unwrap_or(0);
    let total_value = result.payload["total_value"].as_f64().unwrap_or(0.0);
    println!("   ✅ Portfolio: ${:.2} total value", total_value);
    println!("   📊 Rebalancing: {} trades required", trade_count);

    if let Some(trades) = result.payload["rebalancing_trades"].as_array() {
      for trade in trades {
        let symbol = trade["symbol"].as_str().unwrap_or("UNKNOWN");
        let action = trade["action"].as_str().unwrap_or("UNKNOWN");
        let quantity = trade["quantity"].as_f64().unwrap_or(0.0);
        println!("      {} {} {}", action, quantity, symbol);
      }
    }
  } else {
    println!("   ❌ Portfolio rebalancing failed - {}", result.message);
  }

  println!();

  Ok(())
}

async fn real_time_monitoring_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
  println!("📊 SCENARIO 5: Real-time Monitoring");
  println!("===================================");

  // Simulate performance monitoring
  for _i in 0..3 {
    let pnl = (rand::random::<f64>() - 0.5) * 2000.0; // Random P&L
    let win_rate = 0.4 + rand::random::<f64>() * 0.4; // 40-80% win rate
    let sharpe_ratio = rand::random::<f64>() * 2.0; // 0-2 Sharpe ratio
    let max_drawdown = rand::random::<f64>() * 0.15; // 0-15% max drawdown

    let result = cyre.call(
      "monitor.performance",
      json!({
                "pnl": pnl,
                "win_rate": win_rate,
                "sharpe_ratio": sharpe_ratio,
                "max_drawdown": max_drawdown,
                "period": "daily"
            })
    ).await;

    if result.ok {
      let alerts_vec = Vec::new();
      let alerts = result.payload["alerts"].as_array().unwrap_or(&alerts_vec);
      println!("   📊 Performance: P&L: ${:.2}, Win Rate: {:.1}%", pnl, win_rate * 100.0);
      if !alerts.is_empty() {
        println!("   ⚠️  Alerts: {}", alerts.len());
      }
    } else {
      println!("   ❌ Performance monitoring failed - {}", result.message);
    }

    sleep(Duration::from_millis(500)).await;
  }

  println!();

  Ok(())
}

async fn stress_testing_demo(cyre: &Cyre) -> Result<(), Box<dyn std::error::Error>> {
  println!("🔥 SCENARIO 6: Stress Testing");
  println!("=============================");

  // Test system under high load
  println!("   🚀 Running high-frequency trading simulation...");

  let start_time = std::time::Instant::now();
  let mut success_count = 0;
  let mut failure_count = 0;

  // Rapid-fire market data
  for i in 0..100 {
    let symbol = if i % 4 == 0 {
      "AAPL"
    } else if i % 4 == 1 {
      "GOOGL"
    } else if i % 4 == 2 {
      "TSLA"
    } else {
      "MSFT"
    };
    let price = 100.0 + (rand::random::<f64>() - 0.5) * 10.0;

    let result = cyre.call(
      "market.data.ingest",
      json!({
                "symbol": symbol,
                "price": price,
                "volume": 1000,
                "timestamp": current_timestamp(),
                "stress_test": true
            })
    ).await;

    if result.ok {
      success_count += 1;
    } else {
      failure_count += 1;
    }

    // Every 20 iterations, try a risk assessment
    if i % 20 == 0 {
      let _risk_result = cyre.call(
        "risk.assess",
        json!({
                    "symbol": symbol,
                    "position_size": 100.0,
                    "current_price": price,
                    "account_value": 100000.0
                })
      ).await;
    }
  }

  let duration = start_time.elapsed();
  let throughput = ((success_count + failure_count) as f64) / duration.as_secs_f64();

  println!("   📊 Stress Test Results:");
  println!("      Duration: {:.2}s", duration.as_secs_f64());
  println!("      Success: {}", success_count);
  println!("      Failures: {}", failure_count);
  println!("      Throughput: {:.0} ops/sec", throughput);
  println!(
    "      Success Rate: {:.1}%",
    ((success_count as f64) / ((success_count + failure_count) as f64)) * 100.0
  );

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

// Mock random number generator for demo purposes
mod rand {
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{ Hash, Hasher };

  pub fn random<T>() -> T where T: From<f64> {
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    let hash = hasher.finish();
    let normalized = ((hash % 1000) as f64) / 1000.0;
    T::from(normalized)
  }
}
