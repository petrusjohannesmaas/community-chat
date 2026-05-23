use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use tokio::net::TcpStream;

const SERVER: &str = "ws://127.0.0.1:3030/ws";

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub username: String,
    pub content: String,
    pub timestamp: u64,
}

pub type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

pub async fn connect(app: AppHandle, username: String) -> Result<WsWriter, String> {
    let (ws_stream, _) = connect_async(SERVER).await.map_err(|e| e.to_string())?;
    let (mut write, read) = ws_stream.split();

    write
        .send(Message::Text(username.into()))
        .await
        .map_err(|e| e.to_string())?;

    tokio::spawn(async move {
        let mut read = read;
        while let Some(Ok(msg)) = read.next().await {
            if let Message::Text(text) = msg {
                if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                    app.emit("ws-message", chat_msg).ok();
                }
            }
        }
    });

    Ok(write)
}

pub async fn send(writer: &mut WsWriter, username: &str, content: &str) {
    let msg = ChatMessage {
        username: username.to_string(),
        content: content.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let payload = serde_json::to_string(&msg).unwrap();
    writer
        .send(Message::Text(payload.into()))
        .await
        .unwrap();
}
