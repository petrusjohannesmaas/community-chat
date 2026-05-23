# Technical Implementation Guide: Community Chat PoC

This document covers everything needed to get Phase 1 working — a group chat application where community members can send and receive messages in a shared channel.

---

## 1. What We're Building

A native desktop app where:
- A community administrator runs a Mosquitto broker via Docker
- Members install the Tauri desktop client
- Everyone connects to the same broker and chats in `community/general`

Nothing is stored on the server. No user accounts. No history server-side. The broker is a postbox — it receives a message and forwards it to everyone subscribed to that topic.

---

## 2. Mosquitto Broker

The broker is the only server-side component. It runs in Docker and requires no custom code.

**`deploy/docker-compose.yml`**
```yaml
version: "3.9"

services:
  mosquitto:
    image: eclipse-mosquitto:2
    ports:
      - "1883:1883"     # MQTT
      - "9001:9001"     # MQTT over WebSocket (for future web client)
    volumes:
      - ./mosquitto.conf:/mosquitto/config/mosquitto.conf
    restart: unless-stopped
```

**`deploy/mosquitto.conf`**
```
listener 1883
listener 9001
protocol websockets
allow_anonymous true
```

`allow_anonymous true` is intentional for Phase 1. Authentication is a Phase 3 concern. Do not expose port 1883 publicly without adding authentication first.

Start with:
```bash
docker compose up -d
```

---

## 3. Message Format

All messages are JSON published to `community/general`.

```json
{
  "username": "alice",
  "content": "Hello everyone",
  "timestamp": 1716800000
}
```

`timestamp` is Unix time in seconds. The client that receives the message formats it for display — the broker never interprets it.

This payload format is intentionally minimal. When encryption is added in a later phase, `content` becomes a ciphertext string. No other fields change.

---

## 4. Tauri Project Structure

```
├── src/                          # React frontend
│   ├── main.tsx                  # React entry point
│   ├── App.tsx                   # Root component, Tauri event listeners
│   └── components/
│       ├── MessageList.tsx       # Scrolling message history
│       └── MessageInput.tsx      # Text input and send button
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Tauri builder, command registration
│   │   └── mqtt.rs               # MQTT connection manager
│   └── Cargo.toml
│
└── deploy/
    ├── docker-compose.yml
    └── mosquitto.conf
```

---

## 5. Rust — MQTT Connection

Add `rumqttc` to `src-tauri/Cargo.toml`:

```toml
[dependencies]
rumqttc = "0.24"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tauri = { version = "2", features = [] }
```

**`src-tauri/src/mqtt.rs`**

```rust
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS, Event, Packet};
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

    // Subscribe to group channel
    client
        .subscribe(TOPIC, QoS::AtLeastOnce)
        .await
        .unwrap();

    // Spawn listener — forwards incoming messages to React as Tauri events
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Packet::Publish(msg))) => {
                    if let Ok(payload) = std::str::from_utf8(&msg.payload) {
                        if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(payload) {
                            // Emit to React frontend
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
```

**`src-tauri/src/main.rs`**

```rust
mod mqtt;

use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use rumqttc::AsyncClient;

struct MqttState(Arc<Mutex<Option<AsyncClient>>>);
struct UsernameState(Arc<Mutex<String>>);

#[tauri::command]
async fn connect(
    app: tauri::AppHandle,
    username: String,
    mqtt_state: State<'_, MqttState>,
    username_state: State<'_, UsernameState>,
) -> Result<(), String> {
    let client = mqtt::connect(app, username.clone()).await;
    *mqtt_state.0.lock().await = Some(client);
    *username_state.0.lock().await = username;
    Ok(())
}

#[tauri::command]
async fn send_message(
    content: String,
    mqtt_state: State<'_, MqttState>,
    username_state: State<'_, UsernameState>,
) -> Result<(), String> {
    let guard = mqtt_state.0.lock().await;
    let username = username_state.0.lock().await;
    if let Some(client) = guard.as_ref() {
        mqtt::publish(client, &username, &content).await;
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(MqttState(Arc::new(Mutex::new(None))))
        .manage(UsernameState(Arc::new(Mutex::new(String::new()))))
        .invoke_handler(tauri::generate_handler![connect, send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## 6. React Frontend

**`src/App.tsx`**

```tsx
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import MessageList from "./components/MessageList";
import MessageInput from "./components/MessageInput";

