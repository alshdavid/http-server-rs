use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

use unix_mode;

pub fn get_meta_mode(meta: &Metadata) -> String {
  self::unix_mode::to_string(meta.permissions().mode())
}

pub fn get_meta_size(meta: &Metadata) -> String {
  let size = meta.size();
  if size >= 1000 {
    return format!("{} kb", meta.size() / 1000);
  }
  if size >= (1000 * 1000) {
    return format!("{} mb", meta.size() / (1000 * 1000));
  }
  if size >= (1000 * 1000 * 1000) {
    return format!("{} gb", meta.size() / (1000 * 1000 * 1000));
  }
  if size >= (1000 * 1000 * 1000 * 1000) {
    return format!("{} tb", meta.size() / (1000 * 1000 * 1000 * 1000));
  }

  format!("{} b ", size)
}
