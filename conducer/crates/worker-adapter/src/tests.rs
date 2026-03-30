#[cfg(test)]
mod tests {
    use crate::claude_code::ClaudeCodeAdapter;
    use crate::worktree::WorktreeManager;
    use crate::WorkerAdapter;

    // --- Worktree tests ---

    #[tokio::test]
    async fn test_worktree_manager_create_and_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path();

        // Initialize a git repo
        tokio::process::Command::new("git")
            .args(["init"])
            .current_dir(project_root)
            .output()
            .await
            .unwrap();

        // Create an initial commit (worktree requires at least one commit)
        tokio::process::Command::new("git")
            .args(["commit", "--allow-empty", "-m", "init"])
            .current_dir(project_root)
            .output()
            .await
            .unwrap();

        let mgr = WorktreeManager::new(project_root);

        // Create worktree
        let wt_path = mgr.create("worker-1", "feat/test-feature").await.unwrap();
        assert!(wt_path.exists());
        assert!(wt_path.join(".git").exists()); // worktree has a .git file

        // Cleanup
        mgr.cleanup("worker-1").await.unwrap();
        assert!(!wt_path.exists());
    }

    #[tokio::test]
    async fn test_worktree_path() {
        let mgr = WorktreeManager::new("/tmp/test-project");
        let path = mgr.worktree_path("worker-42");
        assert_eq!(path.to_str().unwrap(), "/tmp/test-project/.worktrees/worker-42");
    }

    // --- ClaudeCodeAdapter tests ---

    #[test]
    fn test_claude_settings_generation() {
        let dir = tempfile::tempdir().unwrap();
        let worktree_path = dir.path();

        ClaudeCodeAdapter::generate_settings(worktree_path).unwrap();

        let settings_path = worktree_path.join(".claude/settings.json");
        assert!(settings_path.exists());

        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings_path).unwrap()).unwrap();

        let allow = content["permissions"]["allow"].as_array().unwrap();
        assert!(allow.iter().any(|v| v.as_str() == Some("Read")));
        assert!(allow.iter().any(|v| v.as_str() == Some("Write")));
        assert!(allow
            .iter()
            .any(|v| v.as_str() == Some("Bash(cargo test*)")));

        let deny = content["permissions"]["deny"].as_array().unwrap();
        assert!(deny.iter().any(|v| v.as_str() == Some("Bash(sudo*)")));
    }

    #[test]
    fn test_claude_build_prompt() {
        use crate::SpawnConfig;
        use acp_types::ContextEnvelope;
        use conducer_sandbox::profile::SandboxProfile;

        let mut sandbox = SandboxProfile::new("/tmp/wt", "/project", 7710);
        sandbox.home_dir = "/home/user".to_string();
        let envelope = ContextEnvelope {
            architecture_summary: String::new(),
            relevant_interfaces: vec![],
            allowed_paths: vec![],
            read_paths: vec![],
            constraints: vec![],
            branch_prefix: "feat/test".to_string(),
        };

        let config = SpawnConfig {
            worker_id: "w-1",
            worktree_path: std::path::Path::new("/tmp/wt"),
            context_envelope: &envelope,
            sandbox_profile: &sandbox,
            system_prompt: "You are a worker",
            feature_title: "Add auth module",
            feature_spec: "Implement OAuth2 provider abstraction",
        };

        let prompt = ClaudeCodeAdapter::build_prompt(&config);
        assert!(prompt.contains("Add auth module"));
        assert!(prompt.contains("Implement OAuth2 provider abstraction"));
    }

    #[test]
    fn test_claude_runtime_name() {
        let adapter = ClaudeCodeAdapter::new();
        assert_eq!(adapter.runtime_name(), "claude-code");
    }
}
