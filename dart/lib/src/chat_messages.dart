/// Represents a single message in a chat conversation.
class ChatMessage {
  /// The role of the message author.
  final ChatRole role;

  /// The text content of the message.
  final String content;

  const ChatMessage({required this.role, required this.content});

  /// Creates a user message.
  factory ChatMessage.user(String content) =>
      ChatMessage(role: ChatRole.user, content: content);

  /// Creates an assistant message.
  factory ChatMessage.assistant(String content) =>
      ChatMessage(role: ChatRole.assistant, content: content);

  /// Creates a system message.
  factory ChatMessage.system(String content) =>
      ChatMessage(role: ChatRole.system, content: content);

  /// Serializes the message to a JSON map for the API request body.
  Map<String, dynamic> toJson() => {
        'role': role.value,
        'content': content,
      };

  @override
  String toString() => 'ChatMessage(role: ${role.value}, content: $content)';
}

/// Roles that a chat message author can have.
enum ChatRole {
  /// A message from the end user.
  user('user'),

  /// A message from the AI assistant.
  assistant('assistant'),

  /// A system-level instruction message.
  system('system');

  /// The string value sent to the API.
  final String value;

  const ChatRole(this.value);
}