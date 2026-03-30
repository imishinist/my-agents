use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use acp_types::*;
use conducer_state::queries;

use crate::llm::{LlmClient, LlmError};

/// PM Agent — the project's "brain". Invoked by Orchestrator on events.
pub struct PmAgent {
    llm: Box<dyn LlmClient>,
    pool: SqlitePool,
}

/// Output of Epic decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposedFeature {
    pub title: String,
    pub specification: String,
    pub priority: String,
    pub depends_on_titles: Vec<String>,
    pub allowed_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DecompositionOutput {
    features: Vec<DecomposedFeature>,
}

#[derive(Debug, thiserror::Error)]
pub enum PmError {
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),
    #[error("DB error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

impl PmAgent {
    pub fn new(llm: Box<dyn LlmClient>, pool: SqlitePool) -> Self {
        Self { llm, pool }
    }

    /// Decompose an Epic into Features
    pub async fn decompose_epic(&self, epic_id: &str) -> Result<Vec<FeatureId>, PmError> {
        let epic = queries::get_epic(&self.pool, epic_id)
            .await?
            .ok_or_else(|| PmError::Parse(format!("Epic {} not found", epic_id)))?;

        let system_prompt = include_str!("../../../prompts/pm-system.md");
        let decompose_prompt = include_str!("../../../prompts/pm-decompose.md");

        let user_prompt = decompose_prompt
            .replace("{{epic_title}}", &epic.title)
            .replace("{{epic_description}}", &epic.description);

        let response = self.llm.complete(system_prompt, &user_prompt).await?;

        let output = Self::parse_decomposition(&response)?;
        let feature_ids = self
            .save_features(epic_id, &output.features)
            .await?;

        // Mark epic as active
        queries::update_epic_status(&self.pool, epic_id, "active").await?;

        Ok(feature_ids)
    }

    fn parse_decomposition(response: &str) -> Result<DecompositionOutput, PmError> {
        // Extract JSON from response (may be wrapped in markdown code blocks)
        let json_str = extract_json_block(response);

        serde_json::from_str::<DecompositionOutput>(json_str)
            .map_err(|e| PmError::Parse(format!("Failed to parse decomposition: {}. Response: {}", e, response)))
    }

    async fn save_features(
        &self,
        epic_id: &str,
        features: &[DecomposedFeature],
    ) -> Result<Vec<FeatureId>, PmError> {
        // First pass: create features and build title -> id mapping
        let mut title_to_id: std::collections::HashMap<String, FeatureId> =
            std::collections::HashMap::new();
        let mut feature_ids = Vec::new();

        for f in features {
            let id = FeatureId::new();
            title_to_id.insert(f.title.clone(), id.clone());
            feature_ids.push(id);
        }

        // Second pass: resolve depends_on titles to IDs and save
        for (i, f) in features.iter().enumerate() {
            let depends_on: Vec<String> = f
                .depends_on_titles
                .iter()
                .filter_map(|title| title_to_id.get(title).map(|id| id.as_str().to_string()))
                .collect();
            let depends_on_json = serde_json::to_string(&depends_on)
                .map_err(|e| PmError::Parse(e.to_string()))?;

            queries::create_feature(
                &self.pool,
                feature_ids[i].as_str(),
                epic_id,
                &f.title,
                &f.specification,
                &depends_on_json,
                &f.priority,
            )
            .await?;
        }

        Ok(feature_ids)
    }

    /// Get features that are ready to be assigned (all deps merged, status=pending)
    pub async fn get_assignable_features(&self) -> Result<Vec<conducer_state::models::Feature>, PmError> {
        let features = queries::get_ready_features(&self.pool).await?;
        Ok(features)
    }

