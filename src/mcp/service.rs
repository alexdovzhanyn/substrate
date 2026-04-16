use crate::core::SubstrateCore;
use crate::core::beliefs::belief::{BeliefCommitment, BeliefProposal};
use crate::core::query_types::{BatchQuery, SingleQuery};
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
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::mcp::types::IngestMarkdownPromptParams;

#[derive(Clone)]
pub struct MCPService {
  core: Arc<Mutex<SubstrateCore>>,
  tool_router: ToolRouter<MCPService>,
  prompt_router: PromptRouter<MCPService>,
}

#[tool_router]
impl MCPService {
  pub fn new(core: Arc<Mutex<SubstrateCore>>) -> Self {
    Self {
      core,
      tool_router: Self::tool_router(),
      prompt_router: Self::prompt_router(),
    }
  }

  #[tool(description = "Answer one concrete question from Substrate")]
  async fn query_single(
    &self,
    Parameters(params): Parameters<SingleQuery>,
  ) -> Result<CallToolResult, McpError> {
    let beliefs = self
      .core
      .lock()
      .await
      .query_single(params)
      .await
      .map_err(to_mcp_error)?;

    Ok(CallToolResult::structured(json!({
      "beliefs": beliefs
    })))
  }

  #[tool(description = "Answer multiple concrete questions from Substrate")]
  async fn query_batch(
    &self,
    Parameters(params): Parameters<BatchQuery>,
  ) -> Result<CallToolResult, McpError> {
    let beliefs = self
      .core
      .lock()
      .await
      .query_batch(params)
      .await
      .map_err(to_mcp_error)?;

    Ok(CallToolResult::structured(json!(beliefs)))
  }

  #[tool(description = "Propose a new belief to be added to Substrate")]
  async fn propose(
    &self,
    Parameters(params): Parameters<BeliefProposal>,
  ) -> Result<CallToolResult, McpError> {
    match self
      .core
      .lock()
      .await
      .propose(params)
      .await
      .map_err(to_mcp_error)?
    {
      Some(draft) => Ok(CallToolResult::structured(json!(draft))),
      None => Ok(CallToolResult::success(vec![Content::text(
        "No conflicts, belief recorded successully",
      )])),
    }
  }

  #[tool(description = "Commit a previously proposed belief to Substrate")]
  async fn commit(
    &self,
    Parameters(params): Parameters<BeliefCommitment>,
  ) -> Result<CallToolResult, McpError> {
    self
      .core
      .lock()
      .await
      .commit(params)
      .await
      .map_err(to_mcp_error)?;

    Ok(CallToolResult::success(vec![Content::text("Ok")]))
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
impl MCPService {
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
impl ServerHandler for MCPService {
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
