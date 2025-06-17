// src/talent/types.rs
// Talent type definitions and core functionality

use serde_json::Value;
use crate::types::{
    ActionPayload,
    TalentResult,
    SchemaFunction,
    ConditionFunction,
    TransformFunction,
    SelectorFunction,
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
            .field("configuration", &self.configuration)
            .field("metadata", &self.metadata)
            .finish()
    }
}

//=============================================================================
// TALENT TYPES
//=============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TalentType {
    Schema, // Input validation
    Transform, // Data transformation
    Condition, // Conditional logic
    Selector, // Data selection/filtering
    Composite, // Multiple talent combination
}

impl Default for TalentType {
    fn default() -> Self {
        TalentType::Transform
    }
}

//=============================================================================
// TALENT IMPLEMENTATION
//=============================================================================

impl Talent {
    /// Create new talent
    pub fn new(id: String, name: String, talent_type: TalentType) -> Self {
        Self {
            id,
            name,
            description: None,
            talent_type,
            enabled: true,
            priority: 50,
            schema_fn: None,
            condition_fn: None,
            transform_fn: None,
            selector_fn: None,
            configuration: Value::Null,
            metadata: Value::Null,
        }
    }

    /// Execute talent function
    pub async fn execute(&self, payload: &ActionPayload) -> TalentResult {
        if !self.enabled {
            return TalentResult {
                success: false,
                value: Value::Null,
                payload: payload.clone(),
                message: "Talent is disabled".to_string(),
                execution_time: 0,
                error: Some("Talent is disabled".to_string()),
                metadata: Some(
                    serde_json::json!({
                    "talent_id": self.id,
                    "enabled": false
                })
                ),
            };
        }

        match self.talent_type {
            TalentType::Transform => self.execute_transform(payload),
            TalentType::Schema => self.execute_schema(payload),
            TalentType::Condition => self.execute_condition(payload),
            TalentType::Selector => self.execute_selector(payload),
            TalentType::Composite => self.execute_composite(payload),
        }
    }

    /// Execute transform function
    fn execute_transform(&self, payload: &ActionPayload) -> TalentResult {
        if let Some(ref transform_fn) = self.transform_fn {
            let result_value = transform_fn(payload.clone());
            TalentResult {
                success: true,
                value: result_value.clone(),
                payload: result_value,
                message: "Transform completed successfully".to_string(),
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
                success: true,
                value: payload.clone(),
                payload: payload.clone(),
                message: "No transform function defined, payload passed through".to_string(),
                execution_time: 0,
                error: None,
                metadata: Some(
                    serde_json::json!({
                    "talent_id": self.id,
                    "talent_type": "transform",
                    "passthrough": true
                })
                ),
            }
        }
    }

    /// Execute schema validation
    fn execute_schema(&self, payload: &ActionPayload) -> TalentResult {
        if let Some(ref schema_fn) = self.schema_fn {
            let validation = schema_fn(payload);
            TalentResult {
                success: validation.valid,
                value: payload.clone(),
                payload: payload.clone(),
                message: if validation.valid {
                    "Schema validation passed".to_string()
                } else {
                    "Schema validation failed".to_string()
                },
                execution_time: 0,
                error: if validation.valid {
                    None
                } else {
                    Some(validation.errors.join(", "))
                },
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
                success: true,
                value: payload.clone(),
                payload: payload.clone(),
                message: "No schema function defined, validation skipped".to_string(),
                execution_time: 0,
                error: None,
                metadata: Some(
                    serde_json::json!({
                    "talent_id": self.id,
                    "talent_type": "schema",
                    "validation_skipped": true
                })
                ),
            }
        }
    }

    /// Execute condition check
    fn execute_condition(&self, payload: &ActionPayload) -> TalentResult {
        if let Some(ref condition_fn) = self.condition_fn {
            let passes = condition_fn(payload);
            TalentResult {
                success: passes,
                value: payload.clone(),
                payload: payload.clone(),
                message: if passes {
                    "Condition check passed".to_string()
                } else {
                    "Condition check failed".to_string()
                },
                execution_time: 0,
                error: if passes {
                    None
                } else {
                    Some("Condition check failed".to_string())
                },
                metadata: Some(
                    serde_json::json!({
                    "talent_id": self.id,
                    "talent_type": "condition",
                    "condition_result": passes
                })
                ),
            }
        } else {
            TalentResult {
                success: true,
                value: payload.clone(),
                payload: payload.clone(),
                message: "No condition function defined, check passed by default".to_string(),
                execution_time: 0,
                error: None,
                metadata: Some(
                    serde_json::json!({
                    "talent_id": self.id,
                    "talent_type": "condition",
                    "default_pass": true
                })
                ),
            }
        }
    }

    /// Execute selector function
    fn execute_selector(&self, payload: &ActionPayload) -> TalentResult {
        if let Some(ref selector_fn) = self.selector_fn {
            let selected = selector_fn(payload);
            TalentResult {
                success: true,
                value: selected.clone(),
                payload: selected,
                message: "Selection completed successfully".to_string(),
                execution_time: 0,
                error: None,
                metadata: Some(
                    serde_json::json!({
                    "talent_id": self.id,
                    "talent_type": "selector",
                    "original_payload": payload
                })
                ),
            }
        } else {
            TalentResult {
                success: true,
                value: payload.clone(),
                payload: payload.clone(),
                message: "No selector function defined, payload passed through".to_string(),
                execution_time: 0,
                error: None,
                metadata: Some(
                    serde_json::json!({
                    "talent_id": self.id,
                    "talent_type": "selector",
                    "passthrough": true
                })
                ),
            }
        }
    }

    /// Execute composite (multiple talents)
    fn execute_composite(&self, payload: &ActionPayload) -> TalentResult {
        // Simplified composite execution
        TalentResult {
            success: true,
            value: payload.clone(),
            payload: payload.clone(),
            message: "Composite execution completed".to_string(),
            execution_time: 0,
            error: None,
            metadata: Some(
                serde_json::json!({
                "talent_id": self.id,
                "talent_type": "composite",
                "composite_execution": true
            })
            ),
        }
    }

    /// Builder pattern methods
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_schema_fn(mut self, schema_fn: SchemaFunction) -> Self {
        self.schema_fn = Some(schema_fn);
        self
    }

    pub fn with_transform_fn(mut self, transform_fn: TransformFunction) -> Self {
        self.transform_fn = Some(transform_fn);
        self
    }

    pub fn with_condition_fn(mut self, condition_fn: ConditionFunction) -> Self {
        self.condition_fn = Some(condition_fn);
        self
    }

    pub fn with_selector_fn(mut self, selector_fn: SelectorFunction) -> Self {
        self.selector_fn = Some(selector_fn);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}
