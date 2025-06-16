// src/talent/types.rs
// Talent type definitions and core functionality

use std::sync::Arc;
use serde_json::Value;
use crate::types::{
    ActionPayload,
    TalentResult,
    SchemaFunction,
    ConditionFunction,
    TransformFunction,
    SelectorFunction,
    ValidationResult,
};

/*

      C.Y.R.E - T.A.L.E.N.T - T.Y.P.E.S
      
      Data transformation and validation system

*/

//=============================================================================
// TALENT DEFINITION
//=============================================================================

#[derive(Clone)]
pub struct Talent {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub talent_type: TalentType,
    pub enabled: bool,
    pub priority: u8,

    // Function types (cannot derive Debug due to function pointers)
    pub schema_fn: Option<SchemaFunction>,
    pub condition_fn: Option<ConditionFunction>,
    pub transform_fn: Option<TransformFunction>,
    pub selector_fn: Option<SelectorFunction>,

    pub configuration: Value,
    pub metadata: Value,
}

impl std::fmt::Debug for Talent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Talent")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("talent_type", &self.talent_type)
            .field("enabled", &self.enabled)
            .field("priority", &self.priority)
            .field("schema_fn", &self.schema_fn.as_ref().map(|_| "function"))
            .field("condition_fn", &self.condition_fn.as_ref().map(|_| "function"))
            .field("transform_fn", &self.transform_fn.as_ref().map(|_| "function"))
            .field("selector_fn", &self.selector_fn.as_ref().map(|_| "function"))
            .field("configuration", &self.configuration)
            .field("metadata", &self.metadata)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TalentType {
    Schema,
    Condition,
    Transform,
    Selector,
    Pipeline,
}

//=============================================================================
// TALENT IMPLEMENTATION
//=============================================================================

