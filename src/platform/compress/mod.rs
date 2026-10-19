use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use brotli as brotli_rs;

struct Entry {
  id: u64,
  version: u64,
  compressed: Arc<[u8]>,
}

pub struct Compressor {
  level: u32,
  entries: Mutex<HashMap<String, Entry>>,
  version: AtomicU64,
}

impl Compressor {
  pub fn new(level: u32) -> Self {
    Self {
      level,
      entries: Mutex::new(HashMap::new()),
      version: AtomicU64::new(0),
    }
  }

  pub fn compress(
    &self,
    key: &str,
    id: u64,
    input: &[u8],
  ) -> Vec<u8> {
    let version = self.version.load(Ordering::Acquire);

    if let Ok(entries) = self.entries.lock()
      && let Some(entry) = entries.get(key)
      && entry.id == id
      && entry.version == version
    {
      return entry.compressed.to_vec();
    }

    let compressed = Arc::<[u8]>::from(brotli(input, self.level));
    self.store(key, id, version, compressed.clone());
    compressed.to_vec()
  }

  /// Compress `input` and cache it for `key`, bypassing any existing entry.
  pub fn compress_and_store(
    &self,
    key: &str,
    id: u64,
    input: &[u8],
  ) -> Vec<u8> {
    let version = self.version.load(Ordering::Acquire);
    let compressed = Arc::<[u8]>::from(brotli(input, self.level));
    self.store(key, id, version, compressed.clone());
    compressed.to_vec()
  }

  pub fn clear(&self) {
    self.version.fetch_add(1, Ordering::AcqRel);
    if let Ok(mut entries) = self.entries.lock() {
      entries.clear();
    }
  }

  fn store(
    &self,
    key: &str,
    id: u64,
    version: u64,
    compressed: Arc<[u8]>,
  ) {
    if let Ok(mut entries) = self.entries.lock() {
      entries.insert(
        key.to_string(),
        Entry {
          id,
          version,
          compressed,
        },
      );
    }
  }
}

pub fn brotli(
  input: &[u8],
  level: u32,
) -> Vec<u8> {
  let mut writer = brotli_rs::CompressorWriter::new(Vec::new(), 4096, level, 22);
  if writer.write_all(input).is_err() {
    return Vec::new();
  }
  writer.into_inner()
}

/// Identity of a file's content: size and modification time.
pub fn id(path: &Path) -> u64 {
  let Ok(metadata) = std::fs::metadata(path) else {
    return 0;
  };

  #[cfg(unix)]
  {
    use std::os::unix::fs::MetadataExt;
    (metadata.mtime() as u64) ^ metadata.len()
  }

  #[cfg(windows)]
  {
    use std::os::windows::fs::MetadataExt;
    metadata.last_write_time() ^ metadata.len()
  }
}

pub type SharedCompressor = Arc<Compressor>;
