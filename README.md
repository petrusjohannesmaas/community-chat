# Community Chat

A self-hosted group chat for communities. Deployable by an administrator with a single command — members install the desktop client and connect.

---

## What It Is

Community Chat is built around the idea that a small community should be able to run their own private messaging without depending on a third-party platform. There are no accounts, no tracking, and no data leaving infrastructure the administrator controls.

Messages are persisted on the server, so members receive history they missed while offline. The server is the only component that needs to be hosted — it handles message routing, storage, and history replay. Members install a native desktop client and connect to it.

---

## Deployment

The server runs via Docker Compose. One command brings it up.

```bash
docker compose up -d
```

The desktop client is distributed as a native installer (`.deb`, `.exe`, `.dmg`) built by Tauri.

**Server address:** The client has the server IP hardcoded in `src-tauri/src/client.rs`. Before distributing the client, update `SERVER` to point at the machine running the server. A config file approach is planned for a later phase.

---

## Project Structure

```
├── src/                        # React frontend
│   ├── App.tsx
│   └── components/
│       ├── MessageList.tsx
│       ├── MessageInput.tsx
│       └── MemberList.tsx
├── src-tauri/                  # Rust / Tauri backend
│   ├── src/
│   │   ├── main.rs
│   │   └── client.rs           # WebSocket client connection
│   └── Cargo.toml
├── server/                     # WebSocket server
│   ├── src/
│   │   ├── main.rs
│   │   ├── ws.rs               # WebSocket handler and broadcast
│   │   └── db.rs               # Message persistence
│   └── Cargo.toml
├── deploy/
│   └── docker-compose.yml
└── README.md
```

---

## Message Format

```json
{
  "username": "alice",
  "content": "Hello everyone",
  "timestamp": 1234567890
}
```

---

## Roadmap

### Phase 1 — Group Chat
- Server running in Docker with message persistence
- Desktop client connecting to server
- Group channel where all members can send and receive messages
- Members receive history they missed while offline

### Phase 2 — Direct Messages
- Member list in UI
- Click a member to open a DM conversation

### Phase 3 — Polish
- Username persistence across sessions
- Typing indicators
- Online presence
- Unread message count
- Auto-scroll to latest message

### Phase 4 — Auth and Invite System
- Token-based authentication on connect
- Invite link generation
- Admin member management

### Phase 5 — Encryption
- End-to-end encrypted message payloads
- Key exchange on first contact
- No changes required to server routing or storage

---

## License

GPL-3.0
