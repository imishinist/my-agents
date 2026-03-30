use async_trait::async_trait;

/// Abstract LLM client trait. Implementations can use Anthropic API, Claude Code CLI,
/// or ACP-based coding agents.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Send a prompt and get a text response
    async fn complete(&self, system_prompt: &str, user_prompt: &str) -> Result<String, LlmError>;
}

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("API error: {0}")]
    Api(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// --- Mock (testing) ---

/// Mock LLM client for testing
pub struct MockLlmClient {
    response: String,
}

impl MockLlmClient {
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
        }
    }
}

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn complete(&self, _system_prompt: &str, _user_prompt: &str) -> Result<String, LlmError> {
        Ok(self.response.clone())
    }
}

// --- Claude Code CLI ---

/// Claude Code CLI adapter - runs `claude -p` with system prompt.
/// Uses the user's Claude Code subscription (no API cost).
pub struct ClaudeCodeClient {
    pub model: Option<String>,
}

impl ClaudeCodeClient {
    pub fn new() -> Self {
        Self { model: None }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

#[async_trait]
impl LlmClient for ClaudeCodeClient {
    async fn complete(&self, system_prompt: &str, user_prompt: &str) -> Result<String, LlmError> {
        let mut cmd = tokio::process::Command::new("claude");
        cmd.arg("-p").arg(user_prompt);
        cmd.arg("--system-prompt").arg(system_prompt);
        cmd.arg("--output-format").arg("text");

        if let Some(model) = &self.model {
            cmd.arg("--model").arg(model);
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let msg = if stderr.is_empty() { stdout } else { stderr };
            return Err(LlmError::Api(format!("claude CLI failed: {}", msg.trim())));
        }

        let response = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(response)
    }
}

// --- Kiro CLI ---

/// Kiro CLI adapter - runs `kiro-cli chat --no-interactive` with system prompt.
pub struct KiroCliClient {
    pub model: Option<String>,
}

impl KiroCliClient {
    pub fn new() -> Self {
        Self { model: None }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

#[async_trait]
impl LlmClient for KiroCliClient {
    async fn complete(&self, system_prompt: &str, user_prompt: &str) -> Result<String, LlmError> {
        let prompt = format!("{}\n\n{}", system_prompt, user_prompt);
        let mut cmd = tokio::process::Command::new("kiro-cli");
        cmd.arg("chat")
            .arg("--no-interactive")
            .arg("--trust-all-tools")
            .arg(&prompt);

        if let Some(model) = &self.model {
            cmd.arg("--model").arg(model);
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let msg = if stderr.is_empty() { stdout } else { stderr };
            return Err(LlmError::Api(format!("kiro-cli failed: {}", msg.trim())));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

// --- ACP Agent (HTTP) ---

/// ACP-based LLM client. Sends a prompt to a coding agent via ACP HTTP,
/// and receives the response. This allows using any ACP-compatible agent
/// (Claude Code, Kiro, etc.) as the LLM backend without direct API costs.
pub struct AcpAgentClient {
    /// ACP endpoint URL of the agent to delegate to (e.g. "http://localhost:7800/acp")
    pub endpoint: String,
    /// Agent ID to address in ACP messages
    pub agent_id: String,
    pub http_client: reqwest::Client,
}

impl AcpAgentClient {
    pub fn new(endpoint: impl Into<String>, agent_id: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            agent_id: agent_id.into(),
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmClient for AcpAgentClient {
    async fn complete(&self, system_prompt: &str, user_prompt: &str) -> Result<String, LlmError> {
        // Build an ACP message with a clarification.request-like payload
        // that the target agent can process and respond to
        let message = serde_json::json!({
            "acp_version": "1.0",
            "message_id": format!("msg-{}", uuid::Uuid::new_v4().as_simple()),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source": "orchestrator",
            "destination": self.agent_id,
            "type": "pm.prompt",
            "payload": {
                "system_prompt": system_prompt,
                "user_prompt": user_prompt
            }
        });

        let resp = self
            .http_client
            .post(&self.endpoint)
            .json(&message)
            .send()
            .await
            .map_err(|e| LlmError::Api(format!("ACP request failed: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(LlmError::Api(format!(
                "ACP agent returned {}: {}",
                status, body
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| LlmError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        body["payload"]["response"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| {
                LlmError::InvalidResponse(format!(
                    "Missing payload.response in ACP response: {}",
                    body
                ))
            })
    }
}

/// LLM backend configuration
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LlmBackendConfig {
    /// Use Claude Code CLI (subscription-based, no API cost)
    ClaudeCode {
        model: Option<String>,
    },
    /// Use Kiro CLI
    KiroCli {
        model: Option<String>,
    },
    /// Use an ACP-compatible agent as LLM backend
    AcpAgent {
        endpoint: String,
        agent_id: String,
    },
}

impl LlmBackendConfig {
    pub fn create_client(&self) -> Box<dyn LlmClient> {
        match self {
            LlmBackendConfig::ClaudeCode { model } => {
                let mut client = ClaudeCodeClient::new();
                if let Some(m) = model {
                    client = client.with_model(m);
                }
                Box::new(client)
            }
            LlmBackendConfig::KiroCli { model } => {
                let mut client = KiroCliClient::new();
                if let Some(m) = model {
                    client = client.with_model(m);
                }
                Box::new(client)
            }
            LlmBackendConfig::AcpAgent { endpoint, agent_id } => {
                Box::new(AcpAgentClient::new(endpoint, agent_id))
            }
        }
    }
}
