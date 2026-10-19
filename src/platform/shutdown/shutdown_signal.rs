/// Resolves on Ctrl+C, or on SIGTERM where the platform provides it.
pub async fn shutdown_signal() {
  let ctrl_c = async {
    tokio::signal::ctrl_c().await.ok();
  };

  #[cfg(unix)]
  let terminate = async {
    if let Ok(mut signal) =
      tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
    {
      signal.recv().await;
    } else {
      std::future::pending::<()>().await;
    }
  };

  #[cfg(not(unix))]
  let terminate = std::future::pending::<()>();

  tokio::select! {
    _ = ctrl_c => {},
    _ = terminate => {},
  }
}
