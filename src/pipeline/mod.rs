// src/pipeline/mod.rs
// File location: src/pipeline/mod.rs
// Pipeline module - Array-based operator system

//=============================================================================
// MODULE DECLARATIONS
//=============================================================================

pub mod operators;

//=============================================================================
// RE-EXPORTS
//=============================================================================

// Core pipeline types
pub use operators::{ Pipeline, OperatorResult, ScheduleConfig, PipelineResult };

// Operator implementations
pub use operators::{
    BlockOperator,
    ThrottleOperator,
    DebounceOperator,
    RequiredOperator,
    SchemaOperator,
    TransformOperator,
    ConditionOperator,
    SelectorOperator,
    ScheduleOperator,
    Operator,
};

// Compilation functions
pub use operators::{
    compile_pipeline,
    execute_pipeline,
    is_fast_path,
    estimated_performance,
    estimate_operator_count,
    get_pipeline_stats,
    get_pipeline_info,
    list_compiled_pipelines,
    PipelineStats,
    PipelineInfo,
};

//=============================================================================
// MODULE DOCUMENTATION
//=============================================================================

// # Pipeline Module
//
// Array-based pipeline system for maximum performance:
//
// ## Architecture
// - **Pipeline**: Array of operators that process payload sequentially
// - **Fast Path**: Empty array = zero overhead (1.8M+ ops/sec target)
// - **Operators**: Async functions that transform/validate/route payload
// - **Compilation**: IO config → optimized operator array
//
// ## Operator Categories
// 1. **Protection** (First): Block, Throttle, Debounce
// 2. **Validation**: Required, Schema
// 3. **Processing**: Condition, Selector, Transform
// 4. **Scheduling** (Last): Delay, Interval, Repeat
//
// ## Performance Model
// - 0 operators = 1.8M+ ops/sec (fast path)
// - 1-2 operators = 1.2M+ ops/sec (lightweight)
// - 3-5 operators = 800K+ ops/sec (standard)
// - 6+ operators = 400K+ ops/sec (complex)
//
// ## Usage
// ```rust
// use cyre::pipeline::{compile_pipeline, is_fast_path};
// use cyre::types::IO;
//
// // Fast path configuration
// let fast_config = IO::new("fast-action");
// assert!(is_fast_path(&fast_config));
// let mut fast_config = fast_config;
// compile_pipeline(&mut fast_config);
//
// // Pipeline configuration
// let pipeline_config = IO::new("protected-action")
//     .with_throttle(100)
//     .with_required(true);
// let mut pipeline_config = pipeline_config;
// compile_pipeline(&mut pipeline_config);
// ```

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::IO;
    use serde_json::json;

    #[tokio::test]
    async fn test_fast_path_compilation() {
        let config = IO::new("fast");
        assert!(is_fast_path(&config));

        let mut config = config;
        let result = compile_pipeline(&mut config);
        assert!(result.is_ok());
        assert!(config._has_fast_path);

        // Fast path should process payload without overhead
        let payload = json!({"test": "fast"});
        let result = execute_pipeline(&mut config, payload.clone()).await;
        assert!(result.is_ok());
        match result.unwrap() {
            PipelineResult::Continue(result_payload) => {
                assert_eq!(result_payload, payload);
            }
            _ => panic!("Expected Continue result"),
        }
    }

    #[tokio::test]
    async fn test_pipeline_compilation() {
        let config = IO::new("pipeline").with_throttle(100).with_required(true);

        assert!(!is_fast_path(&config));
        assert_eq!(estimate_operator_count(&config), 2);
        assert_eq!(estimated_performance(&config), 1_200_000);

        let mut config = config;
        let result = compile_pipeline(&mut config);
        assert!(result.is_ok());
        assert!(!config._has_fast_path);
        assert_eq!(config._pipeline.len(), 2);
    }

    #[tokio::test]
    async fn test_operator_ordering() {
        let config = IO::new("ordered")
            .with_delay(1000) // Schedule (last)
            .with_transform("test") // Processing (middle)
            .with_throttle(100) // Protection (first)
            .with_required(true); // Validation (second)

        let mut config = config;
        let result = compile_pipeline(&mut config);
        assert!(result.is_ok());

        let names = &config._pipeline;

        // Verify correct ordering regardless of config order
        assert_eq!(names[0], "throttle"); // Protection first
        assert_eq!(names[1], "required"); // Validation second
        assert_eq!(names[2], "transform"); // Processing third
        assert_eq!(names[3], "schedule"); // Schedule last
    }

    #[test]
    fn test_performance_estimation() {
        // Fast path
        let fast = IO::new("fast");
        assert_eq!(estimated_performance(&fast), 1_800_000);

        // Lightweight (1-2 ops)
        let lightweight = IO::new("light").with_throttle(100);
        assert_eq!(estimated_performance(&lightweight), 1_200_000);

        // Standard (3-5 ops)
        let standard = IO::new("standard")
            .with_throttle(100)
            .with_required(true)
            .with_schema("test");
        assert_eq!(estimated_performance(&standard), 800_000);

        // Complex (6+ ops)
        let complex = IO::new("complex")
            .with_throttle(100)
            .with_debounce(50)
            .with_required(true)
            .with_schema("test")
            .with_transform("test")
            .with_condition("test")
            .with_delay(1000);
        assert_eq!(estimated_performance(&complex), 400_000);
    }
}
