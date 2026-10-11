use std::path::PathBuf;

pub mod helpers;
pub mod tsconfig;
pub mod typescript;

#[derive(Debug)]
pub struct TransformerResult {
  pub code: String,
  pub map: Option<String>,
}

pub struct TransformerContext {
  /// The bytes of the input file
  pub content: Vec<u8>,
  /// Path to the source file
  pub path: PathBuf,
  /// URL path the file is served from, used as the source map source
  pub source_url: String,
  /// Whether to generate a source map for the output
  pub source_maps: bool,
  /// The extension of the file being processed
  pub kind: String,
  /// The nearest `tsconfig.json` compiler options, if one was found
  pub tsconfig: tsconfig::TsConfig,
}
