import asyncio
import websockets
import json
import sys

async def receive_messages(websocket, username):
    """Listen for incoming messages from the server"""
    try:
        async for message in websocket:
            data = json.loads(message)
            msg_type = data.get("type")
            
            if msg_type == "broadcast":
                sender = data.get("username")
                content = data.get("content")
                # Clear the current input line and print the message
                print(f"\r{sender}: {content}")
                print(f"{username}: ", end="", flush=True)
            
            elif msg_type == "message_received":
                # Silently acknowledge (we already showed it was sent)
                pass
                
    except websockets.exceptions.ConnectionClosed:
        print("\n\n✗ Connection to server lost")
    except Exception as e:
        print(f"\n✗ Error receiving messages: {e}")

async def send_messages(websocket, username):
    """Send messages typed by the user"""
    try:
        while True:
            # Get input from user (blocking, but in executor to not block async)
            loop = asyncio.get_event_loop()
            message = await loop.run_in_executor(None, input, f"{username}: ")
            
            # Exit condition
            if message.lower() in ['quit', 'exit', 'q']:
                print("Disconnecting...")
                return False
            
            # Skip empty messages
            if not message.strip():
                continue
            
            # Send message
            chat_msg = json.dumps({
                "type": "message",
                "content": message
            })
            await websocket.send(chat_msg)
            
    except Exception as e:
        print(f"\n✗ Error sending message: {e}")
        return False
    
    return True

async def interactive_client():
    uri = "ws://localhost:8765"
    
    try:
        async with websockets.connect(uri) as websocket:
            print("✓ Connected to server\n")
            
            # Get username from user
            username = input("Enter your username: ").strip()
            while not username:
                print("Username cannot be empty!")
                username = input("Enter your username: ").strip()
            
            # Send username to server
            username_msg = json.dumps({
                "type": "set_username",
                "username": username
            })
            await websocket.send(username_msg)
            
            # Wait for confirmation
            response = await websocket.recv()
            data = json.loads(response)
            
            if data.get("type") == "username_confirmed":
                print(f"✓ Username set to: {data.get('username')}")
                print("\nType your messages (or 'quit' to exit):\n")
            
            # Run receiver and sender concurrently
            receive_task = asyncio.create_task(receive_messages(websocket, username))
            send_task = asyncio.create_task(send_messages(websocket, username))
            
            # Wait for either task to complete (sender exits on 'quit')
            done, pending = await asyncio.wait(
                [receive_task, send_task],
                return_when=asyncio.FIRST_COMPLETED
            )
            
            # Cancel the other task
            for task in pending:
                task.cancel()
                
    except ConnectionRefusedError:
        print("❌ Error: Could not connect. Is server.py running?")
    except KeyboardInterrupt:
        print("\n\nDisconnected (Ctrl+C)")
    except Exception as e:
        print(f"❌ Error: {e}")

if __name__ == "__main__":
    asyncio.run(interactive_client())