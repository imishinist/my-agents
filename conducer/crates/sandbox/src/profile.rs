use std::path::Path;

/// macOS sandbox-exec profile generator.
///
/// Generates a `.sb` profile that restricts a Worker process to:
/// - Read/write its own worktree
/// - Read-only access to the project root
/// - Read/write to ~/.cargo, /tmp
/// - Network only to the orchestrator proxy
pub struct SandboxProfile {
    pub worktree_path: String,
    pub project_root: String,
    pub home_dir: String,
    pub proxy_port: u16,
}

impl SandboxProfile {
    pub fn new(
        worktree_path: impl Into<String>,
        project_root: impl Into<String>,
        proxy_port: u16,
    ) -> Self {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/Users/unknown".to_string());
        Self {
            worktree_path: worktree_path.into(),
            project_root: project_root.into(),
            home_dir,
            proxy_port,
        }
    }

    /// Render the sandbox-exec profile string.
    pub fn render(&self) -> String {
        format!(
            r#"(version 1)
(deny default)

;; Worker worktree: full read/write
(allow file-read* file-write*
  (subpath "{worktree}"))

;; Project root: read-only
(allow file-read*
  (subpath "{project}"))

;; Cargo registry & rustup toolchain
(allow file-read* file-write*
  (subpath "{home}/.cargo"))
(allow file-read*
  (subpath "{home}/.rustup"))

;; Temp files
(allow file-read* file-write*
  (subpath "/tmp"))

;; Process execution
(allow process-exec
  (literal "/usr/bin/git")
  (literal "/usr/bin/env")
  (literal "/bin/sh")
  (subpath "{home}/.cargo/bin")
  (subpath "{home}/.rustup/toolchains"))

;; Network: proxy only
(allow network-outbound
  (remote tcp "localhost:{proxy_port}"))

;; Allow basic system operations
(allow process-fork)
(allow sysctl-read)
(allow mach-lookup)
"#,
            worktree = self.worktree_path,
            project = self.project_root,
            home = self.home_dir,
            proxy_port = self.proxy_port,
        )
    }

    /// Write the profile to a temp file and return the path.
    pub fn write_to(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.render())
    }
}
