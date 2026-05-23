use crate::db::{self, ChatMessage};
use crate::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;

pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    let username = match receiver.next().await {
        Some(Ok(Message::Text(name))) => name,
        _ => return,
    };

    println!("{username} connected");

    let recent = db::recent(&state.pool, 50).await;
    for msg in &recent {
        if let Ok(json) = serde_json::to_string(msg) {
            if sender.send(Message::Text(json)).await.is_err() {
                return;
            }
        }
    }

    let mut rx = state.tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if sender.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    });

    let tx = state.tx.clone();
    let pool = state.pool.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                    db::insert(&pool, &chat_msg).await;
                    if let Ok(json) = serde_json::to_string(&chat_msg) {
                        tx.send(json).ok();
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    println!("{username} disconnected");
}
