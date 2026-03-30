use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Epic {
    /// Epic ID (e.g. "epic-xxxxx")
    pub id: String,
    pub title: String,
    pub description: String,
    /// draft | active | completed | cancelled | error
    pub status: String,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Feature {
    /// Feature ID (e.g. "feat-xxxxx")
    pub id: String,
    pub epic_id: String,
    pub title: String,
    pub specification: String,
    /// pending | assigned | in_progress | pr_submitted | in_review | merged | changes_requested | blocked | cancelled
    pub status: String,
    pub worker_id: Option<String>,
    pub branch_name: Option<String>,
    pub pr_number: Option<i64>,
    /// JSON array of feature IDs this feature depends on
    pub depends_on: String,
    /// low | medium | high | critical
    pub priority: String,
    pub blocked_reason: Option<String>,
    /// JSON-encoded ContextEnvelope
    pub context_envelope: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Worker {
    /// Worker ID (e.g. "worker-xxxxx")
    pub id: String,
    /// claude-code | kiro | agent-sdk | custom
    pub runtime_type: String,
    /// idle | busy | stalled | offline
    pub status: String,
    pub current_feature_id: Option<String>,
    pub worktree_path: Option<String>,
    pub pid: Option<i64>,
    pub last_heartbeat: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Escalation {
    pub id: String,
    pub feature_id: String,
    /// architecture_decision | security_review | requirement_clarification | conflict_resolution | scope_change
    pub escalation_type: String,
    pub title: String,
    pub context: String,
    pub question: String,
    /// JSON array of option objects
    pub options: String,
    pub pm_recommendation: Option<String>,
    pub pm_reasoning: Option<String>,
    /// pending | answered | expired
    pub status: String,
    pub po_answer: Option<String>,
    pub po_notes: Option<String>,
    /// low | medium | high | critical
    pub urgency: String,
    /// JSON array of feature IDs
    pub blocking_features: String,
    pub created_at: String,
    pub answered_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Permission {
    pub id: String,
    pub worker_id: String,
    pub feature_id: String,
    pub action: String,
    /// dependency_add | network_access | file_access | destructive_command | script_execution
    pub category: String,
    pub reason: String,
    /// low | medium | high
    pub risk_level: String,
    /// pending | granted | denied | expired
    pub status: String,
    /// auto | orchestrator | po
    pub decided_by: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub decided_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct ProgressLogEntry {
    pub id: i64,
    pub feature_id: String,
    pub worker_id: String,
    pub step: i64,
    pub total_steps: i64,
    pub current_task: String,
    /// JSON array of file paths
    pub files_modified: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Review {
    pub id: String,
    pub feature_id: String,
    pub pr_number: i64,
    /// pm | po
    pub reviewer: String,
    /// approved | changes_requested | escalated
    pub verdict: String,
    pub summary: Option<String>,
    /// JSON array of review comment objects
    pub comments: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Message {
    pub id: String,
    pub correlation_id: Option<String>,
    pub source: String,
    pub destination: String,
    /// ACP message type (e.g. "feature.assign")
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub message_type: String,
    /// JSON-encoded ACP message payload
    pub payload: String,
    pub timestamp: String,
}

/// Epic with progress info for GUI
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EpicWithProgress {
    #[serde(flatten)]
    pub epic: Epic,
    pub total_features: i64,
    pub merged_features: i64,
}

/// Action item for the GUI action queue (union of escalation + permission)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct ActionItem {
    /// "escalation" or "permission"
    pub action_type: String,
    pub id: String,
    pub title: String,
    pub question: String,
    /// low | medium | high | critical
    pub urgency: String,
    pub created_at: String,
}
