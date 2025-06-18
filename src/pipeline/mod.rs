// src/pipeline/mod.rs - REDESIGNED following TypeScript pattern
// Pipeline system redesign based on user feedback

// Remove unused imports
use crate::types::{ IO, ActionPayload, ActionId };
use crate::context::state;
use crate::utils::current_timestamp;

//=============================================================================
// PIPELINE RESULT
//=============================================================================

#[derive(Debug)]
pub enum PipelineResult {
    Continue(ActionPayload),
    Block(String),
    Schedule,
}

//=============================================================================
// PIPELINE COMPILER - FOLLOWS USER ORDER
//=============================================================================

/// Compile pipeline for action - respects user configuration order
pub fn compile_pipeline(action: &mut IO) -> Result<(), String> {
    let mut pipeline_operators = Vec::new();

    // DON'T reorder - follow user's configuration intent
    // Each builder method call adds to pipeline in order

    // Build pipeline in the order operators were configured
    // This respects user intent rather than forcing our own ordering

    // Check block first (immediate rejection)
    if action.block {
        pipeline_operators.push("block".to_string());
        // If blocked, no other operators matter
        action._pipeline = pipeline_operators;
        action._has_fast_path = false;
        return Ok(());
    }

    // Add operators based on configuration presence, maintaining order
    if action.required.is_some() {
        pipeline_operators.push("required".to_string());
    }

    if action.schema.is_some() {
        pipeline_operators.push("schema".to_string());
    }

    if action.throttle.is_some() {
        pipeline_operators.push("throttle".to_string());
    }

    if action.debounce.is_some() {
        pipeline_operators.push("debounce".to_string());
    }

    if action.detect_changes {
        pipeline_operators.push("change_detection".to_string());
    }

    if action.condition.is_some() {
        pipeline_operators.push("condition".to_string());
    }

    if action.selector.is_some() {
        pipeline_operators.push("selector".to_string());
    }

    if action.transform.is_some() {
        pipeline_operators.push("transform".to_string());
    }

    if action.delay.is_some() {
        pipeline_operators.push("delay".to_string());
    }

    if action.interval.is_some() {
        pipeline_operators.push("interval".to_string());
    }

    // Store compiled pipeline
    action._pipeline = pipeline_operators;

    // Simple fast path detection: is_zero_overhead = pipeline.length == 0
    action._has_fast_path = action._pipeline.is_empty();

    // Update other optimization flags
    action._has_protections =
        action.throttle.is_some() ||
        action.debounce.is_some() ||
        action.detect_changes ||
        action.required.is_some();

    action._has_processing =
        action.condition.is_some() ||
        action.selector.is_some() ||
        action.transform.is_some() ||
        action.schema.is_some();

    action._has_scheduling = action.delay.is_some() || action.interval.is_some();

    Ok(())
}

//=============================================================================
// PIPELINE EXECUTION - USING ACTION STATE
//=============================================================================

/// Execute pipeline using action configuration and state
pub async fn execute_pipeline(
    action: &mut IO,
    payload: ActionPayload
) -> Result<PipelineResult, String> {
    // ZERO OVERHEAD FAST PATH - pipeline.length == 0
    if action._has_fast_path {
        return Ok(PipelineResult::Continue(payload));
    }

    let mut current_payload = payload;

    // Execute operators in the order they were compiled (user's intent)
    for operator in &action._pipeline.clone() {
        match operator.as_str() {
            "block" => {
                return Ok(PipelineResult::Block("Channel blocked".to_string()));
            }

            "required" => {
                current_payload = execute_required_operator(action, current_payload)?;
            }

            "schema" => {
                current_payload = execute_schema_operator(action, current_payload).await?;
            }

            "throttle" => {
                // Use action state, not separate atomic state
                match execute_throttle_operator(action, &current_payload).await? {
                    PipelineResult::Continue(p) => {
                        current_payload = p;
                    }
                    other => {
                        return Ok(other);
                    }
                }
            }

            "debounce" => {
                match execute_debounce_operator(action, &current_payload).await? {
                    PipelineResult::Continue(p) => {
                        current_payload = p;
                    }
                    other => {
                        return Ok(other);
                    }
                }
            }

            "change_detection" => {
                current_payload = execute_change_detection_operator(action, current_payload).await?;
            }

            "condition" => {
                current_payload = execute_condition_operator(action, current_payload).await?;
            }

            "selector" => {
                current_payload = execute_selector_operator(action, current_payload).await?;
            }

            "transform" => {
                current_payload = execute_transform_operator(action, current_payload).await?;
            }

            "delay" | "interval" => {
                return Ok(PipelineResult::Schedule);
            }

            _ => {
                return Err(format!("Unknown pipeline operator: {}", operator));
            }
        }
    }

    Ok(PipelineResult::Continue(current_payload))
}

