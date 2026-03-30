#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tokio::sync::broadcast;
    use tower::ServiceExt;
    use utoipa::OpenApi;

    use crate::llm::MockLlmClient;
    use crate::pm_agent::PmAgent;
    use crate::router::{ApiDoc, SseEvent, create_router};
    use crate::server::AppState;

    async fn setup_state() -> Arc<AppState> {
        setup_state_with_response(mock_decompose_response()).await
    }

    async fn setup_state_with_response(llm_response: &str) -> Arc<AppState> {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        std::mem::forget(dir);
        let pool = conducer_state::db::init_pool(&db_path).await.unwrap();
        let (event_tx, _) = broadcast::channel::<SseEvent>(256);
        let llm = Box::new(MockLlmClient::new(llm_response));
        let pm_agent = PmAgent::new(llm, pool.clone());
        Arc::new(AppState {
            pool,
            event_tx,
            pm_agent,
        })
    }

    fn mock_decompose_response() -> &'static str {
        r#"```json
{
    "features": [
        {
            "title": "Project overview section",
            "specification": "Add project name, description, and goals to README",
            "priority": "high",
            "depends_on_titles": [],
            "allowed_paths": ["README.md"]
        },
        {
            "title": "Setup instructions",
            "specification": "Add installation and configuration steps",
            "priority": "medium",
            "depends_on_titles": ["Project overview section"],
            "allowed_paths": ["README.md"]
        }
    ]
}
```"#
    }

    // --- OpenAPI spec tests ---

    #[test]
    fn test_openapi_spec_generates() {
        let spec = ApiDoc::openapi();
        let json = serde_json::to_string_pretty(&spec).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["openapi"], "3.1.0");
        assert_eq!(parsed["info"]["title"], "conducer API");
        assert_eq!(parsed["info"]["version"], "0.1.0");

        let paths = parsed["paths"].as_object().unwrap();
        assert!(paths.contains_key("/acp"), "Missing /acp endpoint");
        assert!(paths.contains_key("/api/epics"), "Missing /api/epics endpoint");
        assert!(paths.contains_key("/api/features"), "Missing /api/features endpoint");
        assert!(paths.contains_key("/api/workers"), "Missing /api/workers endpoint");
        assert!(paths.contains_key("/api/actions"), "Missing /api/actions endpoint");
        assert!(paths.contains_key("/api/messages"), "Missing /api/messages endpoint");

        let schemas = parsed["components"]["schemas"].as_object().unwrap();
        assert!(schemas.contains_key("Epic"), "Missing Epic schema");
        assert!(schemas.contains_key("Feature"), "Missing Feature schema");
        assert!(schemas.contains_key("Worker"), "Missing Worker schema");
        assert!(schemas.contains_key("ActionItem"), "Missing ActionItem schema");
    }

    #[test]
    fn test_openapi_spec_snapshot() {
        let spec = ApiDoc::openapi();
        let json = serde_json::to_string_pretty(&spec).unwrap();
        assert!(json.len() > 1000, "OpenAPI spec seems too small: {} bytes", json.len());
    }

    // --- PM Agent integration tests ---

    #[tokio::test]
    async fn test_pm_agent_decompose_epic() {
        let state = setup_state().await;

        // Create an epic directly in DB
        conducer_state::queries::create_epic(
            &state.pool,
            "epic-test-1",
            "README更新",
            "プロジェクトの概要、セットアップ手順、使い方を追加",
        )
        .await
        .unwrap();

        // Run PM Agent decomposition
        let feature_ids = state.pm_agent.decompose_epic("epic-test-1").await.unwrap();
        assert_eq!(feature_ids.len(), 2);

        // Verify features were saved to DB
        let features = conducer_state::queries::list_features_by_epic(&state.pool, "epic-test-1")
            .await
            .unwrap();
        assert_eq!(features.len(), 2);
        // ORDER BY priority DESC uses string comparison: "medium" > "high"
        assert_eq!(features[0].title, "Setup instructions");
        assert_eq!(features[0].priority, "medium");
        assert_eq!(features[1].title, "Project overview section");
        assert_eq!(features[1].priority, "high");

        // Verify dependency resolution: "Setup instructions" depends on "Project overview section"
        let deps: Vec<String> = serde_json::from_str(&features[0].depends_on).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], features[1].id);

        // Verify epic status was updated to active
        let epic = conducer_state::queries::get_epic(&state.pool, "epic-test-1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(epic.status, "active");
    }

    #[tokio::test]
    async fn test_pm_agent_decompose_nonexistent_epic() {
        let state = setup_state().await;
        let result = state.pm_agent.decompose_epic("epic-nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pm_agent_assignable_features_respects_deps() {
        let state = setup_state().await;

        conducer_state::queries::create_epic(&state.pool, "epic-a", "Test", "desc")
            .await
            .unwrap();

        // Decompose to create features with dependencies
        state.pm_agent.decompose_epic("epic-a").await.unwrap();

        // Only the first feature (no deps) should be assignable
        let assignable = state.pm_agent.get_assignable_features().await.unwrap();
        assert_eq!(assignable.len(), 1);
        assert_eq!(assignable[0].title, "Project overview section");

        // Merge the first feature
        conducer_state::queries::update_feature_status(&state.pool, &assignable[0].id, "merged")
            .await
            .unwrap();

        // Now the second feature should be assignable
        let assignable = state.pm_agent.get_assignable_features().await.unwrap();
        assert_eq!(assignable.len(), 1);
        assert_eq!(assignable[0].title, "Setup instructions");
    }

    #[tokio::test]
    async fn test_pm_agent_build_context_envelope_with_memory() {
        let state = setup_state().await;

        // Seed project memory
        conducer_state::memory::set(&state.pool, "architecture", "architecture", "Rust monorepo")
            .await
            .unwrap();
        conducer_state::memory::set(&state.pool, "api-spec", "interface", "REST API v1")
            .await
            .unwrap();
        conducer_state::memory::set(&state.pool, "no-unsafe", "constraint", "No unsafe code")
            .await
            .unwrap();

        // Create epic and decompose
        conducer_state::queries::create_epic(&state.pool, "epic-mem", "Test", "desc")
            .await
            .unwrap();
        state.pm_agent.decompose_epic("epic-mem").await.unwrap();

        // Assign features to check context envelope
        conducer_state::queries::create_worker(&state.pool, "w-1", "claude-code")
            .await
            .unwrap();
        let assignments = state.pm_agent.assign_features().await.unwrap();
        assert_eq!(assignments.len(), 1);

        let envelope = &assignments[0].context_envelope;
        assert_eq!(envelope.architecture_summary, "Rust monorepo");
        assert_eq!(envelope.relevant_interfaces, vec!["REST API v1"]);
        assert_eq!(envelope.constraints, vec!["No unsafe code"]);
    }

    // --- HTTP API integration tests ---

    #[tokio::test]
    async fn test_http_create_and_list_epics() {
        let state = setup_state().await;
        let app = create_router(state);

        // Create epic
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/epics")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"title":"Test Epic","description":"A test epic"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let epic: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(epic["title"], "Test Epic");
        assert_eq!(epic["status"], "draft");
        let epic_id = epic["id"].as_str().unwrap().to_string();

        // List epics
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/epics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let epics: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(epics.len(), 1);
        assert_eq!(epics[0]["id"], epic_id);
    }

    #[tokio::test]
    async fn test_http_get_epic_not_found() {
        let state = setup_state().await;
        let app = create_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/epics/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_http_create_epic_triggers_decompose() {
        let state = setup_state().await;
        let pool = state.pool.clone();
        let mut rx = state.event_tx.subscribe();
        let app = create_router(state);

        // Create epic via HTTP
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/epics")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"title":"README更新","description":"概要を追加"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let epic: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let epic_id = epic["id"].as_str().unwrap().to_string();

        // Wait for SSE events: epic.created then epic.decomposed
        let mut got_created = false;
        let mut got_decomposed = false;
        let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(5);
        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(tokio::time::Duration::from_millis(500), rx.recv()).await {
                Ok(Ok(event)) => {
                    if event.event_type == "epic.created" {
                        got_created = true;
                    }
                    if event.event_type == "epic.decomposed" {
                        got_decomposed = true;
                        break;
                    }
                }
                _ => continue,
            }
        }

        assert!(got_created, "Should have received epic.created SSE event");
        assert!(got_decomposed, "Should have received epic.decomposed SSE event");

        // Verify features were created in DB
        let features = conducer_state::queries::list_features_by_epic(&pool, &epic_id)
            .await
            .unwrap();
        assert_eq!(features.len(), 2);
    }

    #[tokio::test]
    async fn test_http_list_features_and_workers() {
        let state = setup_state().await;
        let app = create_router(state);

        // Features should be empty initially
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/features")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let features: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert!(features.is_empty());

        // Workers should be empty initially
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/workers")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let workers: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert!(workers.is_empty());
    }

    #[tokio::test]
    async fn test_http_acp_message_ingestion() {
        let state = setup_state().await;
        let app = create_router(state);

        let msg = serde_json::json!({
            "acp_version": "1.0",
            "message_id": "msg-test-001",
            "timestamp": "2026-03-30T00:00:00Z",
            "source": "worker-1",
            "destination": "orchestrator",
            "type": "heartbeat.response",
            "payload": {
                "type": "heartbeat.response",
                "feature_id": "feat-001",
                "status": "in_progress",
                "last_action": "editing file",
                "health": "ok"
            }
        });

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/acp")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&msg).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        // Verify message was stored
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/messages")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let messages: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["source"], "worker-1");
        assert_eq!(messages[0]["type"], "heartbeat.response");
    }

    #[tokio::test]
    async fn test_http_swagger_ui_accessible() {
        let state = setup_state().await;
        let app = create_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/swagger-ui/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    // --- ProcessManager tests ---

    #[tokio::test]
    async fn test_process_manager_track_and_stop() {
        use crate::process_mgr::ProcessManager;
        use conducer_worker_adapter::{WorkerAdapter, WorkerError, WorkerHandle, SpawnConfig};

        // A mock adapter that doesn't actually spawn anything
        struct MockAdapter;

        #[async_trait::async_trait]
        impl WorkerAdapter for MockAdapter {
            async fn spawn(&self, config: &SpawnConfig<'_>) -> Result<WorkerHandle, WorkerError> {
                Ok(WorkerHandle {
                    pid: 99999,
                    worker_id: config.worker_id.to_string(),
                    worktree_path: config.worktree_path.to_string_lossy().to_string(),
                })
            }
            async fn stop(&self, _handle: &WorkerHandle) -> Result<(), WorkerError> {
                Ok(())
            }
            fn runtime_name(&self) -> &'static str {
                "mock"
            }
        }

        // Use a temp dir as project root with a git repo
        let dir = tempfile::tempdir().unwrap();
        let project_root = dir.path();
        tokio::process::Command::new("git")
            .args(["init"])
            .current_dir(project_root)
            .output()
            .await
            .unwrap();
        tokio::process::Command::new("git")
            .args(["commit", "--allow-empty", "-m", "init"])
            .current_dir(project_root)
            .output()
            .await
            .unwrap();

        let mgr = ProcessManager::new(
            Box::new(MockAdapter),
            project_root,
            7710,
            "test prompt".to_string(),
        );

        assert!(!mgr.is_running("w-1").await);

        let envelope = acp_types::ContextEnvelope {
            architecture_summary: String::new(),
            relevant_interfaces: vec![],
            allowed_paths: vec![],
            read_paths: vec![],
            constraints: vec![],
            branch_prefix: "feat/test".to_string(),
        };

        let handle = mgr
            .spawn_worker("w-1", "feat/test-pm", "Test feature", "spec", &envelope)
            .await
            .unwrap();

        assert_eq!(handle.worker_id, "w-1");
        assert!(mgr.is_running("w-1").await);
        assert_eq!(mgr.active_workers().await.len(), 1);

        mgr.stop_worker("w-1").await.unwrap();
        assert!(!mgr.is_running("w-1").await);
        assert!(mgr.active_workers().await.is_empty());
    }

    // --- HeartbeatMonitor tests ---

    #[tokio::test]
    async fn test_heartbeat_detects_stalled_worker() {
        use crate::heartbeat::HeartbeatMonitor;
        use std::time::Duration;

        let state = setup_state().await;

        // Create a worker and set it to busy with an old heartbeat
        conducer_state::queries::create_worker(&state.pool, "w-stall", "claude-code")
            .await
            .unwrap();
        conducer_state::queries::update_worker_status(&state.pool, "w-stall", "busy")
            .await
            .unwrap();
        // Set heartbeat to 1 hour ago
        sqlx::query(
            "UPDATE workers SET last_heartbeat = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-60 minutes') WHERE id = ?",
        )
        .bind("w-stall")
        .execute(&state.pool)
        .await
        .unwrap();

        let mut rx = state.event_tx.subscribe();

        let monitor = HeartbeatMonitor::new(
            state.pool.clone(),
            state.event_tx.clone(),
            Duration::from_millis(50),
            5, // 5 minute timeout
        );

        // Run monitor in background
        let monitor_handle = tokio::spawn(monitor.run());

        // Wait for stall detection
        let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("Should receive stall event")
            .unwrap();

        assert_eq!(event.event_type, "worker.stalled");
        assert!(event.data.contains("w-stall"));

        // Verify worker status was updated
        let workers = conducer_state::queries::list_workers(&state.pool).await.unwrap();
        assert_eq!(workers[0].status, "stalled");

        monitor_handle.abort();
    }

    // --- Review pipeline tests ---

    fn mock_review_approved_response() -> &'static str {
        r#"```json
{
    "verdict": "approved",
    "summary": "Implementation looks correct and complete",
    "comments": []
}
```"#
    }

    fn mock_review_changes_response() -> &'static str {
        r#"```json
{
    "verdict": "changes_requested",
    "summary": "Missing test coverage",
    "comments": [
        {
            "file": "src/auth.rs",
            "line": 42,
            "severity": "error",
            "message": "No unit tests for the auth module",
            "suggestion": "Add tests for login and logout flows"
        }
    ]
}
```"#
    }

    fn mock_clarification_answer_response() -> &'static str {
        r#"```json
{"answer": "Use Redis for token storage based on the stateless constraint", "escalate": false}
```"#
    }

    fn mock_clarification_escalate_response() -> &'static str {
        r#"```json
{"answer": null, "escalate": true, "escalation_reason": "This is an architectural decision that needs PO input"}
```"#
    }

    #[tokio::test]
    async fn test_review_pr_approved() {
        let state = setup_state_with_response(mock_review_approved_response()).await;

        // Create epic + feature + set feature to pr_submitted
        conducer_state::queries::create_epic(&state.pool, "epic-r1", "Test", "desc")
            .await
            .unwrap();
        conducer_state::queries::create_feature(
            &state.pool, "feat-r1", "epic-r1", "Auth module", "Implement auth", "[]", "high",
        )
        .await
        .unwrap();
        conducer_state::queries::update_feature_status(&state.pool, "feat-r1", "pr_submitted")
            .await
            .unwrap();

        let result = state.pm_agent.review_pr("feat-r1", "diff content here").await.unwrap();
        assert_eq!(result.verdict, "approved");

        // Feature should be merged
        let feature = conducer_state::queries::get_feature(&state.pool, "feat-r1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(feature.status, "merged");

        // Review should be saved
        let reviews = conducer_state::queries::list_reviews_by_feature(&state.pool, "feat-r1")
            .await
            .unwrap();
        assert_eq!(reviews.len(), 1);
        assert_eq!(reviews[0].verdict, "approved");
    }

    #[tokio::test]
    async fn test_review_pr_changes_requested() {
        let state = setup_state_with_response(mock_review_changes_response()).await;

        conducer_state::queries::create_epic(&state.pool, "epic-r2", "Test", "desc")
            .await
            .unwrap();
        conducer_state::queries::create_feature(
            &state.pool, "feat-r2", "epic-r2", "Auth module", "Implement auth", "[]", "high",
        )
        .await
        .unwrap();

        let result = state.pm_agent.review_pr("feat-r2", "diff").await.unwrap();
        assert_eq!(result.verdict, "changes_requested");
        assert_eq!(result.comments.len(), 1);
        assert_eq!(result.comments[0].severity, "error");

        let feature = conducer_state::queries::get_feature(&state.pool, "feat-r2")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(feature.status, "changes_requested");
    }

    #[tokio::test]
    async fn test_answer_clarification_direct() {
        let state = setup_state_with_response(mock_clarification_answer_response()).await;

        conducer_state::queries::create_epic(&state.pool, "epic-c1", "Test", "desc")
            .await
            .unwrap();
        conducer_state::queries::create_feature(
            &state.pool, "feat-c1", "epic-c1", "Token storage", "Implement tokens", "[]", "high",
        )
        .await
        .unwrap();

        let answer = state
            .pm_agent
            .answer_clarification("feat-c1", "Redis or DB?", "Need to store tokens")
            .await
            .unwrap();

        assert!(!answer.escalate);
        assert!(answer.answer.unwrap().contains("Redis"));
    }

    #[tokio::test]
    async fn test_answer_clarification_escalate() {
        let state = setup_state_with_response(mock_clarification_escalate_response()).await;

        conducer_state::queries::create_epic(&state.pool, "epic-c2", "Test", "desc")
            .await
            .unwrap();
        conducer_state::queries::create_feature(
            &state.pool, "feat-c2", "epic-c2", "Architecture", "Design system", "[]", "high",
        )
        .await
        .unwrap();

        let answer = state
            .pm_agent
            .answer_clarification("feat-c2", "Monolith or microservices?", "")
            .await
            .unwrap();

        assert!(answer.escalate);
        assert!(answer.answer.is_none());
        assert!(answer.escalation_reason.is_some());
    }

    #[tokio::test]
    async fn test_check_epic_completion() {
        let state = setup_state().await;

        conducer_state::queries::create_epic(&state.pool, "epic-comp", "Test", "desc")
            .await
            .unwrap();
        state.pm_agent.decompose_epic("epic-comp").await.unwrap();

        // Not complete yet
        assert!(!state.pm_agent.check_epic_completion("epic-comp").await.unwrap());

        // Merge all features
        let features = conducer_state::queries::list_features_by_epic(&state.pool, "epic-comp")
            .await
            .unwrap();
        for f in &features {
            conducer_state::queries::update_feature_status(&state.pool, &f.id, "merged")
                .await
                .unwrap();
        }

        // Now complete
        assert!(state.pm_agent.check_epic_completion("epic-comp").await.unwrap());

        let epic = conducer_state::queries::get_epic(&state.pool, "epic-comp")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(epic.status, "completed");
    }

    #[tokio::test]
    async fn test_event_loop_pr_submitted_triggers_review() {
        let state = setup_state_with_response(mock_review_approved_response()).await;
        let mut rx = state.event_tx.subscribe();

        // Setup: epic + feature
        conducer_state::queries::create_epic(&state.pool, "epic-ev1", "Test", "desc")
            .await
            .unwrap();
        conducer_state::queries::create_feature(
            &state.pool, "feat-ev1", "epic-ev1", "Module", "Build it", "[]", "high",
        )
        .await
        .unwrap();

        // Simulate pr.submitted event
        let payload = serde_json::json!({
            "type": "pr.submitted",
            "feature_id": "feat-ev1",
            "pr_number": 1,
            "branch_name": "feat/module",
            "summary": "Added module implementation",
            "files_changed": ["src/module.rs"],
            "test_results": {"passed": 5, "failed": 0, "skipped": 0},
            "lint_clean": true
        });

        crate::event_loop::handle_acp_event(&state, "pr.submitted", &payload).await;

        // Should receive review.completed event
        let event = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            rx.recv(),
        )
        .await
        .expect("Should receive event")
        .unwrap();

        assert_eq!(event.event_type, "review.completed");
        assert!(event.data.contains("approved"));

        // Feature should be merged
        let feature = conducer_state::queries::get_feature(&state.pool, "feat-ev1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(feature.status, "merged");
    }
}
