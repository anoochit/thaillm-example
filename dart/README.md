# thaillm

A Dart client package for [ThaiLLM](http://thaillm.or.th) — a Thai-language LLM service provider with multiple models.

## Supported models

| Model | Slug | Notes |
|---|---|---|
| OpenThaiGPT | `openthaigpt` | General-purpose Thai LLM |
| Pathumma | `pathumma` | By NECTEC |
| Typhoon | `typhoon` | By SCB 10X |
| KBTG | `kbtg` | By Kasikorn Business Technology Group |

## Installation

Add to your `pubspec.yaml`:

```yaml
dependencies:
  thaillm: ^0.1.0
```

Then run:

```sh
dart pub get
```

## Quick start

```dart
import 'package:thaillm/thaillm.dart';

void main() async {
  final client = ThaiLLMClient(apiKey: 'YOUR_API_KEY');

  // Simple one-shot question
  final answer = await client.ask(
    model: ThaiLLMModel.typhoon,
    prompt: 'กรุงเทพมีชื่อเต็มว่าอะไร',
  );
  print(answer);

  client.close();
}
```

## Multi-turn conversation

```dart
final history = <ChatMessage>[
  ChatMessage.system('ตอบเป็นภาษาไทยเสมอ'),
  ChatMessage.user('สวัสดี'),
];

final reply = await client.chat(
  model: ThaiLLMModel.openThaiGpt,
  messages: history,
);
print(reply.content);
print('Tokens used: ${reply.usage.totalTokens}');

// Continue the conversation
history
  ..add(ChatMessage.assistant(reply.content))
  ..add(ChatMessage.user('แนะนำอาหารไทยหน่อย'));

final reply2 = await client.chat(
  model: ThaiLLMModel.openThaiGpt,
  messages: history,
  temperature: 0.7,
);
print(reply2.content);
```

## API reference

### `ThaiLLMClient`

| Parameter | Type | Default | Description |
|---|---|---|---|
| `apiKey` | `String` | required | Your ThaiLLM API key |
| `defaultMaxTokens` | `int` | `2048` | Max tokens for all requests |
| `defaultTemperature` | `double` | `0.3` | Temperature for all requests |
| `httpClient` | `http.Client?` | auto | Inject a custom HTTP client |

### `chat()`

Sends a full conversation and returns a `ChatResponse`.

```dart
final response = await client.chat(
  model: ThaiLLMModel.typhoon,
  messages: [...],
  maxTokens: 512,   // optional override
  temperature: 0.8, // optional override
);
```

### `ask()`

Convenience wrapper for single-turn prompts; returns the reply string directly.

```dart
final text = await client.ask(
  model: ThaiLLMModel.kbtg,
  prompt: 'อธิบาย blockchain',
  systemPrompt: 'อธิบายให้เข้าใจง่าย', // optional
);
```

## Error handling

```dart
try {
  final reply = await client.ask(...);
} on ThaiLLMAuthException catch (e) {
  // Invalid API key
} on ThaiLLMRateLimitException catch (e) {
  // Too many requests
} on ThaiLLMApiException catch (e) {
  print('HTTP ${e.statusCode}: ${e.message}');
} on ThaiLLMNetworkException catch (e) {
  // Connectivity issues
} on ThaiLLMParseException catch (e) {
  // Unexpected response format
}
```

## Running tests

```sh
dart test
```

## License

MIT
