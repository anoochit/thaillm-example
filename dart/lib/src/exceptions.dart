/// Base class for all ThaiLLM exceptions.
sealed class ThaiLLMException implements Exception {
  /// Human-readable description of the error.
  final String message;

  const ThaiLLMException(this.message);

  @override
  String toString() => '$runtimeType: $message';
}

/// Thrown when the API returns a non-2xx HTTP status code.
final class ThaiLLMApiException extends ThaiLLMException {
  /// The HTTP status code returned by the API.
  final int statusCode;

  /// The raw response body, if available.
  final String? responseBody;

  const ThaiLLMApiException({
    required this.statusCode,
    required String message,
    this.responseBody,
  }) : super(message);

  @override
  String toString() =>
      'ThaiLLMApiException($statusCode): $message'
      '${responseBody != null ? '\nBody: $responseBody' : ''}';
}

/// Thrown when the API returns a 401 Unauthorized response.
final class ThaiLLMAuthException extends ThaiLLMException {
  const ThaiLLMAuthException()
      : super('Invalid or missing API key. Check your ThaiLLM API key.');
}

/// Thrown when the API returns a 429 Too Many Requests response.
final class ThaiLLMRateLimitException extends ThaiLLMException {
  const ThaiLLMRateLimitException()
      : super('Rate limit exceeded. Please slow down your requests.');
}

/// Thrown when the JSON response from the API cannot be parsed.
final class ThaiLLMParseException extends ThaiLLMException {
  const ThaiLLMParseException(String detail)
      : super('Failed to parse API response: $detail');
}

/// Thrown when a network-level error occurs (e.g. no internet connection).
final class ThaiLLMNetworkException extends ThaiLLMException {
  const ThaiLLMNetworkException(String detail)
      : super('Network error: $detail');
}
