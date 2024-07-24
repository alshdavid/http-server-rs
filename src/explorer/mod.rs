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

pub fn render_directory_explorer(
  config: &Config,
  req_uri: &str,
  file_path: &Path,
) -> Result<String, String> {
  let dir_path = PathBuf::from(file_path);
  let dir = self::fs::read_dir(&dir_path).unwrap();
  let mut files = Vec::<(String, String, String, String, String, String)>::new();
  let mut folders = Vec::<(String, String, String, String)>::new();

  for item in dir {
    let item = item.unwrap();
    let meta = item.metadata().unwrap();
    let meta_mode = get_meta_mode(&meta);
    let last_modified: DateTime<Utc> = meta.modified().unwrap().into();

    let abs_path = pathdiff::diff_paths(item.path(), &config.serve_dir_abs).unwrap();
    let rel_path = pathdiff::diff_paths(item.path(), &config.serve_dir_abs.join(req_uri)).unwrap();
    let rel_path_str = rel_path.to_str().unwrap();

    if item.file_type().unwrap().is_dir() {
      folders.push((
        format!("{}", meta_mode),
        format!("{}", last_modified.format("%d %b %Y %H:%M")),
        abs_path.to_str().unwrap().to_string(),
        rel_path_str.to_string(),
      ));
    } else {
      let filename = PathBuf::from(item.file_name().into_string().unwrap());
      let file_extension = filename.extension().unwrap().to_str().unwrap().to_string();
      let size = get_meta_size(&meta);
      files.push((
        file_extension,
        format!("{}", meta_mode),
        format!("{}", last_modified.format("%d %b %Y %H:%M")),
        format!("{}", size),
        abs_path.to_str().unwrap().to_string(),
        rel_path_str.to_string(),
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

  let handlebars = Handlebars::new();
  let output = handlebars
    .render_template(
      DIR_PAGE,
      &json!({
        "parent": parent,
        "path": req_uri,
        "files": files,
        "folders": folders,
        "address": config.address.clone(),
        "port": config.port.clone(),
      }),
    )
    .unwrap();

  Ok(output)
}
