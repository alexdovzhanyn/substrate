use crate::beliefs::belief::Belief;
use crate::debug;
use rmcp::{
  ErrorData as McpError, RoleServer, ServerHandler,
  handler::server::{
    router::{prompt::PromptRouter, tool::ToolRouter},
    wrapper::Parameters,
  },
  model::{
    CallToolResult, Content, GetPromptRequestParams, GetPromptResult, ListPromptsResult,
    PaginatedRequestParams, PromptMessage, PromptMessageRole, ServerCapabilities, ServerInfo,
  },
  prompt, prompt_handler, prompt_router,
  service::RequestContext,
  tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryParams {
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

#[derive(Clone)]
pub struct SubstrateService {
  state: AppState,
  tool_router: ToolRouter<SubstrateService>,
  prompt_router: PromptRouter<SubstrateService>,
}

#[tool_router]
impl SubstrateService {
  pub fn new(state: AppState) -> Self {
    Self {
      state,
      tool_router: Self::tool_router(),
      prompt_router: Self::prompt_router(),
    }
  }

  #[tool(
    description = "Answer one concrete question from Substrate. Alternate phrasings must be near-paraphrases of the same question, not related topics."
  )]
  async fn query_single_topic(
    &self,
    Parameters(params): Parameters<QueryParams>,
  ) -> Result<CallToolResult, McpError> {
    let belief_store = &self.state.belief_store.lock().await;

    let mut flat_queries: Vec<String> = params.paraphrases;
    flat_queries.push(params.query);

    let beliefs = self
      .state
      .semantic_index
      .lock()
      .await
      .query_beliefs(
        flat_queries,
        params.max_result_count.unwrap_or(3),
        belief_store,
      )
      .await
      .map_err(to_mcp_error)?;

    Ok(CallToolResult::structured(json!({
      "beliefs": beliefs
    })))
  }

  #[tool(
    description = "Record a new belief in Substrate. You must follow the established nomenclature in the previously provided instructions."
  )]
  async fn record(
    &self,
    Parameters(params): Parameters<RecordParams>,
  ) -> Result<CallToolResult, McpError> {
    let belief = Belief {
      id: uuid::Uuid::new_v4().to_string(),
      content: params.content,
      tags: params
        .tags
        .into_iter()
        .map(|s| s.trim().to_string())
        .collect(),
      possible_queries: params
        .possible_queries
        .into_iter()
        .map(|s| s.trim().to_string())
        .collect(),
    };

    self
      .state
      .semantic_index
      .lock()
      .await
      .index(&belief)
      .await
      .map_err(to_mcp_error)?;

    self
      .state
      .belief_store
      .lock()
      .await
      .insert(&belief)
      .map_err(to_mcp_error)?;

    debug!("[SemanticIndex] Recorded new belief: {belief:?}");

    Ok(CallToolResult::success(vec![Content::text(
      "Belief Stored",
    )]))
  }

  // This is only necessary as a stop-gap measure until cursor-cli supports showing MCP prompts
  #[tool(
    description = "Returns the canonical workflow instructions for ingesting markdown into Substrate. This tool does not perform ingestion itself."
  )]
  async fn ingest_markdown(
    &self,
    Parameters(params): Parameters<IngestMarkdownPromptParams>,
  ) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::structured(json!({
      "workflow": "markdown_ingestion",
      "goal": "Extract reusable beliefs from markdown and store them in Substrate",
      "rules": [
        "Only store reusable, environment-specific facts",
        "Do not store temporary or speculative notes",
        "Beliefs must be atomic and self-contained"
      ],
      "steps": [
        format!("Read markdown file {}", params.markdown_file_path),
        "Extract beliefs",
        "Generate retrieval queries",
        "Record beliefs"
      ]
    })))
  }
}

#[prompt_router]
impl SubstrateService {
  #[prompt(name = "ingest_markdown_into_substrate")]
  async fn ingest_markdown_into_substrate(
    &self,
    Parameters(args): Parameters<IngestMarkdownPromptParams>,
    _ctx: RequestContext<RoleServer>,
  ) -> Result<GetPromptResult, McpError> {
    let requested_file = args.markdown_file_path;

    let messages = vec![
      PromptMessage::new_text(
        PromptMessageRole::Assistant,
        "I'll analyze the markdown file you referenced and ingest any generalizable, reusable information into Substrate",
      ),
      PromptMessage::new_text(
        PromptMessageRole::User,
        format!(
          "Please take a look at the contents of the markdown file at \"{}\", extract any beliefs you can, and record them in Substrate, according to Substrate's belief creation rules",
          requested_file
        ),
      ),
    ];

    Ok(
      GetPromptResult::new(messages)
        .with_description(format!("Ingest {} into Substrate", requested_file)),
    )
  }
}

#[tool_handler]
#[prompt_handler]
impl ServerHandler for SubstrateService {
  fn get_info(&self) -> ServerInfo {
    let instructions = include_str!("../../assets/instructions.md");

    ServerInfo::new(
      ServerCapabilities::builder()
        .enable_tools()
        .enable_prompts()
        .build(),
    )
    .with_instructions(instructions)
  }
}

fn to_mcp_error<E: std::fmt::Display>(e: E) -> McpError {
  McpError::internal_error(e.to_string(), None)
}
