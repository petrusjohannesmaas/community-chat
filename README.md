# e2ee-chat
A secure chat application

## Phases:
1. Begin without encryption with client + server architecture
2. Add encryption
3. Move to peer to peer
4. Configure expiring local storage per user
5. Cache / sync

## Phase 1:

**Client:**
- Docker container
- Connects to Websocket server
- Serves Web based UI with a view for logging in
- User can create an account if not exist
- User logs in, gets a token and has access to chat stream
- Chat stream is displayed as horizontal cards

**Server**
- Python websocket server
- Authentication service
