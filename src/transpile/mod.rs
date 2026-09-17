use std::path::PathBuf;

pub mod tsconfig;
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
  /// The nearest `tsconfig.json` compiler options, if one was found
  pub tsconfig: tsconfig::TsConfig,
}
