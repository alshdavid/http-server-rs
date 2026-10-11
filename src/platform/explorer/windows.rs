use std::fs::Metadata;
use std::os::windows::fs::MetadataExt;

pub fn get_meta_mode(_meta: &Metadata) -> String {
  "----------".to_string()
}

pub fn get_meta_size(meta: &Metadata) -> String {
  format!("{}", meta.file_size())
}
