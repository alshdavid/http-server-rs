use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::path::MAIN_SEPARATOR_STR;

use clap::Parser;
use normalize_path::NormalizePath;
use pathdiff::diff_paths;

use crate::cli::CliCommand;

// const VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(unused)]
#[derive(Default, Debug)]
pub struct Config {
  // pub cwd: PathBuf,
  // pub version: String,
  pub serve_dir_abs: PathBuf,
  // pub serve_dir_rel: PathBuf,
  pub serve_dir_fmt: String,
  pub address: String,
  pub port: usize,
  pub spa: bool,
  pub domain: String,
  pub domain_pretty: String,
  // pub cache_time: usize,
  pub headers: HashMap<String, Vec<String>>,
  pub quiet: bool,
  pub watch: bool,
}

impl Config {
  pub fn from_cli() -> Result<Self, String> {
    let command = CliCommand::parse();
    let Ok(cwd) = env::current_dir() else {
      return Err("Unable to get cwd".to_string());
    };

    let domain = format!("{}:{}", command.address, command.port);
    let mut domain_pretty = domain.clone();
    if command.address == "0.0.0.0" {
      domain_pretty = format!("localhost:{}", command.port)
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

    if command.cors {
      headers.insert(
        "Access-Control-Allow-Origin".to_string(),
        vec!["*".to_string()],
      );
    }

    if command.cache_time == 0 || command.no_cache {
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
        return Err("Unable to parse header".to_string());
      };
      let key = key.to_string();
      let value = value.to_string();
      headers.entry(key).or_default().push(value);
    }

    Ok(Config {
      // cwd,
      // version: VERSION.to_string(),
      serve_dir_fmt: format!(".{}{}", MAIN_SEPARATOR_STR, serve_dir_rel.to_str().unwrap()),
      serve_dir_abs,
      // serve_dir_rel,
      domain,
      domain_pretty,
      spa: command.spa,
      address: command.address,
      port: command.port,
      // cache_time: command.cache_time,
      headers,
      quiet: command.quiet,
      watch: command.watch,
    })
  }
}
