# Implementing member list and DMs

**Session ID:** ses_19ad559c9ffeb1IGfMKRZ4ZwGM
**Created:** 5/26/2026, 6:42:34 PM
**Updated:** 5/26/2026, 6:54:56 PM

---

## User

Phase 1 of the roadmap is complete. Right now I need to implement a member list in the ui. This will for now have to be all the active connections I guess. Here's t he challenge. We are going to need to also implement the functionality to have DMs. Think this through on how it could be implemented

---

## Assistant

Here's my design analysis and proposed plan.

---

## Design

### 1. Protocol: Typed Message Envelope

Currently all messages are flat `{ username, content, timestamp }` JSON. We need a `type` discriminator so both server and client can distinguish chat messages from DMs and system events.

**Server → Client variants:**
- `{"type": "chat", "username": "alice", "content": "hello", "timestamp": 123}` — public message
- `{"type": "dm", "from": "alice", "to": "bob", "content": "secret", "timestamp": 123}` — DM
- `{"type": "member_list", "members": ["alice", "bob", "charlie"]}` — full member list on connect
- `{"type": "user_joined", "username": "alice"}` — broadcast on new connection
- `{"type": "user_left", "username": "alice"}` — broadcast on disconnect

**Client → Server variants:**
- `{"type": "chat", "content": "hello"}` — send public message (server fills username/timestamp)
- `{"type": "dm", "to": "bob", "content": "secret"}` — send DM

History replay wraps old DB rows with `"type": "chat"`. DMs are **not persisted** in this phase.

### 2. Server: Per-User Channels Instead of Single Broadcast

The current single `broadcast::channel` can't route DMs. I'll replace it with a `HashMap<String, mpsc::Sender<String>>` wrapped in `Arc<RwLock<...>>` in `AppState`.

- **On connect:** create an mpsc channel, insert username→tx into map, send `member_list` to the new client, broadcast `user_joined` to all others
- **On disconnect:** remove from map, broadcast `user_left`
- **On `chat` message:** iterate all mpsc senders and forward the message
- **On `dm` message:** look up the recipient's sender; send to recipient and back to sender only
- **History replay:** reads last 50 public messages from DB as before, now wrapped with `"type":"chat"`

### 3. Tauri Client: Discriminated Events

The read loop deserializes into a `ServerMessage` enum and emits typed Tauri events: `"ws-message"` (chat), `"ws-dm"`, `"ws-member-list"`, `"ws-user-joined"`, `"ws-user-left"`.

The `send_message` command gains an optional `recipient` parameter — if `None`, sends a chat; if `Some(user)`, sends a DM.

### 4. Frontend Architecture

**New state in `App.tsx`:**
```typescript
type ConversationId = "global" | `dm:${string}`;

members: string[]           // from ws-member-list + ws-user-joined/left
conversations: Record<ConversationId, ChatMessage[]>
currentConversation: ConversationId  // "global" by default
```

- Chat messages always go to `conversations["global"]`
- DMs go to `conversations["dm:bob"]` (keyed by the *other* participant)
- `MemberList` shows all members; clicking one sets `currentConversation` to `"dm:{user}"`
- An explicit "Global Chat" entry returns to the global view
- `MessageList` filters to `conversations[currentConversation]`

### 5. Files to Modify/Create

| File | Action | What |
|---|---|---|
| `server/src/main.rs` | Modify | Add `connections: Arc<RwLock<HashMap<String, mpsc::Sender<String>>>>` to `AppState` |
| `server/src/ws.rs` | Rewrite | Typed message protocol, per-user mpsc channels, DM routing, join/leave broadcast |
| `src-tauri/src/client.rs` | Rewrite | `ServerMessage` enum, typed event emission, `send_dm` function |
| `src-tauri/src/lib.rs` | Modify | Add optional `recipient` to `send_message`, new command if needed |
| `src/App.tsx` | Modify | Member state, conversation map, current conversation tracking, handle new event types |
| `src/components/MessageList.tsx` | Tweak | Accept conversation context for display (e.g., show "DM with bob" header) |
| `src/components/MemberList.tsx` | **New** | Sidebar component listing members, click to DM, "Global Chat" button |
| `src/App.css` | Add | Styles for sidebar layout, member list, active conversation |

