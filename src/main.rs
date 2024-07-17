mod cli;
mod config;

use std::convert::Infallible;
use std::fs;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

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
// use tokio::fs;
use tokio::net::TcpListener;

use crate::config::Config;

fn main() -> Result<(), String> {
  let config = Arc::new(Config::from_cli()?);
  dbg!(&config);

  println!("Serving on http://{}", config.domain);

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
        // Trim the leading "/" from the URI
        let req_path = SharedString::from(req.uri().to_string().as_str())
          .get(1..)
          .unwrap();

        // Guess the file path of the file to serve
        let mut file_path = config.serve_dir_abs.join(req_path.to_string());

        // Try to serve index.html
        if file_path.is_dir() && file_path.join("index.html").exists() {
          file_path = file_path.join("index.html");
        }

        // Serve folder structure
        if file_path.is_dir() {
          res = res.header("Content-Type", "text/html");
          let files = fs::read_dir(&file_path).unwrap();
          let mut output = String::new();
          for file in files {
            let rel_path =
              pathdiff::diff_paths(file.unwrap().path(), &config.serve_dir_abs).unwrap();
            let rel_path_str = rel_path.to_str().unwrap();

            output += "<li><a href=\"/";
            output += &format!("{}\">/{}</a></li>", rel_path_str, rel_path_str);
          }
          let resp = res.body(Full::new(Bytes::from(output))).unwrap();
          return Ok(resp);
        }

        // 404 if no file exists
        if !file_path.exists() {
          return Ok(
            res
              .status(404)
              .body(Full::new(Bytes::from("File not found")))
              .unwrap(),
          );
        }

        if let Some(mime) = self::mime_guess::from_path(&file_path).first() {
          res = res.header("Content-Type", mime.to_string());
        }

        for (key, values) in config.headers.iter() {
          for value in values.iter() {
            res = res.header(key, value);
          }
        }

        let file = fs::read(&file_path).unwrap();

        let resp = res.body(Full::new(Bytes::from(file))).unwrap();
        Ok(resp)
      }
    })
  })
}
