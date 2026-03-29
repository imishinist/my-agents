use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MemoryEntry {
    pub key: String,
    pub category: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Upsert a project memory entry.
pub async fn set(pool: &SqlitePool, key: &str, category: &str, content: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO project_memory (key, category, content) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET content = excluded.content, category = excluded.category, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')"
    )
    .bind(key)
    .bind(category)
    .bind(content)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<MemoryEntry>, sqlx::Error> {
    sqlx::query_as::<_, MemoryEntry>("SELECT * FROM project_memory WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
}

pub async fn list_by_category(pool: &SqlitePool, category: &str) -> Result<Vec<MemoryEntry>, sqlx::Error> {
    sqlx::query_as::<_, MemoryEntry>("SELECT * FROM project_memory WHERE category = ? ORDER BY updated_at DESC")
        .bind(category)
        .fetch_all(pool)
        .await
}

pub async fn list_all(pool: &SqlitePool) -> Result<Vec<MemoryEntry>, sqlx::Error> {
    sqlx::query_as::<_, MemoryEntry>("SELECT * FROM project_memory ORDER BY category, key")
        .fetch_all(pool)
        .await
}

pub async fn delete(pool: &SqlitePool, key: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM project_memory WHERE key = ?")
        .bind(key)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
