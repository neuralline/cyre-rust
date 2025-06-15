# Cyre Rust - Modular Architecture Guide

## 🏗️ Modular Structure Overview

The Rust implementation of Cyre uses a clean modular architecture that provides better maintainability, testability, and extensibility while preserving all performance optimizations.

## 📁 Project Structure

```
src/
├── lib.rs              # Main entry point and public API
├── types/              # Core type definitions
│   ├── mod.rs          # Module exports
│   ├── core.rs         # Fundamental types
│   ├── io.rs           # Action configuration
│   └── priority.rs     # Priority system
├── protection/         # Protection mechanisms
│   ├── mod.rs          # Module exports
│   └── state.rs        # Protection implementation
├── talent/             # Talent system (advanced processing)
│   ├── mod.rs          # Module exports
│   ├── types.rs        # Talent definitions
│   ├── registry.rs     # Talent management
│   └── functions.rs    # Common talent functions
├── core/               # Main Cyre implementation
│   ├── mod.rs          # Module exports
│   └── cyre.rs         # Core functionality
├── channel/            # Channel implementation
│   ├── mod.rs          # Module exports
│   └── channel.rs      # Channel logic
├── breathing/          # Quantum breathing system (placeholder)
├── timeline/           # Timeline and scheduling (placeholder)
├── branch/             # Branch system (placeholder)
└── utils.rs            # Utility functions

examples/
├── simple_usage.rs             # Basic usage demonstration
├── modular_performance_test.rs # Performance benchmarks
└── cyre_server.rs             # HTTP server example
```

## 🎯 Design Principles

### 1. **Separation of Concerns**

Each module has a single, well-defined responsibility:

- `types/` - Data structures and configurations
- `protection/` - Rate limiting and safety mechanisms
- `talent/` - Advanced data processing
- `core/` - Main orchestration logic
- `channel/` - Individual action execution

### 2. **Performance Preservation**

All critical optimizations are maintained:

- **Fast path separation** in `channel/channel.rs`
- **Lock-free counters** throughout the system
- **Compile-time optimizations** via generics and inlining
- **Hot path inlining** with `#[inline(always)]`

### 3. **Clean APIs**

Each module exposes a minimal, focused API:

```rust
// Clean imports through the prelude
use cyre_rust::prelude::*;

// Or specific modules
use cyre_rust::talent::{Talent, TalentRegistry};
use cyre_rust::protection::ProtectionBuilder;
```

### 4. **Testability**

Each module can be tested independently:

```rust
// Test just the protection system
#[test]
fn test_throttle_protection() {
    let protection = ProtectionState::new(Some(100), None, false);
    // ... test logic
}
```

## 🚀 Quick Start

### Basic Usage

```rust
use cyre_rust::prelude::*;

#[tokio::main]
async fn main() {
    let mut cyre = Cyre::new();

    // Register action
    cyre.action(IO::new("greet"));

    // Register handler
    cyre.on("greet", |payload| {
        Box::pin(async move {
            CyreResponse {
                ok: true,
                payload: json!({"message": "Hello!"}),
                message: "Success".to_string(),
                error: None,
                timestamp: current_timestamp(),
                metadata: None,
            }
        })
    });

    // Call action
    let response = cyre.call("greet", json!({"name": "World"})).await;
    println!("Response: {}", response.payload);
}
```

### Advanced Usage with Protection

```rust
use cyre_rust::prelude::*;

// Create protected action
let config = IO::new("api-call")
    .with_throttle(1000)           // 1 second rate limit
    .with_change_detection()       // Skip duplicate payloads
    .with_priority(Priority::High); // High priority execution

cyre.action(config);
cyre.on("api-call", |payload| {
    Box::pin(async move {
        // Your handler logic
        CyreResponse::default()
    })
});
```

### Talent System Usage

```rust
use cyre_rust::talent::{Talent, TalentRegistry, functions::*};

let registry = TalentRegistry::new();

// Register validation talent
let validator = Talent::schema(
    "validate-user",
    "User Validation",
    "Validates required fields",
    require_fields(&["name", "email"])
);
registry.register_talent(validator)?;

// Register transform talent
let transformer = Talent::transform(
    "add-timestamp",
    "Add Timestamp",
    "Adds processing timestamp",
    add_timestamp("processed_at")
);
registry.register_talent(transformer)?;

// Execute talent pipeline
let result = registry.execute_talents(
    &["validate-user".to_string(), "add-timestamp".to_string()],
    &IO::new("process"),
    json!({"name": "Alice", "email": "alice@example.com"})
);
```

## 📦 Module Details

### Core Types (`src/types/`)

**Purpose**: Fundamental data structures and type definitions

**Key Types**:

- `ActionId` - Unique action identifier
- `ActionPayload` - JSON payload type
- `CyreResponse` - Standard response structure
- `IO` - Action configuration with builder pattern
- `Priority` - Execution priority levels

