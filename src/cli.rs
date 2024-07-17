use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct CliCommand {
  /// Target directory to serve
  #[arg(default_value = "./dist")]
  pub serve_dir: PathBuf,

  #[arg(short = 'a', long = "address", default_value = "0.0.0.0")]
  pub address: String,

  #[arg(short = 'p', long = "port", default_value = "8080")]
  pub port: usize,

  /// Cache control time
  #[arg(short = 'c', long = "cache-time", default_value = "3600")]
  pub cache_time: usize,

  /// Custom headers (Format "key:value")
  #[arg(short = 'H', long = "header")]
  pub headers: Vec<String>,

  /// Enable CORS header
  #[arg(long = "cors")]
  pub cors: bool,

  /// Disable cache control header
  #[arg(long = "no-cache")]
  pub no_cache: bool,

  /// Don't print any logs to terminal
  #[arg(short = 'Q', long = "quiet")]
  pub quiet: bool,
}
