use std::collections::HashMap;
use std::env;
use std::hash::Hash;
use std::path::PathBuf;

use clap::Parser;
use normalize_path::NormalizePath;
use pathdiff::diff_paths;

use crate::cli::CliCommand;

#[derive(Default, Debug)]
pub struct Config {
  pub cwd: PathBuf,
  pub serve_dir_abs: PathBuf,
  pub serve_dir_rel: PathBuf,
  pub address: String,
  pub port: usize,
  pub domain: String,
  pub cache_time: usize,
  pub headers: HashMap<String, Vec<String>>,
}

impl Config {
  pub fn from_cli() -> Result<Self, String> {
    let command = CliCommand::parse();
    let Ok(cwd) = env::current_dir() else {
      return Err("Unable to get cwd".to_string());
    };

  dbg!(&command);


    let domain = format!("{}:{}", command.address, command.port);

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

    if !serve_dir_abs.exists() {
      return Err(format!(
        "Serve directory not found: '{}'",
        serve_dir_rel.to_str().unwrap()
      ));
    }

    let mut headers = HashMap::<String, Vec<String>>::new();

    if command.cors {
      headers.insert("Access-Control-Allow-Origin".to_string(), vec!["*".to_string()]);
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
      cwd,
      serve_dir_abs,
      serve_dir_rel,
      domain,
      address: command.address,
      port: command.port,
      cache_time: command.cache_time,
      headers,
    })
  }
}
