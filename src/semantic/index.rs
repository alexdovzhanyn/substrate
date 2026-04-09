use std::collections::HashMap;

use crate::beliefs::BeliefStore;
use crate::beliefs::belief::Belief;
use crate::beliefs::candidate::{CandidateBelief, CandidateBeliefEmbeddingEntry};
use crate::beliefs::embedding::BeliefEmbeddingEntry;
use crate::{
    error::AppResult,
    semantic::Reranker,
    semantic::embedding::{EmbeddingDB, EmbeddingResolver},
    util::Config,
};
use futures_util::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase, Select};

pub struct SemanticIndex {
    db: EmbeddingDB,
    embedding_resolver: EmbeddingResolver,
    reranker: Reranker,
    semantic_top_k: usize,
    max_l2_distance: f32,
    min_reranker_score: f32,
}

impl SemanticIndex {
    pub async fn initialize(config: &Config) -> AppResult<Self> {
        let db = EmbeddingDB::initialize(&config).await?;
        let embedding_resolver = EmbeddingResolver::initialize()?;
        let reranker = Reranker::initialize()?;

        Ok(Self {
            db,
            embedding_resolver,
            reranker,
            semantic_top_k: config.retrieval.semantic_top_k,
            max_l2_distance: config.retrieval.max_l2_distance,
            min_reranker_score: config.retrieval.min_reranker_score,
        })
    }

    pub async fn insert_belief_embeddings(&mut self, belief: &Belief) -> AppResult<()> {
        let mut embedding_entries: Vec<BeliefEmbeddingEntry> = Vec::new();

        embedding_entries.push(self.create_embedding_entry(
            &belief.id,
            "subject",
            &belief.subject,
        )?);
        embedding_entries.push(self.create_embedding_entry(&belief.id, "value", &belief.value)?);

        for tag in &belief.tags {
            embedding_entries.push(self.create_embedding_entry(&belief.id, "tag", tag)?);
        }

        for query in &belief.possible_queries {
            embedding_entries.push(self.create_embedding_entry(
                &belief.id,
                "possible_query",
                query,
            )?);
        }

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

    pub async fn query_beliefs(
        &mut self,
        prompt: String,
        belief_store: &BeliefStore,
    ) -> AppResult<Vec<CandidateBelief>> {
        let embedding = self.embedding_resolver.embed(prompt.clone())?;
        let vector = embedding
            .into_iter()
            .next()
            .ok_or("No embedding returned")?;

        let table = self
            .db
            .connection
            .open_table("belief_embeddings")
            .execute()
            .await?;

        let results = table
            .query()
            .nearest_to(vector)?
            .limit(self.semantic_top_k)
            .select(Select::Columns(vec![
                "belief_id".to_string(),
                "entry_id".to_string(),
                "embedding_source".to_string(),
                "embedding_text".to_string(),
                "_distance".to_string(),
            ]))
            .execute()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        let mut best_by_belief_id: HashMap<String, f32> = HashMap::new();
        for entry in CandidateBeliefEmbeddingEntry::from_record_batch_stream(&results)? {
            if entry.score > self.max_l2_distance {
                println!("Discarded due to l2_distance: {entry:#?}");
                continue;
            }

            best_by_belief_id
                .entry(entry.belief_id.clone())
                .and_modify(|existing| {
                    if entry.score < *existing {
                        *existing = entry.score;
                    }
                })
                .or_insert(entry.score);
        }

        let min_reranker_score = self.min_reranker_score;

        let candidates = best_by_belief_id
            .into_keys()
            .map(|belief_id| {
                self.create_candidate_belief_from_embedding(&belief_id, &prompt, &belief_store)
            })
            .collect::<AppResult<Vec<_>>>()?
            .into_iter()
            .filter(|c| {
                let is_kept = c.score > min_reranker_score;

                if !is_kept {
                    println!("Discared due to reranker threshold: {c:#?}");
                }

                is_kept
            })
            .collect();

        Ok(candidates)
    }

    fn create_embedding_entry(
        &mut self,
        belief_id: &str,
        embedding_source: &str,
        embedding_text: &str,
    ) -> AppResult<BeliefEmbeddingEntry> {
        let embedding = self.embedding_resolver.embed(embedding_text.to_string())?;
        let vector = embedding
            .into_iter()
            .next()
            .ok_or("No embedding returned")?;

        let entry = BeliefEmbeddingEntry {
            belief_id: belief_id.to_string(),
            entry_id: uuid::Uuid::new_v4().to_string(),
            embedding_source: embedding_source.to_string(),
            embedding_text: embedding_text.to_string(),
            vector,
        };

        Ok(entry)
    }

    fn create_candidate_belief_from_embedding(
        &mut self,
        belief_id: &str,
        original_query: &str,
        belief_store: &BeliefStore,
    ) -> AppResult<CandidateBelief> {
        let belief = belief_store
            .get(belief_id)?
            .ok_or_else(|| format!("Missing belief in store for id {}", belief_id))?;

        let belief_as_passage = format!(
            "{}: {}. {}",
            belief.subject,
            belief.value,
            belief
                .possible_queries
                .iter()
                .map(|q| format!("Question: {}.", q))
                .collect::<Vec<_>>()
                .join(" ")
        );

        let raw_reranker_score = self.reranker.score(original_query, &belief_as_passage)?;

        Ok(CandidateBelief {
            subject: belief.subject,
            value: belief.value,
            score: Self::sigmoid(raw_reranker_score), // Normalize to 0..1
        })
    }

    fn sigmoid(x: f32) -> f32 {
        1.0 / (1.0 + (-x).exp())
    }
}
