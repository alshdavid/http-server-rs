use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

use unix_mode;

pub fn get_meta_mode(meta: &Metadata) -> String {
  self::unix_mode::to_string(meta.permissions().mode())
}

pub fn get_meta_size(meta: &Metadata) -> String {
  format!("{}", meta.size())
}
