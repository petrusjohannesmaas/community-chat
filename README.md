# e2ee-chat
A secure chat application

## Roadmap:
1. Begin without encryption with client + server architecture
1. Add encryption
2. Move to peer to peer
3. Configure expiring local storage per user
4. Cache / sync

## Architecture

### 📱 Client
**Client flow:** 
User creates keys → User connects via passphrase → Other client connections metadata is displayed → User can initiate a chat → User can disconnect 

**Functions:**
- Generate keys & passphrase if not exist
- Connects to Websocket server
- Serves Web UI


### 🗼 Server
**Server flow:**
Server listens for incoming connections → Registers client session → Relays messages to clients → Handles heartbeat/cleanup on disconnect

**Functions:**
- Websocket server
- Message routing
- Connection management
