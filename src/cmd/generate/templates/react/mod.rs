use std::fs;
use std::path::Path;

use include_dir::Dir;
use include_dir::include_dir;

static TEMPLATE: Dir<'_> =
  include_dir!("$CARGO_MANIFEST_DIR/src/cmd/generate/templates/react/template");

pub fn generate(target: &Path) -> std::io::Result<()> {
  fs::create_dir_all(target)?;
  TEMPLATE.extract(target)
}
