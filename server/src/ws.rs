use crate::db::{self, DmMessage};
use crate::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::mpsc;

#[derive(Serialize)]
#[serde(tag = "type")]
enum ServerMessage {
    #[serde(rename = "chat")]
    Chat {
        username: String,
        content: String,
        timestamp: u64,
    },
    #[serde(rename = "dm")]
    Dm {
        from: String,
        to: String,
        content: String,
        timestamp: u64,
    },
    #[serde(rename = "member_list")]
    MemberList { members: Vec<String> },
    #[serde(rename = "user_joined")]
    UserJoined { username: String },
    #[serde(rename = "user_left")]
    UserLeft { username: String },
    #[serde(rename = "login_result")]
    LoginResult {
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "login")]
    Login { username: String, password: String },
    #[serde(rename = "chat")]
    Chat { content: String },
    #[serde(rename = "dm")]
    Dm { to: String, content: String },
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn json(msg: &ServerMessage) -> String {
    serde_json::to_string(msg).unwrap()
}

async fn send_to(state: &Arc<AppState>, username: &str, msg: &str) {
    let conns = state.connections.read().await;
    if let Some(sender) = conns.get(username) {
        sender.send(msg.to_string()).await.ok();
    }
}

async fn broadcast(state: &Arc<AppState>, msg: &str, exclude: Option<&str>) {
    let mut dead = Vec::new();
    {
        let conns = state.connections.read().await;
        for (username, sender) in conns.iter() {
            if exclude.map_or(false, |e| username == e) {
                continue;
            }
            if sender.send(msg.to_string()).await.is_err() {
                dead.push(username.clone());
            }
        }
    }
    if !dead.is_empty() {
        let mut conns = state.connections.write().await;
        for user in dead {
            conns.remove(&user);
        }
    }
}

pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // --- Login step ---
    let username = match receiver.next().await {
        Some(Ok(Message::Text(text))) => {
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(ClientMessage::Login { username, password }) => {
                    let ok = db::verify_login(&state.pool, &username, &password).await;
                    if ok {
                        sender
                            .send(Message::Text(json(&ServerMessage::LoginResult {
                                success: true,
                                error: None,
                            })))
                            .await
                            .ok();
                        username
                    } else {
                        sender
                            .send(Message::Text(json(&ServerMessage::LoginResult {
                                success: false,
                                error: Some("Invalid username or password".into()),
                            })))
                            .await
                            .ok();
                        return;
                    }
                }
                _ => {
                    sender
                        .send(Message::Text(json(&ServerMessage::LoginResult {
                            success: false,
                            error: Some("Expected login message".into()),
                        })))
                        .await
                        .ok();
                    return;
                }
            }
        }
        _ => return,
    };

    println!("{username} connected");

    // --- Register connection ---
    let (tx, mut rx) = mpsc::channel::<String>(256);

    {
        let mut conns = state.connections.write().await;

        // Remove old connection if reconnecting
        conns.remove(&username);
        conns.insert(username.clone(), tx);
    }

    // --- Send member list to new client ---
    {
        let conns = state.connections.read().await;
        let members: Vec<String> = conns.keys().cloned().collect();
        if sender
            .send(Message::Text(json(&ServerMessage::MemberList { members })))
            .await
            .is_err()
        {
            // Client disconnected before full setup
            let mut conns = state.connections.write().await;
            conns.remove(&username);
            println!("{username} disconnected (early)");
            return;
        }
    }

    // --- Broadcast user_joined ---
    broadcast(
        &state,
        &json(&ServerMessage::UserJoined {
            username: username.clone(),
        }),
        Some(&username),
    )
    .await;

    // --- Send public chat history ---
    let recent_msgs = db::recent(&state.pool, 50).await;
    for msg in &recent_msgs {
        let json = json(&ServerMessage::Chat {
            username: msg.username.clone(),
            content: msg.content.clone(),
            timestamp: msg.timestamp,
        });
        if sender.send(Message::Text(json)).await.is_err() {
            break;
        }
    }

    // --- Send DM history ---
    let recent_dms = db::recent_dms(&state.pool, &username, 50).await;
    for dm in &recent_dms {
        let json = json(&ServerMessage::Dm {
            from: dm.from_username.clone(),
            to: dm.to_username.clone(),
            content: dm.content.clone(),
            timestamp: dm.timestamp,
        });
        if sender.send(Message::Text(json)).await.is_err() {
            break;
        }
    }

    // --- Send task: reads from mpsc rx, writes to WebSocket ---
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // --- Receive task: reads from WebSocket, routes messages ---
    let state2 = state.clone();
    let username2 = username.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(ClientMessage::Chat { content }) => {
                        let chat_msg = db::ChatMessage {
                            username: username2.clone(),
                            content: content.clone(),
                            timestamp: now(),
                        };
                        db::insert(&state2.pool, &chat_msg).await;
                        let payload = json(&ServerMessage::Chat {
                            username: username2.clone(),
                            content,
                            timestamp: chat_msg.timestamp,
                        });
                        broadcast(&state2, &payload, None).await;
                    }
                    Ok(ClientMessage::Dm { to, content }) => {
                        let ts = now();
                        let dm_record = DmMessage {
                            from_username: username2.clone(),
                            to_username: to.clone(),
                            content: content.clone(),
                            timestamp: ts,
                        };
                        db::insert_dm(&state2.pool, &dm_record).await;
                        let payload = json(&ServerMessage::Dm {
                            from: username2.clone(),
                            to: to.clone(),
                            content,
                            timestamp: ts,
                        });
                        send_to(&state2, &username2, &payload).await;
                        send_to(&state2, &to, &payload).await;
                    }
                    _ => {}
                }
            }
        }
    });

    // --- Wait for either task to complete ---
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    // --- Cleanup ---
    {
        let mut conns = state.connections.write().await;
        conns.remove(&username);
    }

    broadcast(
        &state,
        &json(&ServerMessage::UserLeft {
            username: username.clone(),
        }),
        None,
    )
    .await;

    println!("{username} disconnected");
}
