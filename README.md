# Cyre Rust

```sh
Neural Line
Reactive event manager
C.Y.R.E ~/`SAYER`/
action-on-call

NPM CYRE'S RUST COUSIN
```

# 🦀 Cyre Rust - High-Performance Reactive Event Manager

**Cyre is a revolutionary channel-based event management system that brings surgical precision to reactive programming.** Unlike traditional pub/sub systems that broadcast noise, Cyre uses addressable channels with intelligent operators for controlled, high-performance communication.

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Performance](https://img.shields.io/badge/Performance-1.8M+%20ops/sec-red.svg)](BENCHMARKS.md)

## 🎯 **Why Cyre? The Architecture That Changes Everything**

### **Traditional Event Systems: Chaos**

```javascript
// Everyone screams, everyone hears - Event soup! 🌊
emitter.emit("USER_CREATED", data); // Goes everywhere
emitter.emit("USER_CREATED", data); // Duplicate processing
emitter.emit("USER_CREATED", data); // System overload
```

### **Cyre: Controlled Precision**

```rust
// Addressable channels with intelligent operators 🎯
cyre.action(IO::new("user.validation")
    .with_throttle(1000)      // "Slow down there, cowboy"
    .with_debounce(250)       // "Let me finish thinking"
    .with_change_detection()  // "Don't repeat yourself"
);

cyre.call("user.validation", data).await;  // Surgical precision
```

## 🏗️ **Revolutionary Channel-Based Architecture**

### **Addressable Communication (Not Broadcast Noise)**

Cyre forces **intentional communication** through channel IDs. No more event soup!

```rust
// Each channel is a precise endpoint
cyre.call("email.send", user_data).await;        // 📧 Email service
cyre.call("analytics.track", user_data).await;   // 📊 Analytics
cyre.call("audit.log", user_data).await;         // 📝 Audit trail
```

### **Channel Operators: Smart Traffic Controllers**

Every channel can have intelligent operators that control behavior:

```rust
// API rate limiting
cyre.action(IO::new("api-requests")
    .with_throttle(1000)     // Max 1 request per second
    .with_change_detection() // Skip duplicate requests
);

// Real-time search with debouncing
cyre.action(IO::new("search")
    .with_debounce(300)      // Wait for user to stop typing
    .with_priority(Priority::High)
);

// Background processing with protection
cyre.action(IO::new("background-job")
    .with_throttle(5000)     // Don't overwhelm system
    .with_priority(Priority::Low)
);
```

### **TimeKeeper: Scheduling That Just Works**

Seamless integration of timing with the same channel architecture:

```rust
// setTimeout equivalent
cyre.action(IO::delayed("notify-user", 5000));

// setInterval equivalent
cyre.action(IO::interval("health-check", 30000));

// Complex scheduling
cyre.action(IO::complex("backup", 1000, 5000, 3)); // delay + interval + repeat
```

## ⚡ **Performance That Crushes Expectations**

### **Fast Path Optimization**

- **Zero overhead** for simple channels
- **Sub-microsecond latency** for hot paths
- **1.8M+ operations/second** sustained throughput

### **Intelligent Protection**

- **Throttling** prevents system overload
- **Debouncing** eliminates duplicate work
- **Change detection** skips unnecessary processing
- **Priority queues** ensure critical tasks execute first

### **Memory Safety Without Cost**

- **Zero garbage collection** pauses
- **Predictable performance** under load
- **Compile-time safety** guarantees
- **Fearless concurrency** with Rust's type system

## 🚀 **Quick Start**

### **Installation**

Add to your `Cargo.toml`:

```toml
[dependencies]
cyre_rust = "0.1.0"
tokio = { version = "1.40", features = ["full"] }
serde_json = "1.0"
```

### **Basic Usage**

```rust
use cyre_rust::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cyre = Cyre::new();

    // 1. Register a channel
    cyre.action(IO::new("greet"));

    // 2. Register a handler
    cyre.on("greet", |payload| {
        Box::pin(async move {
            let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("World");

            CyreResponse {
                ok: true,
                payload: json!({"greeting": format!("Hello, {}!", name)}),
                message: "Greeting generated".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    });

    // 3. Call the channel
    let result = cyre.call("greet", json!({"name": "Rust"})).await;
    println!("Response: {}", result.payload["greeting"]);

    Ok(())
}
```

### **Advanced Features**

```rust
use cyre_rust::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cyre = Cyre::new();
    cyre.init_timekeeper().await?;

    // Protected API endpoint
    cyre.action(IO::new("api-call")
        .with_throttle(1000)        // Rate limiting
        .with_change_detection()    // Skip duplicates
        .with_priority(Priority::High)
    );

    // Scheduled monitoring
    cyre.action(IO::interval("monitor", 5000));  // Every 5 seconds

    // Complex workflow
    cyre.action(IO::complex("backup", 1000, 3000, 5)); // Delay + repeat

    // Register handlers
    cyre.on("api-call", |payload| {
        Box::pin(async move {
            // Your API logic here
            CyreResponse { /* ... */ }
        })
    });

    // Trigger executions
    cyre.call("api-call", json!({"endpoint": "/users"})).await;
    cyre.call("monitor", json!({"system": "database"})).await;

    Ok(())
}
```

## 🧠 **Architectural Insights**

### **It's Like a Telephone System**

- **Channel ID** = Phone number (precise addressing)
- **Operators** = Call routing/filtering (busy signal, call waiting)
- **Handlers** = The person who answers the phone

### **It's Microservices at the Function Level**

Each channel is essentially a micro-service with:

- **Clear interface** (channel ID)
- **Protection policies** (operators)
- **Isolated behavior** (handlers)
- **Independent scaling** (fast path vs protected path)

### **Channel based architecture**

This is exactly what event-driven systems need - controlled chaos with surgical precision!
🎯 What I Love About Cyre's Channel-Based Design

### Addressable Communication (vs Broadcast Noise)

```rust
// Traditional pub/sub: Everyone screams, everyone hears
publisher.emit("USER_CREATED", data); // Goes everywhere 📢

Cyre: Precise channel addressing
cyre.call("user.validation", data).await;

// Surgical precision 🎯
The channel ID requirement is genius - it forces intentional communication architecture instead of the typical event soup!
```

### Channel Operators as Traffic Controllers

```rust
// Each channel is its own intelligent agent
cyre.action(IO::new("api-requests")
.with_throttle(1000) // "Slow down there, cowboy"
.with_debounce(250) // "Let me finish thinking"
.with_change_detection() // "Don't repeat yourself"
);
```

The channel operators are like smart middleware - they understand the context and can make decisions without global coordination. That's architectural poetry!

### Decoupled but Controlled

Unlike traditional pub/sub where:

Publishers spray events everywhere 🌊
Subscribers filter through noise 🔍
No one controls the flow 🤷

### Cyre gives you:

Addressable endpoints (channel IDs)
Smart routing (operators decide behavior)
Flow control (throttle, debounce, block)
State management (change detection)

## 📊 **Performance Benchmarks**

| Operation Type | Ops/Second | Avg Latency | Memory Usage |
| -------------- | ---------- | ----------- | ------------ |
| Fast Path      | 1,867,327  | 0.54μs      | Constant     |
| Protected      | 892,451    | 1.12μs      | Constant     |
| Scheduled      | 245,678    | 2.1μs       | Constant     |
| Complex        | 156,234    | 3.8μs       | Constant     |

**Key Advantages:**

- ✅ **Zero GC pauses** (deterministic performance)
- ✅ **Sub-millisecond latency** consistently
- ✅ **Predictable memory usage** under load
- ✅ **Graceful degradation** with protection systems

## 🎯 **Use Cases**

### **Web APIs with Rate Limiting**

```rust
// Protect your API from abuse
cyre.action(IO::new("user-registration")
    .with_throttle(2000)        // Max 1 registration per 2 seconds
    .with_change_detection()    // Prevent duplicate registrations
);
```

### **Real-time Search**

```rust
// Efficient search-as-you-type
cyre.action(IO::new("search")
    .with_debounce(300)         // Wait for user to stop typing
    .with_priority(Priority::High)
);
```

### **Background Jobs**

```rust
// Scheduled processing
cyre.action(IO::interval("cleanup", 3600000)); // Every hour
cyre.action(IO::delayed("reminder", 86400000)); // 24 hour delay
```

### **IoT Device Management**

```rust
// Handle thousands of sensor readings efficiently
cyre.action(IO::new("sensor-data")
    .with_throttle(100)         // Rate limit per sensor
    .with_change_detection()    // Only process changes
);
```

### **Microservice Communication**

```rust
// Service-to-service calls with protection
cyre.action(IO::new("payment.process")
    .with_throttle(1000)        // Prevent payment spam
    .with_priority(Priority::Critical)
);
```

## 🛠️ **Examples**

Run the comprehensive examples to see Cyre in action:

```bash
# Basic usage
cargo run --example simple_usage

# IoT simulation
cargo run --example smart_home_demo

# TimeKeeper scheduling
cargo run --example timekeeper_demo

# Performance testing
cargo run --example realistic_performance_test

# HTTP server
cargo run --bin cyre-server --features server
```

## 🎭 **Feature Comparison**

| Feature           | Traditional Pub/Sub   | Cyre Channels             |
| ----------------- | --------------------- | ------------------------- |
| **Addressing**    | Broadcast to all      | Precise channel IDs       |
| **Flow Control**  | None                  | Throttle, debounce, block |
| **Performance**   | Variable (GC pauses)  | Predictable (no GC)       |
| **Protection**    | Manual implementation | Built-in operators        |
| **Scheduling**    | External timers       | Integrated TimeKeeper     |
| **Memory Safety** | Runtime errors        | Compile-time guarantees   |
| **Debugging**     | Event soup chaos      | Clear channel traces      |

## 🏗️ **Production Ready**

### **Why Cyre is Ready for Production**

1. **🐛 Debuggable**: Channel IDs make tracing trivial
2. **🧪 Testable**: Mock individual channels easily
3. **📈 Scalable**: Operators prevent system overload
4. **🔧 Maintainable**: Clear separation of concerns
5. **⚡ Performant**: Fast path for simple cases, protection for complex ones

### **Battle-Tested Patterns**

- **Circuit breakers** via throttling
- **Request deduplication** via change detection
- **Load shedding** via priority queues
- **Graceful degradation** under stress

## 📚 **Documentation**

- **[API Reference](docs/api.md)** - Complete API documentation
- **[Performance Guide](docs/performance.md)** - Optimization techniques
- **[Architecture Deep Dive](docs/architecture.md)** - System design details
- **[Examples](examples/)** - Comprehensive usage examples

## 🤝 **Contributing**

We welcome contributions! See our [Contributing Guide](CONTRIBUTING.md) for details.

## 📜 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🎯 **What Makes Cyre Special**

Cyre feels like **"What if we took the best parts of:**

- **Actor Model** (addressable entities)
- **Reactive Streams** (flow control)
- **Microservices** (isolation + contracts)
- **Node.js EventEmitter** (familiar API)

**...and made it type-safe, memory-safe, and actually usable in production?"**

This is **opinionated without being restrictive**, **powerful without being complex**, and **safe without being slow**.

The channel-based approach with operators solves the fundamental "event-driven systems are hard to reason about" problem by making communication **intentional and controllable**.

---

**🚀 Ready to build the future with precision-engineered reactive systems? Give Cyre a try!**

## Origins

Originally evolved from the Quantum-Inception clock project (2016), Cyre has grown into a full-featured event management system while maintaining its quantum timing heritage. The latest evolution introduces Schema, hooks, and standardized execution behavior to provide a more predictable and powerful developer experience.

```sh
Q0.0U0.0A0.0N0.0T0.0U0.0M0 - I0.0N0.0C0.0E0.0P0.0T0.0I0.0O0.0N0.0S0
Expands HORIZONTALLY as your projects grow
```

**CYRE RUST** - Neural Line Reactive Event Manager  
_The fastest, most reliable reactive state management for modern applications._