    /// Assign ready features to idle workers. Returns list of (feature_id, worker_id) pairs.
    pub async fn assign_features(&self) -> Result<Vec<Assignment>, PmError> {
        let ready_features = self.get_assignable_features().await?;
        let idle_workers = queries::get_idle_workers(&self.pool).await?;

        let mut assignments = Vec::new();

        for (feature, worker) in ready_features.iter().zip(idle_workers.iter()) {
            let branch_name = format!(
                "feat/{}",
                feature.title.to_lowercase().replace(' ', "-")
            );

            queries::assign_feature_to_worker(
                &self.pool,
                &feature.id,
                &worker.id,
                &branch_name,
            )
            .await?;

            queries::update_worker_status(&self.pool, &worker.id, "busy").await?;

            // Build context envelope from feature data
            let context_envelope = self.build_context_envelope(&feature).await?;

            assignments.push(Assignment {
                feature_id: FeatureId::from_string(&feature.id),
                worker_id: WorkerId::from_string(&worker.id),
                branch_name,
                context_envelope,
            });
        }

        Ok(assignments)
    }

    async fn build_context_envelope(
        &self,
        feature: &conducer_state::models::Feature,
    ) -> Result<ContextEnvelope, PmError> {
        // Load architecture summary from project memory
        let architecture_summary = conducer_state::memory::get(&self.pool, "architecture")
            .await?
            .map(|e| e.content)
            .unwrap_or_default();

        // Load relevant interfaces from project memory
        let relevant_interfaces: Vec<String> = conducer_state::memory::list_by_category(&self.pool, "interface")
            .await?
            .into_iter()
            .map(|e| e.content)
            .collect();

        // Load constraints from project memory
        let constraints: Vec<String> = conducer_state::memory::list_by_category(&self.pool, "constraint")
            .await?
            .into_iter()
            .map(|e| e.content)
            .collect();

        // Parse allowed_paths from feature's context_envelope if set
        let allowed_paths = if let Some(envelope_json) = &feature.context_envelope {
            let envelope: serde_json::Value = serde_json::from_str(envelope_json)
                .unwrap_or(serde_json::json!({}));
            envelope["allowed_paths"]
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default()
        } else {
            vec!["src/**".to_string(), "tests/**".to_string()]
        };

        Ok(ContextEnvelope {
            architecture_summary,
            relevant_interfaces,
            allowed_paths,
            read_paths: vec![],
            constraints,
            branch_prefix: format!(
                "feat/{}",
                feature.title.to_lowercase().replace(' ', "-")
            ),
        })
    }
}

/// A feature-to-worker assignment
#[derive(Debug, Clone)]
pub struct Assignment {
    pub feature_id: FeatureId,
    pub worker_id: WorkerId,
    pub branch_name: String,
    pub context_envelope: ContextEnvelope,
}

