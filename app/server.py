import asyncio
import websockets

async def echo_handler(websocket):
    print(f"\nClient connected: {websocket.remote_address}")
    
    try:
        async for message in websocket:
            print(f"\n➡️ {websocket.remote_address}: {message}")
            response = "OK"
            await websocket.send(response)
    except websockets.exceptions.ConnectionClosedOK:
        print("Client disconnected gracefully")

async def main():
    async with websockets.serve(echo_handler, "localhost", 8765):
        print("Server started on ws://localhost:8765")
        await asyncio.Future()  # Keep the server running

if __name__ == "__main__":
    asyncio.run(main())