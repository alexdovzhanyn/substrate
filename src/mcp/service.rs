use crate::beliefs::belief::{
  Belief, BeliefCommitment, BeliefDraft, BeliefProposal, ResolutionAction,
};
use crate::beliefs::candidate::CandidateBelief;
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
use serde_json::json;
use std::collections::HashMap;
use std::time::SystemTime;

use crate::mcp::types::{BatchQueryParams, IngestMarkdownPromptParams, SingleQueryParams};
use crate::state::AppState;

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

  #[tool(description = "Answer one concrete question from Substrate")]
  async fn query_single(
    &self,
    Parameters(params): Parameters<SingleQueryParams>,
  ) -> Result<CallToolResult, McpError> {
    let belief_store = &self.state.belief_store.lock().await;

    let mut flat_queries: Vec<String> = params.paraphrases;
    flat_queries.push(params.query);

    let beliefs = self
      .state
      .semantic_index
      .lock()
      .await
      .query_candidate_beliefs(
        &flat_queries,
        params.max_result_count.unwrap_or(3),
        belief_store,
      )
      .await
      .map_err(to_mcp_error)?;

    Ok(CallToolResult::structured(json!({
      "beliefs": beliefs
    })))
  }

  #[tool(description = "Answer multiple concrete questions from Substrate")]
  async fn query_batch(
    &self,
    Parameters(params): Parameters<BatchQueryParams>,
  ) -> Result<CallToolResult, McpError> {
    let belief_store = &self.state.belief_store.lock().await;
    let mut semantic_index = self.state.semantic_index.lock().await;

    let mut beliefs: HashMap<String, Vec<CandidateBelief>> = HashMap::new();
    for query_set in params.queries {
      let mut flat_queries: Vec<String> = query_set.paraphrases;
      flat_queries.push(query_set.query.clone());

      let candidates = semantic_index
        .query_candidate_beliefs(
          &flat_queries,
          query_set.max_result_count.unwrap_or(3),
          belief_store,
        )
        .await
        .map_err(to_mcp_error)?;

      beliefs.insert(query_set.query, candidates);
    }

    Ok(CallToolResult::structured(json!(beliefs)))
  }

  #[tool(description = "Propose a new belief to be added to Substrate")]
  async fn propose(
    &self,
    Parameters(params): Parameters<BeliefProposal>,
  ) -> Result<CallToolResult, McpError> {
    let belief_store = &self.state.belief_store.lock().await;

    let draft_id = uuid::Uuid::new_v4().to_string();

    let tags = params
      .tags
      .into_iter()
      .map(|s| s.trim().to_string())
      .collect();

    let possible_queries = params
      .possible_queries
      .clone()
      .into_iter()
      .map(|s| s.trim().to_string())
      .collect();

    let mut flat_queries: Vec<String> = params.possible_queries;
    flat_queries.push(params.content.clone());

    let potential_conflicts: Vec<Belief> = self
      .state
      .semantic_index
      .lock()
      .await
      .find_ranked_candidates(&flat_queries, belief_store)
      .await
      .map_err(to_mcp_error)?
      .into_iter()
      .filter(|c| c.score >= 0.75)
      .map(|c| c.belief)
      .collect();

    let time_now: u64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
      Ok(t) => t.as_secs(),
      Err(_) => panic!("Could not get system time!"),
    };

    if potential_conflicts.is_empty() {
      let belief = Belief {
        id: draft_id,
        content: params.content,
        tags,
        possible_queries,
        created_at: time_now,
        updated_at: time_now,
      };

      self
        .state
        .semantic_index
        .lock()
        .await
        .index(&belief)
        .await
        .map_err(to_mcp_error)?;

      belief_store.insert_belief(&belief).map_err(to_mcp_error)?;

      debug!("[SemanticIndex] Recorded new belief: {belief:?}");

      return Ok(CallToolResult::success(vec![Content::text(
        "No conflicts, belief recorded successully",
      )]));
    }

    let draft = BeliefDraft {
      id: draft_id,
      content: params.content,
      tags,
      possible_queries,
      potential_conflicts,
      created_at: time_now,
    };

    belief_store.insert_draft(&draft).map_err(to_mcp_error)?;

    Ok(CallToolResult::structured(json!(draft)))
  }

  #[tool(description = "Commit a previously proposed belief to Substrate")]
  async fn commit(
    &self,
    Parameters(params): Parameters<BeliefCommitment>,
  ) -> Result<CallToolResult, McpError> {
    let time_now: u64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
      Ok(t) => t.as_secs(),
      Err(_) => panic!("Could not get system time!"),
    };

    let draft = self
      .state
      .belief_store
      .lock()
      .await
      .get_draft(&params.draft_id)
      .map_err(to_mcp_error)?;

    let draft = match draft {
      Some(draft) => draft,
      None => {
        return Ok(CallToolResult::error(vec![Content::text(
          "No such draft ID found",
        )]));
      }
    };

    let mut should_promote_draft = true;

    for resolution in params.conflict_resolutions {
      match resolution.action {
        ResolutionAction::Invalidate => {
          self
            .invalidate_belief(&resolution.conflicting_belief_id)
            .await?
        }
        ResolutionAction::MergeDuplicate => {
          should_promote_draft = false;

          let missed_query = match resolution.missed_query {
            Some(q) => q,
            None => continue,
          };

          self
            .add_belief_query(&resolution.conflicting_belief_id, &missed_query)
            .await?;
        }
        ResolutionAction::Ignore => (),
      }
    }

    let belief_store = self.state.belief_store.lock().await;

    if should_promote_draft {
      let belief = Belief {
        id: draft.id.clone(),
        content: draft.content,
        tags: draft.tags,
        possible_queries: draft.possible_queries,
        created_at: time_now,
        updated_at: time_now,
      };

      self
        .state
        .semantic_index
        .lock()
        .await
        .index(&belief)
        .await
        .map_err(to_mcp_error)?;

      belief_store.insert_belief(&belief).map_err(to_mcp_error)?;
    }

    belief_store.remove_draft(&draft.id).map_err(to_mcp_error)?;

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

  async fn add_belief_query(&self, belief_id: &str, query: &str) -> Result<(), McpError> {
    let belief_store = self.state.belief_store.lock().await;

    let mut belief = belief_store
      .get_belief(belief_id)
      .map_err(to_mcp_error)?
      .ok_or_else(|| McpError::internal_error("Belief not found", Some(belief_id.into())))?;

    self
      .state
      .semantic_index
      .lock()
      .await
      .insert_embedding_entry(belief_id, "possible_query", query)
      .await
      .map_err(to_mcp_error)?;

    belief.possible_queries.push(query.into());

    belief_store
      .update_belief(&mut belief)
      .map_err(to_mcp_error)?;

    Ok(())
  }

  async fn invalidate_belief(&self, belief_id: &str) -> Result<(), McpError> {
    self
      .state
      .belief_store
      .lock()
      .await
      .remove_belief(belief_id)
      .map_err(to_mcp_error)?;

    self
      .state
      .semantic_index
      .lock()
      .await
      .remove_embeddings(belief_id)
      .await
      .map_err(to_mcp_error)?;

    Ok(())
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
