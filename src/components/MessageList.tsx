interface Message {
  username: string;
  content: string;
  timestamp: number;
}

interface Props {
  messages: Message[];
  currentUser: string;
  conversationLabel: string;
}

export default function MessageList({ messages, currentUser, conversationLabel }: Props) {
  return (
    <>
      <div className="conversation-header">{conversationLabel}</div>
      <div style={{ flex: 1, overflowY: "auto", marginBottom: "1rem" }}>
        {messages.length === 0 && (
          <div style={{ color: "#888", fontStyle: "italic", padding: "1rem 0" }}>
            No messages yet. Start the conversation!
          </div>
        )}
        {messages.map((msg, i) => (
          <div key={i} style={{ marginBottom: "0.5rem" }}>
            <strong>{msg.username === currentUser ? "You" : msg.username}</strong>
            {" "}
            <span>{msg.content}</span>
            {" "}
            <small>{new Date(msg.timestamp * 1000).toLocaleTimeString()}</small>
          </div>
        ))}
      </div>
    </>
  );
}
