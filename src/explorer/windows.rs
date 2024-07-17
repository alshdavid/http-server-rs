use std::fs::Metadata;
use std::os::windows::fs::MetadataExt;

use chrono::DateTime;
use chrono::Utc;
use unix_mode;

pub fn get_meta_mode(meta: &Metadata) -> String {
  "----------".to_string()
}

pub fn get_meta_size(meta: &Metadata) -> String {
  format!("{}", meta.file_size())
}
