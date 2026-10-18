use std::io;
use std::path::Path;
use std::path::PathBuf;

use normalize_path::NormalizePath;

/// The file system `grass` resolves `@use`/`@import` through.
///
/// `grass` resolves imports itself rather than going through the request
/// resolver, so confinement to the served directory has to be enforced here.
/// Without it a stylesheet could read any path on the host, for example
/// `@use "../../../../etc/passwd"`.
pub struct GrassFs {
  root_dir: PathBuf,
}

impl std::fmt::Debug for GrassFs {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    f.debug_tuple("GrassFs").finish()
  }
}

impl GrassFs {
  pub fn new(root_dir: &Path) -> Self {
    Self {
      root_dir: root_dir.to_path_buf(),
    }
  }

  fn is_allowed(
    &self,
    path: &Path,
  ) -> bool {
    path.normalize().starts_with(&self.root_dir)
  }

  /// Refuse reads outside the served directory with the error `grass` expects
  /// for a missing file, so the import simply fails to resolve.
  fn guard(
    &self,
    path: &Path,
  ) -> io::Result<()> {
    if self.is_allowed(path) {
      return Ok(());
    }

    Err(io::Error::new(
      io::ErrorKind::NotFound,
      format!("{} is outside of the served directory", path.display()),
    ))
  }
}

impl grass::Fs for GrassFs {
  fn is_dir(
    &self,
    path: &Path,
  ) -> bool {
    self.is_allowed(path) && path.is_dir()
  }

  fn is_file(
    &self,
    path: &Path,
  ) -> bool {
    self.is_allowed(path) && path.is_file()
  }

  fn read(
    &self,
    path: &Path,
  ) -> io::Result<Vec<u8>> {
    self.guard(path)?;

    std::fs::read(path)
  }

  fn canonicalize(
    &self,
    path: &Path,
  ) -> io::Result<PathBuf> {
    self.guard(path)?;

    std::fs::canonicalize(path)
  }
}
