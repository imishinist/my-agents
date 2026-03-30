use serde::{Deserialize, Serialize};

/// Command execution policy loaded from command-policy.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPolicy {
    #[serde(default)]
    pub auto_allow: CommandList,
    #[serde(default)]
    pub orchestrator_approve: CommandList,
    #[serde(default)]
    pub po_approve: CommandList,
    #[serde(default)]
    pub deny: CommandList,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandList {
    #[serde(default)]
    pub commands: Vec<String>,
}

/// Network allowlist loaded from network-allowlist.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    #[serde(default)]
    pub auto_allow: DomainList,
    #[serde(default)]
    pub conditional_allow: DomainList,
    #[serde(default)]
    pub deny: DomainList,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomainList {
    #[serde(default)]
    pub domains: Vec<String>,
}

/// What level of approval a command requires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalLevel {
    AutoAllow,
    OrchestratorApprove,
    PoApprove,
    Deny,
}

impl CommandPolicy {
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Determine the approval level for a given command string.
    pub fn check(&self, command: &str) -> ApprovalLevel {
        if self.deny.matches(command) {
            return ApprovalLevel::Deny;
        }
        if self.auto_allow.matches(command) {
            return ApprovalLevel::AutoAllow;
        }
        if self.orchestrator_approve.matches(command) {
            return ApprovalLevel::OrchestratorApprove;
        }
        if self.po_approve.matches(command) {
            return ApprovalLevel::PoApprove;
        }
        // Unknown commands require PO approval
        ApprovalLevel::PoApprove
    }
}

impl NetworkPolicy {
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Check if a domain is allowed, conditionally allowed, or denied.
    pub fn check_domain(&self, domain: &str) -> ApprovalLevel {
        if self.auto_allow.matches_domain(domain) {
            return ApprovalLevel::AutoAllow;
        }
        if self.conditional_allow.matches_domain(domain) {
            return ApprovalLevel::OrchestratorApprove;
        }
        ApprovalLevel::Deny
    }
}

impl CommandList {
    /// Check if a command matches any pattern in this list.
    /// Supports trailing `*` as a simple wildcard.
    fn matches(&self, command: &str) -> bool {
        self.commands.iter().any(|pattern| {
            if let Some(prefix) = pattern.strip_suffix(" *") {
                // "cargo add *" matches "cargo add serde"
                command == prefix || command.starts_with(&format!("{} ", prefix))
            } else if pattern.ends_with('*') {
                command.starts_with(pattern.trim_end_matches('*'))
            } else {
                command == pattern || command.starts_with(&format!("{} ", pattern))
            }
        })
    }
}

impl DomainList {
    fn matches_domain(&self, domain: &str) -> bool {
        self.domains.iter().any(|pattern| {
            if pattern == "*" {
                true
            } else if let Some(suffix) = pattern.strip_prefix("*.") {
                domain == suffix || domain.ends_with(&format!(".{}", suffix))
            } else {
                domain == pattern
            }
        })
    }
}
