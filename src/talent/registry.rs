// src/talent/registry.rs
// Talent registry for managing and executing talents

use std::collections::HashMap;
use std::sync::{ Arc, RwLock, OnceLock };
use serde_json::Value;

use crate::types::{ ActionPayload, TalentResult };
use crate::talent::types::Talent;

/*

      C.Y.R.E - T.A.L.E.N.T - R.E.G.I.S.T.R.Y
      
      Central registry for talent management and execution

*/

//=============================================================================
// TALENT REGISTRY
//=============================================================================

#[derive(Debug)]
pub struct TalentRegistry {
    talents: Arc<RwLock<HashMap<String, Talent>>>,
    pipelines: Arc<RwLock<HashMap<String, Vec<String>>>>,
    execution_stats: Arc<RwLock<HashMap<String, ExecutionStats>>>,
}

#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time: u64,
    pub last_execution_time: u64,
}

impl TalentRegistry {
    pub fn new() -> Self {
        Self {
            talents: Arc::new(RwLock::new(HashMap::new())),
            pipelines: Arc::new(RwLock::new(HashMap::new())),
            execution_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new talent
    pub fn register_talent(&self, talent: Talent) -> Result<(), String> {
        let mut talents = self.talents.write().unwrap();

        if talents.contains_key(&talent.id) {
            return Err(format!("Talent with ID '{}' already exists", talent.id));
        }

        talents.insert(talent.id.clone(), talent);
        Ok(())
    }

    /// Execute a single talent
    pub async fn execute_talent(&self, talent_id: &str, payload: ActionPayload) -> TalentResult {
        let start_time = std::time::Instant::now();

        let talent = {
            let talents = self.talents.read().unwrap();
            match talents.get(talent_id) {
                Some(talent) => talent.clone(),
                None => {
                    return TalentResult {
                        success: false,
                        value: Value::Null,
                        payload: payload.clone(),
                        message: "Talent not found".to_string(),
                        execution_time: start_time.elapsed().as_millis() as u64,
                        error: Some(format!("Talent not found: {}", talent_id)),
                        metadata: Some(
                            serde_json::json!({
                            "talent_id": talent_id,
                            "error_type": "not_found"
                        })
                        ),
                    };
                }
            }
        };

        let result = talent.execute(payload).await;
        let execution_time = start_time.elapsed().as_millis() as u64;

        // Update execution statistics
        self.update_execution_stats(talent_id, &result, execution_time);

        result
    }

    /// Execute a talent pipeline
    pub async fn execute_pipeline(
        &self,
        pipeline_id: &str,
        payload: ActionPayload
    ) -> TalentResult {
        let start_time = std::time::Instant::now();

        let talent_ids = {
            let pipelines = self.pipelines.read().unwrap();
            match pipelines.get(pipeline_id) {
                Some(ids) => ids.clone(),
                None => {
                    return TalentResult {
                        success: false,
                        value: Value::Null,
                        payload: payload.clone(),
                        message: "Pipeline not found".to_string(),
                        execution_time: start_time.elapsed().as_millis() as u64,
                        error: Some(format!("Pipeline '{}' not found", pipeline_id)),
                        metadata: Some(
                            serde_json::json!({
                            "pipeline_id": pipeline_id,
                            "error_type": "not_found"
                        })
                        ),
                    };
                }
            }
        };

        let mut current_payload = payload;
        let mut errors = Vec::new();
        let talent_count = talent_ids.len(); // Store the count before moving

        for talent_id in &talent_ids {
            // Use reference to avoid moving
            let result = self.execute_talent(talent_id, current_payload.clone()).await;

            if result.success {
                current_payload = result.payload;
            } else {
                errors.push(
                    format!("Talent '{}': {}", talent_id, result.error.unwrap_or_default())
                );

                if !errors.is_empty() {
                    return TalentResult {
                        success: false,
                        value: current_payload.clone(),
                        payload: current_payload,
                        message: "Pipeline execution failed".to_string(),
                        execution_time: start_time.elapsed().as_millis() as u64,
                        error: Some(format!("Talent execution failed: {}", errors.join(", "))),
                        metadata: Some(
                            serde_json::json!({
                            "pipeline_id": pipeline_id,
                            "failed_talents": errors,
                            "execution_time": start_time.elapsed().as_millis()
                        })
                        ),
                    };
                }
            }
        }

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Update pipeline metadata
        let mut result = TalentResult {
            success: true,
            value: current_payload.clone(),
            payload: current_payload,
            message: "Pipeline execution completed".to_string(),
            execution_time,
            error: None,
            metadata: Some(
                serde_json::json!({
                "pipeline_id": pipeline_id,
                "talents_executed": talent_count, // Use stored count
                "execution_time": execution_time
            })
            ),
        };

        if let Some(metadata) = result.metadata.as_mut() {
            if let Some(obj) = metadata.as_object_mut() {
                obj.insert("pipeline_success".to_string(), Value::Bool(true));
            }
        }

        result
    }

    /// Register a talent pipeline
    pub fn register_pipeline(
        &self,
        pipeline_id: String,
        talent_ids: Vec<String>
    ) -> Result<(), String> {
        // Validate that all talents exist
        {
            let talents = self.talents.read().unwrap();
            for talent_id in &talent_ids {
                if !talents.contains_key(talent_id) {
                    return Err(format!("Talent '{}' not found", talent_id));
                }
            }
        }

        let mut pipelines = self.pipelines.write().unwrap();
        pipelines.insert(pipeline_id, talent_ids);
        Ok(())
    }

    /// Get talent by ID
    pub fn get_talent(&self, talent_id: &str) -> Option<Talent> {
        let talents = self.talents.read().unwrap();
        talents.get(talent_id).cloned()
    }

    /// List all talents
    pub fn list_talents(&self) -> Vec<String> {
        let talents = self.talents.read().unwrap();
        talents.keys().cloned().collect()
    }

    /// Get execution statistics
    pub fn get_execution_stats(&self, talent_id: &str) -> Option<ExecutionStats> {
        let stats = self.execution_stats.read().unwrap();
        stats.get(talent_id).cloned()
    }

    /// Update execution statistics
    fn update_execution_stats(&self, talent_id: &str, result: &TalentResult, execution_time: u64) {
        let mut stats = self.execution_stats.write().unwrap();
        let stat = stats.entry(talent_id.to_string()).or_insert_with(ExecutionStats::default);

        stat.total_executions += 1;
        stat.last_execution_time = execution_time;

        if result.success {
            stat.successful_executions += 1;
        } else {
            stat.failed_executions += 1;
        }

        // Update average execution time
        stat.average_execution_time =
            (stat.average_execution_time * (stat.total_executions - 1) + execution_time) /
            stat.total_executions;
    }

    /// Remove a talent
    pub fn remove_talent(&self, talent_id: &str) -> Result<(), String> {
        let mut talents = self.talents.write().unwrap();

        if talents.remove(talent_id).is_some() {
            // Also remove from execution stats
            let mut stats = self.execution_stats.write().unwrap();
            stats.remove(talent_id);

            Ok(())
        } else {
            Err(format!("Talent '{}' not found", talent_id))
        }
    }

    /// Clear all talents
    pub fn clear(&self) {
        let mut talents = self.talents.write().unwrap();
        let mut pipelines = self.pipelines.write().unwrap();
        let mut stats = self.execution_stats.write().unwrap();

        talents.clear();
        pipelines.clear();
        stats.clear();
    }
}

//=============================================================================
// GLOBAL TALENT REGISTRY
//=============================================================================

static GLOBAL_TALENT_REGISTRY: OnceLock<TalentRegistry> = OnceLock::new();

/// Get the global talent registry
pub fn get_talent_registry() -> &'static TalentRegistry {
    GLOBAL_TALENT_REGISTRY.get_or_init(|| TalentRegistry::new())
}

//=============================================================================
// CONVENIENCE FUNCTIONS
//=============================================================================

/// Register a talent globally
pub fn register_talent(talent: Talent) -> Result<(), String> {
    get_talent_registry().register_talent(talent)
}

/// Execute a talent globally
pub async fn execute_talent(talent_id: &str, payload: ActionPayload) -> TalentResult {
    get_talent_registry().execute_talent(talent_id, payload).await
}

/// Execute a pipeline globally
pub async fn execute_pipeline(pipeline_id: &str, payload: ActionPayload) -> TalentResult {
    get_talent_registry().execute_pipeline(pipeline_id, payload).await
}

/// Register a pipeline globally
pub fn register_pipeline(pipeline_id: String, talent_ids: Vec<String>) -> Result<(), String> {
    get_talent_registry().register_pipeline(pipeline_id, talent_ids)
}
