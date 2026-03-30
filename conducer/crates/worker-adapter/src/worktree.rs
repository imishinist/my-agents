use std::path::{Path, PathBuf};

use crate::WorkerError;

/// Manages git worktrees for Worker isolation.
pub struct WorktreeManager {
    /// Project root (main worktree).
    project_root: PathBuf,
    /// Directory where worktrees are created (e.g. .worktrees/).
    worktrees_dir: PathBuf,
}

impl WorktreeManager {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        let project_root = project_root.into();
        let worktrees_dir = project_root.join(".worktrees");
        Self {
            project_root,
            worktrees_dir,
        }
    }

    /// Create a new worktree for a worker on the given branch.
    /// Returns the path to the created worktree.
    pub async fn create(
        &self,
        worker_id: &str,
        branch_name: &str,
    ) -> Result<PathBuf, WorkerError> {
        let worktree_path = self.worktrees_dir.join(worker_id);

        // Ensure .worktrees/ directory exists
        tokio::fs::create_dir_all(&self.worktrees_dir)
            .await
            .map_err(WorkerError::Io)?;

        // Create branch from current HEAD if it doesn't exist
        let _ = run_git(
            &self.project_root,
            &["branch", branch_name],
        )
        .await;

        // Create worktree
        run_git(
            &self.project_root,
            &[
                "worktree",
                "add",
                worktree_path.to_str().unwrap_or(""),
                branch_name,
            ],
        )
        .await?;

        Ok(worktree_path)
    }

    /// Remove a worktree and clean up the branch.
    pub async fn cleanup(&self, worker_id: &str) -> Result<(), WorkerError> {
        let worktree_path = self.worktrees_dir.join(worker_id);

        // Remove worktree
        run_git(
            &self.project_root,
            &[
                "worktree",
                "remove",
                worktree_path.to_str().unwrap_or(""),
                "--force",
            ],
        )
        .await?;

        Ok(())
    }

    pub fn worktree_path(&self, worker_id: &str) -> PathBuf {
        self.worktrees_dir.join(worker_id)
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }
}

/// Run a git command in the given directory.
async fn run_git(cwd: &Path, args: &[&str]) -> Result<String, WorkerError> {
    let output = tokio::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .await
        .map_err(WorkerError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WorkerError::Git(format!(
            "git {} failed: {}",
            args.join(" "),
            stderr
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
