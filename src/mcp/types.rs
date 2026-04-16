use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IngestMarkdownPromptParams {
  /// The markdown file to ingest
  pub markdown_file_path: String,
}
