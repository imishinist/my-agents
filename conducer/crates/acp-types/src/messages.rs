use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::ids::*;

// --- Enums ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EpicStatus {
    Draft,
    Active,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FeatureStatus {
    Pending,
    Assigned,
    InProgress,
    PrSubmitted,
    InReview,
    Merged,
    ChangesRequested,
    Blocked,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    Idle,
    Busy,
    Stalled,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    Approved,
    ChangesRequested,
    Escalated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EscalationType {
    ArchitectureDecision,
    SecurityReview,
    RequirementClarification,
    ConflictResolution,
    ScopeChange,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Urgency {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermissionCategory {
    DependencyAdd,
    NetworkAccess,
    FileAccess,
    DestructiveCommand,
    ScriptExecution,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkerHealth {
    Ok,
    Degraded,
    Stuck,
}

// --- Context Envelope ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ContextEnvelope {
    pub architecture_summary: String,
    pub relevant_interfaces: Vec<String>,
    pub allowed_paths: Vec<String>,
    pub read_paths: Vec<String>,
    pub constraints: Vec<String>,
    pub branch_prefix: String,
}

// --- Message Payloads ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessagePayload {
    // PM -> Worker
    #[serde(rename = "feature.assign")]
    FeatureAssign {
        feature_id: FeatureId,
        epic_id: EpicId,
        title: String,
        specification: String,
        context_envelope: ContextEnvelope,
        priority: Priority,
        depends_on: Vec<FeatureId>,
    },

    #[serde(rename = "review.feedback")]
    ReviewFeedback {
        feature_id: FeatureId,
        pr_number: u64,
        verdict: ReviewVerdict,
        summary: Option<String>,
        comments: Vec<ReviewComment>,
    },

    #[serde(rename = "dependency.resolved")]
    DependencyResolved {
        resolved_feature_id: FeatureId,
        summary: String,
        updated_interfaces: Vec<String>,
    },

    #[serde(rename = "feature.cancel")]
    FeatureCancel {
        feature_id: FeatureId,
        reason: String,
    },

    // Worker -> PM
    #[serde(rename = "feature.accepted")]
    FeatureAccepted {
        feature_id: FeatureId,
        estimated_steps: u32,
        branch_name: String,
        worktree_path: String,
    },

    #[serde(rename = "progress.update")]
    ProgressUpdate {
        feature_id: FeatureId,
        step: u32,
        total_steps: u32,
        current_task: String,
        files_modified: Vec<String>,
        status: FeatureStatus,
    },

    #[serde(rename = "pr.submitted")]
    PrSubmitted {
        feature_id: FeatureId,
        pr_number: u64,
        branch_name: String,
        summary: String,
        files_changed: Vec<String>,
        test_results: TestResults,
        lint_clean: bool,
    },

    #[serde(rename = "clarification.request")]
    ClarificationRequest {
        feature_id: FeatureId,
        question: String,
        context: String,
        options: Vec<String>,
        blocking: bool,
    },

    // Orchestrator <-> Worker
    #[serde(rename = "heartbeat.request")]
    HeartbeatRequest {},

    #[serde(rename = "heartbeat.response")]
    HeartbeatResponse {
        feature_id: FeatureId,
        status: FeatureStatus,
        last_action: String,
        health: WorkerHealth,
    },

    // PM -> PO
    #[serde(rename = "escalation.request")]
    EscalationRequest {
        escalation_id: EscalationId,
        feature_id: FeatureId,
        escalation_type: EscalationType,
        title: String,
        context: String,
        question: String,
        options: Vec<EscalationOption>,
        pm_recommendation: Option<String>,
        pm_reasoning: Option<String>,
        urgency: Urgency,
        blocking_features: Vec<FeatureId>,
    },

    #[serde(rename = "status.report")]
    StatusReport {
        epic_id: EpicId,
        summary: String,
        features: Vec<FeatureSummary>,
        blockers: Vec<String>,
        next_actions: Vec<String>,
    },

    // PO -> PM
    #[serde(rename = "escalation.response")]
    EscalationResponse {
        escalation_id: EscalationId,
        answer: String,
        notes: Option<String>,
    },

    // Permission management
    #[serde(rename = "permission.request")]
    PermissionRequest {
        worker_id: WorkerId,
        feature_id: FeatureId,
        action: String,
        category: PermissionCategory,
        reason: String,
    },

    #[serde(rename = "permission.response")]
    PermissionResponse {
        granted: bool,
        reason: Option<String>,
    },

    #[serde(rename = "permission.escalation")]
    PermissionEscalation {
        permission_id: PermissionId,
        worker_id: WorkerId,
        feature_id: FeatureId,
        action: String,
        category: PermissionCategory,
        reason: String,
        worker_context: String,
    },

    #[serde(rename = "permission.decision")]
    PermissionDecision {
        permission_id: PermissionId,
        granted: bool,
        notes: Option<String>,
    },
}

impl MessagePayload {
    pub fn message_type(&self) -> &'static str {
        match self {
            Self::FeatureAssign { .. } => "feature.assign",
            Self::ReviewFeedback { .. } => "review.feedback",
            Self::DependencyResolved { .. } => "dependency.resolved",
            Self::FeatureCancel { .. } => "feature.cancel",
            Self::FeatureAccepted { .. } => "feature.accepted",
            Self::ProgressUpdate { .. } => "progress.update",
            Self::PrSubmitted { .. } => "pr.submitted",
            Self::ClarificationRequest { .. } => "clarification.request",
            Self::HeartbeatRequest { .. } => "heartbeat.request",
            Self::HeartbeatResponse { .. } => "heartbeat.response",
            Self::EscalationRequest { .. } => "escalation.request",
            Self::StatusReport { .. } => "status.report",
            Self::EscalationResponse { .. } => "escalation.response",
            Self::PermissionRequest { .. } => "permission.request",
            Self::PermissionResponse { .. } => "permission.response",
            Self::PermissionEscalation { .. } => "permission.escalation",
            Self::PermissionDecision { .. } => "permission.decision",
        }
    }
}

// --- Supporting types ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReviewComment {
    pub file: String,
    pub line: u32,
    pub severity: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TestResults {
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EscalationOption {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pros: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cons: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeatureSummary {
    pub id: FeatureId,
    pub title: String,
    pub status: FeatureStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker: Option<WorkerId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<String>,
}
