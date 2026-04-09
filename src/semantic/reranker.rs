use ndarray::Array2;
use ort::inputs;
use ort::session::Session;
use ort::value::Tensor;
use std::io::{self, Write};
use tokenizers::Tokenizer;

use crate::error::AppResult;

pub struct Reranker {
  tokenizer: tokenizers::Tokenizer,
  session: ort::session::Session,
}

impl Reranker {
  pub fn initialize() -> AppResult<Self> {
    print!("> Initializing reranker...");
    io::stdout().flush()?;

    let tokenizer = Tokenizer::from_file(Self::get_relative_path(
      "models/bge-reranker-base/tokenizer.json",
    ))?;
    let session = Session::builder()?.commit_from_file(Self::get_relative_path(
      "models/bge-reranker-base/onnx/model.onnx",
    ))?;

    println!(" Done.");
    io::stdout().flush()?;

    Ok(Self { tokenizer, session })
  }

  pub fn score(&mut self, query: &str, passage: &str) -> AppResult<f32> {
    let encoding = self.tokenizer.encode((query, passage), true)?;

    let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&x| x as i64).collect();
    let attention_mask: Vec<i64> = encoding
      .get_attention_mask()
      .iter()
      .map(|&x| x as i64)
      .collect();

    let seq_len = input_ids.len();

    let outputs = self.session.run(inputs![
            "input_ids" => Tensor::from_array(Array2::from_shape_vec((1, seq_len), input_ids)?)?,
            "attention_mask" => Tensor::from_array(Array2::from_shape_vec((1, seq_len), attention_mask)?)?,
        ])?;

    let (_shape, data) = outputs[0].try_extract_tensor::<f32>()?;
    let score = *data.first().ok_or("Empty reranker output")?;

    println!("Query: {query}\nPassage: {passage}\nScore: {score}\n\n");

    Ok(score)
  }

  fn get_relative_path(path: &str) -> std::path::PathBuf {
    std::env::current_exe()
      .unwrap()
      .parent()
      .unwrap()
      .join(path)
  }
}
