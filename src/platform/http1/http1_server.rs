use std::convert::Infallible;
use std::future::Future;
use std::sync::Arc;

use http_body_util::Full;
use http_body_util::combinators::BoxBody;
use hyper::Request;
use hyper::Response;
use hyper::body::Bytes as HyperBytes;
use hyper::body::Incoming;
use hyper::http::response::Builder as ResponseBuilder;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use hyper_util::server::graceful::GracefulShutdown;
use tokio::net::TcpListener;
use tokio::net::ToSocketAddrs;

/// Simple wrapper around hyper to make it a little nicer to use.
///
/// The returned future resolves once the listener stops and every in-flight
/// connection has drained.
pub async fn http1_server<F, Fut, A>(
  addr: A,
  shutdown: impl Future<Output = ()>,
  handle_func: F,
) -> anyhow::Result<()>
where
  A: ToSocketAddrs,
  F: 'static + Send + Sync + Fn(Request<Incoming>, ResponseBuilder) -> Fut,
  Fut: Send + Future<Output = anyhow::Result<Response<BoxBody<HyperBytes, Infallible>>>>,
{
  let listener = TcpListener::bind(&addr).await?;
  let handler_func_ref = Arc::new(handle_func);
  let graceful = GracefulShutdown::new();
  tokio::pin!(shutdown);

  loop {
    tokio::select! {
      _ = &mut shutdown => break,
      accepted = listener.accept() => {
        let Ok((stream, _)) = accepted else {
          continue;
        };
        let io = TokioIo::new(stream);
        let handler_func_ref = handler_func_ref.clone();

        let service_handler = service_fn(move |req| {
          let fut = handler_func_ref(req, Response::builder());

          async move {
            let handler_response = match fut.await {
              Ok(handler_response) => handler_response,
              Err(handler_error) => handle_error(handler_error),
            };

            Ok::<Response<BoxBody<HyperBytes, Infallible>>, anyhow::Error>(handler_response)
          }
        });

        let connection = http1::Builder::new().serve_connection(io, service_handler);

        tokio::spawn(graceful.watch(connection));
      }
    }
  }

  graceful.shutdown().await;

  Ok(())
}

fn handle_error(error: anyhow::Error) -> Response<BoxBody<HyperBytes, Infallible>> {
  let content = HyperBytes::from(format!("{}", error));
  let body = BoxBody::new(Full::new(content));
  let response = Response::builder().status(500).body(body);

  let Ok(response) = response else { todo!() };

  response
}