### Key Questions for You:

1. **DM persistence** — Should DMs be stored at all in this phase, or purely ephemeral (in-memory only)? I'd recommend ephemeral for now; we can add DM-specific storage later.
2. **Layout** — Sidebar (fixed left panel) vs drawer vs modal for the member list? I'd suggest a fixed left sidebar with the member list, since that's the most common and intuitive pattern for chat apps.
3. **Multiple connections per username** — Currently one username = one connection. Should we allow a username to re-connect seamlessly (e.g., if they close and reopen), or should it be a new session? I'd suggest allowing reconnection (remove old entry, add new one).

**Tool: question**

---

## Assistant

Good input. This adds a login/auth dimension to Phase 2. Let me revise the plan accordingly.

---

## Revised Plan

### What changes

The scope expands: we now need a **users table**, a **login flow**, **DM persistence**, seeded users, plus the member list and DM routing.

### 1. Database — 3 new tables

| Table | Columns | Purpose |
|---|---|---|
| `users` | `username TEXT PK`, `password_hash TEXT` | Pre-seeded with 3 users |
| `direct_messages` | `id`, `from_username`, `to_username`, `content`, `timestamp` | Persisted DMs |

`seed_users()` runs on startup if the users table is empty — inserts **alice**, **bob**, **charlie** with SHA-256 hashed passwords.

### 2. Protocol — Login first, then typed messages

```
1. CONNECT WebSocket
2. Client → Server: {"type": "login", "username": "alice", "password": "secret"}
3. Server → Client: {"type": "login_result", "success": true}
   (or {"type": "login_result", "success": false, "error": "..."} + close)
4. Normal typed messaging begins
```

Client → Server thereafter:
```
{"type": "chat", "content": "hello"}
{"type": "dm", "to": "bob", "content": "secret"}
```

Server → Client:
```
{"type": "chat", "username": "alice", "content": "hello", "timestamp": 123}
{"type": "dm", "from": "alice", "to": "bob", "content": "secret", "timestamp": 123}
{"type": "member_list", "members": ["alice","bob","charlie"]}
{"type": "user_joined", "username": "dave"}
{"type": "user_left", "username": "dave"}
```

### 3. Server — Per-user mpsc channels in `AppState`

```rust
pub struct AppState {
    pub pool: SqlitePool,
    pub connections: Arc<RwLock<HashMap<String, mpsc::Sender<String>>>>,
}
```

**Lifecycle:**
- **Login:** verify credentials against DB → fail or continue
- **On success:** insert into `connections` map, send `member_list` + public history (last 50) + DM history (last 50 per conversation), broadcast `user_joined` to others
- **On disconnect:** remove, broadcast `user_left`
- **Public chat:** iterate all `connections` values and forward
- **DMs:** look up recipient in `connections` map, forward only to sender + recipient; persist to `direct_messages` table
- **Duplicate reconnect:** old connection is dropped (removed from map), new one takes over

### 4. Tauri Client Changes

- `connect` command now takes `(username, password)`, sends login message, waits for `login_result`, returns `Ok(())` or `Err`
- `send_message` gains optional `recipient: Option<String>`
- Read loop deserializes into `ServerMessage` enum and emits typed events:
  - `"ws-message"` for chat
  - `"ws-dm"` for DM delivery
  - `"ws-member-list"`, `"ws-user-joined"`, `"ws-user-left"`

### 5. Frontend Architecture

**Login screen** replaces the current single-input join screen with username + password fields + Login button.

**State** in `App.tsx`:
```typescript
type ConversationId = "global" | `dm:${string}`;

members: string[]
conversations: Record<ConversationId, ChatMessage[]>
currentConversation: ConversationId  // "global" by default
```

**Layout:** Fixed left sidebar with:
- "Global Chat" button at top
- List of connected members below (self highlighted)
- Click a member → switches `currentConversation` to `"dm:{user}"`

**Message flow:**
- Chat messages → `conversations["global"]`
- DMs → `conversations["dm:{other_user}"]`
- `MessageList` renders only `conversations[currentConversation]` with a header like "Global Chat" or "DM with bob"
- On connect, any persisted DM history is placed into the appropriate conversation buckets

### 6. Files to Change

