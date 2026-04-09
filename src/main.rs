mod beliefs;
mod error;
mod semantic;
mod util;

use std::io::{self, Write};

use crate::error::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
    println!("Tesseract starting...");

    let config = util::Config::load("config/default.toml")?;

    let mut semantic_index = semantic::SemanticIndex::initialize(&config).await?;
    let belief_store = beliefs::BeliefStore::initialize(&config)?;

    loop {
        let action = prompt("[q]uery or [i]nsert? ")?;

        if action == "q" {
            query_beliefs(&mut semantic_index, &belief_store).await?;
        } else {
            create_belief(&mut semantic_index, &belief_store).await?;
        }
    }
}

async fn create_belief(
    semantic_index: &mut semantic::SemanticIndex,
    belief_store: &beliefs::BeliefStore,
) -> AppResult<()> {
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
        possible_queries: possible_queries
            .split(',')
            .map(|s| s.trim().to_string())
            .collect(),
    };

    semantic_index.insert_belief_embeddings(&belief).await?;
    belief_store.insert(&belief)?;

    Ok(())
}

async fn query_beliefs(
    semantic_index: &mut semantic::SemanticIndex,
    belief_store: &beliefs::BeliefStore,
) -> AppResult<()> {
    let prompt = prompt("Query: ")?;

    let beliefs = semantic_index.query_beliefs(prompt, &belief_store).await?;

    println!("Results: {beliefs:#?}");

    Ok(())
}

fn prompt(label: &str) -> Result<String, io::Error> {
    print!("{label}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}
