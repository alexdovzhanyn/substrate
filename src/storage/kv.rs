use redb::{Database, ReadableDatabase, TableDefinition};
use serde_json;

use crate::beliefs::belief::Belief;
use crate::util::{get_storage_path, Config};

const BELIEF_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("beliefs");

pub struct KVStore {
    connection: Database
}

impl KVStore {
    pub fn initialize(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let connection = Database::create(get_storage_path(&config.storage.redb_file))?;

        Ok(Self { connection })
    }

    pub fn insert_belief(&self, belief: &Belief) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = serde_json::to_vec(belief)?;

        let write_txn = self.connection.begin_write()?;

        {
            let mut table = write_txn.open_table(BELIEF_TABLE)?;
            table.insert(belief.id.as_str(), serialized.as_slice())?;
        }

        write_txn.commit()?;

        Ok(())
    }

    pub fn get_belief(&self, belief_id: &str) -> Result<Option<Belief>, Box<dyn std::error::Error>> {
        let read_txn = self.connection.begin_read()?;
        let table = read_txn.open_table(BELIEF_TABLE)?;

        let value = match table.get(belief_id)? {
            Some(value) => value,
            None => return Ok(None)
        };

        let belief: Belief = serde_json::from_slice(value.value())?;
        Ok(Some(belief))
    }
}