| File | Action |
|---|---|
| `server/Cargo.toml` | Add `sha2` dependency |
| `server/src/db.rs` | Add users table, DMs table, seed_users, verify_login, insert_dm, recent_dms |
| `server/src/ws.rs` | Rewrite: ClientMessage/ServerMessage enums, login flow, per-user mpsc routing |
| `server/src/main.rs` | Update AppState, add connections map, pass pool to ws handler |
| `src-tauri/src/client.rs` | Rewrite: ServerMessage enum, login handshake, typed event emission |
| `src-tauri/src/lib.rs` | Update connect/send_message commands for auth + DM |
| `src/App.tsx` | Login form, members state, conversation map, new event handlers |
| `src/components/MemberList.tsx` | **New** — sidebar with Global Chat + member list, click to DM |
| `src/components/MessageList.tsx` | Tweak — show conversation context header, accept conversation-scoped messages |
| `src/App.css` | Add sidebar layout, member list, login form styles |

### Questions before I proceed

1. **Default passwords for the 3 seeded users** — Should I just use `password123` for all three (alice, bob, charlie), or do you have specific credentials in mind?
2. **Password hashing** — I'll use SHA-256 via the `sha2` crate. This is adequate for a demo and will be replaced by proper auth in Phase 4. Good with you?
3. **DM history on reconnect** — When a user logs in, I'll send the last 50 DMs from each conversation they've been in. That's the same limit as public chat history. OK?

---

## User

Yes I am happy. Build it out and then give me a walkthrough on what has happened so I can test it.

---

## Assistant

Everything compiles and builds clean. Here's the full walkthrough:

---

## Walkthrough: What Changed and How to Test

### Architecture Recap

```
Login → Member List → Global Chat or DM with a member
```

The system now has **4 layers** instead of 2:

| Layer | Purpose |
|---|---|
| **Server** (Rust/Axum) | Authenticates users, tracks active connections, routes public messages to all, routes DMs to specific recipients, persists chat + DMs to SQLite |
| **Tauri Client** (Rust) | Connects WebSocket, performs login handshake, deserializes typed server messages, emits typed Tauri events to frontend |
| **React Frontend** | Login form, sidebar with member list, conversation-aware message list and input |
| **SQLite DB** | `messages` (public), `users` (seeded), `direct_messages` (DMs) |

### Database (`server/src/db.rs`)

Three tables now exist:

```sql
-- Public messages (Phase 1)
CREATE TABLE messages (id, username, content, timestamp);

-- 3 pre-seeded users (SHA-256 hashed passwords)
CREATE TABLE users (username TEXT PK, password_hash TEXT);

-- Persisted DMs
CREATE TABLE direct_messages (id, from_username, to_username, content, timestamp);
```

On server startup, `seed_users()` checks if `users` is empty and inserts:

| Username | Password |
|---|---|
| `alice` | `password123` |
| `bob` | `password123` |
| `charlie` | `password123` |

### WebSocket Protocol (`server/src/ws.rs`)

Messages now use a `"type"` discriminator:

**Login handshake (first message only):**
```
C→S: {"type":"login","username":"alice","password":"password123"}
S→C: {"type":"login_result","success":true}
  or: {"type":"login_result","success":false,"error":"Invalid username or password"}
```
If login fails, the server closes the connection immediately.

**After login — all messages are typed:**
```
C→S: {"type":"chat","content":"hello everyone"}
C→S: {"type":"dm","to":"bob","content":"secret"}

S→C: {"type":"chat","username":"alice","content":"hello everyone","timestamp":123}
S→C: {"type":"dm","from":"alice","to":"bob","content":"secret","timestamp":123}
S→C: {"type":"member_list","members":["alice","bob","charlie"]}
S→C: {"type":"user_joined","username":"dave"}
S→C: {"type":"user_left","username":"dave"}
```

### Connection Tracking (`server/src/main.rs` → `AppState`)

The old `broadcast::channel<String>` is replaced with a connections map:

```rust
pub struct AppState {
    pub pool: SqlitePool,
    pub connections: Arc<RwLock<HashMap<String, mpsc::Sender<String>>>>,
}
```

