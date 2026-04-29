# Project Status Report - Reboot Phase

## Current State Assessment
The project has been stripped of its Dart/Flutter frontend and is now a pure Python codebase.

### Backend (Python)
- **Technology:** Python with `websockets` library.
- **Components:**
  - `server.py`: A lightweight WebSocket server that manages user sessions (usernames) and broadcasts messages to all connected clients.
  - `requirements.txt`: Currently only lists `websockets`.
- **Capabilities:**
  - Connection management.
  - Username registration.
  - Global message broadcasting.
- **Missing Features:**
  - Authentication.
  - End-to-end encryption (previously prototyped in Dart but not yet implemented in Python).
  - Persistence (database for history).

### Frontend / Client
- **Current State:** The Dart/Flutter code has been removed.
- **Existing Logic:** A functional CLI client exists in `backend/tests/cli_test.py` which provides a dual-task (async) environment for sending and receiving messages.

## Actions Taken
1. **Repository Audit:** Identified key logic in `server.py` and researched legacy encryption tests in the now-deleted `client/` directory.
2. **Frontend Stripping:** Hard deleted the `client/` directory and all Dart/Flutter dependencies.
3. **Documentation:** Created this status report and a POC suggestion for the next phase.

## Technical Debt & Observations
- The server currently uses `websockets.serve`, which is fine for prototypes but less extensible than a full framework like FastAPI.
- Encryption logic needs to be ported from the research findings (RSA-2048 and PBKDF2) to a Python-native library like `cryptography`.
