import 'package:test/test.dart';
import 'package:client/socket_client.dart'; // Import using project name

void main() {
  group('WebSocket Integration Tests', () {
    late SocketClient client;

    setUp(() {
      // Initialize before each test
      client = SocketClient('ws://localhost:8765');
    });

    tearDown(() {
      // Clean up after each test
      client.close();
    });

    test('Server should echo sent message', () async {
      client.connect();
      
      const testMessage = "Hello from Dart!";
      client.send(testMessage);

      // Listen for the first response from the server
      final response = await client.messages.first;

      print('Received from server: $response');
      expect(response, isNotNull);
      // If your server echoes, you can use: expect(response, equals(testMessage));
    });
  });
}