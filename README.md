# e2ee-chat
A secure chat application

## Roadmap:
1. Begin without encryption with client + server architecture
2. Add encryption
3. Move to peer to peer
4. Configure expiring local storage per user
5. Cache / sync

## Architecture

### 📱 Client
**Client flow:** 
User can create an account if not exist → User logs in → Gets a token → Chat stream is displayed → User can disconnect 

**Functions:**
- Connects to Websocket server
- Serves Web UI
- Displays messages
- Sends messages

### 🗼 Server
**Server flow:**
Server listens for incoming connections → Validates Authentication token → Registers client session → Forwards messages to clients → Handles heartbeat/cleanup on disconnect

**Functions:**
- Websocket server
- Broadcast messages
- Authentication
