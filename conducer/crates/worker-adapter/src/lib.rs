pub mod claude_code;
pub mod worktree;
mod tests;

use std::path::Path;

use async_trait::async_trait;

use acp_types::ContextEnvelope;
use conducer_sandbox::profile::SandboxProfile;

/// Handle to a running Worker process.
#[derive(Debug, Clone)]
pub struct WorkerHandle {
    pub pid: u32,
    pub worker_id: String,
    pub worktree_path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Worker spawn failed: {0}")]
    SpawnFailed(String),
    #[error("Worker not running")]
    NotRunning,
    #[error("Git error: {0}")]
    Git(String),
}

/// Configuration for spawning a Worker.
pub struct SpawnConfig<'a> {
    pub worker_id: &'a str,
    pub worktree_path: &'a Path,
    pub context_envelope: &'a ContextEnvelope,
    pub sandbox_profile: &'a SandboxProfile,
    pub system_prompt: &'a str,
    pub feature_title: &'a str,
    pub feature_spec: &'a str,
}

/// Trait abstracting over different Worker runtimes (Claude Code, Kiro, etc.).
#[async_trait]
pub trait WorkerAdapter: Send + Sync {
    /// Spawn a Worker process in the given worktree.
    async fn spawn(&self, config: &SpawnConfig<'_>) -> Result<WorkerHandle, WorkerError>;

    /// Stop a running Worker process.
    async fn stop(&self, handle: &WorkerHandle) -> Result<(), WorkerError>;

    /// Runtime name for logging/DB.
    fn runtime_name(&self) -> &'static str;
}
