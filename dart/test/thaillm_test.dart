import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:test/test.dart';
import 'package:thaillm/thaillm.dart';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const _fakeApiKey = 'test-key-123';

/// Builds a minimal valid API JSON response string.
String _buildResponse({
  String content = 'สวัสดีครับ',
  String finishReason = 'stop',
}) =>
    jsonEncode({
      'id': 'chatcmpl-test',
      'object': 'chat.completion',
      'created': 1776590343,
      'model': '/model',
      'choices': [
        {
          'index': 0,
          'message': {'role': 'assistant', 'content': content},
          'finish_reason': finishReason,
        }
      ],
      'usage': {
        'prompt_tokens': 10,
        'completion_tokens': 20,
        'total_tokens': 30,
      },
    });

ThaiLLMClient _clientWithMock(MockClientHandler handler) => ThaiLLMClient(
      apiKey: _fakeApiKey,
      httpClient: MockClient(handler),
    );

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

void main() {
  group('ThaiLLMModel', () {
    test('slugs are correct', () {
      expect(ThaiLLMModel.openThaiGpt.slug, 'openthaigpt');
      expect(ThaiLLMModel.pathumma.slug, 'pathumma');
      expect(ThaiLLMModel.typhoon.slug, 'typhoon');
      expect(ThaiLLMModel.kbtg.slug, 'kbtg');
    });

    test('fromSlug returns correct model', () {
      expect(ThaiLLMModel.fromSlug('typhoon'), ThaiLLMModel.typhoon);
      expect(ThaiLLMModel.fromSlug('unknown'), isNull);
    });
  });

  group('ChatMessage', () {
    test('factory constructors set correct roles', () {
      expect(ChatMessage.user('hi').role, ChatRole.user);
      expect(ChatMessage.assistant('hi').role, ChatRole.assistant);
      expect(ChatMessage.system('hi').role, ChatRole.system);
    });

    test('toJson serialises correctly', () {
      final msg = ChatMessage.user('สวัสดี');
      expect(msg.toJson(), {'role': 'user', 'content': 'สวัสดี'});
    });
  });

  group('ChatResponse', () {
    test('parses valid JSON correctly', () {
      final json =
          jsonDecode(_buildResponse(content: 'ดีครับ')) as Map<String, dynamic>;
      final response = ChatResponse.fromJson(json);

      expect(response.id, 'chatcmpl-test');
      expect(response.content, 'ดีครับ');
      expect(response.finishReason, 'stop');
      expect(response.usage.totalTokens, 30);
      expect(response.usage.promptTokens, 10);
      expect(response.usage.completionTokens, 20);
      expect(response.created,
          DateTime.fromMillisecondsSinceEpoch(1776590343 * 1000));
    });

    test('content returns empty string when choices list is empty', () {
      final json = jsonDecode(jsonEncode({
        'id': 'x',
        'created': 0,
        'model': '/model',
        'choices': [],
        'usage': {'prompt_tokens': 0, 'completion_tokens': 0, 'total_tokens': 0},
      })) as Map<String, dynamic>;

      expect(ChatResponse.fromJson(json).content, '');
    });
  });

  group('ThaiLLMClient.chat()', () {
    test('sends correct URL for each model', () async {
      for (final model in ThaiLLMModel.values) {
        Uri? capturedUri;
        final client = _clientWithMock((request) async {
          capturedUri = request.url;
          return http.Response(_buildResponse(), 200);
        });

        await client.chat(
          model: model,
          messages: [ChatMessage.user('test')],
        );
        client.close();

        expect(
          capturedUri.toString(),
          'http://thaillm.or.th/api/${model.slug}/v1/chat/completions',
        );
      }
    });

    test('sends apikey header', () async {
      String? capturedKey;
      final client = _clientWithMock((request) async {
        capturedKey = request.headers['apikey'];
        return http.Response(_buildResponse(), 200);
      });

      await client.chat(
        model: ThaiLLMModel.typhoon,
        messages: [ChatMessage.user('hi')],
      );
      client.close();

      expect(capturedKey, _fakeApiKey);
    });

    test('returns ChatResponse on 200', () async {
      final client = _clientWithMock(
          (_) async => http.Response(_buildResponse(content: 'ตอบแล้ว'), 200));

      final response = await client.chat(
        model: ThaiLLMModel.typhoon,
        messages: [ChatMessage.user('สวัสดี')],
      );
      client.close();

      expect(response.content, 'ตอบแล้ว');
    });

    test('throws ThaiLLMAuthException on 401', () async {
      final client =
          _clientWithMock((_) async => http.Response('Unauthorized', 401));

      expect(
        () => client.chat(
            model: ThaiLLMModel.typhoon, messages: [ChatMessage.user('hi')]),
        throwsA(isA<ThaiLLMAuthException>()),
      );
      client.close();
    });

    test('throws ThaiLLMRateLimitException on 429', () async {
      final client =
          _clientWithMock((_) async => http.Response('Too Many Requests', 429));

      expect(
        () => client.chat(
            model: ThaiLLMModel.typhoon, messages: [ChatMessage.user('hi')]),
        throwsA(isA<ThaiLLMRateLimitException>()),
      );
      client.close();
    });

    test('throws ThaiLLMApiException on 500', () async {
      final client = _clientWithMock(
          (_) async => http.Response('Internal Server Error', 500));

      expect(
        () => client.chat(
            model: ThaiLLMModel.typhoon, messages: [ChatMessage.user('hi')]),
        throwsA(isA<ThaiLLMApiException>()),
      );
      client.close();
    });

    test('throws ThaiLLMParseException on malformed JSON', () async {
      final client = _clientWithMock(
          (_) async => http.Response('not valid json {{', 200));

      expect(
        () => client.chat(
            model: ThaiLLMModel.typhoon, messages: [ChatMessage.user('hi')]),
        throwsA(isA<ThaiLLMParseException>()),
      );
      client.close();
    });
  });

  group('ThaiLLMClient.ask()', () {
    test('returns content string directly', () async {
      final client = _clientWithMock((_) async =>
          http.Response(_buildResponse(content: 'คำตอบ'), 200));

      final result =
          await client.ask(model: ThaiLLMModel.kbtg, prompt: 'คำถาม');
      client.close();

      expect(result, 'คำตอบ');
    });

    test('includes system message when provided', () async {
      List<dynamic>? sentMessages;
      final client = _clientWithMock((request) async {
        final body = jsonDecode(request.body) as Map<String, dynamic>;
        sentMessages = body['messages'] as List<dynamic>;
        return http.Response(_buildResponse(), 200);
      });

      await client.ask(
        model: ThaiLLMModel.pathumma,
        prompt: 'hi',
        systemPrompt: 'You are helpful.',
      );
      client.close();

      expect(sentMessages?.first['role'], 'system');
      expect(sentMessages?.last['role'], 'user');
    });
  });
}
