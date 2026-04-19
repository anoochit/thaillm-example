import 'package:thaillm/thaillm.dart';

Future<void> main() async {
  // ── 1. Create a client ────────────────────────────────────────────────────
  final client = ThaiLLMClient(
    apiKey: 'ABC1234567890XYZ', // replace with your actual API key
    defaultMaxTokens: 1024,
    defaultTemperature: 0.5,
  );

  try {
    // ── 2. Simple single-prompt call ─────────────────────────────────────────
    print('=== ask() example ===');
    final answer = await client.ask(
      model: ThaiLLMModel.typhoon,
      prompt: 'กรุงเทพมีชื่อเต็มว่าอะไร',
    );
    print('Answer: $answer\n');

    // ── 3. Multi-turn conversation ───────────────────────────────────────────
    print('=== chat() multi-turn example ===');
    final history = [
      ChatMessage.system('คุณเป็นผู้ช่วยที่ตอบคำถามด้วยภาษาไทยเสมอ'),
      ChatMessage.user('สวัสดี'),
    ];

    final firstReply = await client.chat(
      model: ThaiLLMModel.openThaiGpt,
      messages: history,
    );
    print('Assistant: ${firstReply.content}');
    print('Tokens used: ${firstReply.usage.totalTokens}\n');

    // Append the reply and continue the conversation
    history.add(ChatMessage.assistant(firstReply.content));
    history.add(ChatMessage.user('ช่วยแนะนำอาหารไทยยอดนิยม 3 อย่างหน่อย'));

    final secondReply = await client.chat(
      model: ThaiLLMModel.openThaiGpt,
      messages: history,
      temperature: 0.7, // more creative for this turn
    );
    print('Assistant: ${secondReply.content}');

    // ── 4. Try each model ────────────────────────────────────────────────────
    print('\n=== Trying all models ===');
    for (final model in ThaiLLMModel.values) {
      try {
        final reply = await client.ask(
          model: model,
          prompt: 'บอกชื่อของคุณ',
          maxTokens: 64,
        );
        print('${model.slug}: $reply');
      } on ThaiLLMApiException catch (e) {
        print('${model.slug} error: $e');
      }
    }
  } on ThaiLLMAuthException catch (e) {
    print('Auth error: $e');
  } on ThaiLLMRateLimitException catch (e) {
    print('Rate limit: $e');
  } on ThaiLLMNetworkException catch (e) {
    print('Network error: $e');
  } finally {
    client.close();
  }
}