**Builder Pattern Example**:

```rust
let config = IO::new("my-action")
    .with_name("My Action")
    .with_priority(Priority::High)
    .with_throttle(500)
    .with_change_detection();
```

### Protection System (`src/protection/`)

**Purpose**: Rate limiting, throttling, and safety mechanisms

**Key Features**:

- **Compile-time optimization** via `ProtectionType` enum
- **Fast path detection** for zero-overhead channels
- **Thread-safe atomic counters** for statistics
- **Hot path inlining** for maximum performance

**Types**:

- `ProtectionType` - Optimization categories (None, ThrottleOnly, etc.)
- `ProtectionState` - Thread-safe protection logic
- `ProtectionBuilder` - Fluent configuration API

### Talent System (`src/talent/`)

**Purpose**: Advanced payload processing and validation

**Architecture**:

- `types.rs` - Talent definitions and execution logic
- `registry.rs` - Talent management and pipeline execution
- `functions.rs` - Common utility functions

**Talent Types**:

- **Schema** - Data validation
- **Condition** - Conditional execution
- **Transform** - Data transformation
- **Selector** - Data filtering/selection

**Pipeline Example**:

```rust
let pipeline = TalentPipeline::new("user-processing", "User Processing")
    .add_talent("validate-user")
    .add_talent("transform-data")
    .add_talent("cleanup-sensitive")
    .fail_fast(true);
```

### Core Implementation (`src/core/`)

**Purpose**: Main Cyre orchestration and execution logic

**Key Features**:

- **Fast path optimization** - separate cache for unprotected channels
- **Performance counters** - lock-free atomic metrics
- **Pipeline compilation** - pre-computed execution paths
- **Hot path inlining** - critical call path optimization

### Channel System (`src/channel/`)

**Purpose**: Individual action execution and metrics

**Key Features**:

- **Dual execution paths** - fast path vs. protected path
- **Performance tracking** - execution time, error rates
- **Builder pattern** - fluent channel configuration
- **Health monitoring** - automatic performance rating

### Utility Functions (`src/utils.rs`)

**Purpose**: Common utilities and helper functions

**Categories**:

- **Time utilities** - timestamps, duration formatting
- **Performance utilities** - ops/sec calculation, percentiles
- **Hash utilities** - payload hashing for change detection
- **Validation utilities** - ID validation, timing checks
- **Formatting utilities** - human-readable output

## 🔧 Development Guidelines

### Adding New Modules

1. **Create module directory** under `src/`
2. **Add `mod.rs`** with exports
3. **Implement functionality** in separate files
4. **Add module declaration** to `src/lib.rs`
5. **Write comprehensive tests**
6. **Update documentation**

### Performance Considerations

1. **Use `#[inline(always)]`** for hot paths
2. **Prefer atomic operations** over mutexes
3. **Minimize allocations** in critical paths
4. **Use compile-time optimizations** via generics
5. **Profile before optimizing** - measure impact

### Testing Strategy

1. **Unit tests** for individual functions
2. **Integration tests** for module interactions
3. **Performance tests** for optimization verification
4. **Example programs** for usage demonstration

## 📊 Performance Results

The modular architecture maintains excellent performance:

```
Fast Path Baseline:     200,000+ ops/sec
Protected Channels:     100,000+ ops/sec
Mixed Workload:         150,000+ ops/sec
```

Key optimizations preserved:

- ✅ **Zero-cost fast path** for unprotected channels
- ✅ **Lock-free performance counters**
- ✅ **Compile-time protection specialization**
- ✅ **Hot path inlining** throughout

## 🧪 Running Examples

```bash
# Basic usage demonstration
cargo run --example simple_usage

# Performance benchmarking
cargo run --example modular_performance_test --release

# HTTP server example
cargo run --example cyre_server --features server

# Run all tests
cargo test

# Run benchmarks
cargo bench
```

## 🎯 Benefits of Modular Architecture

### **Maintainability**

- Clear separation of concerns
- Focused, single-purpose modules
- Easy to locate and modify functionality

### **Testability**

- Independent module testing
- Isolated test environments
- Comprehensive test coverage

### **Extensibility**

- Easy to add new features
- Plugin-like talent system
- Clean extension points

### **Performance**

- All optimizations preserved
- Hot path inlining maintained
- Lock-free operations throughout

### **Documentation**

- Self-documenting module structure
- Clear API boundaries
- Comprehensive examples

## 🔮 Future Extensions

The modular architecture makes these additions straightforward:

- **Additional talent types** - Custom processing logic
- **Plugin system** - Dynamic talent loading
- **Monitoring backends** - Metrics export modules
- **Storage adapters** - Persistence layer modules
- **Network protocols** - Communication modules

## 📚 API Documentation

Generate full API documentation:

```bash
cargo doc --open
```

This provides comprehensive documentation for all modules, types, and functions with examples and usage patterns.
