/// ThaiLLM — Dart client for the ThaiLLM API.
///
/// Supports multiple Thai-language models:
/// - OpenThaiGPT
/// - Pathumma
/// - Typhoon
/// - KBTG
///
/// Example usage:
/// ```dart
/// import 'package:thaillm/thaillm.dart';
///
/// final client = ThaiLLMClient(apiKey: 'YOUR_API_KEY');
///
/// final response = await client.chat(
///   model: ThaiLLMModel.typhoon,
///   messages: [ChatMessage.user('สวัสดี')],
/// );
///
/// print(response.content);
/// ```
library thaillm;

export 'src/client.dart';
export 'src/models.dart';
export 'src/chat_message.dart';
export 'src/chat_response.dart';
export 'src/exceptions.dart';
