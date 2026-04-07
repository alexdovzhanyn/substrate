use lancedb::{connect, Connection};
use lancedb::database::CreateTableMode;
use lancedb::query::{Select, QueryBase, ExecutableQuery};
use arrow_array::{Float32Array, LargeStringArray, Array};
use futures_util::TryStreamExt;

use crate::beliefs::belief::Belief;
use crate::beliefs::embedding::BeliefEmbeddingEntry;
use crate::beliefs::candidate::CandidateBeliefEmbeddingEntry;
use crate::semantic::resolver::Resolver;
use crate::util::{get_storage_path, Config};

pub struct EmbeddingDB {
    connection: Connection,
    semantic_resolver: Resolver,
    semantic_top_k: usize,
    max_l2_distance: f32
}

impl EmbeddingDB {
    pub async fn initialize(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let connection = connect(&get_storage_path(&config.storage.lancedb_file)).execute().await?;
        let semantic_resolver = Resolver::initialize()?;

        let db = Self {
            connection,
            semantic_resolver,
            semantic_top_k: config.retrieval.semantic_top_k,
            max_l2_distance: config.retrieval.max_l2_distance
        };

        db.create_tables().await?;

        Ok(db)
    }

    async fn create_tables(&self) -> Result<(), Box<dyn std::error::Error>> {
        let existing_tables = self.connection.table_names().execute().await?;
        let required_tables = vec![
            ("belief_embeddings", BeliefEmbeddingEntry::get_schema)
        ];

        for (table_name, schema_resolver) in required_tables {
            if existing_tables.contains(&table_name.to_string()) {
                continue;
            }

            self.connection
                .create_empty_table(table_name, schema_resolver())
                .mode(CreateTableMode::exist_ok(|request| request))
                .execute()
                .await?;
        }

        Ok(())
    }

    pub async fn insert_embeddings(&mut self, belief: &Belief) -> Result<(), Box<dyn std::error::Error>> {
        let mut embedding_entries: Vec<BeliefEmbeddingEntry> = Vec::new();

        embedding_entries.push(self.create_embedding_entry(
            &belief.id,
            "subject",
            &belief.subject
        )?);

        embedding_entries.push(self.create_embedding_entry(
            &belief.id,
            "value",
            &belief.value
        )?);

        for tag in &belief.tags {
            embedding_entries.push(self.create_embedding_entry(
                &belief.id,
                "tag",
                tag,
            )?);
        }

        for query in &belief.possible_queries {
            embedding_entries.push(self.create_embedding_entry(
                &belief.id,
                "possible_query",
                query,
            )?);
        }

        let batch = BeliefEmbeddingEntry::to_record_batch(&embedding_entries)?;

        let table = self.connection.open_table("belief_embeddings").execute().await?;
        table.add(batch).execute().await?;

        Ok(()) 
    } 

    fn create_embedding_entry(
        &mut self,
        belief_id: &str,
        embedding_source: &str,
        embedding_text: &str
    ) -> Result<BeliefEmbeddingEntry, Box<dyn std::error::Error>> {
        let embedding = self.semantic_resolver.embed(embedding_text.to_string())?;
        let vector = embedding.into_iter().next().ok_or("No embedding returned")?;

        let entry = BeliefEmbeddingEntry {
            belief_id: belief_id.to_string(),
            entry_id: uuid::Uuid::new_v4().to_string(),
            embedding_source: embedding_source.to_string(),
            embedding_text: embedding_text.to_string(),
            vector
        };

        Ok(entry)
    }

    pub async fn query_beliefs(&mut self, prompt: String) -> Result<Vec<CandidateBeliefEmbeddingEntry>, Box<dyn std::error::Error>> {
        let embedding = self.semantic_resolver.embed(prompt.clone())?;
        let vector = embedding.into_iter().next().ok_or("No embedding returned")?;

        let table = self.connection.open_table("belief_embeddings").execute().await?;

        let results = table
            .query()
            .nearest_to(vector)?
            .limit(self.semantic_top_k)
            .select(Select::Columns(vec![
                "belief_id".to_string(),
                "entry_id".to_string(),
                "embedding_source".to_string(),
                "embedding_text".to_string(),
                "_distance".to_string()
            ]))
            .execute()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        let mut candidates: Vec<CandidateBeliefEmbeddingEntry> = Vec::new();

        for batch in results {
            let belief_id_col = batch
                .column_by_name("belief_id")
                .ok_or("Missing belief_id column")?
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .ok_or("belief_id column had wrong type")?;

            let entry_id_col = batch
                .column_by_name("entry_id")
                .ok_or("Missing entry_id column")?
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .ok_or("entry_id column had wrong type")?;

            let embedding_source_col = batch
                .column_by_name("embedding_source")
                .ok_or("Missing embedding_source column")?
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .ok_or("embedding_source column had wrong type")?;

            let embedding_text_col = batch
                .column_by_name("embedding_text")
                .ok_or("Missing embedding_text column")?
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .ok_or("embedding_text column had wrong type")?;

            let distance_col = batch
                .column_by_name("_distance")
                .ok_or("Missing _distance column")?
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or("_distance column had wrong type")?;

            for row_idx in 0..batch.num_rows() {
                if distance_col.is_null(row_idx) {
                    continue;
                }

                let distance = distance_col.value(row_idx);

                if distance > self.max_l2_distance {
                    continue;
                }

                let candidate = CandidateBeliefEmbeddingEntry {
                    belief_id: belief_id_col.value(row_idx).to_string(),
                    entry_id: entry_id_col.value(row_idx).to_string(),
                    embedding_source: embedding_source_col.value(row_idx).to_string(),
                    embedding_text: embedding_text_col.value(row_idx).to_string(),
                    score: distance_col.value(row_idx)
                };

                candidates.push(candidate);    
            }
        }

        Ok(candidates)
    }
}