**Lifecycle:**
- **Connect + login success:** Insert into map → send `member_list` to new client + broadcast `user_joined` to others → send public history (last 50) + DM history (last 50)
- **Disconnect:** Remove from map → broadcast `user_left`
- **Public chat:** Iterate all connections and forward
- **DM:** Look up recipient's sender; forward to sender AND recipient (silently dropped if recipient offline, but persisted for later reconnect)
- **Reconnect (same username):** Old connection is removed, new one replaces it

### Tauri Client (`src-tauri/src/client.rs`)

- On `connect(app, username, password)`: Connects WebSocket, sends login, waits for `login_result` response. Returns `Ok(writer)` on success or `Err(message)` on failure.
- On success: spawns a read loop that deserializes `ServerMessage` and emits typed Tauri events:
  - `"ws-message"` → public chat message
  - `"ws-dm"` → DM delivery
  - `"ws-member-list"` → full member list array
  - `"ws-user-joined"` → single username
  - `"ws-user-left"` → single username
- `send_chat(writer, content)`: sends `{"type":"chat","content":"..."}`
- `send_dm(writer, to, content)`: sends `{"type":"dm","to":"...","content":"..."}`
- `send_message` command now takes `optional recipient: string | null` — if `null`, sends chat; if string, sends DM

### Frontend (`src/`)

**App.tsx — State shape:**
```typescript
type ConversationId = "global" | `dm:${string}`;

members: string[]
messagesByConversation: Record<string, ChatMessage[]>  // { global: [...], "dm:bob": [...], ... }
currentConversation: ConversationId  // "global" by default
```

**Login screen** — Username + Password inputs, Login button, error display.

**Chat layout** — Fixed 240px sidebar + flexible main area:
- **Sidebar:** "Global Chat" button at top, member list below (sorted: self first, then alphabetical). Green dot for online status. Active conversation highlighted in blue.
- **Main area:** Conversation header (e.g. "# Global Chat" or "DM with bob") + MessageList + MessageInput.

**Event flow:**
| Tauri Event | Handler |
|---|---|
| `ws-message` | Append to `messagesByConversation.global` |
| `ws-dm` | Determine other party → append to `messagesByConversation["dm:{other}"]` |
| `ws-member-list` | Replace `members` array |
| `ws-user-joined` | Add to `members` (if not already present) |
| `ws-user-left` | Remove from `members` |

### How to Test

1. **Rebuild and start the server:**
   ```bash
   cd server && cargo build --release && ./target/release/chat-server
   ```
   Or use Docker:
   ```bash
   docker compose -f deploy/docker-compose.yml up --build -d
   ```
   Wait for the "Chat server running on 0.0.0.0:3030" message.

2. **Start a Tauri client (dev mode):**
   ```bash
   npm run tauri dev
   ```

3. **Login** with one of the seeded accounts:
   - `alice` / `password123`
   - `bob` / `password123`
   - `charlie` / `password123`

4. **Test scenarios:**
   - Login with invalid credentials → error message shown
   - Login with valid credentials → sidebar shows connected members, history loads
   - Open a second client as a different user → both see each other in member list
   - Send a public chat message → all connected users see it
   - Click a member (e.g., bob) → switches to DM view
   - Send a DM → visible only to sender and recipient
   - Disconnect a client → other clients see them disappear from member list

### Files Changed (Summary)

| File | Change |
|---|---|
| `server/Cargo.toml` | Added `sha2`, `hex` |
| `server/src/db.rs` | New tables (users, direct_messages), seed_users, verify_login, insert_dm, recent_dms |
| `server/src/main.rs` | AppState: removed broadcast tx, added connections map |
| `server/src/ws.rs` | Full rewrite: typed protocol, login flow, per-user mpsc routing, DM handling |
| `src-tauri/src/client.rs` | Full rewrite: ServerMessage enum, login handshake, typed event emission |
| `src-tauri/src/lib.rs` | connect takes password, send_message takes optional recipient |
| `src/App.tsx` | Full rewrite: login form, members state, conversation map, event handlers |
| `src/App.css` | Full rewrite: sidebar layout, login form, member list, conversation header |
| `src/components/MemberList.tsx` | **New** — sidebar component with member list |
| `src/components/MessageList.tsx` | Updated: conversation header, empty state |

---

