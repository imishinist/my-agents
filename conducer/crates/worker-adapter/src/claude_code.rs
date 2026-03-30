use std::path::Path;

use async_trait::async_trait;

use crate::{SpawnConfig, WorkerAdapter, WorkerError, WorkerHandle};

/// Claude Code CLI adapter.
/// Spawns `claude -p` with a system prompt and feature specification.
/// Generates `.claude/settings.json` in the worktree for permission control.
pub struct ClaudeCodeAdapter;

impl ClaudeCodeAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Generate `.claude/settings.json` for the Worker's worktree.
    pub(crate) fn generate_settings(worktree_path: &Path) -> Result<(), WorkerError> {
        let claude_dir = worktree_path.join(".claude");
        std::fs::create_dir_all(&claude_dir).map_err(WorkerError::Io)?;

        let settings = serde_json::json!({
            "permissions": {
                "allow": [
                    "Bash(cargo build*)",
                    "Bash(cargo test*)",
                    "Bash(cargo check*)",
                    "Bash(cargo fmt*)",
                    "Bash(cargo clippy*)",
                    "Bash(git add*)",
                    "Bash(git commit*)",
                    "Bash(git diff*)",
                    "Bash(git log*)",
                    "Bash(git status*)",
                    "Read",
                    "Write",
                    "Edit",
                    "Glob",
                    "Grep"
                ],
                "deny": [
                    "Bash(sudo*)",
                    "Bash(rm -rf*)"
                ]
            }
        });

        let settings_path = claude_dir.join("settings.json");
        std::fs::write(&settings_path, serde_json::to_string_pretty(&settings).unwrap())
            .map_err(WorkerError::Io)?;

        Ok(())
    }

    /// Build the user prompt from the feature context.
    pub(crate) fn build_prompt(config: &SpawnConfig<'_>) -> String {
        format!(
            "## Feature: {title}\n\n{spec}\n\n## Constraints\n\
             - Work only in this worktree\n\
             - Create a branch, implement, test, and commit\n\
             - Report progress regularly",
            title = config.feature_title,
            spec = config.feature_spec,
        )
    }
}

#[async_trait]
impl WorkerAdapter for ClaudeCodeAdapter {
    async fn spawn(&self, config: &SpawnConfig<'_>) -> Result<WorkerHandle, WorkerError> {
        // Generate .claude/settings.json
        Self::generate_settings(config.worktree_path)?;

        // Write sandbox profile
        let profile_path = config.worktree_path.join(".conducer-sandbox.sb");
        config
            .sandbox_profile
            .write_to(&profile_path)
            .map_err(WorkerError::Io)?;

        let prompt = Self::build_prompt(config);

        // Spawn claude -p in the worktree
        let child = tokio::process::Command::new("sandbox-exec")
            .arg("-f")
            .arg(&profile_path)
            .arg("claude")
            .arg("-p")
            .arg(&prompt)
            .arg("--system-prompt")
            .arg(config.system_prompt)
            .arg("--output-format")
            .arg("text")
            .current_dir(config.worktree_path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| WorkerError::SpawnFailed(format!("Failed to spawn claude: {}", e)))?;

        let pid = child.id().ok_or_else(|| {
            WorkerError::SpawnFailed("Failed to get PID of spawned process".to_string())
        })?;

        Ok(WorkerHandle {
            pid,
            worker_id: config.worker_id.to_string(),
            worktree_path: config.worktree_path.to_string_lossy().to_string(),
        })
    }

    async fn stop(&self, handle: &WorkerHandle) -> Result<(), WorkerError> {
        // Send SIGTERM to the worker process
        let status = tokio::process::Command::new("kill")
            .arg(handle.pid.to_string())
            .status()
            .await
            .map_err(WorkerError::Io)?;

        if !status.success() {
            tracing::warn!("kill {} returned non-zero, process may already be dead", handle.pid);
        }

        Ok(())
    }

    fn runtime_name(&self) -> &'static str {
        "claude-code"
    }
}
