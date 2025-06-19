// src/schema/mod.rs - Complete schema module with separated compilation and execution

// Submodules
pub mod data_definitions;
pub mod operators;
pub mod compiler;
pub mod executor;

// Re-export data validation functionality
pub use data_definitions::{
    validate_field,
    DataDefResult,
    describe_value,
    PROTECTION_TALENTS,
    PROCESSING_TALENTS,
    SCHEDULING_TALENTS,
};

// Re-export compilation functionality
pub use compiler::{ compile_pipeline, CompileResult };

// Re-export execution functionality
pub use executor::{
    execute_pipeline,
    execute_pipeline_to_response,
    requires_pipeline_execution,
    PipelineResult,
};

// Re-export operators functionality
pub use operators::{
    // Operator implementations
    Operator,
    BlockOperator,
    ThrottleOperator,
    DebounceOperator,
    RequiredOperator,
    SchemaOperator,
    TransformOperator,
    ConditionOperator,
    SelectorOperator,
    ScheduleOperator,
};
