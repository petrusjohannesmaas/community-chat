import asyncio
import websockets

async def echo_handler(websocket):
    print(f"Client connected: {websocket.remote_address}")
    
    try:
        async for message in websocket:
            print(f"""
            New message: {message}
            \nFrom: {websocket.remote_address}
            """)
            response = f"OK"
            await websocket.send(response)
    except websockets.exceptions.ConnectionClosedOK:
        print("Client disconnected gracefully")

async def main():
    async with websockets.serve(echo_handler, "localhost", 8765):
        print("Server started on ws://localhost:8765")
        await asyncio.Future()  # Keep the server running

if __name__ == "__main__":
    asyncio.run(main())