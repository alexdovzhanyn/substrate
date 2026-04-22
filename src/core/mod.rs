use crate::{
  core::{
    beliefs::{
      belief::{Belief, BeliefCommitment, BeliefDraft, BeliefProposal, ResolutionAction},
      candidate::CandidateBelief,
    },
    query_types::{BatchQuery, SingleQuery},
  },
  error::AppResult,
  util::Config,
};

pub mod beliefs;
pub mod query_types;
use std::{collections::HashMap, time::SystemTime};
mod semantic;

use beliefs::store::BeliefStore;
use semantic::SemanticIndex;

pub struct SubstrateCore {
  pub semantic_index: semantic::SemanticIndex,
  pub belief_store: beliefs::BeliefStore,
}

impl SubstrateCore {
  pub async fn initialize(config: &Config) -> AppResult<Self> {
    let semantic_index = semantic::SemanticIndex::initialize(&config).await?;
    let belief_store = beliefs::BeliefStore::initialize(&config)?;

    Ok(Self {
      semantic_index,
      belief_store,
    })
  }

  pub async fn query_single(&mut self, query: SingleQuery) -> AppResult<Vec<CandidateBelief>> {
    let mut queries: Vec<String> = query.paraphrases;
    queries.push(query.query);

    self
      .semantic_index
      .query_candidate_beliefs(
        &queries,
        query.max_result_count.unwrap_or(3),
        &self.belief_store,
      )
      .await
  }

  pub async fn query_batch(
    &mut self,
    batch: BatchQuery,
  ) -> AppResult<HashMap<String, Vec<CandidateBelief>>> {
    let mut beliefs: HashMap<String, Vec<CandidateBelief>> = HashMap::new();

    for query_set in batch.queries {
      let mut flat_queries: Vec<String> = query_set.paraphrases;
      flat_queries.push(query_set.query.clone());

      let candidates = self
        .semantic_index
        .query_candidate_beliefs(
          &flat_queries,
          query_set.max_result_count.unwrap_or(3),
          &self.belief_store,
        )
        .await?;

      beliefs.insert(query_set.query, candidates);
    }

    Ok(beliefs)
  }

  pub async fn propose(&mut self, proposal: BeliefProposal) -> AppResult<Option<BeliefDraft>> {
    let draft_id = uuid::Uuid::new_v4().to_string();

    let tags = proposal
      .tags
      .into_iter()
      .map(|s| s.trim().to_string())
      .collect();

    let possible_queries = proposal
      .possible_queries
      .clone()
      .into_iter()
      .map(|s| s.trim().to_string())
      .collect();

    let mut flat_queries: Vec<String> = proposal.possible_queries;
    flat_queries.push(proposal.content.clone());

    let potential_conflicts: Vec<Belief> = self
      .semantic_index
      .find_ranked_candidates(&flat_queries, &self.belief_store)
      .await?
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
        content: proposal.content,
        tags,
        possible_queries,
        created_at: time_now,
        updated_at: time_now,
      };

      self.semantic_index.index(&belief).await?;

      self.belief_store.insert_belief(&belief, false)?;

      return Ok(None);
    }

    let draft = BeliefDraft {
      belief: Belief {
        id: draft_id,
        content: proposal.content,
        tags,
        possible_queries,
        created_at: time_now,
        updated_at: time_now,
      },
      potential_conflicts,
    };

    self.belief_store.insert_belief(&draft, true)?;

    Ok(Some(draft))
  }

  pub async fn commit(&mut self, commitment: BeliefCommitment) -> AppResult<()> {
    let time_now: u64 = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)?
      .as_secs();

    let mut draft = self
      .belief_store
      .get_belief(&commitment.draft_id, true)?
      .ok_or_else(|| "No matching draft ID found")?;

    let mut should_promote_draft = true;

    for resolution in commitment.conflict_resolutions {
      match resolution.action {
        ResolutionAction::Invalidate => {
          self
            .belief_store
            .remove_belief(&resolution.conflicting_belief_id)?;

          self
            .semantic_index
            .remove_embeddings(&resolution.conflicting_belief_id)
            .await?;
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

    if should_promote_draft {
      draft.updated_at = time_now;

      self.semantic_index.index(&draft).await?;

      self.belief_store.promote_draft(&draft.id)?;

      return Ok(());
    }

    self.belief_store.remove_belief(&draft.id)?;

    Ok(())
  }

  pub async fn flush() -> AppResult<()> {
    let config = Config::load("config/default.toml")?;

    SemanticIndex::flush(&config).await?;
    BeliefStore::flush(&config)?;

    Ok(())
  }

  async fn add_belief_query(&mut self, belief_id: &str, query: &str) -> AppResult<()> {
    let mut belief = self
      .belief_store
      .get_belief(belief_id, false)?
      .ok_or_else(|| format!("Belief not found: {}", belief_id))?;

    self
      .semantic_index
      .insert_embedding_entry(belief_id, "possible_query", query)
      .await?;

    belief.possible_queries.push(query.into());

    self.belief_store.update_belief(&mut belief)?;

    Ok(())
  }
}