/// Review result from PM Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub verdict: String,
    pub summary: String,
    #[serde(default)]
    pub comments: Vec<ReviewCommentOutput>,
    pub escalation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCommentOutput {
    pub file: String,
    pub line: u32,
    pub severity: String,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Clarification answer from PM Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationAnswer {
    pub answer: Option<String>,
    pub escalate: bool,
    pub escalation_reason: Option<String>,
}

impl PmAgent {
    /// Review a PR diff against the Feature specification.
    pub async fn review_pr(
        &self,
        feature_id: &str,
        pr_diff: &str,
    ) -> Result<ReviewResult, PmError> {
        let feature = queries::get_feature(&self.pool, feature_id)
            .await?
            .ok_or_else(|| PmError::Parse(format!("Feature {} not found", feature_id)))?;

        let system_prompt = include_str!("../../../prompts/pm-system.md");
        let review_template = include_str!("../../../prompts/pm-review.md");

        let user_prompt = review_template
            .replace("{{feature_title}}", &feature.title)
            .replace("{{feature_spec}}", &feature.specification)
            .replace("{{pr_diff}}", pr_diff);

        let response = self.llm.complete(system_prompt, &user_prompt).await?;
        let json_str = extract_json_block(&response);

        let result: ReviewResult = serde_json::from_str(json_str)
            .map_err(|e| PmError::Parse(format!("Failed to parse review: {}. Response: {}", e, response)))?;

        // Save review to DB
        let review_id = ReviewId::new();
        let comments_json = serde_json::to_string(&result.comments)
            .unwrap_or_else(|_| "[]".to_string());

        queries::create_review(
            &self.pool,
            review_id.as_str(),
            feature_id,
            feature.pr_number.unwrap_or(0),
            "pm",
            &result.verdict,
            Some(&result.summary),
            &comments_json,
        )
        .await?;

        // Update feature status based on verdict
        match result.verdict.as_str() {
            "approved" => {
                queries::update_feature_status(&self.pool, feature_id, "merged").await?;
            }
            "changes_requested" => {
                queries::update_feature_status(&self.pool, feature_id, "changes_requested")
                    .await?;
            }
            "escalated" => {
                queries::update_feature_status(&self.pool, feature_id, "blocked").await?;
            }
            _ => {}
        }

        Ok(result)
    }

    /// Answer a clarification request from a Worker.
    /// Returns an answer if PM can handle it, or signals escalation to PO.
    pub async fn answer_clarification(
        &self,
        feature_id: &str,
        question: &str,
        context: &str,
    ) -> Result<ClarificationAnswer, PmError> {
        let feature = queries::get_feature(&self.pool, feature_id)
            .await?
            .ok_or_else(|| PmError::Parse(format!("Feature {} not found", feature_id)))?;

        let system_prompt = include_str!("../../../prompts/pm-system.md");

        let user_prompt = format!(
            "A Worker implementing feature \"{}\" has a question.\n\n\
             ## Feature Specification\n{}\n\n\
             ## Worker's Question\n{}\n\n\
             ## Context\n{}\n\n\
             If you can answer confidently based on the specification and project context, respond with:\n\
             ```json\n{{\"answer\": \"your answer\", \"escalate\": false}}\n```\n\n\
             If you cannot answer and need PO input, respond with:\n\
             ```json\n{{\"answer\": null, \"escalate\": true, \"escalation_reason\": \"why PO needs to decide\"}}\n```",
            feature.title, feature.specification, question, context
        );

        let response = self.llm.complete(system_prompt, &user_prompt).await?;
        let json_str = extract_json_block(&response);

        serde_json::from_str(json_str)
            .map_err(|e| PmError::Parse(format!("Failed to parse clarification answer: {}. Response: {}", e, response)))
    }

    /// Check if all features of an epic are merged, and if so mark epic as completed.
    pub async fn check_epic_completion(&self, epic_id: &str) -> Result<bool, PmError> {
        let features = queries::list_features_by_epic(&self.pool, epic_id).await?;
        let all_merged = features.iter().all(|f| f.status == "merged");

        if all_merged && !features.is_empty() {
            queries::update_epic_status(&self.pool, epic_id, "completed").await?;
        }

        Ok(all_merged)
    }
}

/// Extract a JSON block from a response that may contain markdown fences
fn extract_json_block(response: &str) -> &str {
    // Try to find ```json ... ``` block
    if let Some(start) = response.find("```json") {
        let after_fence = &response[start + 7..];
        if let Some(end) = after_fence.find("```") {
            return after_fence[..end].trim();
        }
    }
    // Try to find ``` ... ``` block
    if let Some(start) = response.find("```") {
        let after_fence = &response[start + 3..];
        if let Some(end) = after_fence.find("```") {
            return after_fence[..end].trim();
        }
    }
    // Try raw JSON (starts with { or [)
    let trimmed = response.trim();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return trimmed;
    }
    response.trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_block_fenced() {
        let response = r#"Here is the decomposition:

```json
{"features": []}
```

Done."#;
        assert_eq!(extract_json_block(response), r#"{"features": []}"#);
    }

    #[test]
    fn test_extract_json_block_raw() {
        let response = r#"{"features": []}"#;
        assert_eq!(extract_json_block(response), r#"{"features": []}"#);
    }

    #[test]
    fn test_parse_decomposition() {
        let json = r#"{"features": [
            {
                "title": "Auth module",
                "specification": "Build auth",
                "priority": "high",
                "depends_on_titles": [],
                "allowed_paths": ["src/auth/**"]
            },
            {
                "title": "Login UI",
                "specification": "Build login page",
                "priority": "medium",
                "depends_on_titles": ["Auth module"],
                "allowed_paths": ["src/ui/**"]
            }
        ]}"#;

        let output = PmAgent::parse_decomposition(json).unwrap();
        assert_eq!(output.features.len(), 2);
        assert_eq!(output.features[0].title, "Auth module");
        assert_eq!(output.features[1].depends_on_titles, vec!["Auth module"]);
    }
}
