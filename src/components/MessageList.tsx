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
    <div style={{ flex: 1, overflowY: "auto", marginBottom: "1rem" }}>
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
  );
}