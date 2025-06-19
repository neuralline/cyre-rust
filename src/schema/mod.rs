// src/schema/mod.rs - Complete schema module with all submodules

// Submodules
pub mod data_definitions;
pub mod operators;
pub mod compile_execute;

// Re-export data validation functionality
pub use data_definitions::{
    validate_field,
    DataDefResult,
    describe_value,
    PROTECTION_TALENTS,
    PROCESSING_TALENTS,
    SCHEDULING_TALENTS,
};

// Re-export compilation and execution functionality
pub use compile_execute::{
    compile_pipeline,
    execute_pipeline,
    PipelineResult,
    is_fast_path,
    estimate_operator_count,
    estimated_performance,
    ConfigFields,
};

// Re-export operators functionality
pub use operators::{
    PipelineStats,
    PipelineInfo,
    get_pipeline_stats,
    get_pipeline_info,
    list_compiled_pipelines,
    Pipeline,
    OperatorResult,
    ScheduleConfig,
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
