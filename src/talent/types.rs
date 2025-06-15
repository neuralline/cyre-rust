// src/talent/types.rs
// Talent type definitions

use serde::{Serialize, Deserialize};
use crate::types::{ActionPayload, TalentResult, SchemaFunction, ConditionFunction, TransformFunction, SelectorFunction};

//=============================================================================
// TALENT TYPES
//=============================================================================

/// Types of talents available in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TalentType {
    Schema,      // Data validation
    Condition,   // Conditional execution
    Transform,   // Data transformation
    Selector,    // Data selection/filtering
}

/// A talent represents a reusable piece of functionality
#[derive(Debug, Clone)]
pub struct Talent {
    pub id: String,
    pub name: String,
    pub talent_type: TalentType,
    pub description: String,
    pub active: bool,
    
    // Function pointers for different talent types
    pub schema_fn: Option<SchemaFunction>,
    pub condition_fn: Option<ConditionFunction>,
    pub transform_fn: Option<TransformFunction>,
    pub selector_fn: Option<SelectorFunction>,
    
    // Metadata
    pub version: String,
    pub author: Option<String>,
    pub created_at: u64,
}

impl Talent {
    /// Create a new schema validation talent
    pub fn schema(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        schema_fn: SchemaFunction,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            talent_type: TalentType::Schema,
            description: description.into(),
            active: true,
            schema_fn: Some(schema_fn),
            condition_fn: None,
            transform_fn: None,
            selector_fn: None,
            version: "1.0.0".to_string(),
            author: None,
            created_at: crate::utils::current_timestamp(),
        }
    }

    /// Create a new condition talent
    pub fn condition(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        condition_fn: ConditionFunction,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            talent_type: TalentType::Condition,
            description: description.into(),
            active: true,
            schema_fn: None,
            condition_fn: Some(condition_fn),
            transform_fn: None,
            selector_fn: None,
            version: "1.0.0".to_string(),
            author: None,
            created_at: crate::utils::current_timestamp(),
        }
    }

    /// Create a new transform talent
    pub fn transform(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        transform_fn: TransformFunction,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            talent_type: TalentType::Transform,
            description: description.into(),
            active: true,
            schema_fn: None,
            condition_fn: None,
            transform_fn: Some(transform_fn),
            selector_fn: None,
            version: "1.0.0".to_string(),
            author: None,
            created_at: crate::utils::current_timestamp(),
        }
    }

    /// Create a new selector talent
    pub fn selector(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        selector_fn: SelectorFunction,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            talent_type: TalentType::Selector,
            description: description.into(),
            active: true,
            schema_fn: None,
            condition_fn: None,
            transform_fn: None,
            selector_fn: Some(selector_fn),
            version: "1.0.0".to_string(),
            author: None,
            created_at: crate::utils::current_timestamp(),
        }
    }

    /// Set talent version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set talent author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Enable or disable the talent
    pub fn set_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Execute the talent based on its type
    pub fn execute(&self, payload: ActionPayload) -> TalentResult {
        if !self.active {
            return TalentResult {
                success: false,
                payload,
                error: Some("Talent is inactive".to_string()),
                metadata: Some(serde_json::json!({
                    "talent_id": self.id,
                    "reason": "inactive"
                })),
            };
        }

        match self.talent_type {
            TalentType::Schema => {
                if let Some(schema_fn) = self.schema_fn {
                    let validation = schema_fn(payload.clone());
                    if validation.valid {
                        TalentResult {
                            success: true,
                            payload,
                            error: None,
                            metadata: Some(serde_json::json!({
                                "talent_id": self.id,
                                "talent_type": "schema",
                                "validation": validation
                            })),
                        }
                    } else {
                        TalentResult {
                            success: false,
                            payload,
                            error: Some(format!("Schema validation failed: {}", validation.errors.join(", "))),
                            metadata: Some(serde_json::json!({
                                "talent_id": self.id,
                                "validation": validation
                            })),
                        }
                    }
                } else {
                    TalentResult {
                        success: false,
                        payload,
                        error: Some("Schema function not defined".to_string()),
                        metadata: None,
                    }
                }
            },
            TalentType::Condition => {
                if let Some(condition_fn) = self.condition_fn {
                    let passed = condition_fn(payload.clone());
                    if passed {
                        TalentResult {
                            success: true,
                            payload,
                            error: None,
                            metadata: Some(serde_json::json!({
                                "talent_id": self.id,
                                "talent_type": "condition",
                                "condition_passed": true
                            })),
                        }
                    } else {
                        TalentResult {
                            success: false,
                            payload,
                            error: Some("Condition check failed".to_string()),
                            metadata: Some(serde_json::json!({
                                "talent_id": self.id,
                                "talent_type": "condition",
                                "condition_passed": false
                            })),
                        }
                    }
                } else {
                    TalentResult {
                        success: false,
                        payload,
                        error: Some("Condition function not defined".to_string()),
                        metadata: None,
                    }
                }
            },
            TalentType::Transform => {
                if let Some(transform_fn) = self.transform_fn {
                    let transformed_payload = transform_fn(payload);
                    TalentResult {
                        success: true,
                        payload: transformed_payload,
                        error: None,
                        metadata: Some(serde_json::json!({
                            "talent_id": self.id,
                            "talent_type": "transform"
                        })),
                    }
                } else {
                    TalentResult {
                        success: false,
                        payload,
                        error: Some("Transform function not defined".to_string()),
                        metadata: None,
                    }
                }
            },
            TalentType::Selector => {
                if let Some(selector_fn) = self.selector_fn {
                    let selected_payload = selector_fn(payload);
                    TalentResult {
                        success: true,
                        payload: selected_payload,
                        error: None,
                        metadata: Some(serde_json::json!({
                            "talent_id": self.id,
                            "talent_type": "selector"
                        })),
                    }
                } else {
                    TalentResult {
                        success: false,
                        payload,
                        error: Some("Selector function not defined".to_string()),
                        metadata: None,
                    }
                }
            },
        }
    }

    /// Get talent metadata
    pub fn get_metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "name": self.name,
            "type": self.talent_type,
            "description": self.description,
            "active": self.active,
            "version": self.version,
            "author": self.author,
            "created_at": self.created_at
        })
    }
}

