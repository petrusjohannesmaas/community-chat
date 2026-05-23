# Project TODO

## Phase 1 — Group Chat (current)
- [ ] Mosquitto broker running via Docker Compose
- [ ] Tauri project scaffolded (`cargo tauri init`)
- [ ] `rumqttc` MQTT client connecting to broker from Rust
- [ ] Subscribe to `community/general` on connect
- [ ] Publish JSON message to `community/general`
- [ ] Emit incoming messages from Rust to React via Tauri events
- [ ] React connection screen (username input, join button)
- [ ] React message list displaying incoming messages
- [ ] React message input (send on Enter or button click)
- [ ] Sender sees their own messages (received back from broker)

## Phase 2 — Direct Messages
- [ ] Member list in UI (manually configured for now)
- [ ] DM topic structure: `community/dm/{userA}/{userB}`
- [ ] Subscribe to own DM topic on connect
- [ ] Click member name to open DM conversation
- [ ] Separate message history per conversation in UI state

## Phase 3 — Polish
- [ ] Persist username locally (Tauri store or local file)
- [ ] Local message history (SQLite via `tauri-plugin-sql`)
- [ ] Online presence (MQTT retained messages / will messages)
- [ ] Typing indicators
- [ ] Unread message count per conversation
- [ ] Auto-scroll to latest message
- [ ] Timestamp formatting

## Phase 4 — Auth and Invite System
- [ ] Mosquitto password authentication (disable `allow_anonymous`)
- [ ] Invite link generation (simple FastAPI endpoint)
- [ ] Username registration tied to invite token
- [ ] Admin view: list members, revoke access

## Phase 5 — Encryption
- [ ] Keypair generation on first launch (stored locally)
- [ ] Public key published to server on registration
- [ ] Fetch recipient public key before sending DM
- [ ] Encrypt message payload before publish
- [ ] Decrypt on receive
- [ ] Key fingerprint display for manual verification
- [ ] Group chat encryption strategy (decide: shared key vs per-member)
