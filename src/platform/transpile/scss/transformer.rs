use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::thread::JoinHandle;

use crossbeam::channel::Sender;
use grass::OutputStyle;
use tokio::sync::oneshot;

use super::fs::GrassFs;

struct ScssTransformerEvent {
  path: PathBuf,
  kind: String,
  tx: oneshot::Sender<anyhow::Result<String>>,
}

/// The worker threads a [`ScssTransformer`] compiles on, along with the channel
/// that feeds them.
struct ScssTransformerWorker {
  tx: Sender<ScssTransformerEvent>,
  _handles: Vec<JoinHandle<()>>,
}

/// Compiles SCSS and Sass for a served directory.
///
/// The transformer reads the source, compiles it and rewrites import specifiers
/// so the output can be requested from the server, leaving the caller to name
/// the file it wants rendered.
pub struct ScssTransformer {
  root_dir: PathBuf,
  output_style: OutputStyle,
  worker_threads: usize,
  worker: OnceLock<io::Result<ScssTransformerWorker>>,
}

impl ScssTransformer {
  pub fn new(
    root_dir: &Path,
    output_style: OutputStyle,
    worker_threads: usize,
  ) -> Self {
    Self {
      root_dir: root_dir.to_path_buf(),
      output_style,
      worker_threads: worker_threads.max(1),
      worker: OnceLock::new(),
    }
  }

  pub async fn transpile(
    &self,
    path: &Path,
    kind: &str,
  ) -> anyhow::Result<String> {
    let (tx, rx) = oneshot::channel();

    self
      .worker()?
      .tx
      .send(ScssTransformerEvent {
        path: path.to_path_buf(),
        kind: kind.to_string(),
        tx,
      })
      .map_err(|_| io::Error::other("the SCSS transpiler is not running"))?;

    match rx.await {
      Ok(result) => result,
      Err(error) => Err(io::Error::other(error).into()),
    }
  }

  fn worker(&self) -> io::Result<&ScssTransformerWorker> {
    self
      .worker
      .get_or_init(|| {
        let (tx, rx) = crossbeam::channel::unbounded::<ScssTransformerEvent>();

        let root_dir = self.root_dir.clone();
        let output_style = self.output_style;

        let mut handles = Vec::<JoinHandle<()>>::new();

        for i in 0..self.worker_threads {
          let rx = rx.clone();
          let root_dir = root_dir.clone();

          handles.push(
            std::thread::Builder::new()
              .name(format!("scss-transformer-{}", i))
              .spawn(move || {
                let fs = GrassFs::new(&root_dir);

                while let Ok(ScssTransformerEvent { path, kind, tx }) = rx.recv() {
                  let _ = tx.send('block: {
                    let mut code =
                      match super::transpile::transpile(&fs, &path, &kind, output_style) {
                        Ok(code) => code,
                        Err(error) => break 'block Err(error),
                      };

                    for (specifier, replacement) in
                      super::transpile::candidates(&fs, &code, &path, &root_dir)
                    {
                      code =
                        code.replace(&format!("\"{specifier}\""), &format!("\"{replacement}\""));
                      code = code.replace(&format!("'{specifier}'"), &format!("'{replacement}'"));
                    }

                    Ok(code)
                  });
                }
              })?,
          );
        }

        Ok(ScssTransformerWorker {
          tx,
          _handles: handles,
        })
      })
      .as_ref()
      .map_err(|error| io::Error::new(error.kind(), error.to_string()))
  }
}