interface ChatMessage {
  username: string;
  content: string;
  timestamp: number;
}

export default function App() {
  const [username, setUsername] = useState("");
  const [connected, setConnected] = useState(false);
  const [messages, setMessages] = useState<ChatMessage[]>([]);

  useEffect(() => {
    const unlisten = listen<ChatMessage>("mqtt-message", (event) => {
      setMessages((prev) => [...prev, event.payload]);
    });
    return () => { unlisten.then(f => f()); };
  }, []);

  async function handleConnect() {
    await invoke("connect", { username });
    setConnected(true);
  }

  async function handleSend(content: string) {
    await invoke("send_message", { content });
  }

  if (!connected) {
    return (
      <div>
        <input
          placeholder="Enter your username"
          value={username}
          onChange={e => setUsername(e.target.value)}
        />
        <button onClick={handleConnect}>Join</button>
      </div>
    );
  }

  return (
    <div>
      <MessageList messages={messages} currentUser={username} />
      <MessageInput onSend={handleSend} />
    </div>
  );
}
```

**`src/components/MessageList.tsx`**

```tsx
interface Message {
  username: string;
  content: string;
  timestamp: number;
}

interface Props {
  messages: Message[];
  currentUser: string;
}

export default function MessageList({ messages, currentUser }: Props) {
  return (
    <div>
      {messages.map((msg, i) => (
        <div key={i}>
          <strong>{msg.username === currentUser ? "You" : msg.username}</strong>
          <span>{msg.content}</span>
          <small>{new Date(msg.timestamp * 1000).toLocaleTimeString()}</small>
        </div>
      ))}
    </div>
  );
}
```

**`src/components/MessageInput.tsx`**

```tsx
import { useState } from "react";

interface Props {
  onSend: (content: string) => void;
}

export default function MessageInput({ onSend }: Props) {
  const [value, setValue] = useState("");

  function handleSend() {
    if (!value.trim()) return;
    onSend(value.trim());
    setValue("");
  }

  return (
    <div>
      <input
        value={value}
        onChange={e => setValue(e.target.value)}
        onKeyDown={e => e.key === "Enter" && handleSend()}
        placeholder="Type a message..."
      />
      <button onClick={handleSend}>Send</button>
    </div>
  );
}
```

---

## 7. End-to-End Flow

```
User types message and presses Enter
        ↓
React calls invoke("send_message", { content })
        ↓
Rust builds ChatMessage JSON, publishes to community/general (QoS 1)
        ↓
Mosquitto receives and forwards to all subscribers
        ↓
Rust eventloop receives Publish packet
        ↓
app.emit("mqtt-message", chat_msg) → React
        ↓
React appends message to state → MessageList re-renders
```

Sender also receives their own message back from the broker, which is correct behaviour — it confirms delivery and keeps all clients in sync.

---

## 8. Verification Checklist

| Step | Action | Expected result |
|---|---|---|
| 1 | `docker compose up -d` | Mosquitto running on port 1883 |
| 2 | `cargo tauri dev` | App opens, shows username input |
| 3 | Enter username, click Join | Connected to broker |
| 4 | Open second instance with different username | Both connected |
| 5 | Send message from instance A | Appears in both A and B |
| 6 | Send message from instance B | Appears in both A and B |
| 7 | Close instance B, send from A | Message published (no receiver — expected) |
| 8 | Reopen instance B | Does not receive missed messages (expected in Phase 1) |

---

## Next Steps

- **Phase 2:** Add member list, DM topic structure (`community/dm/{a}/{b}`), click to open DM
- **Phase 3:** Username persistence, local message history (SQLite via Tauri), online presence via MQTT will/retain
- **Phase 4:** Authentication on Mosquitto, invite link system
- **Phase 5:** E2EE — encrypt payload before publish, decrypt on receive. No broker or topic changes needed.
