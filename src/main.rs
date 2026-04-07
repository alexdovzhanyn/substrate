mod semantic;
mod beliefs;
mod storage;

use std::io;

fn main() {
  println!("Tesseract starting...");

  loop {
      test_io_embeddings()
  }
}

fn test_io_embeddings() {
    let mut input = String::new();

    println!("Enter text for embedding: ");

    if let Err(e) = io::stdin().read_line(&mut input) {
        eprintln!("Failed to read input: {}", e);
        return;
    }

    let mut embedding_resolver = match semantic::resolver::Resolver::initialize() {
        Ok(resolver) => resolver,
        Err(error) => panic!("Unable to initialize embedding resolver: {error:?}"),
    };

    println!("You entered: {}", input.trim());

    match embedding_resolver.embed(input.trim().to_string()) {
        Ok(embedding) => {
            println!("Embedded as: {:?}", embedding);
        },
        Err(error) => eprintln!("Failed to create embedding :("),
    };
}
