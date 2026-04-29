# Project TODO: Python Reboot

## Phase 1: Infrastructure
- [ ] Migrate `backend/server.py` from raw `websockets` to `FastAPI`.
- [ ] Update `requirements.txt` with `fastapi`, `uvicorn`, and `python-multipart`.
- [ ] Implement a basic `common/schemas.py` using Pydantic for message validation.

## Phase 2: GTK Client (Proof of Concept)
- [ ] Set up a basic GTK 4 window using `PyGObject`.
- [ ] Implement the "Connection Screen" (Server URL, Username).
- [ ] Integrate `asyncio` with the GTK main loop (using `GLib.idle_add` or a dedicated library like `gbulb`).
- [ ] Implement the message display list and message entry field.

## Phase 3: Encryption & Security
- [ ] Add `cryptography` to dependencies.
- [ ] Port RSA key generation logic from legacy research.
- [ ] Port PBKDF2 key derivation logic.
- [ ] Implement "First-Contact" key exchange (Public Key broadcasting).

## Phase 4: Polish
- [ ] Add persistence (SQLite) for local chat history.
- [ ] Add "typing..." indicators.
- [ ] Implement a basic "Settings" menu for GTK UI.
