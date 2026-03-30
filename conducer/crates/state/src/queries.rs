use sqlx::SqlitePool;

use crate::models::*;

// --- Epic ---

pub async fn create_epic(pool: &SqlitePool, id: &str, title: &str, description: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO epics (id, title, description, status) VALUES (?, ?, ?, 'draft')")
        .bind(id)
        .bind(title)
        .bind(description)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_epic(pool: &SqlitePool, id: &str) -> Result<Option<Epic>, sqlx::Error> {
    sqlx::query_as::<_, Epic>("SELECT * FROM epics WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn list_epics(pool: &SqlitePool) -> Result<Vec<Epic>, sqlx::Error> {
    sqlx::query_as::<_, Epic>("SELECT * FROM epics ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn update_epic_status(pool: &SqlitePool, id: &str, status: &str) -> Result<(), sqlx::Error> {
    let mut query = String::from("UPDATE epics SET status = ?, last_error = NULL, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')");
    if status == "completed" {
        query.push_str(", completed_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')");
    }
    query.push_str(" WHERE id = ?");
    sqlx::query(&query)
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_epic_error(pool: &SqlitePool, id: &str, error: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE epics SET status = 'error', last_error = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?"
    )
    .bind(error)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

// --- Feature ---

pub async fn create_feature(
    pool: &SqlitePool,
    id: &str,
    epic_id: &str,
    title: &str,
    specification: &str,
    depends_on: &str,
    priority: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO features (id, epic_id, title, specification, depends_on, priority) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(epic_id)
    .bind(title)
    .bind(specification)
    .bind(depends_on)
    .bind(priority)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_feature(pool: &SqlitePool, id: &str) -> Result<Option<Feature>, sqlx::Error> {
    sqlx::query_as::<_, Feature>("SELECT * FROM features WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn list_features_by_epic(pool: &SqlitePool, epic_id: &str) -> Result<Vec<Feature>, sqlx::Error> {
    sqlx::query_as::<_, Feature>("SELECT * FROM features WHERE epic_id = ? ORDER BY priority DESC, created_at")
        .bind(epic_id)
        .fetch_all(pool)
        .await
}

pub async fn list_all_features(pool: &SqlitePool) -> Result<Vec<Feature>, sqlx::Error> {
    sqlx::query_as::<_, Feature>("SELECT * FROM features ORDER BY created_at")
        .fetch_all(pool)
        .await
}

pub async fn update_feature_status(pool: &SqlitePool, id: &str, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE features SET status = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn assign_feature_to_worker(
    pool: &SqlitePool,
    feature_id: &str,
    worker_id: &str,
    branch_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE features SET status = 'assigned', worker_id = ?, branch_name = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?",
    )
    .bind(worker_id)
    .bind(branch_name)
    .bind(feature_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get features whose dependencies are all merged
pub async fn get_ready_features(pool: &SqlitePool) -> Result<Vec<Feature>, sqlx::Error> {
    // Features that are pending and have no unmerged dependencies
    sqlx::query_as::<_, Feature>(
        r#"
        SELECT f.* FROM features f
        WHERE f.status = 'pending'
          AND NOT EXISTS (
            SELECT 1 FROM json_each(f.depends_on) dep
            JOIN features dep_f ON dep_f.id = dep.value
            WHERE dep_f.status != 'merged'
          )
        "#,
    )
    .fetch_all(pool)
    .await
}

// --- Worker ---

pub async fn create_worker(pool: &SqlitePool, id: &str, runtime_type: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO workers (id, runtime_type) VALUES (?, ?)")
        .bind(id)
        .bind(runtime_type)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_workers(pool: &SqlitePool) -> Result<Vec<Worker>, sqlx::Error> {
    sqlx::query_as::<_, Worker>("SELECT * FROM workers")
        .fetch_all(pool)
        .await
}

pub async fn get_idle_workers(pool: &SqlitePool) -> Result<Vec<Worker>, sqlx::Error> {
    sqlx::query_as::<_, Worker>("SELECT * FROM workers WHERE status = 'idle'")
        .fetch_all(pool)
        .await
}

pub async fn update_worker_status(pool: &SqlitePool, id: &str, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE workers SET status = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_worker_heartbeat(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE workers SET last_heartbeat = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'), updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_stalled_workers(pool: &SqlitePool, timeout_minutes: i64) -> Result<Vec<Worker>, sqlx::Error> {
    sqlx::query_as::<_, Worker>(
        "SELECT * FROM workers WHERE status = 'busy' AND last_heartbeat < strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ? || ' minutes')",
    )
    .bind(-timeout_minutes)
    .fetch_all(pool)
    .await
}

// --- Messages ---

pub async fn insert_message(
    pool: &SqlitePool,
    id: &str,
    correlation_id: Option<&str>,
    source: &str,
    destination: &str,
    message_type: &str,
    payload: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO messages (id, correlation_id, source, destination, type, payload) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(correlation_id)
    .bind(source)
    .bind(destination)
    .bind(message_type)
    .bind(payload)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_messages(pool: &SqlitePool, limit: i64) -> Result<Vec<Message>, sqlx::Error> {
    sqlx::query_as::<_, Message>("SELECT * FROM messages ORDER BY timestamp DESC LIMIT ?")
        .bind(limit)
        .fetch_all(pool)
        .await
}

// --- Escalations ---

pub async fn create_escalation(
    pool: &SqlitePool,
    escalation: &Escalation,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO escalations (id, feature_id, escalation_type, title, context, question, options, pm_recommendation, pm_reasoning, urgency, blocking_features) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&escalation.id)
    .bind(&escalation.feature_id)
    .bind(&escalation.escalation_type)
    .bind(&escalation.title)
    .bind(&escalation.context)
    .bind(&escalation.question)
    .bind(&escalation.options)
    .bind(&escalation.pm_recommendation)
    .bind(&escalation.pm_reasoning)
    .bind(&escalation.urgency)
    .bind(&escalation.blocking_features)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_pending_actions(pool: &SqlitePool) -> Result<Vec<ActionItem>, sqlx::Error> {
    sqlx::query_as::<_, ActionItem>(
        r#"
        SELECT 'escalation' as action_type, id, title, question, urgency, created_at
        FROM escalations WHERE status = 'pending'
        UNION ALL
        SELECT 'permission' as action_type, id, action as title, reason as question, risk_level as urgency, created_at
        FROM permissions WHERE status = 'pending'
        ORDER BY created_at
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn answer_escalation(
    pool: &SqlitePool,
    id: &str,
    answer: &str,
    notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE escalations SET status = 'answered', po_answer = ?, po_notes = ?, answered_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?",
    )
    .bind(answer)
    .bind(notes)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

// --- Permissions ---

pub async fn create_permission(
    pool: &SqlitePool,
    perm: &Permission,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO permissions (id, worker_id, feature_id, action, category, reason, risk_level) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&perm.id)
    .bind(&perm.worker_id)
    .bind(&perm.feature_id)
    .bind(&perm.action)
    .bind(&perm.category)
    .bind(&perm.reason)
    .bind(&perm.risk_level)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn decide_permission(
    pool: &SqlitePool,
    id: &str,
    granted: bool,
    decided_by: &str,
    notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    let status = if granted { "granted" } else { "denied" };
    sqlx::query(
        "UPDATE permissions SET status = ?, decided_by = ?, notes = ?, decided_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?",
    )
    .bind(status)
    .bind(decided_by)
    .bind(notes)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

// --- Reviews ---

pub async fn create_review(
    pool: &SqlitePool,
    id: &str,
    feature_id: &str,
    pr_number: i64,
    reviewer: &str,
    verdict: &str,
    summary: Option<&str>,
    comments: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO reviews (id, feature_id, pr_number, reviewer, verdict, summary, comments) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(feature_id)
    .bind(pr_number)
    .bind(reviewer)
    .bind(verdict)
    .bind(summary)
    .bind(comments)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_reviews_by_feature(pool: &SqlitePool, feature_id: &str) -> Result<Vec<Review>, sqlx::Error> {
    sqlx::query_as::<_, Review>("SELECT * FROM reviews WHERE feature_id = ? ORDER BY created_at DESC")
        .bind(feature_id)
        .fetch_all(pool)
        .await
}
