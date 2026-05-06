# Technical Implementation Guide: FastAPI + GTK P2P Chat PoC

This document outlines the exact infrastructure, schemas, and integration steps required to build the Proof of Concept. It follows a local-first architecture where clients own all message state and the server functions exclusively as a stateless WebSocket relay.

---

## 1. Required Toolchain & Dependencies
| Component | Package | Purpose |
|-----------|---------|---------|
| **Runtime** | Python 3.11+ | Async/await, type hints, standard library improvements |
| **Server Framework** | `fastapi`, `uvicorn[standard]` | WebSocket endpoint, routing, HTTP health checks |
| **Data Validation** | `pydantic` (v2) | Shared message schemas for client/server validation |
| **Client GUI** | `PyGObject` (GTK 4 bindings) | Native desktop UI, event loop |
| **Async DB** | `aiosqlite` | Non-blocking local message persistence |
| **WebSocket Client** | `websockets` | Async bidirectional communication with FastAPI |
| **Utilities** | `uuid`, `time`, `json`, `logging` | Message IDs, timestamps, serialization, diagnostics |

**Project Structure**
```
poc_chat/
├── common/
│   └── models.py          # Shared Pydantic schemas
├── server/
│   ├── main.py            # FastAPI WebSocket relay
│   └── requirements.txt
├── client/
│   ├── main.py            # GTK UI + async integration
│   ├── db.py              # aiosqlite schema & queries
│   └── ws_client.py       # WebSocket connection manager
└── pyproject.toml
```

---

## 2. Define Shared Schemas
Both server and client must validate messages against identical structures. Place these in `common/models.py`.

```python
from pydantic import BaseModel, Field
from typing import Literal, Optional
from uuid import uuid4
from time import time

class Message(BaseModel):
    id: str = Field(default_factory=lambda: str(uuid4()))
    from_user: str
    to_user: str
    type: Literal["message", "ack", "read", "sync_request"]
    timestamp: float = Field(default_factory=time)
    seq: int = 0
    payload: str

class ServerEnvelope(BaseModel):
    # Wraps routing metadata for the server layer
    type: Literal["route", "system"]
    payload: Message | dict  # Message for route, status dict for system
```

**Validation Rules**
- `id`: Globally unique UUID4. Used for deduplication.
- `seq`: Monotonic integer incremented by the sender per conversation. Guarantees ordering when timestamps collide.
- `type`: Determines client-side state transitions and server routing behavior.
- `payload`: Plaintext string in PoC phase. Will become base64-encoded ciphertext in encryption phase.

---

## 3. Implement FastAPI WebSocket Relay
The server maintains only active WebSocket connections in memory. It does not persist messages, keys, or history.

**`server/main.py`**
```python
from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from common.models import ServerEnvelope, Message
import json

app = FastAPI()

# In-memory connection registry: {username: WebSocket}
active_connections: dict[str, WebSocket] = {}

@app.websocket("/ws/{username}")
async def websocket_endpoint(websocket: WebSocket, username: str):
    await websocket.accept()
    active_connections[username] = websocket
    try:
        while True:
            raw = await websocket.receive_text()
            envelope = ServerEnvelope.model_validate_json(raw)

            if envelope.type == "route":
                msg = Message.model_validate(envelope.payload)
                target_ws = active_connections.get(msg.to_user)
                if target_ws:
                    await target_ws.send_text(raw)  # Forward exact payload
                # If target offline, server drops message. Client handles offline queue.
    except WebSocketDisconnect:
        active_connections.pop(username, None)
```

**Infrastructure Notes**
- Run with: `uvicorn server.main:app --reload --host 0.0.0.0 --port 8000`
- No database, no authentication layer in PoC.
- `active_connections` dict resets on server restart. Clients must implement reconnection logic.

---

## 4. Implement GTK Client with Async Integration
GTK runs on a single main thread. WebSockets and SQLite must execute asynchronously without blocking the UI.

**Integration Pattern**
1. Start `asyncio` event loop in a background thread.
2. Use `GLib.idle_add()` to marshal WebSocket receipts and DB callbacks into GTK's main thread.
3. Never call GTK widgets directly from `asyncio` tasks.

**`client/ws_client.py`**
```python
import asyncio
import websockets
from gi.repository import GLib
from common.models import ServerEnvelope, Message
import json

class WSManager:
    def __init__(self, on_message_received, on_disconnect):
        self.uri = "ws://localhost:8000/ws"
        self.ws = None
        self.on_message = on_message_received
        self.on_disconnect = on_disconnect
        self.username = None

    async def connect(self, username: str):
        self.username = username
        self.ws = await websockets.connect(f"{self.uri}/{username}")
        await self.listen()

    async def listen(self):
        try:
            async for raw in self.ws:
                GLib.idle_add(self.on_message, raw)
        except websockets.ConnectionClosed:
            GLib.idle_add(self.on_disconnect)

    async def send(self, msg: Message):
        envelope = ServerEnvelope(type="route", payload=msg)
        if self.ws:
            await self.ws.send(envelope.model_dump_json())
```

