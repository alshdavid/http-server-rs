use std::{os::unix::fs::PermissionsExt, path::Path};
use handlebars::Handlebars;
use chrono::{DateTime, Utc}; 
use serde_json::json;
use unix_mode;
use std::fs;

use crate::config::Config;

const DIR_PAGE: &str = include_str!("./dir.hbs");

pub fn render_directory_explorer(config: &Config, req_uri: &str, file_path: &Path) -> Result<String, String> {
  let dir = self::fs::read_dir(&file_path).unwrap();
  let mut files = Vec::<(String, String, String)>::new();
  let mut folders = Vec::<(String, String, String)>::new();
  
  for item in dir {
    let item = item.unwrap();
    let meta = item.metadata().unwrap();
    let meta_mode = self::unix_mode::to_string(meta.permissions().mode());
    let last_modified: DateTime<Utc> =  meta.modified().unwrap().into();

    let rel_path = pathdiff::diff_paths(item.path(), &config.serve_dir_abs).unwrap();
    let rel_path_str = rel_path.to_str().unwrap();
    if item.file_type().unwrap().is_dir() {
      folders.push((format!("{}", meta_mode), format!("{}", last_modified.format("%d %b %Y %H:%M")), rel_path_str.to_string()));
    } else {
      files.push((format!("{}", meta_mode), format!("{}", last_modified.format("%d %b %Y %H:%M")), rel_path_str.to_string()));
    };
  }

  let handlebars = Handlebars::new();
  let output = handlebars.render_template(DIR_PAGE, &json!({
    "path": req_uri.clone(),
    "files": files,
    "folders": folders,
    "address": config.address.clone(),
    "port": config.port.clone(),
  })).unwrap();

  Ok(output)
}