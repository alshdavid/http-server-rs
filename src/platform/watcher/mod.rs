use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use colored::Colorize;
use notify_debouncer_full::DebounceEventResult;
use notify_debouncer_full::Debouncer;
use notify_debouncer_full::RecommendedCache;
use notify_debouncer_full::new_debouncer;
use notify_debouncer_full::notify::EventKind;
use notify_debouncer_full::notify::RecommendedWatcher;
use notify_debouncer_full::notify::RecursiveMode;
use tokio::sync::watch;

use crate::platform::logger::Logger;

pub struct WatcherOptions {
  pub target_dir: PathBuf,
  pub logger: Arc<Logger>,
  pub debounce_delay: Duration,
}

#[derive(Clone)]
pub struct Watcher {
  tx_watch: watch::Sender<Vec<PathBuf>>,
  _debouncer: Arc<Debouncer<RecommendedWatcher, RecommendedCache>>,
}

impl Watcher {
  pub fn new(options: WatcherOptions) -> anyhow::Result<Self> {
    let (tx_watch, _) = watch::channel(Vec::<PathBuf>::new());

    let (tx, rx) = std::sync::mpsc::channel::<DebounceEventResult>();

    thread::spawn({
      let tx_watch = tx_watch.clone();
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
            tx_watch.send_replace(paths);
          }
        }
      }
    });

    let mut debouncer = new_debouncer(options.debounce_delay, None, tx)?;
    debouncer.watch(&options.target_dir, RecursiveMode::Recursive)?;

    Ok(Self {
      tx_watch,
      _debouncer: Arc::new(debouncer),
    })
  }

  pub fn subscribe(&self) -> watch::Receiver<Vec<PathBuf>> {
    self.tx_watch.subscribe()
  }
}
