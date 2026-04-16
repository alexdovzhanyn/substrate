use std::fs::remove_file;
use std::time::SystemTime;

use redb::{Database, ReadableDatabase, TableDefinition};
use serde_json;

use crate::core::beliefs::belief::{Belief, BeliefDraft};
use crate::debug;
use crate::error::AppResult;
use crate::util::{Config, get_storage_path};

const BELIEF_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("beliefs");
const BELIEF_DRAFT_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("belief_drafts");

pub struct BeliefStore {
  connection: Database,
}

impl BeliefStore {
  pub fn initialize(config: &Config) -> AppResult<Self> {
    let connection = Database::create(get_storage_path(&config.storage.redb_file))?;

    Ok(Self { connection })
  }

  pub fn insert_belief(&self, belief: &Belief) -> AppResult<()> {
    let serialized = serde_json::to_vec(belief)?;

    let write_txn = self.connection.begin_write()?;

    {
      let mut table = write_txn.open_table(BELIEF_TABLE)?;
      table.insert(belief.id.as_str(), serialized.as_slice())?;
    }

    write_txn.commit()?;

    debug!("[BeliefStore] Inserted new belief: \n{belief:#?}");

    Ok(())
  }

  pub fn update_belief(&self, belief: &mut Belief) -> AppResult<()> {
    let time_now: u64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
      Ok(t) => t.as_secs(),
      Err(_) => panic!("Could not get system time!"),
    };

    belief.updated_at = time_now;

    let serialized = serde_json::to_vec(belief)?;

    let write_txn = self.connection.begin_write()?;

    {
      let mut table = write_txn.open_table(BELIEF_TABLE)?;
      table.insert(belief.id.as_str(), serialized.as_slice())?;
    }

    write_txn.commit()?;

    debug!("[BeliefStore] Updated belief: \n{belief:#?}");

    Ok(())
  }

  pub fn remove_belief(&self, belief_id: &str) -> AppResult<()> {
    let write_txn = self.connection.begin_write()?;

    {
      let mut table = write_txn.open_table(BELIEF_TABLE)?;
      table.remove(belief_id)?;
    }

    write_txn.commit()?;

    debug!("[BeliefStore] Deleted belief \n{belief_id:?}");

    Ok(())
  }

  pub fn get_belief(&self, belief_id: &str) -> AppResult<Option<Belief>> {
    let read_txn = self.connection.begin_read()?;
    let table = read_txn.open_table(BELIEF_TABLE)?;

    let value = match table.get(belief_id)? {
      Some(value) => value,
      None => return Ok(None),
    };

    let belief: Belief = serde_json::from_slice(value.value())?;

    Ok(Some(belief))
  }

  pub fn insert_draft(&self, draft: &BeliefDraft) -> AppResult<()> {
    let serialized = serde_json::to_vec(draft)?;

    let write_txn = self.connection.begin_write()?;

    {
      let mut table = write_txn.open_table(BELIEF_DRAFT_TABLE)?;
      table.insert(draft.id.as_str(), serialized.as_slice())?;
    }

    write_txn.commit()?;

    debug!("[BeliefStore] Created new draft: \n{draft:#?}");

    Ok(())
  }

  pub fn get_draft(&self, draft_id: &str) -> AppResult<Option<BeliefDraft>> {
    let read_txn = self.connection.begin_read()?;
    let table = read_txn.open_table(BELIEF_DRAFT_TABLE)?;

    let value = match table.get(draft_id)? {
      Some(value) => value,
      None => return Ok(None),
    };

    let draft: BeliefDraft = serde_json::from_slice(value.value())?;

    Ok(Some(draft))
  }

  pub fn remove_draft(&self, draft_id: &str) -> AppResult<()> {
    let write_txn = self.connection.begin_write()?;

    {
      let mut table = write_txn.open_table(BELIEF_DRAFT_TABLE)?;
      table.remove(draft_id)?;
    }

    write_txn.commit()?;

    debug!("[BeliefStore] Deleted draft \n{draft_id:?}");
    Ok(())
  }

  pub fn flush(config: &Config) -> AppResult<()> {
    remove_file(get_storage_path(&config.storage.redb_file))?;

    Ok(())
  }
}
