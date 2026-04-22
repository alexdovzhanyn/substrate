use std::collections::HashMap;
use std::fs::remove_file;
use std::time::SystemTime;

use rusqlite::{Connection, OptionalExtension, named_params, params_from_iter};

use crate::core::beliefs::belief::Belief;
use crate::debug;
use crate::error::AppResult;
use crate::util::{Config, get_storage_path};

pub struct BeliefStore {
  connection: std::sync::Mutex<Connection>,
}

impl BeliefStore {
  pub fn initialize(config: &Config) -> AppResult<Self> {
    let connection = Connection::open(get_storage_path(&config.storage.sqlite_file))?;

    Self::ensure_tables(&connection)?;

    Ok(Self {
      connection: std::sync::Mutex::new(connection),
    })
  }

  fn ensure_tables(connection: &Connection) -> AppResult<()> {
    connection.execute_batch(
      "BEGIN;

       CREATE TABLE IF NOT EXISTS beliefs (
         id               TEXT PRIMARY KEY,
         content          TEXT NOT NULL,
         is_draft         BOOLEAN DEFAULT FALSE,
         created_at       INTEGER NOT NULL,
         updated_at       INTEGER NOT NULL
       );

       CREATE TABLE IF NOT EXISTS belief_tags (
         belief_id TEXT NOT NULL,
         tag       TEXT NOT NULL,
         PRIMARY KEY (belief_id, tag),
         FOREIGN KEY (belief_id) REFERENCES beliefs(id) ON DELETE CASCADE
       );

       CREATE TABLE IF NOT EXISTS belief_queries (
         belief_id TEXT NOT NULL,
         query     TEXT NOT NULL,
         PRIMARY KEY (belief_id, query),
         FOREIGN KEY (belief_id) REFERENCES beliefs(id) ON DELETE CASCADE
       );

       PRAGMA foreign_keys = ON;

       COMMIT;
      ",
    )?;

    Ok(())
  }

  pub fn insert_belief(&mut self, belief: &Belief, is_draft: bool) -> AppResult<()> {
    let mut connection = self.connection.lock().unwrap();
    let transaction = connection.transaction()?;

    transaction.execute(
      "INSERT INTO beliefs (id, content, created_at, updated_at, is_draft) VALUES (?1, ?2, ?3, ?4, ?5)",
      (
        &belief.id,
        &belief.content,
        belief.created_at as i64,
        belief.updated_at as i64,
        is_draft
      ),
    )?;

    for tag in &belief.tags {
      transaction.execute(
        "INSERT INTO belief_tags (belief_id, tag) VALUES (?1, ?2)",
        (&belief.id, tag),
      )?;
    }

    for query in &belief.possible_queries {
      transaction.execute(
        "INSERT INTO belief_queries (belief_id, query) VALUES (?1, ?2)",
        (&belief.id, query),
      )?;
    }

    transaction.commit()?;

    debug!("[BeliefStore] Inserted new belief: \n{belief:#?}");

    Ok(())
  }

  pub fn update_belief(&mut self, belief: &mut Belief) -> AppResult<()> {
    let time_now: u64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
      Ok(t) => t.as_secs(),
      Err(_) => panic!("Could not get system time!"),
    };

    let mut connection = self.connection.lock().unwrap();
    let transaction = connection.transaction()?;

    transaction.execute(
      "UPDATE beliefs SET content = :content, updated_at = :updated WHERE id = :id",
      named_params! {
        ":id": &belief.id,
        ":content": &belief.content,
        ":updated": time_now as i64,
      },
    )?;

    transaction.execute("DELETE FROM belief_tags WHERE belief_id = ?1", [&belief.id])?;
    transaction.execute(
      "DELETE FROM belief_queries WHERE belief_id = ?1",
      [&belief.id],
    )?;

    for tag in &belief.tags {
      transaction.execute(
        "INSERT INTO belief_tags (belief_id, tag) VALUES (?1, ?2)",
        (&belief.id, tag),
      )?;
    }

    for query in &belief.possible_queries {
      transaction.execute(
        "INSERT INTO belief_queries (belief_id, query) VALUES (?1, ?2)",
        (&belief.id, query),
      )?;
    }

    transaction.commit()?;

    debug!("[BeliefStore] Updated belief: \n{belief:#?}");

