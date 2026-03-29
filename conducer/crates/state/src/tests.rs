#[cfg(test)]
mod tests {
    use crate::{db, memory, queries};
    use std::path::PathBuf;
    use tempfile::tempdir;

    async fn setup_pool() -> sqlx::SqlitePool {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        // Keep dir alive by leaking it (test only)
        let db_path_owned: PathBuf = db_path.to_path_buf();
        std::mem::forget(dir);
        db::init_pool(&db_path_owned).await.unwrap()
    }

    #[tokio::test]
    async fn test_epic_crud() {
        let pool = setup_pool().await;

        queries::create_epic(&pool, "epic-001", "Test Epic", "Build something great")
            .await
            .unwrap();

        let epic = queries::get_epic(&pool, "epic-001").await.unwrap().unwrap();
        assert_eq!(epic.title, "Test Epic");
        assert_eq!(epic.status, "draft");

        queries::update_epic_status(&pool, "epic-001", "active")
            .await
            .unwrap();

        let epic = queries::get_epic(&pool, "epic-001").await.unwrap().unwrap();
        assert_eq!(epic.status, "active");

        let epics = queries::list_epics(&pool).await.unwrap();
        assert_eq!(epics.len(), 1);
    }

    #[tokio::test]
    async fn test_feature_crud() {
        let pool = setup_pool().await;

        queries::create_epic(&pool, "epic-001", "Test", "desc")
            .await
            .unwrap();

        queries::create_feature(
            &pool,
            "feat-001",
            "epic-001",
            "Auth module",
            "Implement auth",
            "[]",
            "high",
        )
        .await
        .unwrap();

        queries::create_feature(
            &pool,
            "feat-002",
            "epic-001",
            "Login UI",
            "Build login page",
            r#"["feat-001"]"#,
            "medium",
        )
        .await
        .unwrap();

        let features = queries::list_features_by_epic(&pool, "epic-001")
            .await
            .unwrap();
        assert_eq!(features.len(), 2);

        // feat-001 has no deps, should be ready
        let ready = queries::get_ready_features(&pool).await.unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "feat-001");

        // Mark feat-001 as merged
        queries::update_feature_status(&pool, "feat-001", "merged")
            .await
            .unwrap();

        // Now feat-002 should be ready
        let ready = queries::get_ready_features(&pool).await.unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "feat-002");
    }

    #[tokio::test]
    async fn test_worker_lifecycle() {
        let pool = setup_pool().await;

        queries::create_worker(&pool, "worker-1", "claude-code")
            .await
            .unwrap();

        let idle = queries::get_idle_workers(&pool).await.unwrap();
        assert_eq!(idle.len(), 1);

        queries::update_worker_status(&pool, "worker-1", "busy")
            .await
            .unwrap();

        let idle = queries::get_idle_workers(&pool).await.unwrap();
        assert_eq!(idle.len(), 0);
    }

    #[tokio::test]
    async fn test_message_logging() {
        let pool = setup_pool().await;

        queries::insert_message(
            &pool,
            "msg-001",
            None,
            "orchestrator",
            "worker-1",
            "heartbeat.request",
            "{}",
        )
        .await
        .unwrap();

        let msgs = queries::list_messages(&pool, 10).await.unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].message_type, "heartbeat.request");
    }

    // --- Project Memory tests ---

    #[tokio::test]
    async fn test_memory_set_and_get() {
        let pool = setup_pool().await;

        memory::set(&pool, "architecture", "architecture", "Monorepo with Rust workspace")
            .await
            .unwrap();

        let entry = memory::get(&pool, "architecture").await.unwrap().unwrap();
        assert_eq!(entry.category, "architecture");
        assert_eq!(entry.content, "Monorepo with Rust workspace");
    }

    #[tokio::test]
    async fn test_memory_upsert() {
        let pool = setup_pool().await;

        memory::set(&pool, "db-choice", "architecture", "SQLite")
            .await
            .unwrap();
        memory::set(&pool, "db-choice", "architecture", "PostgreSQL")
            .await
            .unwrap();

        let entry = memory::get(&pool, "db-choice").await.unwrap().unwrap();
        assert_eq!(entry.content, "PostgreSQL");

        // Should still be only one entry
        let all = memory::list_all(&pool).await.unwrap();
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn test_memory_list_by_category() {
        let pool = setup_pool().await;

        memory::set(&pool, "api-v1", "interface", "REST API v1 endpoints")
            .await
            .unwrap();
        memory::set(&pool, "api-v2", "interface", "REST API v2 endpoints")
            .await
            .unwrap();
        memory::set(&pool, "no-unsafe", "constraint", "No unsafe Rust")
            .await
            .unwrap();

        let interfaces = memory::list_by_category(&pool, "interface").await.unwrap();
        assert_eq!(interfaces.len(), 2);

        let constraints = memory::list_by_category(&pool, "constraint").await.unwrap();
        assert_eq!(constraints.len(), 1);
        assert_eq!(constraints[0].content, "No unsafe Rust");
    }

    #[tokio::test]
    async fn test_memory_delete() {
        let pool = setup_pool().await;

        memory::set(&pool, "temp-note", "architecture", "temporary")
            .await
            .unwrap();

        let deleted = memory::delete(&pool, "temp-note").await.unwrap();
        assert!(deleted);

        let entry = memory::get(&pool, "temp-note").await.unwrap();
        assert!(entry.is_none());

        // Deleting non-existent key returns false
        let deleted = memory::delete(&pool, "nonexistent").await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_memory_get_nonexistent() {
        let pool = setup_pool().await;

        let entry = memory::get(&pool, "does-not-exist").await.unwrap();
        assert!(entry.is_none());
    }
}
