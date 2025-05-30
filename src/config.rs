use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::path::MAIN_SEPARATOR_STR;

use clap::Parser;
use normalize_path::NormalizePath;
use pathdiff::diff_paths;

use crate::cli::CliCommand;

#[derive(Default, Debug)]
pub struct Config {
  pub serve_dir_abs: PathBuf,
  pub serve_dir_fmt: String,
  pub address: String,
  pub port: usize,
  pub spa: bool,
  pub cors: bool,
  pub compress: bool,
  pub sab: bool,
  pub domain: String,
  pub domain_pretty: String,
  pub headers: HashMap<String, Vec<String>>,
  pub quiet: bool,
  pub watch: bool,
  pub watch_dir: PathBuf,
  pub no_watch_inject: bool,
  pub stream_buffer_size: usize,
}

impl Config {
  pub fn from_cli() -> anyhow::Result<Self> {
    let command = CliCommand::parse();
    let Ok(cwd) = env::current_dir() else {
      return Err(anyhow::anyhow!("Unable to get cwd"));
    };

    let domain = format!("{}:{}", command.address, command.port);
    let mut domain_pretty = domain.clone();
    if command.address == "0.0.0.0" || command.address == "::" {
      // 127.0.0.1 is more stable, localhost it's possible user change hosts file
      domain_pretty = format!("127.0.0.1:{}", command.port)
    }

    let serve_dir_abs: PathBuf;
    let serve_dir_rel: PathBuf;

    if command.serve_dir.is_absolute() {
      let serve_dir = command.serve_dir.normalize();
      serve_dir_abs = serve_dir;
      serve_dir_rel = diff_paths(&serve_dir_abs, &cwd).unwrap();
    } else {
      serve_dir_abs = cwd.join(&command.serve_dir).normalize();
      serve_dir_rel = diff_paths(&serve_dir_abs, &cwd).unwrap();
    }

    let mut headers = HashMap::<String, Vec<String>>::new();

    if command.sab {
      headers.insert(
        "Cross-Origin-Opener-Policy".to_string(),
        vec!["same-origin".to_string()],
      );

      headers.insert(
        "Cross-Origin-Embedder-Policy".to_string(),
        vec!["require-corp".to_string()],
      );
    } else {
      headers.insert(
        "Cross-Origin-Opener-Policy".to_string(),
        vec!["unsafe-none".to_string()],
      );

      headers.insert(
        "Cross-Origin-Embedder-Policy".to_string(),
        vec!["unsafe-none".to_string()],
      );
    }

    if command.cors {
      headers.insert(
        "Access-Control-Allow-Origin".to_string(),
        vec!["*".to_string()],
      );
    } else {
      headers.insert(
        "Access-Control-Allow-Origin".to_string(),
        vec!["null".to_string()],
      );
    }

    if command.cache_time == 0 {
      headers.insert(
        "Cache-Control".to_string(),
        vec![format!("no-cache, no-store, must-revalidate")],
      );
    } else {
      headers.insert(
        "Cache-Control".to_string(),
        vec![format!("max-age={}", command.cache_time)],
      );
    }

    for header in command.headers {
      let Some((key, value)) = header.split_once(":") else {
        return Err(anyhow::anyhow!("Unable to parse header"));
      };
      let key = key.to_string();
      let value = value.to_string();
      headers.entry(key).or_default().push(value);
    }

    Ok(Config {
      serve_dir_fmt: format!(".{}{}", MAIN_SEPARATOR_STR, serve_dir_rel.to_str().unwrap()),
      serve_dir_abs: serve_dir_abs.clone(),
      domain,
      domain_pretty,
      spa: command.spa,
      cors: command.cors,
      compress: command.compress,
      sab: command.sab,
      address: command.address,
      port: command.port,
      headers,
      quiet: command.quiet,
      watch: command.watch,
      watch_dir: command.watch_dir.unwrap_or(serve_dir_abs),
      no_watch_inject: command.no_watch_inject,
      stream_buffer_size: command.stream_buffer_size,
    })
  }
}
