mod config;
pub mod logging;

pub use config::Config;
use std::fs;

pub fn get_storage_path(filename: &str) -> String {
  let app_support = dirs::data_local_dir()
    .expect("Unable to determine data directory")
    .join("Tesseract");

  fs::create_dir_all(&app_support)
    .expect("Unable to create Tesseract application support directory");

  app_support.join(filename).to_string_lossy().into_owned()
}
