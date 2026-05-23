use rumqttc::{AsyncClient, MqttOptions, QoS, Event, Packet};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

const TOPIC: &str = "community/general";
const BROKER: &str = "localhost";
const PORT: u16 = 1883;

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub username: String,
    pub content: String,
    pub timestamp: u64,
}

pub async fn connect(app: AppHandle, username: String) -> AsyncClient {
    let mut options = MqttOptions::new(&username, BROKER, PORT);
    options.set_keep_alive(std::time::Duration::from_secs(30));

    let (client, mut eventloop) = AsyncClient::new(options, 10);

    client
        .subscribe(TOPIC, QoS::AtLeastOnce)
        .await
        .unwrap();

    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Packet::Publish(msg))) => {
                    if let Ok(payload) = std::str::from_utf8(&msg.payload) {
                        if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(payload) {
                            app.emit("mqtt-message", chat_msg).ok();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("MQTT error: {e}");
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
                _ => {}
            }
        }
    });

    client
}

pub async fn publish(client: &AsyncClient, username: &str, content: &str) {
    let msg = ChatMessage {
        username: username.to_string(),
        content: content.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let payload = serde_json::to_string(&msg).unwrap();

    client
        .publish(TOPIC, QoS::AtLeastOnce, false, payload)
        .await
        .unwrap();
}