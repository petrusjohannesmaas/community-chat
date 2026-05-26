import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import MessageList from "./components/MessageList";
import MessageInput from "./components/MessageInput";
import MemberList from "./components/MemberList";

interface ChatMessage {
  username: string;
  content: string;
  timestamp: number;
}

interface DmPayload {
  from: string;
  to: string;
  content: string;
  timestamp: number;
}

type ConversationId = "global" | `dm:${string}`;

function conversationLabel(id: ConversationId): string {
  if (id === "global") return "# Global Chat";
  return `DM with ${id.slice(3)}`;
}

function App() {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [loginError, setLoginError] = useState("");
  const [connected, setConnected] = useState(false);
  const [members, setMembers] = useState<string[]>([]);
  const [messagesByConversation, setMessagesByConversation] = useState<
    Record<string, ChatMessage[]>
  >({ global: [] });
  const [currentConversation, setCurrentConversation] =
    useState<ConversationId>("global");

  const receivedMessage = useCallback(
    (msg: ChatMessage) => {
      setMessagesByConversation((prev) => ({
        ...prev,
        global: [...(prev.global || []), msg],
      }));
    },
    [],
  );

  const receivedDm = useCallback(
    (dm: DmPayload, currentUser: string) => {
      const other = dm.from === currentUser ? dm.to : dm.from;
      const key = `dm:${other}`;
      const chatMsg: ChatMessage = {
        username: dm.from,
        content: dm.content,
        timestamp: dm.timestamp,
      };
      setMessagesByConversation((prev) => ({
        ...prev,
        [key]: [...(prev[key] || []), chatMsg],
      }));
    },
    [],
  );

  useEffect(() => {
    const unlisteners: (() => void)[] = [];

    listen<ChatMessage>("ws-message", (event) => {
      receivedMessage(event.payload);
    }).then((unlisten) => unlisteners.push(unlisten));

    listen<DmPayload>("ws-dm", (event) => {
      receivedDm(event.payload, username);
    }).then((unlisten) => unlisteners.push(unlisten));

    listen<string[]>("ws-member-list", (event) => {
      setMembers(event.payload);
    }).then((unlisten) => unlisteners.push(unlisten));

    listen<string>("ws-user-joined", (event) => {
      setMembers((prev) =>
        prev.includes(event.payload) ? prev : [...prev, event.payload],
      );
    }).then((unlisten) => unlisteners.push(unlisten));

    listen<string>("ws-user-left", (event) => {
      setMembers((prev) => prev.filter((m) => m !== event.payload));
    }).then((unlisten) => unlisteners.push(unlisten));

    return () => {
      unlisteners.forEach((f) => f());
    };
  }, [receivedMessage, receivedDm, username]);

  async function handleLogin() {
    setLoginError("");
    try {
      await invoke("connect", { username, password });
      setConnected(true);
    } catch (e: any) {
      setLoginError(typeof e === "string" ? e : "Login failed");
    }
  }

  async function handleSend(content: string) {
    const recipient =
      currentConversation === "global"
        ? null
        : currentConversation.slice(3);
    await invoke("send_message", { content, recipient });
  }

  function handleSelectMember(member: string) {
    if (member === username) return;
    setCurrentConversation(`dm:${member}`);
  }

  function handleSelectGlobal() {
    setCurrentConversation("global");
  }

  function getCurrentMessages(): ChatMessage[] {
    return messagesByConversation[currentConversation] || [];
  }

  if (!connected) {
    return (
      <div className="login-container">
        <div className="login-form">
          <h1>Community Chat</h1>
          <input
            placeholder="Username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            autoFocus
          />
          <input
            type="password"
            placeholder="Password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleLogin()}
          />
          {loginError && <div className="login-error">{loginError}</div>}
          <button onClick={handleLogin}>Login</button>
        </div>
      </div>
    );
  }

  return (
    <div className="chat-layout">
      <MemberList
        members={members}
        currentUser={username}
        activeConversation={currentConversation}
        onSelectGlobal={handleSelectGlobal}
        onSelectMember={handleSelectMember}
      />
      <div className="chat-main">
        <MessageList
          messages={getCurrentMessages()}
          currentUser={username}
          conversationLabel={conversationLabel(currentConversation)}
        />
        <MessageInput onSend={handleSend} />
      </div>
    </div>
  );
}

export default App;
