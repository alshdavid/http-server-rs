use std::sync::Arc;

use tokio::sync::watch;

/// Shared shutdown handle. Dropping every clone does not signal.
#[derive(Clone)]
pub struct ShutdownSignal(Arc<watch::Sender<bool>>);

impl ShutdownSignal {
  pub fn new() -> Self {
    Self(Arc::new(watch::Sender::new(false)))
  }

  /// Signals shutdown and wakes every subscriber.
  pub fn trigger(&self) {
    self.0.send_replace(true);
  }

  /// Resolves once shutdown has been triggered.
  pub async fn wait(&self) {
    let mut rx = self.0.subscribe();
    while rx.wait_for(|shutting_down| *shutting_down).await.is_ok() {}
  }

  /// Owned variant of [`Self::wait`], for moving into stream adaptors.
  pub async fn wait_owned(self) {
    self.wait().await
  }
}
