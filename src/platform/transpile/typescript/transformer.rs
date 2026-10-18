use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::thread::JoinHandle;

use crossbeam::channel::Sender;
use tokio::sync::oneshot;

use super::TransformerContext;
use super::TransformerResult;
use super::tsconfig::TsConfigCache;

enum TypeScriptTransformerEvent {
  Transpile {
    content: Vec<u8>,
    path: PathBuf,
    source_url: String,
    kind: String,
    source_maps: bool,
    tx: oneshot::Sender<anyhow::Result<TransformerResult>>,
  },
}

/// The worker threads a [`TypeScriptTransformer`] compiles on, along with the
/// channel that feeds them.
struct TypeScriptTransformerWorker {
  tx: Sender<TypeScriptTransformerEvent>,
  _handles: Vec<JoinHandle<()>>,
}

/// Transpiles TypeScript and TSX for a served directory.
///
/// The transformer owns reading the source, resolving the nearest `tsconfig`
/// and building the source map, so a caller only has to name the file it wants
/// rendered.
pub struct TypeScriptTransformer {
  root_dir: PathBuf,
  tsconfig_cache: Arc<TsConfigCache>,
  worker_threads: usize,
  worker: OnceLock<io::Result<TypeScriptTransformerWorker>>,
}

impl TypeScriptTransformer {
  pub fn new(
    root_dir: &Path,
    worker_threads: usize,
  ) -> Self {
    Self {
      root_dir: root_dir.to_path_buf(),
      tsconfig_cache: Arc::new(TsConfigCache::new()),
      worker_threads: worker_threads.max(1),
      worker: OnceLock::new(),
    }
  }

  /// The name of the swc helper a request path names, if it names one.
  ///
  /// Helpers are not files on disk, so they are served by the call site before
  /// the request is resolved.
  pub fn helper_module_name(path: &str) -> Option<&str> {
    super::helpers::module_name(path)
  }

  /// One swc helper, plus its dependencies, as an ES module.
  pub fn helper_module_source(name: &str) -> Option<String> {
    super::helpers::module_source(name)
  }

  /// Transpile the file at `path`.
  ///
  /// Returns `None` when the file could not be read, so a caller can fall back
  /// to serving something else.
  pub async fn transpile(
    &self,
    path: &Path,
    source_url: &str,
    kind: &str,
    source_maps: bool,
  ) -> anyhow::Result<Option<TransformerResult>> {
    let Ok(content) = tokio::fs::read(path).await else {
      return Ok(None);
    };

    let (tx, rx) = oneshot::channel();

    self
      .worker()?
      .tx
      .send(TypeScriptTransformerEvent::Transpile {
        content,
        path: path.to_path_buf(),
        source_url: source_url.to_string(),
        kind: kind.to_string(),
        source_maps,
        tx,
      })
      .map_err(|_| io::Error::other("the TypeScript transpiler is not running"))?;

    match rx.await {
      Ok(result) => result.map(Some),
      Err(error) => Err(io::Error::other(error).into()),
    }
  }

  fn worker(&self) -> io::Result<&TypeScriptTransformerWorker> {
    self
      .worker
      .get_or_init(|| {
        let (tx, rx) = crossbeam::channel::unbounded::<TypeScriptTransformerEvent>();

        let tsconfig_cache = Arc::clone(&self.tsconfig_cache);
        let root_dir = self.root_dir.clone();

        let mut handles = Vec::<JoinHandle<()>>::new();

        for i in 0..self.worker_threads {
          let rx = rx.clone();
          let tsconfig_cache = Arc::clone(&tsconfig_cache);
          let root_dir = root_dir.clone();

          handles.push(
            std::thread::Builder::new()
              .name(format!("typescript-transformer-{}", i))
              .spawn(move || {
                while let Ok(TypeScriptTransformerEvent::Transpile {
                  content,
                  path,
                  source_url,
                  kind,
                  source_maps,
                  tx,
                }) = rx.recv()
                {
                  let tsconfig = tsconfig_cache.resolve(&path, &root_dir);

                  let _ = tx.send(super::transpile::transpile(TransformerContext {
                    content,
                    path,
                    source_url,
                    source_maps,
                    kind,
                    tsconfig,
                  }));
                }
              })?,
          );
        }

        Ok(TypeScriptTransformerWorker {
          tx,
          _handles: handles,
        })
      })
      .as_ref()
      .map_err(|error| io::Error::new(error.kind(), error.to_string()))
  }
}
