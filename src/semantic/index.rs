use std::collections::HashSet;

use crate::beliefs::BeliefStore;
use crate::beliefs::belief::{Belief, RankedBelief};
use crate::beliefs::candidate::{CandidateBelief, CandidateBeliefEmbeddingEntry};
use crate::beliefs::embedding::BeliefEmbeddingEntry;
use crate::semantic;
use crate::{
  debug,
  error::AppResult,
  info,
  semantic::Reranker,
  semantic::embedding::{EmbeddingDB, EmbeddingResolver},
  trace,
  util::Config,
};
use fastembed::EmbeddingModel;
use futures_util::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase, Select};

pub struct SemanticIndex {
  db: EmbeddingDB,
  embedding_resolver: EmbeddingResolver,
  reranker: Reranker,
  semantic_top_k: usize,
  max_l2_distance: f32,
  reranker_top_k: usize,
}

impl SemanticIndex {
  pub async fn initialize(config: &Config) -> AppResult<Self> {
    info!("[SemanticIndex] Initializing...");

    let db = EmbeddingDB::initialize(&config).await?;
    let embedding_resolver = EmbeddingResolver::initialize()?;
    let reranker = Reranker::initialize()?;

    info!("[SemanticIndex] Initialized.");
    Ok(Self {
      db,
      embedding_resolver,
      reranker,
      semantic_top_k: config.retrieval.semantic_top_k,
      max_l2_distance: config.retrieval.max_l2_distance,
      reranker_top_k: config.retrieval.reranker_top_k,
    })
  }

  pub async fn index(&mut self, belief: &Belief) -> AppResult<()> {
    let mut embedding_entries_to_create: Vec<(&str, &str, &str)> = Vec::new();

    embedding_entries_to_create.push((&belief.id, "content", &belief.content));

    for tag in &belief.tags {
      embedding_entries_to_create.push((&belief.id, "tag", tag));
    }

    for query in &belief.possible_queries {
      embedding_entries_to_create.push((&belief.id, "possible_query", query));
    }

    let embedding_passages: Vec<String> = embedding_entries_to_create
      .iter()
      .map(|(_, _, passage)| passage.to_string())
      .collect();

    let embedding_vectors = self.embedding_resolver.embed(&embedding_passages)?;

    let embedding_entries: Vec<BeliefEmbeddingEntry> = embedding_entries_to_create
      .iter()
      .zip(embedding_vectors.into_iter())
      .map(
        |((belief_id, embedding_source, embedding_text), vector)| BeliefEmbeddingEntry {
          belief_id: belief_id.to_string(),
          entry_id: uuid::Uuid::new_v4().to_string(),
          embedding_source: embedding_source.to_string(),
          embedding_text: embedding_text.to_string(),
          vector,
        },
      )
      .collect();

    let batch = BeliefEmbeddingEntry::to_record_batch(&embedding_entries)?;

    let table = self
      .db
      .connection
      .open_table("belief_embeddings")
      .execute()
      .await?;

    table.add(batch).execute().await?;

    Ok(())
  }

  pub async fn query_candidate_beliefs(
    &mut self,
    query: &[String],
    limit: usize,
    belief_store: &BeliefStore,
  ) -> AppResult<Vec<CandidateBelief>> {
    let ranked = self.find_ranked_candidates(query, belief_store).await?;

    let limit = std::cmp::max(limit, self.reranker_top_k);

    let candidates = ranked
      .iter()
      .take(limit)
      .map(|r| CandidateBelief {
        content: r.content.clone(),
        score: r.score,
      })
      .collect();

    debug!("[SemanticIndex] Query [LIMIT {limit}]: {query:?}");
    debug!("[SemanticIndex] Results: {candidates:?}");

    Ok(candidates)
  }

  pub async fn flush(config: &Config) -> AppResult<()> {
    EmbeddingDB::flush(config)?;

    Ok(())
  }

  pub async fn find_ranked_candidates(
    &mut self,
    query: &[String],
    belief_store: &BeliefStore,
  ) -> AppResult<Vec<RankedBelief>> {
    let table = self
      .db
      .connection
      .open_table("belief_embeddings")
      .execute()
      .await?;

    let query_vectors = self.embedding_resolver.embed(&query)?;

    let semantic_top_k = self.semantic_top_k;

    let mut query_futures = Vec::new();
    for vector in query_vectors {
      let query_builder = table.query();

      let future = async move {
        query_builder
          .nearest_to(vector)?
          .limit(semantic_top_k)
          .select(Select::Columns(vec![
            "belief_id".to_string(),
            "entry_id".to_string(),
            "embedding_source".to_string(),
            "embedding_text".to_string(),
            "_distance".to_string(),
          ]))
          .execute()
          .await
      };

      query_futures.push(future);
    }

    let query_streams = futures::future::try_join_all(query_futures).await?;

    let mut results = Vec::new();
    for stream in query_streams {
      let mut batches = stream.try_collect::<Vec<_>>().await?;
      results.append(&mut batches);
    }

    let mut semantically_similar_beliefs: HashSet<String> = HashSet::new();
    for entry in CandidateBeliefEmbeddingEntry::from_record_batch_stream(&results)? {
      if entry.score > self.max_l2_distance {
        trace!("[SemanticIndex] Discarded due to l2_distance: {entry:?}");
        continue;
      }

      semantically_similar_beliefs.insert(entry.belief_id.clone());
    }

    let mut candidates = semantically_similar_beliefs
      .into_iter()
      .map(|belief_id| self.reranked_candidate_belief_from_id(&belief_id, query, belief_store))
      .collect::<AppResult<Vec<_>>>()?;

    candidates.sort_unstable_by(|a, b| {
      b.score
        .partial_cmp(&a.score)
        .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(candidates)
  }

  fn reranked_candidate_belief_from_id(
    &mut self,
    belief_id: &str,
    queries: &[String],
    belief_store: &BeliefStore,
  ) -> AppResult<RankedBelief> {
    let belief = belief_store
      .get(belief_id)?
      .ok_or_else(|| format!("Missing belief in store for id {}", belief_id))?;

    let belief_as_passage = format!(
      "{}. Related questions are: ({})",
      belief.content,
      belief
        .possible_queries
        .iter()
        .map(|q| format!("\"{}\"", q))
        .collect::<Vec<_>>()
        .join(", ")
    );

    let mut best_raw_reranker_score = f32::MIN;

    for query in queries {
      let raw_reranker_score = self.reranker.score(query, &belief_as_passage)?;

      best_raw_reranker_score = best_raw_reranker_score.max(raw_reranker_score);
    }

    Ok(RankedBelief {
      id: belief_id.into(),
      content: belief.content,
      tags: belief.tags,
      possible_queries: belief.possible_queries,
      score: Self::sigmoid(best_raw_reranker_score), // Normalize to 0..1
    })
  }

  fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
  }
}
