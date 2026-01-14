# e2ee-chat
A secure multi platform chat application

## Roadmap:
1. Begin without encryption with client + server architecture
2. Add encryption
3. Move to peer to peer
4. Configure expiring local storage per user
5. Cache / sync

## Architecture: P2P Storage + Server relay

### Overview

✔️ Simple to build
✔️ Works on mobile networks
✔️ Scales well

Think of the server like a post office that can’t open envelopes and doesn’t keep copies. **It only knows:**
1. Sender
2. Recipient
3. “Here’s a blob”

| Feature           | Who owns it    |
| ----------------- | -------------- |
| Message content   | **Peers only** |
| Message storage   | **Peers only** |
| Encryption keys   | **Peers only** |
| Server visibility | None           |
| Server role       | Router         |
This is **peer-to-peer trust**, not peer-to-peer transport — and that’s our sweet spot.
### Components

#### 1️⃣ WebSocket Relay Server

Minimal responsibilities:

* Authenticate user
* Maintain online connections
* Forward encrypted messages

**It never writes messages to disk.**

Example payload:

```json
{
  "type": "message",
  "from": "alice_id",
  "to": "bob_id",
  "ciphertext": "BASE64(...)",
  "timestamp": 1736400000
}
```

---

#### 2️⃣ Flutter Client

Local-first design:

**Local storage**

* SQLite (via Drift) or Hive
* Stores:

  * Conversations
  * Messages
  * Delivery state

**Crypto**

* Generate keypair on first install
* Exchange public keys via server
* Encrypt every message client-side

---

#### 3️⃣ Offline + sync behavior

When user is offline:

* Messages are queued locally
* Sent when WebSocket reconnects

If recipient is offline:

* Server holds message **in memory or short-lived queue**
* Or returns “offline” → sender retries later

> This still counts as peer-to-peer because **the server is not a source of truth**

---