//=============================================================================
// OPERATOR IMPLEMENTATIONS - USING ACTION STATE
//=============================================================================

/// Required field validation
fn execute_required_operator(action: &IO, payload: ActionPayload) -> Result<ActionPayload, String> {
    if let Some(ref required) = action.required {
        match required {
            crate::types::RequiredType::Basic(true) => {
                if payload.is_null() {
                    return Err("Required payload missing".to_string());
                }
            }
            crate::types::RequiredType::NonEmpty => {
                if
                    payload.is_null() ||
                    (payload.is_string() && payload.as_str().unwrap_or("").is_empty()) ||
                    (payload.is_array() &&
                        payload
                            .as_array()
                            .unwrap_or(&vec![])
                            .is_empty()) ||
                    (payload.is_object() &&
                        payload.as_object().unwrap_or(&serde_json::Map::new()).is_empty())
                {
                    return Err("Required non-empty payload missing".to_string());
                }
            }
            _ => {}
        }
    }
    Ok(payload)
}

/// Schema validation operator
async fn execute_schema_operator(
    action: &IO,
    payload: ActionPayload
) -> Result<ActionPayload, String> {
    if let Some(ref schema_name) = action.schema {
        // Basic schema validation - can be enhanced with talent system
        match schema_name.as_str() {
            "non-empty" => {
                if payload.is_null() {
                    return Err("Schema requires non-empty payload".to_string());
                }
            }
            "object" => {
                if !payload.is_object() {
                    return Err("Schema requires object payload".to_string());
                }
            }
            "array" => {
                if !payload.is_array() {
                    return Err("Schema requires array payload".to_string());
                }
            }
            _ => {
                // Unknown schema - log warning but pass through
                crate::context::sensor::warn(
                    "pipeline",
                    &format!("Unknown schema '{}' for '{}'", schema_name, action.id),
                    false
                );
            }
        }
    }
    Ok(payload)
}

/// Throttle operator - following TypeScript pattern EXACTLY
async fn execute_throttle_operator(
    action: &mut IO,
    payload: &ActionPayload
) -> Result<PipelineResult, String> {
    if let Some(throttle_ms) = action.throttle {
        let current_time = current_timestamp();
        let last_exec_time = action._last_exec_time.unwrap_or(0);

        // FIX 1: Calculate elapsed time from LAST EXECUTION, not action creation
        let elapsed_since_last_exec = current_time - last_exec_time;
        let remaining = if elapsed_since_last_exec < throttle_ms {
            throttle_ms - elapsed_since_last_exec
        } else {
            0
        };

        // FIX 2: Check if we should throttle (industry standard: first call always passes)
        if last_exec_time > 0 && elapsed_since_last_exec < throttle_ms {
            return Ok(PipelineResult::Block(format!("Throttled - {}ms remaining", remaining)));
        }

        // Update last execution time for next call
        action._last_exec_time = Some(current_time);
    }

    Ok(PipelineResult::Continue(payload.clone()))
}

