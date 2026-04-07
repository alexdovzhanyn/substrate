mod semantic;
mod beliefs;
mod storage;
mod util;

use std::collections::HashMap;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Tesseract starting...");

    let config = util::Config::load("config/default.toml")?;

    let mut embedding_db = storage::embedding::EmbeddingDB::initialize(&config).await?;
    let kv_store = storage::kv::KVStore::initialize(&config)?;

    loop {
        let action = prompt("[q]uery or [i]nsert? ")?;

        if action == "q" {
            query_beliefs(&mut embedding_db, &kv_store).await?;
        } else {
            create_belief(&mut embedding_db, &kv_store).await?;
        }
    }
}

async fn create_belief(
    embedding_db: &mut storage::embedding::EmbeddingDB,
    kv_store: &storage::kv::KVStore
) -> Result<(), Box<dyn std::error::Error>> {
    println!("======== NEW BELIEF =========");

    let subject = prompt("Subject: ")?;
    let value = prompt("Value: ")?;
    let tags = prompt("Tags: ")?;
    let possible_queries = prompt("Possible queries: ")?;

    let belief = beliefs::belief::Belief {
        id: uuid::Uuid::new_v4().to_string(),
        subject,
        value,
        tags: tags.split(',').map(|s| s.trim().to_string()).collect(),
        possible_queries: possible_queries.split(',').map(|s| s.trim().to_string()).collect()
    };

    embedding_db.insert_embeddings(&belief).await?;
    kv_store.insert_belief(&belief)?;
 
    Ok(())
}

async fn query_beliefs(
    embedding_db: &mut storage::embedding::EmbeddingDB,
    kv_store: &storage::kv::KVStore
) -> Result<(), Box<dyn std::error::Error>> {
    let prompt = prompt("Query: ")?;

    let mut beliefs_by_id = HashMap::new(); 
    for candidate in embedding_db.query_beliefs(prompt).await? {
        if beliefs_by_id.contains_key(&candidate.belief_id) {
            continue;
        } 

        let Some(belief) = kv_store.get_belief(&candidate.belief_id)? else {
            continue;
        };

        beliefs_by_id.insert(candidate.belief_id, belief);
    }  

    let unique_beliefs: Vec<beliefs::belief::Belief> = beliefs_by_id.into_values().collect(); 

    println!("Results: {unique_beliefs:#?}");

    Ok(())
}

fn prompt(label: &str) -> Result<String, io::Error> {
    print!("{label}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}
