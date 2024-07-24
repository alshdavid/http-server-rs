#![deny(unused_crate_dependencies)]

mod utils;
mod cli;
mod config;
mod explorer;
mod fmt;

use std::collections::HashSet;
use std::convert::Infallible;
use std::fs;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use colored::Colorize;
use explorer::render_directory_explorer;
use futures::TryStreamExt;
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use http_body_util::Full;
use http_body_util::StreamBody;
use hyper::body::Bytes;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Request;
use hyper::Response;
use hyper_util::rt::TokioIo;
use mime_guess;
use normalize_path::NormalizePath;
use notify::RecommendedWatcher;
use notify::Watcher;
use shared_string::SharedString;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use utils::broadcast::BroadcastChannel;

use crate::config::Config;


fn main() -> Result<(), String> {
  let config = Arc::new(Config::from_cli()?);
  if !config.quiet {
    println!("{}", format!("🚀 HTTP Server 🌏").green().bold());
    println!("");

    println!("📁 {}", format!("{}", config.serve_dir_fmt).bold());
    fmt::print_config("Directory Listings", "enabled"); // TODO
    fmt::print_config("GZIP", "disabled"); // TODO
    fmt::print_config("Brotli", "disabled"); // TODO

    for (key, values) in config.headers.iter() {
      fmt::print_config(key, &values.join(", "));
    }
    println!("");

    println!(
      "{}",
      format!("🔗 http://{}", config.domain).bold().bright_white()
    );
    if config.domain != config.domain_pretty {
      println!(
        "{}",
        format!("🔗 http://{}", config.domain_pretty)
          .bold()
          .bright_white()
      );
    }
    println!("");

    println!("{}", "📜 LOGS 📜".blue().bold());
  }

  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .worker_threads(num_cpus::get_physical())
    .build()
    .unwrap()
    .block_on(async {
      let trx_watch = Arc::new(BroadcastChannel::<Vec<PathBuf>>::new());

      let _watcher = {
        if config.watch {
          let trx_watch = trx_watch.clone();
    
          let mut watcher = RecommendedWatcher::new(
            move |result: Result<notify::Event, notify::Error>| {
              let event = result.unwrap();
              if event.kind.is_modify() {
                trx_watch.send(event.paths).unwrap();
              }
            },
            notify::Config::default(),
          ).unwrap();
        
          watcher.watch(&config.serve_dir_abs, notify::RecursiveMode::Recursive).unwrap();
          Some(watcher)
        } else {
          None
        }
      };

      let listener = TcpListener::bind(&config.domain).await.unwrap();
      
      loop {
        let trx_watch = trx_watch.clone();
        let config = config.clone();
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
          http1::Builder::new()
            .serve_connection(io, service_fn(server(config, trx_watch)))
            .await
            .ok();
        });
      }
    });

  Ok(())
}

fn server(
  config: Arc<Config>,
  trx_watch: Arc<BroadcastChannel<Vec<PathBuf>>>,
) -> Box<
  dyn Send
    + Fn(
      Request<Incoming>,
    ) -> Pin<Box<dyn Send + Future<Output = Result<Response<BoxBody<Bytes, std::io::Error>>, Infallible>>>>,
> {
  Box::new(move |req| {
    Box::pin({
      let config = config.clone();
      let trx_watch = trx_watch.clone();
      async move {
        let mut res = Response::builder();
        let (mut writer, reader) = tokio::io::duplex(512);
        let reader_stream = tokio_util::io::ReaderStream::new(reader);
        let stream_body = StreamBody::new(reader_stream.map_ok(hyper::body::Frame::data));
        let boxed_body = stream_body.boxed();

        // Trim the leading "/" from the URI
        let req_path = SharedString::from(req.uri().path())
          .get(1..)
          .unwrap()
          .to_string();

        if req_path == ".http-server-rs/reload" {
          let trx_watch = trx_watch.clone();

          tokio::task::spawn(async move {
            let mut rx = trx_watch.subscribe();
            while let Some(changes) = rx.recv().await {
              let msg = format!("event:changed\ndata:{}\n\n",changes.into_iter().map(|v| v.to_str().unwrap().to_string()).collect::<Vec<String>>().join(","));
              if writer
                .write_all(msg.as_bytes()).await.is_err() {
                  break
                }
            }
          });

          return Ok(
            res
            .header("X-Accel-Buffering", "no")
            .header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .status(hyper::StatusCode::OK)
            .body(boxed_body)
            .expect("not to fail")
          );
        }

        // Guess the file path of the file to serve
        let mut file_path = config.serve_dir_abs.join(req_path.clone());

        // hyper handles preventing access to parent directories via "../../"
        // but this is an extra layer of protection
        if !file_path.normalize().starts_with(&config.serve_dir_abs) {
          println!("{} {}", "[403]".red().bold(), req.uri());
          return Ok(
            res
              .status(403)
              .body(Full::new(Bytes::from("File not found")))
              .unwrap(),
          );
        }

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

          tokio::task::spawn(async move {
            writer.write_all(output.as_bytes()).await.unwrap();
          });

          return Ok(
            res
              .header("Content-Type", "text/html")
              .status(200)
              .body(boxed_body)
              .expect("failed to send  dir explorer")
          );
        }

        // 404 if no file exists
        if config.spa && !file_path.exists() {
          file_path = config.serve_dir_abs.join("index.html");
        }

        if !config.spa && !file_path.exists() {
          file_path = config.serve_dir_abs.join("404.html");
        }

        // 404 if no file exists
        if !file_path.exists() {
          if !config.quiet {
            println!("{} {}", "[404]".red().bold(), req.uri());
          }

          tokio::task::spawn(async move {
            writer.write_all(b"File not found").await.unwrap();
          });

          return Ok(
            res
              .status(404)
              .body(boxed_body)
              .expect("failed to send 404")
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
              .body(boxed_body)
              .expect("failed to send 500")
          );
        };

        if !config.quiet {
          println!("{} {}", "[200]".green().bold(), req.uri());
        }

        tokio::task::spawn(async move {
          writer.write_all(&file).await.unwrap();
        });
        
        Ok(res
          .status(200)
          .body(boxed_body)
          .expect("failed to send file")
        )
      }
    })
  })
}
