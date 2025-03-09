use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use colored::Colorize;
use notify_debouncer_full::new_debouncer;
use notify_debouncer_full::notify::EventKind;
use notify_debouncer_full::notify::RecommendedWatcher;
use notify_debouncer_full::notify::RecursiveMode;
use notify_debouncer_full::DebounceEventResult;
use notify_debouncer_full::Debouncer;
use notify_debouncer_full::RecommendedCache;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::logger::Logger;
use crate::utils::broadcast::BroadcastChannel;

pub struct WatcherOptions {
  pub target_dir: PathBuf,
  pub logger: Arc<Logger>,
}

#[derive(Clone)]
pub struct Watcher {
  trx_watch: Arc<BroadcastChannel<Vec<PathBuf>>>,
  _debouncer: Arc<Debouncer<RecommendedWatcher, RecommendedCache>>,
}

impl Watcher {
  pub fn new(options: WatcherOptions) -> anyhow::Result<Self> {
    let trx_watch = Arc::new(BroadcastChannel::<Vec<PathBuf>>::new());

    let (tx, rx) = std::sync::mpsc::channel::<DebounceEventResult>();

    thread::spawn({
      let trx_watch = trx_watch.clone();
      let logger = options.logger.clone();

      move || {
        while let Ok(Ok(mut result)) = rx.recv() {
          let mut paths = vec![];

          while let Some(ev) = result.pop() {
            match ev.event.kind {
              EventKind::Create(_) => {}
              EventKind::Modify(_) => {}
              EventKind::Remove(_) => {}
              _ => continue,
            }
            paths.extend(ev.paths.clone());
          }

          if !paths.is_empty() {
            logger.println(format!("{}", "[CNG] ".yellow().bold()));
            trx_watch.send(paths).unwrap();
          }
        }
      }
    });

    let mut debouncer = new_debouncer(Duration::from_millis(1000), None, tx)?;
    debouncer.watch(&options.target_dir, RecursiveMode::Recursive)?;

    Ok(Self {
      trx_watch,
      _debouncer: Arc::new(debouncer),
    })
  }

  pub fn subscribe(&self) -> UnboundedReceiver<Vec<PathBuf>> {
    self.trx_watch.subscribe()
  }
}
