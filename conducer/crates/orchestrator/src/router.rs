use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::server::AppState;
use conducer_state::{models::*, queries};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "conducer API",
        description = "Autonomous coding agent orchestrator API",
        version = "0.1.0",
    ),
    paths(
        handle_acp_message,
        create_epic,
        list_epics,
        get_epic,
        retry_epic_decompose,
        list_features,
        get_feature,
        list_workers,
        list_actions,
        respond_to_action,
        list_messages,
    ),
    components(schemas(
        Epic,
        Feature,
        Worker,
        Escalation,
        Permission,
        ProgressLogEntry,
        Review,
        Message,
        EpicWithProgress,
        ActionItem,
        CreateEpicRequest,
        ActionResponse,
        acp_types::AcpMessage,
        acp_types::MessagePayload,
    ))
)]
pub struct ApiDoc;

pub fn create_router(state: Arc<AppState>) -> Router {
    let app = Router::new()
        // ACP message ingestion
        .route("/acp", post(handle_acp_message))
        // State API for GUI
        .route("/api/epics", get(list_epics).post(create_epic))
        .route("/api/epics/{id}", get(get_epic))
        .route("/api/epics/{id}/retry", post(retry_epic_decompose))
        .route("/api/features", get(list_features))
        .route("/api/features/{id}", get(get_feature))
        .route("/api/workers", get(list_workers))
        .route("/api/actions", get(list_actions))
        .route("/api/actions/{id}/respond", post(respond_to_action))
        .route("/api/messages", get(list_messages))
        .route("/api/events", get(sse_events))
        .with_state(state);

    // Merge Swagger UI
    let swagger = utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi());

    app.merge(swagger)
}

// --- ACP endpoint ---

/// Ingest an ACP message from an agent
#[utoipa::path(
    post,
    path = "/acp",
    request_body = acp_types::AcpMessage,
    responses(
        (status = 202, description = "Message accepted"),
        (status = 400, description = "Invalid message"),
        (status = 500, description = "Internal error"),
    ),
    tag = "acp"
)]
async fn handle_acp_message(
    State(state): State<Arc<AppState>>,
    Json(message): Json<acp_types::AcpMessage>,
) -> Result<StatusCode, (StatusCode, String)> {
    let payload_json = serde_json::to_string(&message.payload)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    queries::insert_message(
        &state.pool,
        message.message_id.as_str(),
        message.correlation_id.as_ref().map(|id| id.as_str()),
        &message.source,
        &message.destination,
        &message.message_type,
        &payload_json,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Broadcast event for SSE
    let _ = state.event_tx.send(SseEvent {
        event_type: message.message_type.clone(),
        data: payload_json.clone(),
    });

    // Trigger event-driven actions in background
    {
        let state = Arc::clone(&state);
        let msg_type = message.message_type.clone();
        let payload: serde_json::Value =
            serde_json::from_str(&payload_json).unwrap_or_default();
        tokio::spawn(async move {
            crate::event_loop::handle_acp_event(&state, &msg_type, &payload).await;
        });
    }

    Ok(StatusCode::ACCEPTED)
}

// --- Epic endpoints ---

#[derive(Deserialize, ToSchema)]
struct CreateEpicRequest {
    /// Epic title
    title: String,
    /// Epic description in natural language
    description: String,
}

/// Create a new epic
#[utoipa::path(
    post,
    path = "/api/epics",
    request_body = CreateEpicRequest,
    responses(
        (status = 201, description = "Epic created", body = Epic),
        (status = 500, description = "Internal error"),
    ),
    tag = "epics"
)]
async fn create_epic(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateEpicRequest>,
) -> Result<(StatusCode, Json<Epic>), (StatusCode, String)> {
    let id = acp_types::EpicId::new();
    queries::create_epic(&state.pool, id.as_str(), &req.title, &req.description)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let epic = queries::get_epic(&state.pool, id.as_str())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Epic not found after creation".to_string()))?;

    let _ = state.event_tx.send(SseEvent {
        event_type: "epic.created".to_string(),
        data: serde_json::to_string(&epic).unwrap_or_default(),
    });

    spawn_decompose(Arc::clone(&state), id.as_str().to_string());

    Ok((StatusCode::CREATED, Json(epic)))
}

/// Retry decomposition for a failed epic
#[utoipa::path(
    post,
    path = "/api/epics/{id}/retry",
    params(("id" = String, Path, description = "Epic ID")),
    responses(
        (status = 200, description = "Retry started"),
        (status = 404, description = "Epic not found"),
    ),
    tag = "epics"
)]
async fn retry_epic_decompose(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let epic = queries::get_epic(&state.pool, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Epic not found".to_string()))?;

    if epic.status != "error" && epic.status != "draft" {
        return Err((StatusCode::BAD_REQUEST, format!("Epic is in '{}' status, cannot retry", epic.status)));
    }

    // Reset to draft
    queries::update_epic_status(&state.pool, &id, "draft")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    spawn_decompose(Arc::clone(&state), id);

    Ok(StatusCode::OK)
}

/// Spawn PM Agent decomposition in background.
fn spawn_decompose(state: Arc<AppState>, epic_id: String) {
    tokio::spawn(async move {
        tracing::info!("PM Agent: decomposing epic {}", epic_id);
        match state.pm_agent.decompose_epic(&epic_id).await {
            Ok(feature_ids) => {
                tracing::info!("PM Agent: created {} features for epic {}", feature_ids.len(), epic_id);
                let _ = state.event_tx.send(SseEvent {
                    event_type: "epic.decomposed".to_string(),
                    data: serde_json::json!({
                        "epic_id": epic_id,
                        "feature_count": feature_ids.len(),
                    }).to_string(),
                });
            }
            Err(e) => {
                let error_msg = e.to_string();
                tracing::error!("PM Agent: failed to decompose epic {}: {}", epic_id, error_msg);
                let _ = queries::update_epic_error(&state.pool, &epic_id, &error_msg).await;
                let _ = state.event_tx.send(SseEvent {
                    event_type: "epic.decompose_failed".to_string(),
                    data: serde_json::json!({
                        "epic_id": epic_id,
                        "error": error_msg,
                    }).to_string(),
                });
            }
        }
    });
}

/// List all epics
#[utoipa::path(
    get,
    path = "/api/epics",
    responses(
        (status = 200, description = "List of epics", body = Vec<Epic>),
    ),
    tag = "epics"
)]
async fn list_epics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Epic>>, (StatusCode, String)> {
    let epics = queries::list_epics(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(epics))
}