    Ok(())
  }

  pub fn remove_belief(&self, belief_id: &str) -> AppResult<()> {
    self
      .connection
      .lock()
      .unwrap()
      .execute("DELETE FROM beliefs WHERE id = ?1", [belief_id])?;

    debug!("[BeliefStore] Deleted belief \n{belief_id:?}");

    Ok(())
  }

  pub fn get_belief(&self, belief_id: &str, is_draft: bool) -> AppResult<Option<Belief>> {
    let connection = self.connection.lock().unwrap();

    let belief_row = connection
      .query_row(
        "SELECT id, content, created_at, updated_at FROM beliefs WHERE id = :belief_id and is_draft = :is_draft",
        named_params! {
          ":belief_id": belief_id,
          ":is_draft": is_draft
        },
        |row| {
          Ok(BeliefRow {
            id: row.get(0)?,
            content: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
          })
        },
      )
      .optional()?;

    let belief_row = match belief_row {
      Some(row) => row,
      None => return Ok(None),
    };

    let tags = connection
      .prepare("SELECT tag FROM belief_tags WHERE belief_id = ?1")?
      .query_map([belief_id], |row| row.get(0))?
      .collect::<Result<Vec<String>, _>>()?;

    let possible_queries = connection
      .prepare("SELECT query FROM belief_queries WHERE belief_id = ?1")?
      .query_map([belief_id], |row| row.get(0))?
      .collect::<Result<Vec<String>, _>>()?;

    let belief = Belief {
      id: belief_row.id,
      content: belief_row.content,
      possible_queries,
      tags,
      created_at: belief_row.created_at as u64,
      updated_at: belief_row.updated_at as u64,
    };

    Ok(Some(belief))
  }

  pub fn get_beliefs(
    &self,
    limit: usize,
    search: Option<String>,
    offset: Option<usize>,
  ) -> AppResult<Vec<Belief>> {
    let connection = self.connection.lock().unwrap();

    let belief_rows = connection
      .prepare(
        "
        SELECT b.id, b.content, b.created_at, b.updated_at FROM beliefs b 
        JOIN (
          SELECT id as belief_id FROM beliefs
          WHERE content LIKE '%' || ?1 || '%'

          UNION

          SELECT belief_id FROM belief_tags
          WHERE tag LIKE '%' || ?1 || '%'

          UNION

          SELECT belief_id FROM belief_queries
          WHERE query LIKE '%' || ?1 || '%'
        ) matches
        ON matches.belief_id = b.id
        WHERE b.is_draft = FALSE
        ORDER BY b.created_at DESC
        LIMIT ?2 OFFSET ?3
    ",
      )?
      .query_map(
        (
          search.unwrap_or_default(),
          limit as i32,
          offset.unwrap_or(0) as i32,
        ),
        |row| {
          Ok(BeliefRow {
            id: row.get(0)?,
            content: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
          })
        },
      )?
      .collect::<Result<Vec<BeliefRow>, _>>()?;

    if belief_rows.is_empty() {
      return Ok(vec![]);
    }

    let mut belief_ids: Vec<String> = Vec::new();
    let mut beliefs_by_id: HashMap<String, Belief> = HashMap::new();

    for row in belief_rows {
      belief_ids.push(row.id.clone());

      beliefs_by_id.insert(
        row.id.clone(),
        Belief {
          id: row.id,
          content: row.content,
          tags: Vec::new(),
          possible_queries: Vec::new(),
          created_at: row.created_at as u64,
          updated_at: row.updated_at as u64,
        },
      );
    }

    let id_placeholders = std::iter::repeat_n("?", belief_ids.len())
      .collect::<Vec<_>>()
      .join(", ");

    let tags_by_belief = connection
      .prepare(&format!(
        "SELECT belief_id, tag FROM belief_tags WHERE belief_id IN ({})",
        id_placeholders
      ))?
      .query_map(params_from_iter(belief_ids.iter()), |row| {
        Ok((row.get(0)?, row.get(1)?))
      })?
      .collect::<Result<Vec<(String, String)>, _>>()?;

    let queries_by_belief = connection
      .prepare(&format!(
        "SELECT belief_id, query FROM belief_queries WHERE belief_id IN ({})",
        id_placeholders
      ))?
      .query_map(params_from_iter(belief_ids.iter()), |row| {
        Ok((row.get(0)?, row.get(1)?))
      })?
      .collect::<Result<Vec<(String, String)>, _>>()?;

    for tag_row in tags_by_belief {
      if let Some(belief) = beliefs_by_id.get_mut(&tag_row.0) {
        belief.tags.push(tag_row.1);
      }
    }

    for query_row in queries_by_belief {
      if let Some(belief) = beliefs_by_id.get_mut(&query_row.0) {
        belief.possible_queries.push(query_row.1);
      }
    }

    let mut beliefs = beliefs_by_id.into_values().collect::<Vec<_>>();
    beliefs.sort_by_key(|b| b.created_at);

    Ok(beliefs)
  }

  pub fn promote_draft(&self, draft_id: &str) -> AppResult<()> {
    self.connection.lock().unwrap().execute(
      "UPDATE beliefs SET (is_draft = FALSE) WHERE id = ?1",
      [draft_id],
    )?;

    debug!("[BeliefStore] Promoted belief {draft_id} from draft");

    Ok(())
  }

  pub fn flush(config: &Config) -> AppResult<()> {
    let path = get_storage_path(&config.storage.sqlite_file);

    if std::fs::exists(&path)? {
      remove_file(path)?;
    }

    Ok(())
  }
}

struct BeliefRow {
  id: String,
  content: String,
  created_at: i64,
  updated_at: i64,
}
