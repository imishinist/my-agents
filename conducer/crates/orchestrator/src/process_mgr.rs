use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;

use acp_types::ContextEnvelope;
use conducer_sandbox::profile::SandboxProfile;
use conducer_worker_adapter::worktree::WorktreeManager;
use conducer_worker_adapter::{SpawnConfig, WorkerAdapter, WorkerError, WorkerHandle};

/// Manages Worker process lifecycle: spawn, track, stop.
pub struct ProcessManager {
    adapter: Box<dyn WorkerAdapter>,
    worktree_mgr: WorktreeManager,
    project_root: PathBuf,
    proxy_port: u16,
    workers: Arc<Mutex<HashMap<String, WorkerHandle>>>,
    system_prompt: String,
}

impl ProcessManager {
    pub fn new(
        adapter: Box<dyn WorkerAdapter>,
        project_root: impl Into<PathBuf>,
        proxy_port: u16,
        system_prompt: String,
    ) -> Self {
        let project_root = project_root.into();
        let worktree_mgr = WorktreeManager::new(&project_root);
        Self {
            adapter,
            worktree_mgr,
            project_root,
            proxy_port,
            workers: Arc::new(Mutex::new(HashMap::new())),
            system_prompt,
        }
    }

    /// Spawn a Worker for a Feature assignment.
    pub async fn spawn_worker(
        &self,
        worker_id: &str,
        branch_name: &str,
        feature_title: &str,
        feature_spec: &str,
        context_envelope: &ContextEnvelope,
    ) -> Result<WorkerHandle, WorkerError> {
        // Create worktree
        let worktree_path = self
            .worktree_mgr
            .create(worker_id, branch_name)
            .await?;

        // Generate sandbox profile
        let sandbox_profile = SandboxProfile::new(
            worktree_path.to_string_lossy().as_ref(),
            self.project_root.to_string_lossy().as_ref(),
            self.proxy_port,
        );

        let config = SpawnConfig {
            worker_id,
            worktree_path: &worktree_path,
            context_envelope,
            sandbox_profile: &sandbox_profile,
            system_prompt: &self.system_prompt,
            feature_title,
            feature_spec,
        };

        let handle = self.adapter.spawn(&config).await?;

        self.workers
            .lock()
            .await
            .insert(worker_id.to_string(), handle.clone());

        tracing::info!(
            worker_id = worker_id,
            pid = handle.pid,
            "Worker spawned"
        );

        Ok(handle)
    }

    /// Stop a Worker and clean up its worktree.
    pub async fn stop_worker(&self, worker_id: &str) -> Result<(), WorkerError> {
        let handle = {
            self.workers.lock().await.remove(worker_id)
        };

        if let Some(handle) = handle {
            self.adapter.stop(&handle).await?;
            tracing::info!(worker_id = worker_id, "Worker stopped");
        }

        // Clean up worktree (best-effort)
        if let Err(e) = self.worktree_mgr.cleanup(worker_id).await {
            tracing::warn!(worker_id = worker_id, error = %e, "Worktree cleanup failed");
        }

        Ok(())
    }

    /// Get a snapshot of all active workers.
    pub async fn active_workers(&self) -> HashMap<String, WorkerHandle> {
        self.workers.lock().await.clone()
    }

    /// Check if a worker is tracked.
    pub async fn is_running(&self, worker_id: &str) -> bool {
        self.workers.lock().await.contains_key(worker_id)
    }
}
