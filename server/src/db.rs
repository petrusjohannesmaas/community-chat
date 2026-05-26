use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub username: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct DmMessage {
    pub from_username: String,
    pub to_username: String,
    pub content: String,
    pub timestamp: u64,
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

const SEED_USERS: &[(&str, &str)] = &[
    ("alice", "password123"),
    ("bob", "password123"),
    ("charlie", "password123"),
];

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

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY,
            password_hash TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS direct_messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            from_username TEXT NOT NULL,
            to_username TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await
    .unwrap();

    seed_users(pool).await;
}

async fn seed_users(pool: &SqlitePool) {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if count.0 > 0 {
        return;
    }

    for (username, password) in SEED_USERS {
        let hash = hash_password(password);
        sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
            .bind(username)
            .bind(&hash)
            .execute(pool)
            .await
            .unwrap();
    }

    println!("Seeded {} users", SEED_USERS.len());
}

pub async fn verify_login(pool: &SqlitePool, username: &str, password: &str) -> bool {
    let hash = hash_password(password);

    let result: Result<(String,), _> =
        sqlx::query_as("SELECT password_hash FROM users WHERE username = ? AND password_hash = ?")
            .bind(username)
            .bind(&hash)
            .fetch_one(pool)
            .await;

    result.is_ok()
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

pub async fn insert_dm(pool: &SqlitePool, msg: &DmMessage) {
    sqlx::query(
        "INSERT INTO direct_messages (from_username, to_username, content, timestamp) VALUES (?, ?, ?, ?)",
    )
    .bind(&msg.from_username)
    .bind(&msg.to_username)
    .bind(&msg.content)
    .bind(msg.timestamp as i64)
    .execute(pool)
    .await
    .unwrap();
}

pub async fn recent_dms(pool: &SqlitePool, username: &str, limit: i32) -> Vec<DmMessage> {
    let rows = sqlx::query_as::<_, (String, String, String, i64)>(
        "SELECT from_username, to_username, content, timestamp
         FROM direct_messages
         WHERE from_username = ? OR to_username = ?
         ORDER BY id DESC LIMIT ?",
    )
    .bind(username)
    .bind(username)
    .bind(limit)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .rev()
        .map(|(f, t, c, ts)| DmMessage {
            from_username: f,
            to_username: t,
            content: c,
            timestamp: ts as u64,
        })
        .collect()
}
