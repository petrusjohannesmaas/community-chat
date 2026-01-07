import asyncio
import websockets

async def interactive_client():
    uri = "ws://localhost:8765"
    
    try:
        async with websockets.connect(uri) as websocket:
            print("✓ Connected to server")
            print("Type your messages (or 'quit' to exit):\n")
            
            while True:
                # Get input from user
                message = input("You: ")
                
                # Exit condition
                if message.lower() in ['quit', 'exit', 'q']:
                    print("Disconnecting...")
                    break
                
                # Skip empty messages
                if not message.strip():
                    continue
                
                # Send message
                await websocket.send(message)
                
                # Wait for response
                response = await websocket.recv()
                print(f"Server: {response}\n")
                
    except ConnectionRefusedError:
        print("❌ Error: Could not connect. Is server.py running?")
    except KeyboardInterrupt:
        print("\n\nDisconnected (Ctrl+C)")

if __name__ == "__main__":
    asyncio.run(interactive_client())