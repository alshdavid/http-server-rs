#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use chrono::DateTime;
use chrono::Utc;
use handlebars::Handlebars;
use normalize_path::NormalizePath;
use serde_json::json;

#[cfg(unix)]
use self::unix::get_meta_mode;
#[cfg(unix)]
use self::unix::get_meta_size;
#[cfg(windows)]
use self::windows::get_meta_mode;
#[cfg(windows)]
use self::windows::get_meta_size;
use crate::config::Config;

const DIR_PAGE: &str = include_str!("./dir.hbs");
const RELOAD_SCRIPT: &str = include_str!("./reload.mjs");

pub fn reload_script() -> &'static str {
  RELOAD_SCRIPT
}

pub fn render_directory_explorer(
  config: &Config,
  req_uri: &str,
  file_path: &Path,
) -> anyhow::Result<String> {
  let dir_path = PathBuf::from(file_path);
  let dir = fs::read_dir(&dir_path)?;
  let mut files = Vec::<(String, String, String, String, String, String)>::new();
  let mut folders = Vec::<(String, String, String, String)>::new();

  for item in dir {
    let Ok(item) = item else {
      return Ok("Access error".to_string());
    };

    let meta = item.metadata()?;
    let meta_mode = get_meta_mode(&meta);
    let last_modified: DateTime<Utc> = meta.modified()?.into();

    let Some(abs_path) = pathdiff::diff_paths(item.path(), &config.serve_dir_abs) else {
      return Ok(format!(
        "Unable to diff path (absolute) \n\t{:?}\n\t{:?}",
        item.path(),
        config.serve_dir_abs
      ));
    };

    let Some(rel_path) = pathdiff::diff_paths(item.path(), config.serve_dir_abs.join(req_uri))
    else {
      return Ok(format!(
        "Unable to diff path (relative) \n\t{:?}\n\t{:?}",
        item.path(),
        config.serve_dir_abs
      ));
    };

    let abs_path_str = abs_path.to_str().unwrap().to_string();
    let rel_path_str = rel_path.to_str().unwrap().to_string();

    if item.file_type()?.is_dir() {
      folders.push((
        meta_mode.to_string(),
        format!("{}", last_modified.format("%d %b %Y %H:%M")),
        abs_path_str,
        rel_path_str,
      ));
    } else {
      let filename = PathBuf::from(item.file_name().into_string().unwrap());
      let file_extension = match filename.extension() {
        Some(ext) => ext.to_str().unwrap().to_string(),
        None => "".to_string(),
      };

      let size = get_meta_size(&meta);
      files.push((
        file_extension,
        meta_mode.to_string(),
        format!("{}", last_modified.format("%d %b %Y %H:%M")),
        size.to_string(),
        abs_path_str,
        rel_path_str,
      ));
    };
  }

  let mut parent = None::<String>;
  if let Some(up_one) = dir_path.parent() {
    if dir_path != config.serve_dir_abs.normalize() {
      let diff = pathdiff::diff_paths(up_one, &config.serve_dir_abs).unwrap();
      let diff_str = diff.to_str().unwrap();
      parent = Some(format!("/{}", diff_str));
    }
  }

  folders.sort_by(|a, b| a.3.to_lowercase().cmp(&b.3.to_lowercase()));
  files.sort_by(|a, b| a.5.to_lowercase().cmp(&b.5.to_lowercase()));

  let handlebars = Handlebars::new();
  let Ok(output) = handlebars.render_template(
    DIR_PAGE,
    &json!({
      "parent": parent,
      "path": req_uri,
      "files": files,
      "folders": folders,
      "address": config.address.clone(),
      "port": config.port.clone(),
    }),
  ) else {
    return Ok("Unable to render page".to_string());
  };

  Ok(output)
}
