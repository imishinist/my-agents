use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::ids::MessageId;
use crate::messages::MessagePayload;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AcpMessage {
    pub acp_version: String,
    pub message_id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<MessageId>,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub destination: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub payload: MessagePayload,
}

impl AcpMessage {
    pub fn new(
        source: impl Into<String>,
        destination: impl Into<String>,
        payload: MessagePayload,
    ) -> Self {
        let message_type = payload.message_type().to_string();
        Self {
            acp_version: "1.0".to_string(),
            message_id: MessageId::new(),
            correlation_id: None,
            timestamp: Utc::now(),
            source: source.into(),
            destination: destination.into(),
            message_type,
            payload,
        }
    }

    pub fn with_correlation(mut self, correlation_id: MessageId) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
}
