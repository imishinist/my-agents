#[cfg(test)]
mod tests {
    use crate::permission::{PermissionChecker, PermissionDecision};
    use crate::policy::{ApprovalLevel, CommandPolicy, NetworkPolicy};
    use crate::profile::SandboxProfile;

    // --- Profile tests ---

    #[test]
    fn test_profile_render_contains_paths() {
        let mut profile = SandboxProfile::new("/tmp/worktree-1", "/home/user/project", 7710);
        profile.home_dir = "/home/user".to_string();
        let rendered = profile.render();

        assert!(rendered.contains("(subpath \"/tmp/worktree-1\")"));
        assert!(rendered.contains("(subpath \"/home/user/project\")"));
        assert!(rendered.contains("(subpath \"/home/user/.cargo\")"));
        assert!(rendered.contains("localhost:7710"));
        assert!(rendered.contains("(deny default)"));
    }

    #[test]
    fn test_profile_write_to_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.sb");
        let mut profile = SandboxProfile::new("/tmp/wt", "/project", 7710);
        profile.home_dir = "/home/user".to_string();
        profile.write_to(&path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("(version 1)"));
    }

    // --- Command policy tests ---

    fn sample_command_policy() -> CommandPolicy {
        let toml = r#"
[auto_allow]
commands = ["cargo build", "cargo test", "git add", "git commit", "ls", "cat"]

[orchestrator_approve]
commands = ["cargo add *", "git push", "gh pr create"]

[po_approve]
commands = ["git push --force", "rm -rf *", "curl *"]

[deny]
commands = ["sudo *", "chmod 777 *"]
"#;
        CommandPolicy::from_toml(toml).unwrap()
    }

    #[test]
    fn test_command_policy_auto_allow() {
        let policy = sample_command_policy();
        assert_eq!(policy.check("cargo build"), ApprovalLevel::AutoAllow);
        assert_eq!(policy.check("cargo test"), ApprovalLevel::AutoAllow);
        assert_eq!(policy.check("git add ."), ApprovalLevel::AutoAllow);
        assert_eq!(policy.check("ls"), ApprovalLevel::AutoAllow);
    }

    #[test]
    fn test_command_policy_orchestrator_approve() {
        let policy = sample_command_policy();
        assert_eq!(policy.check("cargo add serde"), ApprovalLevel::OrchestratorApprove);
        assert_eq!(policy.check("git push"), ApprovalLevel::OrchestratorApprove);
        assert_eq!(policy.check("gh pr create"), ApprovalLevel::OrchestratorApprove);
    }

    #[test]
    fn test_command_policy_deny_takes_precedence() {
        let policy = sample_command_policy();
        assert_eq!(policy.check("sudo rm -rf /"), ApprovalLevel::Deny);
        assert_eq!(policy.check("chmod 777 /tmp"), ApprovalLevel::Deny);
    }

    #[test]
    fn test_command_policy_unknown_requires_po() {
        let policy = sample_command_policy();
        assert_eq!(policy.check("python script.py"), ApprovalLevel::PoApprove);
    }

    // --- Network policy tests ---

    fn sample_network_policy() -> NetworkPolicy {
        let toml = r#"
[auto_allow]
domains = ["crates.io", "github.com", "api.github.com"]

[conditional_allow]
domains = ["api.openai.com", "api.anthropic.com"]

[deny]
domains = ["*"]
"#;
        NetworkPolicy::from_toml(toml).unwrap()
    }

    #[test]
    fn test_network_auto_allow() {
        let policy = sample_network_policy();
        assert_eq!(policy.check_domain("crates.io"), ApprovalLevel::AutoAllow);
        assert_eq!(policy.check_domain("github.com"), ApprovalLevel::AutoAllow);
    }

    #[test]
    fn test_network_conditional_allow() {
        let policy = sample_network_policy();
        assert_eq!(policy.check_domain("api.openai.com"), ApprovalLevel::OrchestratorApprove);
    }

    #[test]
    fn test_network_deny_unknown() {
        let policy = sample_network_policy();
        assert_eq!(policy.check_domain("evil.example.com"), ApprovalLevel::Deny);
    }

    // --- Permission checker tests ---

    #[test]
    fn test_permission_checker_command() {
        let checker = PermissionChecker::new(sample_command_policy(), sample_network_policy());

        assert_eq!(checker.check_command("cargo build"), PermissionDecision::Allow);
        assert!(matches!(
            checker.check_command("cargo add tokio"),
            PermissionDecision::NeedsOrchestratorApproval { .. }
        ));
        assert!(matches!(
            checker.check_command("sudo apt install"),
            PermissionDecision::Denied { .. }
        ));
    }

    #[test]
    fn test_permission_checker_network() {
        let checker = PermissionChecker::new(sample_command_policy(), sample_network_policy());

        assert_eq!(checker.check_network("crates.io"), PermissionDecision::Allow);
        assert!(matches!(
            checker.check_network("api.anthropic.com"),
            PermissionDecision::NeedsOrchestratorApproval { .. }
        ));
        assert!(matches!(
            checker.check_network("unknown.com"),
            PermissionDecision::Denied { .. }
        ));
    }
}