/// Get an epic by ID
#[utoipa::path(
    get,
    path = "/api/epics/{id}",
    params(("id" = String, Path, description = "Epic ID")),
    responses(
        (status = 200, description = "Epic details", body = Epic),
        (status = 404, description = "Epic not found"),
    ),
    tag = "epics"
)]
async fn get_epic(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Epic>, (StatusCode, String)> {
    let epic = queries::get_epic(&state.pool, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Epic not found".to_string()))?;
    Ok(Json(epic))
}

// --- Feature endpoints ---

/// List all features (optionally filter by epic_id query param)
#[utoipa::path(
    get,
    path = "/api/features",
    responses(
        (status = 200, description = "List of features", body = Vec<Feature>),
    ),
    tag = "features"
)]
async fn list_features(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Feature>>, (StatusCode, String)> {
    let features = queries::list_all_features(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(features))
}

/// Get a feature by ID
#[utoipa::path(
    get,
    path = "/api/features/{id}",
    params(("id" = String, Path, description = "Feature ID")),
    responses(
        (status = 200, description = "Feature details", body = Feature),
        (status = 404, description = "Feature not found"),
    ),
    tag = "features"
)]
async fn get_feature(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Feature>, (StatusCode, String)> {
    let feature = queries::get_feature(&state.pool, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Feature not found".to_string()))?;
    Ok(Json(feature))
}

// --- Worker endpoints ---

/// List all workers and their status
#[utoipa::path(
    get,
    path = "/api/workers",
    responses(
        (status = 200, description = "List of workers", body = Vec<Worker>),
    ),
    tag = "workers"
)]
async fn list_workers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Worker>>, (StatusCode, String)> {
    let workers = queries::list_workers(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(workers))
}

// --- Action queue ---

/// List pending actions (escalations + permission requests) for PO
#[utoipa::path(
    get,
    path = "/api/actions",
    responses(
        (status = 200, description = "Pending actions", body = Vec<ActionItem>),
    ),
    tag = "actions"
)]
async fn list_actions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ActionItem>>, (StatusCode, String)> {
    let actions = queries::get_pending_actions(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(actions))
}

#[derive(Deserialize, ToSchema)]
struct ActionResponse {
    /// Answer text (for escalations)
    answer: Option<String>,
    /// Grant or deny (for permission requests)
    granted: Option<bool>,
    /// Optional notes from PO
    notes: Option<String>,
}

/// Respond to an action (escalation answer or permission decision)
#[utoipa::path(
    post,
    path = "/api/actions/{id}/respond",
    params(("id" = String, Path, description = "Action ID (escalation or permission)")),
    request_body = ActionResponse,
    responses(
        (status = 200, description = "Response recorded"),
        (status = 400, description = "Must provide 'answer' or 'granted'"),
        (status = 500, description = "Internal error"),
    ),
    tag = "actions"
)]
async fn respond_to_action(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ActionResponse>,
) -> Result<StatusCode, (StatusCode, String)> {
    if let Some(answer) = &req.answer {
        queries::answer_escalation(&state.pool, &id, answer, req.notes.as_deref())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    } else if let Some(granted) = req.granted {
        queries::decide_permission(&state.pool, &id, granted, "po", req.notes.as_deref())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    } else {
        return Err((StatusCode::BAD_REQUEST, "Must provide 'answer' or 'granted'".to_string()));
    }

    let _ = state.event_tx.send(SseEvent {
        event_type: "action.responded".to_string(),
        data: serde_json::json!({ "id": id }).to_string(),
    });

    Ok(StatusCode::OK)
}

// --- Messages ---

/// List recent ACP messages (activity feed)
#[utoipa::path(
    get,
    path = "/api/messages",
    responses(
        (status = 200, description = "Recent messages", body = Vec<Message>),
    ),
    tag = "messages"
)]
async fn list_messages(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Message>>, (StatusCode, String)> {
    let messages = queries::list_messages(&state.pool, 100)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(messages))
}

// --- SSE ---

#[derive(Debug, Clone, Serialize)]
pub struct SseEvent {
    pub event_type: String,
    pub data: String,
}

async fn sse_events(
    State(state): State<Arc<AppState>>,
) -> Sse<impl futures_core::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let mut rx = state.event_tx.subscribe();

    let stream = async_stream::stream! {
        while let Ok(event) = rx.recv().await {
            let sse_event = Event::default()
                .event(event.event_type)
                .data(event.data);
            yield Ok(sse_event);
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
