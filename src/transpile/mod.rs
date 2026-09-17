use std::path::PathBuf;

pub mod typescript;

#[derive(Debug)]
pub struct TransformerResult {
  pub code: String,
}

pub struct TransformerContext {
  /// The bytes of the input file
  pub content: Vec<u8>,
  /// Path to the source file
  pub path: PathBuf,
  /// The extension of the file being processed
  pub kind: String,
}
