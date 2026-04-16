use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SingleQuery {
  /// One or more natural language queries used to retrieve a single belief.
  ///
  /// This should describe exactly one thing you want to know.
  ///
  /// Examples:
  /// - "how do i start the project"
  /// - "run app locally"
  /// - "start dev server"
  pub query: String,

  /// Alternate phrasings of `query`.
  ///
  /// Every entry must ask for the same exact fact or answer as `query`.
  /// These are for recall improvement only.
  ///
  /// Do NOT include:
  /// - broader topic exploration
  /// - neighboring questions
  /// - related subproblems
  /// - general background questions
  ///
  /// If a phrasing would return a different correct answer than `query`,
  /// it must not be included here.
  pub paraphrases: Vec<String>,

  /// The maximum number of beliefs to return.
  ///
  /// Use a small value for narrow questions with one likely answer.
  /// Use a larger value only when the question is broader and may require multiple beliefs.
  ///
  /// Examples:
  /// - "how do i run c-fe-ai" -> 1 or 2
  /// - "c-fe-ai frontend rules" -> 3 to 6
  ///
  /// Do NOT increase this just because you are uncertain. Increase it only when the expected answer
  /// is genuinely distributed across multiple beliefs.
  pub max_result_count: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BatchQuery {
  pub queries: Vec<SingleQuery>,
}
