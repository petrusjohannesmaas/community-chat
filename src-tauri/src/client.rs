use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

const SERVER: &str = "ws://127.0.0.1:3030/ws";

#[derive(Serialize)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "login")]
    Login { username: String, password: String },
    #[serde(rename = "chat")]
    Chat { content: String },
    #[serde(rename = "dm")]
    Dm { to: String, content: String },
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ServerMessage {
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
        #[serde(default)]
        error: Option<String>,
    },
}

pub type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

pub async fn connect(
    app: AppHandle,
    username: String,
    password: String,
) -> Result<WsWriter, String> {
    let (ws_stream, _) = connect_async(SERVER).await.map_err(|e| e.to_string())?;
    let (mut write, read) = ws_stream.split();

    let login_msg = serde_json::to_string(&ClientMessage::Login {
        username: username.clone(),
        password,
    })
    .unwrap();

    write
        .send(Message::Text(login_msg.into()))
        .await
        .map_err(|e| e.to_string())?;

    // Wait for login_result
    let mut read = read;
    match read.next().await {
        Some(Ok(Message::Text(text))) => {
            match serde_json::from_str::<ServerMessage>(&text) {
                Ok(ServerMessage::LoginResult {
                    success: true,
                    error: _,
                }) => {
                    // Login successful — spawn the read loop
                    let app_clone = app.clone();
                    tokio::spawn(async move {
                        while let Some(Ok(msg)) = read.next().await {
                            if let Message::Text(text) = msg {
                                match serde_json::from_str::<ServerMessage>(&text) {
                                    Ok(ServerMessage::Chat {
                                        username,
                                        content,
                                        timestamp,
                                    }) => {
                                        app_clone
                                            .emit(
                                                "ws-message",
                                                serde_json::json!({
                                                    "username": username,
                                                    "content": content,
                                                    "timestamp": timestamp
                                                }),
                                            )
                                            .ok();
                                    }
                                    Ok(ServerMessage::Dm {
                                        from,
                                        to,
                                        content,
                                        timestamp,
                                    }) => {
                                        app_clone
                                            .emit(
                                                "ws-dm",
                                                serde_json::json!({
                                                    "from": from,
                                                    "to": to,
                                                    "content": content,
                                                    "timestamp": timestamp
                                                }),
                                            )
                                            .ok();
                                    }
                                    Ok(ServerMessage::MemberList { members }) => {
                                        app_clone
                                            .emit("ws-member-list", members)
                                            .ok();
                                    }
                                    Ok(ServerMessage::UserJoined { username }) => {
                                        app_clone
                                            .emit("ws-user-joined", username)
                                            .ok();
                                    }
                                    Ok(ServerMessage::UserLeft { username }) => {
                                        app_clone
                                            .emit("ws-user-left", username)
                                            .ok();
                                    }
                                    _ => {}
                                }
                            }
                        }
                    });

                    Ok(write)
                }
                Ok(ServerMessage::LoginResult {
                    success: false,
                    error,
                }) => Err(error.unwrap_or_else(|| "Login failed".into())),
                _ => Err("Unexpected response from server".into()),
            }
        }
        Some(Ok(Message::Close(_))) => Err("Connection closed by server".into()),
        Some(Err(e)) => Err(e.to_string()),
        None => Err("Connection closed".into()),
        _ => Err("Unexpected message".into()),
    }
}

pub async fn send_chat(writer: &mut WsWriter, content: &str) {
    let msg = ClientMessage::Chat {
        content: content.to_string(),
    };
    let payload = serde_json::to_string(&msg).unwrap();
    writer
        .send(Message::Text(payload.into()))
        .await
        .unwrap();
}

pub async fn send_dm(writer: &mut WsWriter, to: &str, content: &str) {
    let msg = ClientMessage::Dm {
        to: to.to_string(),
        content: content.to_string(),
    };
    let payload = serde_json::to_string(&msg).unwrap();
    writer
        .send(Message::Text(payload.into()))
        .await
        .unwrap();
}
