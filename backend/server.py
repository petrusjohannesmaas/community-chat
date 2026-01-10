import asyncio
import websockets
import json
from datetime import datetime

# Store connected clients with their usernames
clients = {}

async def broadcast_message(sender_ws, username, content):
    """Broadcast message to all connected clients except sender"""
    if clients:
        message = json.dumps({
            "type": "broadcast",
            "username": username,
            "content": content
        })
        
        # Send to all clients except the sender
        disconnected = []
        for client_ws in clients:
            if client_ws != sender_ws:
                try:
                    await client_ws.send(message)
                except websockets.exceptions.ConnectionClosed:
                    disconnected.append(client_ws)
        
        # Clean up disconnected clients
        for ws in disconnected:
            if ws in clients:
                del clients[ws]

async def handle_client(websocket):
    print(f"\nClient connected: {websocket.remote_address}")
    username = None
    
    try:
        async for message in websocket:
            try:
                data = json.loads(message)
                msg_type = data.get("type")
                
                if msg_type == "set_username":
                    username = data.get("username")
                    clients[websocket] = username
                    print(f"✓ Username set: {username}")
                    
                    response = json.dumps({
                        "type": "username_confirmed",
                        "username": username
                    })
                    await websocket.send(response)
                
                elif msg_type == "message":
                    sender = clients.get(websocket, "Unknown")
                    content = data.get("content")
                    timestamp = datetime.now().strftime("%H:%M:%S")
                    print(f"\n[{timestamp}] from {sender}: {content}")
                    
                    # Broadcast to other users
                    await broadcast_message(websocket, sender, content)
                    
                    # Confirm to sender
                    response = json.dumps({
                        "type": "message_received",
                        "status": "OK"
                    })
                    await websocket.send(response)
                
                else:
                    print(f"Unknown message type: {msg_type}")
                    
            except json.JSONDecodeError:
                print(f"Invalid JSON received: {message}")
                
    except websockets.exceptions.ConnectionClosedOK:
        print(f"Client disconnected gracefully: {username or 'Unknown'}")
    except websockets.exceptions.ConnectionClosedError as e:
        print(f"Connection closed with error: {username or 'Unknown'} - {e}")
    except Exception as e:
        print(f"Unexpected error: {e}")
    finally:
        # Clean up on disconnect
        if websocket in clients:
            username = clients[websocket]
            del clients[websocket]
            print(f"Removed user: {username}")

async def main():
    async with websockets.serve(
        handle_client, 
        "localhost", 
        8765,
        ping_interval=None,  # Disable ping/pong timeout
        ping_timeout=None
    ):
        print("Server started on ws://localhost:8765")
        print("Waiting for connections...\n")
        await asyncio.Future()

if __name__ == "__main__":
    asyncio.run(main())