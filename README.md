# 🐍 PyRelay Chat

A local-first, end-to-end encrypted (E2EE) messaging system built entirely in Python. The architecture follows a **peer-to-peer trust model with a minimal server relay**, ensuring the server never sees plaintext messages or stores conversation history.

## 🏗 Architecture Overview

This project has been rebooted with a unified Python stack, prioritizing simplicity, security, and native Linux integration.

- **Server (FastAPI):** Acts as a blind WebSocket relay. Handles authentication, connection management, and routes encrypted ciphertext between peers. Messages are held only in-memory for offline delivery and never written to disk.
- **Client (GTK 4):** A native desktop interface built with `PyGObject`. Fully local-first, handling key generation, message encryption/decryption, offline queueing, and persistent storage via SQLite.
- **Security:** End-to-end encryption using RSA-2048 for key exchange and PBKDF2 for payload encryption. The server only sees routing metadata and opaque ciphertext.

## 🛠 Tech Stack

| Component | Technology |
|---|---|
| **Backend Server** | FastAPI, Starlette WebSockets, `pydantic` |
| **Desktop Client** | GTK 4 (`PyGObject`), `asyncio` |
| **Cryptography** | `cryptography` library (RSA-2048 + PBKDF2) |
| **Local Storage** | SQLite (`aiosqlite`) |
| **Shared Models** | Pydantic (`common/` module) |

## ✨ Key Features

- 🔒 **True E2EE:** Messages are encrypted client-side before transmission. The server acts purely as a router.
- 📦 **Local-First Design:** All conversations, keys, and delivery states are stored on the device. You own your data.
- 🔄 **Offline Sync & Queuing:** Messages sent while offline are queued locally and flushed upon reconnection. The server temporarily buffers messages for offline recipients in-memory.
- 🖥️ **Native Linux UI:** Lightweight, fast, and integrated desktop experience using GTK 4.
- 🐍 **Unified Python Codebase:** Eliminates cross-language friction, making development, testing, and maintenance straightforward.

## 🚀 Getting Started

### Prerequisites
- Python 3.10+
- `libgtk-4-dev` and `gir1.2-gtk-4.0` (for GTK client)
- `pip` & `venv`

### 1. Setup Environment
```bash
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

### 2. Run the Server
```bash
uvicorn backend.server:app --host 0.0.0.0 --port 8000 --reload
```

### 3. Launch the Client
```bash
python client/client_gtk.py
```
*Default connection:* `ws://localhost:8000/ws`

## 📁 Project Structure (Proposed)
```
├── backend/
│   ├── server.py          # FastAPI WebSocket relay & routing
│   └── models.py          # Pydantic message schemas
├── client/
│   ├── client_gtk.py      # GTK 4 UI & async WebSocket loop
│   └── storage.py         # SQLite local storage & offline queue
├── common/                # Shared crypto utils & message types
├── requirements.txt
└── README.md
```

## 🗺 Roadmap
- [x] Strip legacy Dart/Flutter frontend
- [ ] Migrate server to FastAPI with WebSocket support
- [ ] Implement GTK 4 client skeleton with async event loop integration
- [ ] Port RSA/PBKDF2 encryption logic using `cryptography`
- [ ] Add local SQLite persistence & offline message queuing
- [ ] Implement authentication & secure key exchange flow

## 📜 License
[MIT](LICENSE)