//=============================================================================
// TALENT EXECUTION CONTEXT
//=============================================================================

/// Context information for talent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalentContext {
    pub action_id: String,
    pub channel_priority: crate::types::Priority,
    pub execution_count: u64,
    pub timestamp: u64,
    pub metadata: serde_json::Value,
}

impl TalentContext {
    pub fn new(action_id: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            channel_priority: crate::types::Priority::Medium,
            execution_count: 0,
            timestamp: crate::utils::current_timestamp(),
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_priority(mut self, priority: crate::types::Priority) -> Self {
        self.channel_priority = priority;
        self
    }

    pub fn with_execution_count(mut self, count: u64) -> Self {
        self.execution_count = count;
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

//=============================================================================
// TALENT PIPELINE
//=============================================================================

/// A pipeline of talents that execute in sequence
#[derive(Debug, Clone)]
pub struct TalentPipeline {
    pub id: String,
    pub name: String,
    pub talent_ids: Vec<String>,
    pub fail_fast: bool,
    pub created_at: u64,
}

impl TalentPipeline {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            talent_ids: Vec::new(),
            fail_fast: true,
            created_at: crate::utils::current_timestamp(),
        }
    }

    pub fn add_talent(mut self, talent_id: impl Into<String>) -> Self {
        self.talent_ids.push(talent_id.into());
        self
    }

    pub fn fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    pub fn talent_ids(&self) -> &[String] {
        &self.talent_ids
    }
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_schema_fn(payload: ActionPayload) -> ValidationResult {
        let valid = payload.get("required_field").is_some();
        ValidationResult {
            valid,
            errors: if valid { vec![] } else { vec!["required_field is missing".to_string()] },
            warnings: vec![],
        }
    }

    fn test_condition_fn(payload: ActionPayload) -> bool {
        payload.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false)
    }

    fn test_transform_fn(mut payload: ActionPayload) -> ActionPayload {
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("transformed".to_string(), json!(true));
        }
        payload
    }

    fn test_selector_fn(payload: ActionPayload) -> ActionPayload {
        json!({
            "selected": payload.get("data")
        })
    }

    #[test]
    fn test_schema_talent() {
        let talent = Talent::schema("test-schema", "Test Schema", "Validates payload", test_schema_fn);
        
        // Valid payload
        let result = talent.execute(json!({"required_field": "value"}));
        assert!(result.success);
        
        // Invalid payload
        let result = talent.execute(json!({"other_field": "value"}));
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_condition_talent() {
        let talent = Talent::condition("test-condition", "Test Condition", "Checks enabled flag", test_condition_fn);
        
        // Condition passes
        let result = talent.execute(json!({"enabled": true}));
        assert!(result.success);
        
        // Condition fails
        let result = talent.execute(json!({"enabled": false}));
        assert!(!result.success);
    }

    #[test]
    fn test_transform_talent() {
        let talent = Talent::transform("test-transform", "Test Transform", "Adds transformed flag", test_transform_fn);
        
        let result = talent.execute(json!({"data": "test"}));
        assert!(result.success);
        assert_eq!(result.payload.get("transformed").unwrap(), &json!(true));
    }

    #[test]
    fn test_selector_talent() {
        let talent = Talent::selector("test-selector", "Test Selector", "Selects data field", test_selector_fn);
        
        let result = talent.execute(json!({"data": "selected_value", "other": "ignored"}));
        assert!(result.success);
        assert_eq!(result.payload.get("selected").unwrap(), &json!("selected_value"));
    }

    #[test]
    fn test_inactive_talent() {
        let talent = Talent::schema("test", "Test", "Test", test_schema_fn).set_active(false);
        
        let result = talent.execute(json!({"required_field": "value"}));
        assert!(!result.success);
        assert!(result.error.unwrap().contains("inactive"));
    }

    #[test]
    fn test_talent_pipeline() {
        let pipeline = TalentPipeline::new("test-pipeline", "Test Pipeline")
            .add_talent("schema")
            .add_talent("transform")
            .add_talent("condition")
            .fail_fast(true);
        
        assert_eq!(pipeline.talent_ids().len(), 3);
        assert_eq!(pipeline.talent_ids()[0], "schema");
        assert!(pipeline.fail_fast);
    }
}