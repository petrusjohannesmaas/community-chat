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

Our **WebSocket Relay Server** is deliberately minimal, with responsibilities limited to:

- **Authenticate user**: Verify identity before allowing a connection.
- **Maintain online connections**: Keep track of active sessions and ensure reliable WebSocket channels.
- **Forward encrypted messages**: Act purely as a relay, passing ciphertext payloads between connected peers.

Critically, the server **never writes messages to disk** — it only holds them in memory briefly if needed for delivery, preserving the **peer‑to‑peer, local‑first design**.

```json
{
  "type": "message",
  "from": "alice_id",
  "to": "bob_id",
  "ciphertext": "BASE64(...)",
  "timestamp": 1736400000
}
```

This ensures the server only sees opaque ciphertext and routing metadata, never plaintext.

---

#### 2️⃣ Flutter Client

We’re designing a **local‑first, end‑to‑end encrypted chat client** where the device itself is the source of truth.

Core requirements of the client:

- **Local storage layer**: Using SQLite (via Drift) or Hive to persist conversations, messages, and delivery state directly on the device.
- **Cryptographic foundation**: Generating a keypair on first install, keeping the private key local, and exchanging public keys through the server.
- **Message security**: Every message is encrypted client‑side before leaving the device, ensuring the server never sees plaintext.
- **Delivery state tracking**: Stored locally so the client can reconcile sent, delivered, and read states without relying on server‑side truth.

This setup means your client is responsible for both **secure message handling** and **state management**, while the server acts mainly as a relay and key exchange facilitator.

---

#### 3️⃣ Offline + sync behavior

- **User offline**: Messages are queued locally in the client’s storage. Once the WebSocket reconnects, the client flushes the queue and sends them out.
- **Recipient offline**: The server holds the message in a expiring memory queue until the recipient reconnects.
- **Peer‑to‑peer integrity**: The server never becomes a permanent store of messages or states. It only facilitates delivery, keeping the architecture true to peer‑to‑peer principles.

This approach gives users resilience against disconnections while preserving our local‑first, end‑to‑end encrypted model.

---