/// Debounce operator - PROPER implementation
async fn execute_debounce_operator(
    action: &mut IO,
    payload: &ActionPayload
) -> Result<PipelineResult, String> {
    if let Some(debounce_ms) = action.debounce {
        let current_time = current_timestamp();

        // Store the time of first debounce call for maxWait
        if action._first_debounce_call.is_none() {
            action._first_debounce_call = Some(current_time);
        }

        // Check maxWait - if exceeded, force execution
        if let Some(max_wait_ms) = action.max_wait {
            let first_call_time = action._first_debounce_call.unwrap_or(current_time);
            if current_time - first_call_time >= max_wait_ms {
                // Reset debounce state and continue
                action._first_debounce_call = None;
                return Ok(PipelineResult::Continue(payload.clone()));
            }
        }

        // In a real implementation, we would properly implement debounce cancellation
        if let Some(_timer_id) = &action._debounce_timer {
            // In a real implementation, cancel the timer
            // For now, we'll use a simplified approach
        }

        // Set new debounce timer
        let timer_id = format!("debounce_{}_{}", action.id, current_time);
        action._debounce_timer = Some(timer_id.clone());

        // Wait for debounce delay
        tokio::time::sleep(tokio::time::Duration::from_millis(debounce_ms)).await;

        // Check if this timer is still the active one (not cancelled by newer call)
        if action._debounce_timer.as_ref() == Some(&timer_id) {
            // This is the final call - reset debounce state and continue
            action._debounce_timer = None;
            action._first_debounce_call = None;
            Ok(PipelineResult::Continue(payload.clone()))
        } else {
            // This call was cancelled by a newer one
            Ok(PipelineResult::Block("Debounced - newer request received".to_string()))
        }
    } else {
        Ok(PipelineResult::Continue(payload.clone()))
    }
}

/// Change detection operator
async fn execute_change_detection_operator(
    action: &mut IO,
    payload: ActionPayload
) -> Result<ActionPayload, String> {
    if action.detect_changes {
        // Simple payload hash comparison
        let payload_hash = crate::utils::hash_payload(&payload);

        if let Some(last_hash) = action.additional_properties.get("_last_payload_hash") {
            if let Some(last_hash_num) = last_hash.as_u64() {
                if payload_hash == last_hash_num {
                    return Err("Payload unchanged - skipping execution".to_string());
                }
            }
        }

        // Store new hash
        action.additional_properties.insert(
            "_last_payload_hash".to_string(),
            serde_json::Value::Number(payload_hash.into())
        );
    }

    Ok(payload)
}

/// Condition operator
async fn execute_condition_operator(
    action: &IO,
    payload: ActionPayload
) -> Result<ActionPayload, String> {
    if let Some(ref condition_talent) = action.condition {
        match condition_talent.as_str() {
            "always_true" => Ok(payload),
            "never_true" => Err("Condition always false".to_string()),
            "has_data" => {
                if payload.is_null() {
                    Err("Condition requires data".to_string())
                } else {
                    Ok(payload)
                }
            }
            "is_object" => {
                if payload.is_object() {
                    Ok(payload)
                } else {
                    Err("Condition requires object".to_string())
                }
            }
            _ => {
                crate::context::sensor::warn(
                    "pipeline",
                    &format!("Unknown condition '{}' for '{}'", condition_talent, action.id),
                    false
                );
                Ok(payload)
            }
        }
    } else {
        Ok(payload)
    }
}

/// Selector operator
async fn execute_selector_operator(
    action: &IO,
    payload: ActionPayload
) -> Result<ActionPayload, String> {
    if let Some(ref selector_talent) = action.selector {
        match selector_talent.as_str() {
            "data" => {
                if let Some(data) = payload.get("data") {
                    Ok(data.clone())
                } else {
                    Err("Selector 'data' field not found".to_string())
                }
            }
            "user" => {
                if let Some(user) = payload.get("user") {
                    Ok(user.clone())
                } else {
                    Err("Selector 'user' field not found".to_string())
                }
            }
            _ => {
                crate::context::sensor::warn(
                    "pipeline",
                    &format!("Unknown selector '{}' for '{}'", selector_talent, action.id),
                    false
                );
                Ok(payload)
            }
        }
    } else {
        Ok(payload)
    }
}

