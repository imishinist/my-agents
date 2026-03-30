use std::sync::Arc;

use crate::router::SseEvent;
use crate::server::AppState;
use conducer_state::queries;

/// Handle an incoming ACP message by type, triggering PM Agent actions.
pub async fn handle_acp_event(state: &Arc<AppState>, message_type: &str, payload: &serde_json::Value) {
    match message_type {
        "pr.submitted" => handle_pr_submitted(state, payload).await,
        "clarification.request" => handle_clarification_request(state, payload).await,
        _ => {}
    }
}

/// PR submitted → PM Agent reviews → update status → if merged, resolve deps
async fn handle_pr_submitted(state: &Arc<AppState>, payload: &serde_json::Value) {
    let feature_id = match payload["feature_id"].as_str() {
        Some(id) => id.to_string(),
        None => return,
    };
    let pr_diff = payload["summary"].as_str().unwrap_or("").to_string();

    // Update feature status to in_review
    let _ = queries::update_feature_status(&state.pool, &feature_id, "in_review").await;

    tracing::info!(feature_id = %feature_id, "PM Agent: reviewing PR");

    match state.pm_agent.review_pr(&feature_id, &pr_diff).await {
        Ok(result) => {
            tracing::info!(
                feature_id = %feature_id,
                verdict = %result.verdict,
                "PM Agent: review complete"
            );

            let _ = state.event_tx.send(SseEvent {
                event_type: "review.completed".to_string(),
                data: serde_json::json!({
                    "feature_id": feature_id,
                    "verdict": result.verdict,
                    "summary": result.summary,
                })
                .to_string(),
            });

            // If approved (merged), check for next wave
            if result.verdict == "approved" {
                resolve_dependencies_and_spawn(state, &feature_id).await;
            }
        }
        Err(e) => {
            tracing::error!(feature_id = %feature_id, error = %e, "PM Agent: review failed");
        }
    }
}

/// After a feature is merged, check if dependent features are now unblocked.
async fn resolve_dependencies_and_spawn(state: &Arc<AppState>, merged_feature_id: &str) {
    // Get the epic_id of the merged feature
    let feature = match queries::get_feature(&state.pool, merged_feature_id).await {
        Ok(Some(f)) => f,
        _ => return,
    };

    // Check if epic is complete
    match state.pm_agent.check_epic_completion(&feature.epic_id).await {
        Ok(true) => {
            tracing::info!(epic_id = %feature.epic_id, "Epic completed!");
            let _ = state.event_tx.send(SseEvent {
                event_type: "epic.completed".to_string(),
                data: serde_json::json!({ "epic_id": feature.epic_id }).to_string(),
            });
            return;
        }
        Ok(false) => {}
        Err(e) => {
            tracing::error!(error = %e, "Failed to check epic completion");
        }
    }

    // Find newly assignable features
    match state.pm_agent.get_assignable_features().await {
        Ok(ready) if !ready.is_empty() => {
            tracing::info!(
                count = ready.len(),
                "Features unblocked after merge, ready for assignment"
            );
            let _ = state.event_tx.send(SseEvent {
                event_type: "features.unblocked".to_string(),
                data: serde_json::json!({
                    "count": ready.len(),
                    "feature_ids": ready.iter().map(|f| &f.id).collect::<Vec<_>>(),
                })
                .to_string(),
            });
        }
        _ => {}
    }
}

/// Clarification request → PM Agent answers or escalates to PO
async fn handle_clarification_request(state: &Arc<AppState>, payload: &serde_json::Value) {
    let feature_id = match payload["feature_id"].as_str() {
        Some(id) => id.to_string(),
        None => return,
    };
    let question = payload["question"].as_str().unwrap_or("").to_string();
    let context = payload["context"].as_str().unwrap_or("").to_string();

    tracing::info!(feature_id = %feature_id, "PM Agent: answering clarification");

    match state
        .pm_agent
        .answer_clarification(&feature_id, &question, &context)
        .await
    {
        Ok(answer) => {
            if answer.escalate {
                tracing::info!(feature_id = %feature_id, "PM Agent: escalating to PO");
                let _ = state.event_tx.send(SseEvent {
                    event_type: "clarification.escalated".to_string(),
                    data: serde_json::json!({
                        "feature_id": feature_id,
                        "question": question,
                        "reason": answer.escalation_reason,
                    })
                    .to_string(),
                });
            } else {
                let _ = state.event_tx.send(SseEvent {
                    event_type: "clarification.answered".to_string(),
                    data: serde_json::json!({
                        "feature_id": feature_id,
                        "answer": answer.answer,
                    })
                    .to_string(),
                });
            }
        }
        Err(e) => {
            tracing::error!(feature_id = %feature_id, error = %e, "PM Agent: clarification failed");
        }
    }
}
