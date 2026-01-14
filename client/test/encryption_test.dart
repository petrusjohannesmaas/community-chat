import 'dart:convert';
import 'dart:typed_data';
import 'package:pointycastle/export.dart';
import 'package:test/test.dart';

Uint8List deriveKey(String passphrase, String salt, {int length = 32}) {
  final pbkdf2 = KeyDerivator('SHA-256/HMAC/PBKDF2')
    ..init(
      Pbkdf2Parameters(Uint8List.fromList(utf8.encode(salt)), 10000, length),
    ); // 10k iterations
  return pbkdf2.process(Uint8List.fromList(utf8.encode(passphrase)));
}

AsymmetricKeyPair<PublicKey, PrivateKey> generateKeyPair(Uint8List seed) {
  final keyGen = KeyGenerator('RSA')
    ..init(
      ParametersWithRandom(
        RSAKeyGeneratorParameters(BigInt.parse('65537'), 2048, 64),
        SecureRandom('AES/CTR/AUTO-SEED-PRNG')..seed(KeyParameter(seed)),
      ),
    );
  return keyGen.generateKeyPair();
}

void main() {
  group('Encryption tests with mock data', () {
    test('Key derivation prints mock output', () {
      final seed = deriveKey('mockPassphrase', 'mockSalt');
      print(
        'Derived seed (hex): ${seed.map((b) => b.toRadixString(16).padLeft(2, '0')).join()}',
      );
      expect(seed.length, equals(32));
    });

    test('Keypair generation prints mock RSA keys', () {
      final seed = deriveKey('anotherPass', 'userSalt');
      final keyPair = generateKeyPair(seed);

      final pubKey = keyPair.publicKey as RSAPublicKey;
      final privKey = keyPair.privateKey as RSAPrivateKey;

      print(
        'Public Key modulus (first 50 chars): ${pubKey.modulus.toString().substring(0, 50)}...',
      );
      print('Public Key exponent: ${pubKey.exponent}');
      print(
        'Private Key modulus (first 50 chars): ${privKey.modulus.toString().substring(0, 50)}...',
      );
      print('Private Key exponent: ${privKey.exponent}');

      expect(pubKey.exponent, equals(BigInt.from(65537)));
      expect(privKey.modulus, equals(pubKey.modulus));
    });
  });
}
