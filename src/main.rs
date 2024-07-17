mod cli;
mod config;
mod explorer;

use std::convert::Infallible;
use std::fs;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use explorer::render_directory_explorer;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Request;
use hyper::Response;
use hyper_util::rt::TokioIo;
use mime_guess;
use shared_string::SharedString;
use tokio::net::TcpListener;

use crate::config::Config;

fn main() -> Result<(), String> {
  let config = Arc::new(Config::from_cli()?);
  if !config.quiet {
    println!("Serving on http://{}", config.domain);
  }

  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .worker_threads(num_cpus::get_physical())
    .build()
    .unwrap()
    .block_on(async {
      let listener = TcpListener::bind(&config.domain).await.unwrap();

      loop {
        let config = config.clone();
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
          http1::Builder::new()
            .serve_connection(io, service_fn(server(config)))
            .await
            .unwrap();
        });
      }
    });

  Ok(())
}

fn server(
  config: Arc<Config>
) -> Box<
  dyn Send
    + Fn(
      Request<Incoming>,
    ) -> Pin<Box<dyn Send + Future<Output = Result<Response<Full<Bytes>>, Infallible>>>>,
> {
  Box::new(move |req| {
    Box::pin({
      let mut res = Response::builder();
      let config = config.clone();
      async move {
        let req_uri = req.uri().to_string();

        // Trim the leading "/" from the URI
        let req_path = SharedString::from(req_uri.as_str())
          .get(1..)
          .unwrap()
          .to_string();

        // Guess the file path of the file to serve
        let mut file_path = config.serve_dir_abs.join(req_path.clone());

        // Try to serve index.html
        if file_path.is_dir() && file_path.join("index.html").exists() {
          file_path = file_path.join("index.html");
        }

        // Apply custom headers
        for (key, values) in config.headers.iter() {
          for value in values.iter() {
            res = res.header(key, value);
          }
        }

        // Serve folder structure
        if file_path.is_dir() {
          let output = render_directory_explorer(&config, &req_path, &file_path).unwrap();
          res = res.header("Content-Type", "text/html");
          return Ok(res.body(Full::new(Bytes::from(output))).unwrap());
        }

        // 404 if no file exists
        if !file_path.exists() {
          if !config.quiet {
            println!("  [404] {}", req.uri());
          }

          return Ok(
            res
              .status(404)
              .body(Full::new(Bytes::from("File not found")))
              .unwrap(),
          );
        }

        // Apply mime type
        if let Some(mime) = self::mime_guess::from_path(&file_path).first() {
          res = res.header("Content-Type", mime.to_string());
        }

        // Read file
        // TODO not sure why tokio file read doesn't work here
        let Ok(file) = fs::read(&file_path) else {
          return Ok(
            res
              .status(500)
              .body(Full::new(Bytes::from("Unable to read file")))
              .unwrap(),
          );
        };

        if !config.quiet {
          println!("  [200] {}", req.uri());
        }

        let resp = res.body(Full::new(Bytes::from(file))).unwrap();
        Ok(resp)
      }
    })
  })
}
