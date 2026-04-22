/// Supported ThaiLLM model providers.
enum ThaiLLMModel {
  /// OpenThaiGPT model — general-purpose Thai language model.
  openThaiGpt('openthaigpt'),

  /// Pathumma model — developed by NECTEC.
  pathumma('pathumma'),

  /// Typhoon model — by SCB 10X.
  typhoon('typhoon'),

  /// KBTG model — by Kasikorn Business Technology Group.
  kbtg('kbtg');

  /// The URL path segment used to identify the model.
  final String slug;

  const ThaiLLMModel(this.slug);

  /// Returns the ThaiLLMModel that matches [slug], or null if not found.
  static ThaiLLMModel? fromSlug(String slug) {
    for (final model in ThaiLLMModel.values) {
      if (model.slug == slug) return model;
    }
    return null;
  }
}