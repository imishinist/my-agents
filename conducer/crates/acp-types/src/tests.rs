#[cfg(test)]
mod tests {
    use crate::envelope::AcpMessage;
    use crate::ids::*;
    use crate::messages::*;

    #[test]
    fn test_message_roundtrip() {
        let msg = AcpMessage::new(
            "orchestrator",
            "worker-1",
            MessagePayload::FeatureAssign {
                feature_id: FeatureId::from_string("feat-001"),
                epic_id: EpicId::from_string("epic-001"),
                title: "OAuth2 abstraction".to_string(),
                specification: "Implement provider trait".to_string(),
                context_envelope: ContextEnvelope {
                    architecture_summary: "axum web api".to_string(),
                    relevant_interfaces: vec!["src/auth/mod.rs".to_string()],
                    allowed_paths: vec!["src/auth/**".to_string()],
                    read_paths: vec!["src/config.rs".to_string()],
                    constraints: vec!["stateless".to_string()],
                    branch_prefix: "feat/oauth2".to_string(),
                },
                priority: Priority::High,
                depends_on: vec![],
            },
        );

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: AcpMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.acp_version, "1.0");
        assert_eq!(parsed.source, "orchestrator");
        assert_eq!(parsed.destination, "worker-1");
        assert_eq!(parsed.message_type, "feature.assign");
    }

    #[test]
    fn test_heartbeat_roundtrip() {
        let msg = AcpMessage::new(
            "orchestrator",
            "worker-1",
            MessagePayload::HeartbeatRequest {},
        );

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: AcpMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.message_type, "heartbeat.request");
    }

    #[test]
    fn test_escalation_roundtrip() {
        let msg = AcpMessage::new(
            "pm",
            "po",
            MessagePayload::EscalationRequest {
                escalation_id: EscalationId::from_string("esc-001"),
                feature_id: FeatureId::from_string("feat-001"),
                escalation_type: EscalationType::ArchitectureDecision,
                title: "Token storage".to_string(),
                context: "Need to decide storage backend".to_string(),
                question: "Redis or DB?".to_string(),
                options: vec![
                    EscalationOption {
                        value: "redis".to_string(),
                        pros: Some("fast".to_string()),
                        cons: Some("extra infra".to_string()),
                    },
                ],
                pm_recommendation: Some("redis".to_string()),
                pm_reasoning: Some("lower latency".to_string()),
                urgency: Urgency::Medium,
                blocking_features: vec![FeatureId::from_string("feat-001")],
            },
        );

        let json = serde_json::to_string_pretty(&msg).unwrap();
        let parsed: AcpMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.message_type, "escalation.request");
    }

    #[test]
    fn test_id_generation() {
        let id1 = EpicId::new();
        let id2 = EpicId::new();
        assert_ne!(id1, id2);
        assert!(id1.as_str().starts_with("epic-"));
    }

    #[test]
    fn test_all_message_types() {
        // Verify message_type() returns correct strings
        let payloads = vec![
            (MessagePayload::HeartbeatRequest {}, "heartbeat.request"),
            (
                MessagePayload::HeartbeatResponse {
                    feature_id: FeatureId::from_string("f"),
                    status: FeatureStatus::InProgress,
                    last_action: "coding".to_string(),
                    health: WorkerHealth::Ok,
                },
                "heartbeat.response",
            ),
            (
                MessagePayload::PermissionRequest {
                    worker_id: WorkerId::from_string("w"),
                    feature_id: FeatureId::from_string("f"),
                    action: "cargo add serde".to_string(),
                    category: PermissionCategory::DependencyAdd,
                    reason: "need it".to_string(),
                },
                "permission.request",
            ),
        ];

        for (payload, expected_type) in payloads {
            assert_eq!(payload.message_type(), expected_type);
        }
    }
}