**GTK UI Skeleton (`client/main.py`)**
```python
import gi
gi.require_version('Gtk', '4.0')
from gi.repository import Gtk, GLib, GObject
import asyncio, threading

class ChatApp(Gtk.Application):
    def __init__(self):
        super().__init__(application_id="com.localfirst.chat")
        self.ws_manager = None
        self.loop = None

    def do_activate(self):
        win = Gtk.ApplicationWindow(application=self)
        win.set_default_size(600, 400)
        # Add UI elements here (ListBox, Entry, Button)
        win.show()

        # Start asyncio loop in background thread
        self.loop = asyncio.new_event_loop()
        threading.Thread(target=self.loop.run_forever, daemon=True).start()
        asyncio.run_coroutine_threadsafe(self._init_ws(), self.loop)

    def _init_ws(self):
        self.ws_manager = WSManager(
            on_message_received=self._handle_incoming,
            on_disconnect=self._handle_disconnect
        )
        asyncio.run_coroutine_threadsafe(self.ws_manager.connect("alice"), self.loop)

    def _handle_incoming(self, raw: str):
        # Executes on GTK main thread via GLib.idle_add
        msg = Message.model_validate_json(raw)
        # Update UI and trigger local DB insert
        return False  # Required by GLib.idle_add

    def _handle_disconnect(self):
        # Update connection indicator
        return False
```

---

## 5. Local State Management & SQLite Schema
Each client maintains its own conversation state. The server does not track delivery, reads, or history.

**`client/db.py`**
```sql
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    from_user TEXT NOT NULL,
    to_user TEXT NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('message','ack','read')),
    timestamp REAL NOT NULL,
    seq INTEGER NOT NULL,
    payload TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending','sent','delivered','read')),
    created_at REAL DEFAULT (strftime('%s','now'))
);
CREATE INDEX IF NOT EXISTS idx_conversation ON messages(from_user, to_user, timestamp);
```

**State Machine Transitions**
| Status | Trigger | Action |
|--------|---------|--------|
| `pending` | User presses Enter, network unavailable | Store in SQLite |
| `sent` | WebSocket successfully transmits | Update status to `sent` |
| `delivered` | Receive `type: "ack"` matching `id` | Update status to `delivered` |
| `read` | Receive `type: "read"` or UI scrolls into view | Update status to `read` |

**Async DB Operations**
Use `aiosqlite` to query/insert without blocking GTK. Always execute `INSERT` or `UPDATE` inside `asyncio.create_task()`, then marshal result to UI via `GLib.idle_add()`.

---

## 6. Integration & Message Routing Flow

1. **Client A** generates `Message` with UUID, `seq=1`, `status="pending"`. Inserts into SQLite.
2. **Client A** calls `ws_manager.send(msg)`. `aiosqlite` updates status to `"sent"` on successful transmission.
3. **FastAPI** receives envelope, validates Pydantic schema, looks up `to_user` in `active_connections`.
4. **FastAPI** forwards raw JSON to **Client B**'s WebSocket.
5. **Client B** deserializes, inserts into SQLite with `status="delivered"` (implicit on receipt), renders in GTK.
6. **Client B** generates `ack` message with original `id`, sends back through same relay path.
7. **Client A** receives `ack`, updates local SQLite row to `status="delivered"`.

All state transitions occur locally. The server only sees opaque JSON and performs routing.

---

## 7. Offline Queue & Reconnection Logic

**Client-Side Queue**
```python
async def flush_pending_queue(self):
    async with aiosqlite.connect("chat.db") as db:
        cursor = await db.execute(
            "SELECT * FROM messages WHERE status = 'pending' ORDER BY created_at ASC"
        )
        rows = await cursor.fetchall()

    for row in rows:
        msg = Message(...) # reconstruct from row
        await self.ws_manager.send(msg)
        # Update status to 'sent' after successful send
```

**Reconnection Strategy**
1. Detect `WebSocketDisconnect` or `ConnectionClosed`.
2. Mark connection state as `disconnected` in UI.
3. Start exponential backoff timer (`1s, 2s, 4s, 8s...`).
4. On successful reconnect, call `flush_pending_queue()`.
5. Send `sync_request` with `last_timestamp` to request missed messages if server implements ephemeral buffer (optional in PoC).

---

## 8. Verification Checklist

| Step | Command/Action | Expected Result |
|------|----------------|-----------------|
| 1 | `uvicorn server.main:app` | FastAPI starts on `:8000`, Swagger at `/docs` |
| 2 | Run `client/main.py` (Instance A) | GTK window opens, connects, indicator green |
| 3 | Run `client/main.py` (Instance B) | Second window connects independently |
| 4 | Send message A→B | B receives instantly, SQLite updated, UI appended |
| 5 | Kill B, send A→B | A shows `pending`. Restart B. |
| 6 | B reconnects | B does not receive old message (server drops). A retries until delivered or manually cleared. |
| 7 | Validate schema mismatch | Server rejects/ignores invalid JSON, client logs error |

---

## Next Steps (Post-PoC)
1. Replace `payload: str` with base64-encoded AES-GCM ciphertext.
2. Implement RSA key exchange over separate WebSocket channel.
3. Add PBKDF2 key derivation from user passphrase.
4. Replace ephemeral server drop with in-memory TTL queue for offline delivery.
5. Add GTK message status icons (⏳ pending, ✓ sent, ✓✓ delivered).

This guide contains the exact schemas, infrastructure boundaries, and integration patterns required to execute the PoC. Begin with Phase 2 (Shared Schemas), then implement the FastAPI relay, followed by the GTK async client and SQLite state layer.
