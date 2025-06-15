// src/talent/registry.rs
// Talent registry for managing and executing talents

use std::sync::{Arc, RwLock, atomic::{AtomicU64, Ordering}};
use crate::types::{ActionPayload, TalentResult, FastMap, IO};
use super::{Talent, TalentPipeline, TalentContext};

//=============================================================================
// TALENT REGISTRY
//=============================================================================

/// Registry for managing talents and their execution
#[derive(Debug)]
pub struct TalentRegistry {
    talents: Arc<RwLock<FastMap<String, Talent>>>,
    pipelines: Arc<RwLock<FastMap<String, TalentPipeline>>>,
    
    // Performance metrics
    execution_count: AtomicU64,
    talent_executions: AtomicU64,
    pipeline_executions: AtomicU64,
    error_count: AtomicU64,
}

impl TalentRegistry {
    /// Create a new talent registry
    pub fn new() -> Self {
        Self {
            talents: Arc::new(RwLock::new(FastMap::default())),
            pipelines: Arc::new(RwLock::new(FastMap::default())),
            execution_count: AtomicU64::new(0),
            talent_executions: AtomicU64::new(0),
            pipeline_executions: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }

    /// Register a talent in the registry
    pub fn register_talent(&self, talent: Talent) -> Result<bool, String> {
        if talent.id.is_empty() {
            return Err("Talent ID cannot be empty".to_string());
        }

        let mut talents = self.talents.write().unwrap();
        let existed = talents.contains_key(&talent.id);
        talents.insert(talent.id.clone(), talent);
        
        Ok(!existed)
    }

    /// Register a talent pipeline
    pub fn register_pipeline(&self, pipeline: TalentPipeline) -> Result<bool, String> {
        if pipeline.id.is_empty() {
            return Err("Pipeline ID cannot be empty".to_string());
        }

        // Validate that all talents in the pipeline exist
        {
            let talents = self.talents.read().unwrap();
            for talent_id in &pipeline.talent_ids {
                if !talents.contains_key(talent_id) {
                    return Err(format!("Talent '{}' not found in pipeline '{}'", talent_id, pipeline.id));
                }
            }
        }

        let mut pipelines = self.pipelines.write().unwrap();
        let existed = pipelines.contains_key(&pipeline.id);
        pipelines.insert(pipeline.id.clone(), pipeline);
        
        Ok(!existed)
    }

    /// Execute a list of talents
    pub fn execute_talents(&self, talent_ids: &[String], _config: &IO, payload: ActionPayload) -> TalentResult {
        self.execution_count.fetch_add(1, Ordering::Relaxed);
        
        if talent_ids.is_empty() {
            return TalentResult {
                success: true,
                payload,
                error: None,
                metadata: Some(serde_json::json!({
                    "message": "No talents to execute",
                    "talent_count": 0
                })),
            };
        }

        let talents = self.talents.read().unwrap();
        let mut current_payload = payload;
        let mut executed_talents = Vec::new();
        let mut errors = Vec::new();
        
        for talent_id in talent_ids {
            if let Some(talent) = talents.get(talent_id) {
                self.talent_executions.fetch_add(1, Ordering::Relaxed);
                
                let result = talent.execute(current_payload.clone());
                executed_talents.push(talent_id.clone());
                
                if result.success {
                    current_payload = result.payload;
                } else {
                    self.error_count.fetch_add(1, Ordering::Relaxed);
                    let error_msg = result.error.unwrap_or_else(|| "Unknown talent error".to_string());
                    errors.push(format!("{}: {}", talent_id, error_msg));
                    
                    // Fail fast on first error
                    return TalentResult {
                        success: false,
                        payload: current_payload,
                        error: Some(format!("Talent execution failed: {}", errors.join(", "))),
                        metadata: Some(serde_json::json!({
                            "executed_talents": executed_talents,
                            "failed_talent": talent_id,
                            "errors": errors
                        })),
                    };
                }
            } else {
                self.error_count.fetch_add(1, Ordering::Relaxed);
                errors.push(format!("Talent '{}' not found", talent_id));
                
                return TalentResult {
                    success: false,
                    payload: current_payload,
                    error: Some(format!("Talent not found: {}", talent_id)),
                    metadata: Some(serde_json::json!({
                        "executed_talents": executed_talents,
                        "missing_talent": talent_id,
                        "errors": errors
                    })),
                };
            }
        }
        
        TalentResult {
            success: true,
            payload: current_payload,
            error: None,
            metadata: Some(serde_json::json!({
                "executed_talents": executed_talents,
                "talent_count": executed_talents.len(),
                "timestamp": crate::utils::current_timestamp()
            })),
        }
    }

    /// Execute a talent pipeline
    pub fn execute_pipeline(&self, pipeline_id: &str, payload: ActionPayload) -> TalentResult {
        self.pipeline_executions.fetch_add(1, Ordering::Relaxed);
        
        let pipeline = {
            let pipelines = self.pipelines.read().unwrap();
            pipelines.get(pipeline_id).cloned()
        };

        if let Some(pipeline) = pipeline {
            // Create a temporary IO config for pipeline execution
            let config = IO::new(pipeline_id);
            self.execute_talents(&pipeline.talent_ids, &config, payload)
        } else {
            self.error_count.fetch_add(1, Ordering::Relaxed);
            TalentResult {
                success: false,
                payload,
                error: Some(format!("Pipeline '{}' not found", pipeline_id)),
                metadata: Some(serde_json::json!({
                    "pipeline_id": pipeline_id,
                    "error": "not_found"
                })),
            }
        }
    }

    /// Execute talents with context
    pub fn execute_with_context(&self, talent_ids: &[String], context: TalentContext, payload: ActionPayload) -> TalentResult {
        // For now, execute normally but include context in metadata
        let config = IO::new(&context.action_id);
        let mut result = self.execute_talents(talent_ids, &config, payload);
        
        // Add context to metadata
        if let Some(metadata) = result.metadata.as_mut() {
            if let Some(obj) = metadata.as_object_mut() {
                obj.insert("context".to_string(), serde_json::json!(context));
            }
        }
        
        result
    }

    /// Get a talent by ID
    pub fn get_talent(&self, talent_id: &str) -> Option<Talent> {
        let talents = self.talents.read().unwrap();
        talents.get(talent_id).cloned()
    }

    /// Get a pipeline by ID
    pub fn get_pipeline(&self, pipeline_id: &str) -> Option<TalentPipeline> {
        let pipelines = self.pipelines.read().unwrap();
        pipelines.get(pipeline_id).cloned()
    }

    /// List all talent IDs
    pub fn list_talents(&self) -> Vec<String> {
        let talents = self.talents.read().unwrap();
        talents.keys().cloned().collect()
    }

    /// List all pipeline IDs
    pub fn list_pipelines(&self) -> Vec<String> {
        let pipelines = self.pipelines.read().unwrap();
        pipelines.keys().cloned().collect()
    }

    /// Remove a talent
    pub fn remove_talent(&self, talent_id: &str) -> bool {
        let mut talents = self.talents.write().unwrap();
        talents.remove(talent_id).is_some()
    }

    /// Remove a pipeline
    pub fn remove_pipeline(&self, pipeline_id: &str) -> bool {
        let mut pipelines = self.pipelines.write().unwrap();
        pipelines.remove(pipeline_id).is_some()
    }

    /// Enable or disable a talent
    pub fn set_talent_active(&self, talent_id: &str, active: bool) -> bool {
        let mut talents = self.talents.write().unwrap();
        if let Some(talent) = talents.get_mut(talent_id) {
            talent.active = active;
            true
        } else {
            false
        }
    }

    /// Get registry statistics
    pub fn get_stats(&self) -> (u64, u64) {
        (
            self.execution_count.load(Ordering::Relaxed),
            self.talent_executions.load(Ordering::Relaxed)
        )
    }

    /// Get comprehensive metrics
    pub fn get_metrics(&self) -> serde_json::Value {
        let talents = self.talents.read().unwrap();
        let pipelines = self.pipelines.read().unwrap();
        
        let total = self.execution_count.load(Ordering::Relaxed);
        let errors = self.error_count.load(Ordering::Relaxed);
        let success_rate = if total > 0 {
            ((total - errors) as f64 / total as f64) * 100.0
        } else {
            100.0
        };
        
        serde_json::json!({
            "talents": {
                "count": talents.len(),
                "active": talents.values().filter(|t| t.active).count(),
                "inactive": talents.values().filter(|t| !t.active).count()
            },
            "pipelines": {
                "count": pipelines.len()
            },
            "execution": {
                "total_executions": self.execution_count.load(Ordering::Relaxed),
                "talent_executions": self.talent_executions.load(Ordering::Relaxed),
                "pipeline_executions": self.pipeline_executions.load(Ordering::Relaxed),
                "error_count": self.error_count.load(Ordering::Relaxed),
                "success_rate": success_rate
            }
        })
    }

    /// Clear all talents and pipelines
    pub fn clear(&self) {
        {
            let mut talents = self.talents.write().unwrap();
            talents.clear();
        }
        {
            let mut pipelines = self.pipelines.write().unwrap();
            pipelines.clear();
        }
        
        // Reset counters
        self.execution_count.store(0, Ordering::Relaxed);
        self.talent_executions.store(0, Ordering::Relaxed);
        self.pipeline_executions.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
    }

    /// Get talent count
    pub fn talent_count(&self) -> usize {
        self.talents.read().unwrap().len()
    }

    /// Get pipeline count
    pub fn pipeline_count(&self) -> usize {
        self.pipelines.read().unwrap().len()
    }

    /// Get active talent count
    pub fn active_talent_count(&self) -> usize {
        let talents = self.talents.read().unwrap();
        talents.values().filter(|t| t.active).count()
    }

    /// Validate talent dependencies (check if all referenced talents exist)
    pub fn validate_dependencies(&self) -> Vec<String> {
        let mut errors = Vec::new();
        let talents = self.talents.read().unwrap();
        let pipelines = self.pipelines.read().unwrap();
        
        for pipeline in pipelines.values() {
            for talent_id in &pipeline.talent_ids {
                if !talents.contains_key(talent_id) {
                    errors.push(format!("Pipeline '{}' references missing talent '{}'", pipeline.id, talent_id));
                }
            }
        }
        
        errors
    }
}

impl Default for TalentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ValidationResult};
    use serde_json::json;

    fn create_test_talent() -> Talent {
        Talent::transform("test-transform", "Test Transform", "Adds test flag", |mut payload| {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("transformed".to_string(), json!(true));
            }
            payload
        })
    }

    fn create_failing_talent() -> Talent {
        Talent::schema("test-schema", "Test Schema", "Always fails", |_| {
            ValidationResult {
                valid: false,
                errors: vec!["Always fails".to_string()],
                warnings: vec![],
            }
        })
    }

    #[test]
    fn test_register_talent() {
        let registry = TalentRegistry::new();
        let talent = create_test_talent();
        
        let result = registry.register_talent(talent);
        assert!(result.is_ok());
        assert!(result.unwrap()); // New talent
        
        assert_eq!(registry.talent_count(), 1);
        assert!(registry.get_talent("test-transform").is_some());
    }

    #[test]
    fn test_register_duplicate_talent() {
        let registry = TalentRegistry::new();
        let talent1 = create_test_talent();
        let talent2 = create_test_talent();
        
        let result1 = registry.register_talent(talent1);
        assert!(result1.is_ok());
        assert!(result1.unwrap()); // New
        
        let result2 = registry.register_talent(talent2);
        assert!(result2.is_ok());
        assert!(!result2.unwrap()); // Replaced existing
        
        assert_eq!(registry.talent_count(), 1);
    }

    #[test]
    fn test_execute_talents() {
        let registry = TalentRegistry::new();
        let talent = create_test_talent();
        registry.register_talent(talent).unwrap();
        
        let config = IO::new("test");
        let payload = json!({"data": "test"});
        
        let result = registry.execute_talents(&["test-transform".to_string()], &config, payload);
        assert!(result.success);
        assert_eq!(result.payload.get("transformed").unwrap(), &json!(true));
    }

    #[test]
    fn test_execute_failing_talent() {
        let registry = TalentRegistry::new();
        let talent = create_failing_talent();
        registry.register_talent(talent).unwrap();
        
        let config = IO::new("test");
        let payload = json!({"data": "test"});
        
        let result = registry.execute_talents(&["test-schema".to_string()], &config, payload);
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Always fails"));
    }

    #[test]
    fn test_execute_missing_talent() {
        let registry = TalentRegistry::new();
        
        let config = IO::new("test");
        let payload = json!({"data": "test"});
        
        let result = registry.execute_talents(&["missing-talent".to_string()], &config, payload);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_talent_pipeline() {
        let registry = TalentRegistry::new();
        
        // Register talents
        let talent1 = create_test_talent();
        let talent2 = Talent::transform("add-id", "Add ID", "Adds ID field", |mut payload| {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("id".to_string(), json!(123));
            }
            payload
        });
        
        registry.register_talent(talent1).unwrap();
        registry.register_talent(talent2).unwrap();
        
        // Create pipeline
        let pipeline = TalentPipeline::new("test-pipeline", "Test Pipeline")
            .add_talent("test-transform")
            .add_talent("add-id");
        
        registry.register_pipeline(pipeline).unwrap();
        
        // Execute pipeline
        let payload = json!({"data": "test"});
        let result = registry.execute_pipeline("test-pipeline", payload);
        
        assert!(result.success);
        assert_eq!(result.payload.get("transformed").unwrap(), &json!(true));
        assert_eq!(result.payload.get("id").unwrap(), &json!(123));
    }

    #[test]
    fn test_talent_activation() {
        let registry = TalentRegistry::new();
        let talent = create_test_talent();
        registry.register_talent(talent).unwrap();
        
        // Deactivate talent
        assert!(registry.set_talent_active("test-transform", false));
        
        let config = IO::new("test");
        let payload = json!({"data": "test"});
        
        let result = registry.execute_talents(&["test-transform".to_string()], &config, payload);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("inactive"));
    }

    #[test]
    fn test_registry_metrics() {
        let registry = TalentRegistry::new();
        let talent = create_test_talent();
        registry.register_talent(talent).unwrap();
        
        let config = IO::new("test");
        let payload = json!({"data": "test"});
        
        // Execute a few times
        for _ in 0..3 {
            registry.execute_talents(&["test-transform".to_string()], &config, payload.clone());
        }
        
        let metrics = registry.get_metrics();
        assert_eq!(metrics["execution"]["total_executions"].as_u64().unwrap(), 3);
        assert_eq!(metrics["execution"]["talent_executions"].as_u64().unwrap(), 3);
        assert_eq!(metrics["execution"]["success_rate"].as_f64().unwrap(), 100.0);
    }
}