impl Talent {
    pub fn new(id: String, name: String, talent_type: TalentType) -> Self {
        Self {
            id,
            name,
            description: None,
            talent_type,
            enabled: true,
            priority: 0,
            schema_fn: None,
            condition_fn: None,
            transform_fn: None,
            selector_fn: None,
            configuration: Value::Object(Default::default()),
            metadata: Value::Object(Default::default()),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_schema_function(mut self, schema_fn: SchemaFunction) -> Self {
        self.schema_fn = Some(schema_fn);
        self
    }

    pub fn with_condition_function(mut self, condition_fn: ConditionFunction) -> Self {
        self.condition_fn = Some(condition_fn);
        self
    }

    pub fn with_transform_function(mut self, transform_fn: TransformFunction) -> Self {
        self.transform_fn = Some(transform_fn);
        self
    }

    pub fn with_selector_function(mut self, selector_fn: SelectorFunction) -> Self {
        self.selector_fn = Some(selector_fn);
        self
    }

    pub fn with_configuration(mut self, config: Value) -> Self {
        self.configuration = config;
        self
    }

    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Execute talent based on its type
    pub async fn execute(&self, payload: ActionPayload) -> TalentResult {
        if !self.enabled {
            return TalentResult {
                success: false,
                value: Value::Null,
                payload: payload.clone(),
                message: "Talent is inactive".to_string(),
                execution_time: 0,
                error: Some("Talent is inactive".to_string()),
                metadata: Some(
                    serde_json::json!({
                    "talent_id": self.id,
                    "enabled": false
                })
                ),
            };
        }

        let start_time = std::time::Instant::now();

        let result = match self.talent_type {
            TalentType::Schema => self.execute_schema(payload).await,
            TalentType::Condition => self.execute_condition(payload).await,
            TalentType::Transform => self.execute_transform(payload).await,
            TalentType::Selector => self.execute_selector(payload).await,
            TalentType::Pipeline => self.execute_pipeline(payload).await,
        };

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Update execution time in result
        TalentResult {
            execution_time,
            ..result
        }
    }

    async fn execute_schema(&self, payload: ActionPayload) -> TalentResult {
        if let Some(schema_fn) = &self.schema_fn {
            let validation = schema_fn(&payload);

            if validation.valid {
                TalentResult {
                    success: true,
                    value: payload.clone(),
                    payload: payload.clone(),
                    message: "Schema validation passed".to_string(),
                    execution_time: 0,
                    error: None,
                    metadata: Some(
                        serde_json::json!({
                        "talent_id": self.id,
                        "talent_type": "schema",
                        "validation": {
                            "valid": validation.valid,
                            "errors": validation.errors,
                            "warnings": validation.warnings
                        }
                    })
                    ),
                }
            } else {
                TalentResult {
                    success: false,
                    value: payload.clone(),
                    payload: payload.clone(),
                    message: "Schema validation failed".to_string(),
                    execution_time: 0,
                    error: Some(
                        format!("Schema validation failed: {}", validation.errors.join(", "))
                    ),
                    metadata: Some(
                        serde_json::json!({
                        "talent_id": self.id,
                        "validation": {
                            "valid": validation.valid,
                            "errors": validation.errors,
                            "warnings": validation.warnings
                        }
                    })
                    ),
                }
            }
        } else {
            TalentResult {
                success: false,
                value: payload.clone(),
                payload: payload.clone(),
                message: "Schema function not defined".to_string(),
                execution_time: 0,
                error: Some("Schema function not defined".to_string()),
                metadata: None,
            }
        }
    }

    async fn execute_condition(&self, payload: ActionPayload) -> TalentResult {
        if let Some(condition_fn) = &self.condition_fn {
            let passed = condition_fn(&payload);

            if passed {
                TalentResult {
                    success: true,
                    value: payload.clone(),
                    payload: payload.clone(),
                    message: "Condition check passed".to_string(),
                    execution_time: 0,
                    error: None,
                    metadata: Some(
                        serde_json::json!({
                        "talent_id": self.id,
                        "talent_type": "condition",
                        "condition_passed": true
                    })
                    ),
                }
            } else {
                TalentResult {
                    success: false,
                    value: payload.clone(),
                    payload: payload.clone(),
                    message: "Condition check failed".to_string(),
                    execution_time: 0,
                    error: Some("Condition check failed".to_string()),
                    metadata: Some(
                        serde_json::json!({
                        "talent_id": self.id,
                        "talent_type": "condition",
                        "condition_passed": false
                    })
                    ),
                }
            }
        } else {
            TalentResult {
                success: false,
                value: payload.clone(),
                payload: payload.clone(),
                message: "Condition function not defined".to_string(),
                execution_time: 0,
                error: Some("Condition function not defined".to_string()),
                metadata: None,
            }
        }
    }

    async fn execute_transform(&self, payload: ActionPayload) -> TalentResult {
        if let Some(transform_fn) = &self.transform_fn {
            let transformed_payload = transform_fn(payload.clone());

            TalentResult {
                success: true,
                value: transformed_payload.clone(),
                payload: transformed_payload,
                message: "Transform completed".to_string(),
                execution_time: 0,
                error: None,
                metadata: Some(
                    serde_json::json!({
                    "talent_id": self.id,
                    "talent_type": "transform"
                })
                ),
            }
        } else {
            TalentResult {
                success: false,
                value: payload.clone(),
                payload: payload.clone(),
                message: "Transform function not defined".to_string(),
                execution_time: 0,
                error: Some("Transform function not defined".to_string()),
                metadata: None,
            }
        }
    }

    async fn execute_selector(&self, payload: ActionPayload) -> TalentResult {
        if let Some(selector_fn) = &self.selector_fn {
            let selected_payload = selector_fn(&payload);

            TalentResult {
                success: true,
                value: selected_payload.clone(),
                payload: selected_payload,
                message: "Selection completed".to_string(),
                execution_time: 0,
                error: None,
                metadata: Some(
                    serde_json::json!({
                    "talent_id": self.id,
                    "talent_type": "selector"
                })
                ),
            }
        } else {
            TalentResult {
                success: false,
                value: payload.clone(),
                payload: payload.clone(),
                message: "Selector function not defined".to_string(),
                execution_time: 0,
                error: Some("Selector function not defined".to_string()),
                metadata: None,
            }
        }
    }

    async fn execute_pipeline(&self, payload: ActionPayload) -> TalentResult {
        // Pipeline execution would chain multiple talent operations
        // For now, just return the payload as-is
        TalentResult {
            success: true,
            value: payload.clone(),
            payload: payload.clone(),
            message: "Pipeline execution completed".to_string(),
            execution_time: 0,
            error: None,
            metadata: Some(
                serde_json::json!({
                "talent_id": self.id,
                "talent_type": "pipeline"
            })
            ),
        }
    }
}

//=============================================================================
// BUILDER PATTERNS
//=============================================================================

pub struct TalentBuilder {
    talent: Talent,
}

impl TalentBuilder {
    pub fn new(id: String, name: String, talent_type: TalentType) -> Self {
        Self {
            talent: Talent::new(id, name, talent_type),
        }
    }

    pub fn description(mut self, description: String) -> Self {
        self.talent = self.talent.with_description(description);
        self
    }

    pub fn priority(mut self, priority: u8) -> Self {
        self.talent = self.talent.with_priority(priority);
        self
    }

    pub fn schema_function(mut self, schema_fn: SchemaFunction) -> Self {
        self.talent = self.talent.with_schema_function(schema_fn);
        self
    }

    pub fn condition_function(mut self, condition_fn: ConditionFunction) -> Self {
        self.talent = self.talent.with_condition_function(condition_fn);
        self
    }

    pub fn transform_function(mut self, transform_fn: TransformFunction) -> Self {
        self.talent = self.talent.with_transform_function(transform_fn);
        self
    }

    pub fn selector_function(mut self, selector_fn: SelectorFunction) -> Self {
        self.talent = self.talent.with_selector_function(selector_fn);
        self
    }

    pub fn configuration(mut self, config: Value) -> Self {
        self.talent = self.talent.with_configuration(config);
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.talent = self.talent.with_metadata(metadata);
        self
    }

    pub fn build(self) -> Talent {
        self.talent
    }
}

//=============================================================================
// UTILITY FUNCTIONS
//=============================================================================

impl Default for TalentType {
    fn default() -> Self {
        TalentType::Transform
    }
}

/// Create a talent configuration with sensible defaults
pub fn create_talent_config() -> serde_json::Value {
    serde_json::json!({
        "channel_priority": crate::types::Priority::Normal,
        "execution_timeout": 5000,
        "retry_attempts": 3,
        "fallback_enabled": true
    })
}
