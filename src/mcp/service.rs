use crate::beliefs::belief::Belief;
use rmcp::{
  ErrorData as McpError, ServerHandler,
  handler::server::router::tool::ToolRouter,
  handler::server::wrapper::Parameters,
  model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
  tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryParams {
  /// One or more natural language queries used to retrieve beliefs.
  ///
  /// Provide multiple phrasings of the same intent to improve recall.
  /// Each query should represent a realistic way this information might be searched for.
  ///
  /// Examples:
  /// - "how do i start the project"
  /// - "run app locally"
  /// - "start dev server"
  pub query: Vec<String>,
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

  /// 5–10 distinct query variations used for retrieval.
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

#[derive(Clone)]
pub struct TesseractService {
  state: AppState,
  tool_router: ToolRouter<TesseractService>,
}

#[tool_router]
impl TesseractService {
  pub fn new(state: AppState) -> Self {
    Self {
      state,
      tool_router: Self::tool_router(),
    }
  }

  #[tool(description = "Query beliefs from Tesseract")]
  async fn query(
    &self,
    Parameters(params): Parameters<QueryParams>,
  ) -> Result<CallToolResult, McpError> {
    let query = params
      .query
      .first()
      .ok_or_else(|| McpError::invalid_params("query must contain at least one entry", None))?;

    let belief_store = &self.state.belief_store.lock().await;

    let beliefs = self
      .state
      .semantic_index
      .lock()
      .await
      .query_beliefs(query.to_string(), belief_store)
      .await
      .map_err(to_mcp_error)?;

    Ok(CallToolResult::structured(json!({
      "beliefs": beliefs
    })))
  }

  #[tool(
    description = "Record a new belief in Tesseract. You must follow the established nomenclature in the previouslt provided instructions."
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
      .insert_belief_embeddings(&belief)
      .await
      .map_err(to_mcp_error)?;

    self
      .state
      .belief_store
      .lock()
      .await
      .insert(&belief)
      .map_err(to_mcp_error)?;

    Ok(CallToolResult::success(vec![Content::text(
      "Belief Stored",
    )]))
  }
}

#[tool_handler]
impl ServerHandler for TesseractService {
  fn get_info(&self) -> ServerInfo {
    let instructions = r#"
      Tesseract is a retrieval-optimized memory system for technical environments.

      Use Tesseract as your first source of truth for reusable knowledge about the local development environment.

      Before searching the local machine directly, you should query Tesseract first whenever you need to find:

      - where a repository, workspace, or file is located
      - how to run, build, test, deploy, or start something
      - what tool, command, config, or dependency is available
      - how a system in the environment works
      - reusable facts about the local setup

      Do not immediately inspect the filesystem, search the repo, or probe the environment if the answer may already exist in Tesseract. Query Tesseract first. If Tesseract does not return a useful answer, then fall back to direct inspection.

      Use Tesseract to store reusable technical knowledge that may be needed again later through natural language retrieval.

      Do NOT store:
      - temporary or one-off information
      - facts tied only to the current conversation
      - information that cannot be generalized or reused

      ## Retrieval Rule

      When you need information about the environment, tools, commands, repositories, or workflows:

      1. Query Tesseract first
      2. If the result is sufficient, use it
      3. If the result is missing, incomplete, or uncertain, inspect the local machine directly
      4. If you discover reusable information through direct inspection, store it in Tesseract as a new belief

      ## Belief Creation Rules

      When creating a belief:

      1. Content must be a complete, self-contained natural-language statement
         - It must make sense in isolation
         - Do not use relative language such as "this", "here", or "current" unless replaced by a stable identifier

      2. Provide 5–10 query variations
         - Include different verbs, nouns, and phrasings
         - Do not rely on semantic similarity alone
         - If two phrasings are similar, include both explicitly

      3. Keep beliefs atomic
         - One belief should contain one piece of information

      4. Prefer recall over precision
         - Missing a likely query phrasing makes the belief hard to retrieve later

      5. Use tags only as lightweight categorical metadata
         - Tags are secondary to `possible_queries`

      ## Source of Truth Rule

      Prefer storing **observed facts about this environment** over general knowledge.

      Beliefs should reflect what is TRUE in this environment, not what is typically true in general.

      ### Preferred (observed / verified):

      - "Rust is installed at /Users/alex/.cargo/bin/rustc"
      - "The cargo binary is located at /usr/local/bin/cargo"
      - "The project root is /Users/alex/projects/corefe-root"

      ### Acceptable (derived but still specific):

      - "Rust binaries are located in ~/.cargo/bin in this environment"

      ### NOT allowed (generic knowledge):

      - "Rust is typically installed in ~/.cargo/bin"
      - "On macOS, rustup installs toolchains under ~/.rustup"
      - "This is usually how Rust installations work"

      ### Rule:

      Only store general or “typical” information if:
      - you cannot determine the actual value in this environment, AND
      - the information is still useful for future reasoning

      Otherwise, always prefer concrete, environment-specific facts.

      ## Belief Maintenance Rule

      Beliefs must remain accurate over time.

      If you retrieve a belief from Tesseract and then attempt to use it, you must verify that it is still correct.

      If the belief is:

      - incorrect
      - outdated
      - no longer relevant
      - or fails when used in practice

      you must take corrective action.

      ### Required behavior:

      1. Do not continue relying on incorrect beliefs
      2. Determine the correct or updated information
      3. Update or replace the belief with the correct version

      ### Guidelines:

      - Prefer updating an existing belief rather than creating duplicates
      - If the correct information cannot be determined, do not store a replacement
      - Do not leave known-bad beliefs in the system

      ### Example:

      If a belief says:
      "Run pnpm dev to start the project"

      and this command fails or has changed, you must:
      - identify the correct command
      - update the belief accordingly

      ---

      Tesseract is a living memory system.

      Accuracy is more important than preserving old information.

      ## Mental Model

      Tesseract is a reusable memory layer.

      A belief is:
      - one self-contained piece of knowledge
      - plus multiple likely retrieval phrasings

      ## Example Retrieval Behavior

      If you need to know:
      - how to start a project
      - whether a tool is installed
      - where a repo lives
      - how configuration is loaded

      query Tesseract before searching the machine.

      ## Example Belief

      Content:
      "Run pnpm dev from the root directory to start the project locally."

      Possible queries:
      - how do i start the project
      - run app locally
      - start dev server
      - launch application

      Tags:
      - startup
      - local-dev
      - npm
    "#;

    ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
      .with_instructions(instructions)
  }
}

fn to_mcp_error<E: std::fmt::Display>(e: E) -> McpError {
  McpError::internal_error(e.to_string(), None)
}
