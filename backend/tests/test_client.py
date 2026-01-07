import asyncio
import websockets

async def test_client():
    uri = "ws://localhost:8765"
    
    try:
        async with websockets.connect(uri) as websocket:
            message = "Is this thing on?"
            
            print(f"Sending: {message}")
            await websocket.send(message)
            
            response = await websocket.recv()
            print(f"{response}")
            
    except ConnectionRefusedError:
        print("Error: Could not connect. Is server.py running?")

if __name__ == "__main__":
    asyncio.run(test_client())