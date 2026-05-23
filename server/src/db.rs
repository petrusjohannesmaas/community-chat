use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub username: String,
    pub content: String,
    pub timestamp: u64,
}

pub async fn init(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await
    .unwrap();
}

pub async fn insert(pool: &SqlitePool, msg: &ChatMessage) {
    sqlx::query("INSERT INTO messages (username, content, timestamp) VALUES (?, ?, ?)")
        .bind(&msg.username)
        .bind(&msg.content)
        .bind(msg.timestamp as i64)
        .execute(pool)
        .await
        .unwrap();
}

pub async fn recent(pool: &SqlitePool, limit: i32) -> Vec<ChatMessage> {
    let rows = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT username, content, timestamp FROM messages ORDER BY id DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .rev()
        .map(|(u, c, t)| ChatMessage {
            username: u,
            content: c,
            timestamp: t as u64,
        })
        .collect()
}
