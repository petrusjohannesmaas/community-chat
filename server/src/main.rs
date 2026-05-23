mod db;
mod ws;

use axum::{routing::get, Router};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:/data/chat.db".to_string());

    let connect_options = SqliteConnectOptions::from_str(&database_url)
        .expect("Invalid database URL")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .connect_with(connect_options)
        .await
        .expect("Failed to connect to SQLite");

    db::init(&pool).await;

    let (tx, _rx) = broadcast::channel::<String>(100);

    let app_state = Arc::new(AppState { pool, tx });

    let app = Router::new()
        .route("/ws", get(ws::handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();

    println!("Chat server running on 0.0.0.0:3030");
    axum::serve(listener, app).await.unwrap();
}
