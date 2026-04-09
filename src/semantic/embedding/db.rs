use lancedb::database::CreateTableMode;
use lancedb::{Connection, connect};

use crate::beliefs::embedding::BeliefEmbeddingEntry;
use crate::error::AppResult;
use crate::util::{Config, get_storage_path};

pub struct EmbeddingDB {
    pub connection: Connection,
}

impl EmbeddingDB {
    pub async fn initialize(config: &Config) -> AppResult<Self> {
        let connection = connect(&get_storage_path(&config.storage.lancedb_file))
            .execute()
            .await?;

        let db = Self { connection };

        db.create_tables().await?;

        Ok(db)
    }

    async fn create_tables(&self) -> AppResult<()> {
        let existing_tables = self.connection.table_names().execute().await?;
        let required_tables = vec![("belief_embeddings", BeliefEmbeddingEntry::get_schema)];

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
}
