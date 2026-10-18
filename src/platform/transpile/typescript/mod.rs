use std::path::PathBuf;

mod helpers;
mod transformer;
mod transpile;
mod tsconfig;

pub use transformer::TypeScriptTransformer;

#[derive(Debug)]
pub struct TransformerResult {
  pub code: String,
  pub map: Option<String>,
}

struct TransformerContext {
  pub content: Vec<u8>,
  pub path: PathBuf,
  /// URL path the file is served from, used as the source map source
  pub source_url: String,
  pub source_maps: bool,
  /// The extension of the file being processed
  pub kind: String,
  /// The nearest `tsconfig.json` or `deno.json` compiler options, if one was found
  pub tsconfig: tsconfig::TsConfig,
}
