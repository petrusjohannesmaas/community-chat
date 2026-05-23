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

function App() {
  const [username, setUsername] = useState("");
  const [connected, setConnected] = useState(false);
  const [messages, setMessages] = useState<ChatMessage[]>([]);

  useEffect(() => {
    const unlisten = listen<ChatMessage>("ws-message", (event) => {
      setMessages((prev) => [...prev, event.payload]);
    });
    return () => {
      unlisten.then((f) => f());
    };
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
      <div style={{ padding: "2rem", fontFamily: "sans-serif" }}>
        <h1>Community Chat</h1>
        <input
          placeholder="Enter your username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleConnect()}
          style={{ marginRight: "0.5rem", padding: "0.5rem" }}
        />
        <button onClick={handleConnect} style={{ padding: "0.5rem" }}>
          Join
        </button>
      </div>
    );
  }

  return (
    <div style={{ padding: "1rem", fontFamily: "sans-serif", height: "100vh", display: "flex", flexDirection: "column", boxSizing: "border-box" }}>
      <MessageList messages={messages} currentUser={username} />
      <MessageInput onSend={handleSend} />
    </div>
  );
}

export default App;
