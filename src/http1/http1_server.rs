use std::convert::Infallible;
use std::future::Future;

use http_body_util::combinators::BoxBody;
use http_body_util::Full;
use hyper::body::Bytes as HyperBytes;
use hyper::body::Incoming;
use hyper::http::response::Builder as ResponseBuilder;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Request;
use hyper::Response;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::net::ToSocketAddrs;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::oneshot::channel as oneshot_channel;
use tokio::sync::oneshot::Sender as OneshotSender;

/// Simple wrapper around hyper to make it a little nicer to use
pub async fn http1_server<F, Fut, A>(
  addr: A,
  handle_func: F,
) -> anyhow::Result<()>
where
  A: ToSocketAddrs,
  F: 'static + Send + Fn(Request<Incoming>, ResponseBuilder) -> Fut,
  Fut: Send + Future<Output = anyhow::Result<Response<BoxBody<HyperBytes, Infallible>>>>,
{
  let listener = TcpListener::bind(&addr).await?;

  let (tx, mut rx) = unbounded_channel::<(
    Request<Incoming>,
    OneshotSender<anyhow::Result<Response<BoxBody<HyperBytes, Infallible>>>>,
  )>();

  tokio::task::spawn(async move {
    while let Some((req, tx_res)) = rx.recv().await {
      let res = Response::builder();
      tx_res.send(handle_func(req, res).await).unwrap();
    }
  });

  loop {
    // let config = config.clone();
    let tx = tx.clone();
    let (stream, _) = listener.accept().await.unwrap();
    let io = TokioIo::new(stream);

    tokio::task::spawn({
      async move {
        http1::Builder::new()
          .serve_connection(
            io,
            service_fn({
              move |req| {
                let tx = tx.clone();

                async move {
                  let (tx_rex, rx_res) = oneshot_channel();
                  if let Err(_err) = tx.send((req, tx_rex)) {
                    return Err("Unable to handle request".to_string());
                  };
                  let Ok(res) = rx_res.await else {
                    return Err("Unable to handle request".to_string());
                  };
                  let res = match res {
                    Ok(res) => res,
                    Err(err) => Response::builder()
                      .status(500)
                      .body(BoxBody::new(Full::new(HyperBytes::from(format!(
                        "{}",
                        err
                      )))))
                      .unwrap(),
                  };

                  Ok(res)
                }
              }
            }),
          )
          .await
          .unwrap();
      }
    });
  }
}
