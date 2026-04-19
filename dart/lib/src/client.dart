import 'dart:convert';

import 'package:http/http.dart' as http;

import 'chat_message.dart';
import 'chat_response.dart';
import 'exceptions.dart';
import 'models.dart';

/// HTTP client for the ThaiLLM API.
///
/// Construct once and reuse across your application for efficient
/// connection pooling.
///
/// ```dart
/// final client = ThaiLLMClient(apiKey: 'YOUR_API_KEY');
///
/// final response = await client.chat(
///   model: ThaiLLMModel.typhoon,
///   messages: [ChatMessage.user('อธิบาย AI ให้หน่อย')],
/// );
///
/// print(response.content);
/// client.close();
/// ```
class ThaiLLMClient {
  static const String _baseUrl = 'http://thaillm.or.th/api';
  static const String _modelPath = '/model';

  /// Your ThaiLLM API key.
  final String apiKey;

  /// Base URL for the ThaiLLM API. Override for testing or proxies.
  final String baseUrl;

  /// Default maximum tokens to generate. Can be overridden per request.
  final int defaultMaxTokens;

  /// Default sampling temperature. Can be overridden per request.
  final double defaultTemperature;

  final http.Client _http;

  /// Creates a [ThaiLLMClient].
  ///
  /// [apiKey] is required. Optionally pass a custom [httpClient] for
  /// testing or advanced configuration (e.g. proxies, timeouts).
  ThaiLLMClient({
    required this.apiKey,
    this.baseUrl = _baseUrl,
    this.defaultMaxTokens = 2048,
    this.defaultTemperature = 0.3,
    http.Client? httpClient,
  }) : _http = httpClient ?? http.Client();

  /// Sends a chat completion request to the ThaiLLM API.
  ///
  /// - [model] — which Thai LLM to use (required).
  /// - [messages] — the conversation history (required, at least one message).
  /// - [maxTokens] — overrides [defaultMaxTokens] for this request.
  /// - [temperature] — overrides [defaultTemperature] for this request.
  ///
  /// Throws a [ThaiLLMAuthException] for 401 responses,
  /// [ThaiLLMRateLimitException] for 429, [ThaiLLMApiException] for other
  /// non-2xx codes, [ThaiLLMParseException] if the body cannot be decoded,
  /// and [ThaiLLMNetworkException] for connectivity issues.
  Future<ChatResponse> chat({
    required ThaiLLMModel model,
    required List<ChatMessage> messages,
    int? maxTokens,
    double? temperature,
  }) async {
    assert(messages.isNotEmpty, 'messages must not be empty');

    final uri = Uri.parse('$baseUrl/${model.slug}/v1/chat/completions');

    final body = jsonEncode({
      'model': _modelPath,
      'messages': messages.map((m) => m.toJson()).toList(),
      'max_tokens': maxTokens ?? defaultMaxTokens,
      'temperature': temperature ?? defaultTemperature,
    });

    try {
      final response = await _http.post(
        uri,
        headers: {
          'Content-Type': 'application/json',
          'apikey': apiKey,
        },
        body: body,
      );

      return _handleResponse(response);
    } on ThaiLLMException {
      rethrow;
    } catch (e) {
      throw ThaiLLMNetworkException(e.toString());
    }
  }

  /// Convenience method to send a single user message without managing
  /// a full conversation list.
  ///
  /// ```dart
  /// final reply = await client.ask(
  ///   model: ThaiLLMModel.openThaiGpt,
  ///   prompt: 'กรุงเทพมีชื่อเต็มว่าอะไร',
  /// );
  /// ```
  Future<String> ask({
    required ThaiLLMModel model,
    required String prompt,
    String? systemPrompt,
    int? maxTokens,
    double? temperature,
  }) async {
    final messages = [
      if (systemPrompt != null) ChatMessage.system(systemPrompt),
      ChatMessage.user(prompt),
    ];

    final response = await chat(
      model: model,
      messages: messages,
      maxTokens: maxTokens,
      temperature: temperature,
    );

    return response.content;
  }

  /// Releases the underlying HTTP client resources.
  ///
  /// Call this when the [ThaiLLMClient] is no longer needed.
  void close() => _http.close();

  // ---------------------------------------------------------------------------
  // Private helpers
  // ---------------------------------------------------------------------------

  ChatResponse _handleResponse(http.Response response) {
    switch (response.statusCode) {
      case 200:
      case 201:
        return _parseBody(response.body);
      case 401:
        throw const ThaiLLMAuthException();
      case 429:
        throw const ThaiLLMRateLimitException();
      default:
        throw ThaiLLMApiException(
          statusCode: response.statusCode,
          message: 'Unexpected status code ${response.statusCode}',
          responseBody: response.body,
        );
    }
  }

  ChatResponse _parseBody(String body) {
    try {
      final json = jsonDecode(body) as Map<String, dynamic>;
      return ChatResponse.fromJson(json);
    } catch (e) {
      throw ThaiLLMParseException(e.toString());
    }
  }
}
