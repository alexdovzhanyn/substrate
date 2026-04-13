use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SingleQueryParams {
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
pub struct BatchQueryParams {
  pub queries: Vec<SingleQueryParams>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecordParams {
  /// A complete, self-contained belief statement.
  ///
  /// Requirements:
  /// - Must be written as a full natural-language statement
  /// - Must make sense in isolation (no "this", "here", "current", etc.)
  /// - Must be directly usable when retrieved
  /// - Must represent exactly ONE piece of information
  ///
  /// Good:
  /// - "Run pnpm dev from the root directory to start the project locally."
  ///
  /// Bad:
  /// - "start project"
  /// - "how this repo starts"
  pub content: String,

  /// 3-6 distinct query variations that may be used for retrieval.
  ///
  /// This is the MOST IMPORTANT field.
  ///
  /// Requirements:
  /// - Include different verbs (start, run, launch, initialize)
  /// - Include different nouns (app, project, server, service)
  /// - Include different contexts (local, dev, development)
  /// - Include both full questions and short keyword-style queries
  /// - Do NOT rely on semantic similarity — explicitly include variations
  ///
  /// Missing a phrasing can make the belief unretrievable.
  ///
  /// Examples:
  /// - "how do i start the project"
  /// - "run app locally"
  /// - "start dev server"
  /// - "launch application"
  pub possible_queries: Vec<String>,

  /// 1–5 short categorical tags describing the belief.
  ///
  /// Tags are secondary and are NOT the primary retrieval mechanism.
  ///
  /// Use tags to capture:
  /// - technology (e.g., node, postgres, docker)
  /// - domain (e.g., frontend, backend, testing)
  /// - purpose (e.g., setup, deployment, logging)
  ///
  /// Examples:
  /// - ["node", "startup", "local-dev"]
  /// - ["database", "postgres", "prisma"]
  pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IngestMarkdownPromptParams {
  /// The markdown file to ingest
  pub markdown_file_path: String,
}
