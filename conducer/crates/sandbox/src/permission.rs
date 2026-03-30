use crate::policy::{ApprovalLevel, CommandPolicy, NetworkPolicy};

/// Permission checker that combines command and network policies.
pub struct PermissionChecker {
    pub command_policy: CommandPolicy,
    pub network_policy: NetworkPolicy,
}

/// Result of a permission check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDecision {
    /// Allowed without any approval.
    Allow,
    /// Needs orchestrator/PM auto-approval.
    NeedsOrchestratorApproval { reason: String },
    /// Needs PO manual approval.
    NeedsPoApproval { reason: String },
    /// Denied outright.
    Denied { reason: String },
}

impl PermissionChecker {
    pub fn new(command_policy: CommandPolicy, network_policy: NetworkPolicy) -> Self {
        Self {
            command_policy,
            network_policy,
        }
    }

    pub fn check_command(&self, command: &str) -> PermissionDecision {
        match self.command_policy.check(command) {
            ApprovalLevel::AutoAllow => PermissionDecision::Allow,
            ApprovalLevel::OrchestratorApprove => PermissionDecision::NeedsOrchestratorApproval {
                reason: format!("Command requires orchestrator approval: {}", command),
            },
            ApprovalLevel::PoApprove => PermissionDecision::NeedsPoApproval {
                reason: format!("Command requires PO approval: {}", command),
            },
            ApprovalLevel::Deny => PermissionDecision::Denied {
                reason: format!("Command is denied by policy: {}", command),
            },
        }
    }

    pub fn check_network(&self, domain: &str) -> PermissionDecision {
        match self.network_policy.check_domain(domain) {
            ApprovalLevel::AutoAllow => PermissionDecision::Allow,
            ApprovalLevel::OrchestratorApprove => PermissionDecision::NeedsOrchestratorApproval {
                reason: format!("Network access to {} requires approval", domain),
            },
            _ => PermissionDecision::Denied {
                reason: format!("Network access to {} is denied", domain),
            },
        }
    }
}