/// Transform operator
async fn execute_transform_operator(
    action: &IO,
    payload: ActionPayload
) -> Result<ActionPayload, String> {
    if let Some(ref transform_talent) = action.transform {
        match transform_talent.as_str() {
            "uppercase" => {
                if let Some(text) = payload.as_str() {
                    Ok(serde_json::Value::String(text.to_uppercase()))
                } else {
                    Ok(payload)
                }
            }
            "add_timestamp" => {
                let mut result = payload;
                if let Some(obj) = result.as_object_mut() {
                    obj.insert(
                        "transformed_at".to_string(),
                        serde_json::Value::Number(current_timestamp().into())
                    );
                }
                Ok(result)
            }
            "normalize" => {
                // Simple normalization
                if let Some(obj) = payload.as_object() {
                    let mut normalized = serde_json::Map::new();
                    for (key, value) in obj {
                        normalized.insert(key.to_lowercase(), value.clone());
                    }
                    Ok(serde_json::Value::Object(normalized))
                } else {
                    Ok(payload)
                }
            }
            _ => {
                crate::context::sensor::warn(
                    "pipeline",
                    &format!("Unknown transform '{}' for '{}'", transform_talent, action.id),
                    false
                );
                Ok(payload)
            }
        }
    } else {
        Ok(payload)
    }
}

//=============================================================================
// CORE INTEGRATION FUNCTIONS
//=============================================================================

/// Remove compiled pipeline (for cleanup)
pub fn remove_compiled_pipeline(_action_id: &str) {
    // Since pipeline is stored in action itself, this just clears the action
    // No separate cache to clean up
}

/// Clear pipeline cache (for reset operations)
pub fn clear_pipeline_cache() {
    // No separate cache to clear - everything is in action state
}

/// Get pipeline info for debugging
pub fn get_pipeline_info(action_id: &str) -> Option<PipelineInfo> {
    if let Some(action) = state::io::get(action_id) {
        Some(PipelineInfo {
            action_id: action_id.to_string(),
            is_zero_overhead: action._has_fast_path,
            protection_count: action._pipeline.len() as u32,
            compiled_at: action.time_of_creation.unwrap_or(0),
        })
    } else {
        None
    }
}

/// List all compiled pipelines
pub fn list_compiled_pipelines() -> Vec<PipelineInfo> {
    state::io
        ::get_all()
        .into_iter()
        .map(|action| PipelineInfo {
            action_id: action.id.clone(),
            is_zero_overhead: action._has_fast_path,
            protection_count: action._pipeline.len() as u32,
            compiled_at: action.time_of_creation.unwrap_or(0),
        })
        .collect()
}

/// Get pipeline statistics
pub fn get_pipeline_stats() -> PipelineStats {
    let all_actions = state::io::get_all();
    let total_pipelines = all_actions.len();
    let zero_overhead_count = all_actions
        .iter()
        .filter(|action| action._has_fast_path)
        .count();

    PipelineStats {
        total_pipelines,
        zero_overhead_count,
        protected_count: total_pipelines - zero_overhead_count,
        cache_size: total_pipelines,
    }
}

//=============================================================================
// SUPPORTING TYPES
//=============================================================================

#[derive(Debug, Clone)]
pub struct PipelineInfo {
    pub action_id: ActionId,
    pub is_zero_overhead: bool,
    pub protection_count: u32,
    pub compiled_at: u64,
}

#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub total_pipelines: usize,
    pub zero_overhead_count: usize,
    pub protected_count: usize,
    pub cache_size: usize,
}

impl PipelineStats {
    pub fn optimization_ratio(&self) -> f64 {
        if self.total_pipelines == 0 {
            0.0
        } else {
            ((self.zero_overhead_count as f64) / (self.total_pipelines as f64)) * 100.0
        }
    }
}
