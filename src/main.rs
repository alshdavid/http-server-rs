#![deny(unused_crate_dependencies)]

mod cli;
mod config;
mod explorer;
mod fmt;

use std::convert::Infallible;
use std::fs;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use colored::Colorize;
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
use normalize_path::NormalizePath;
use notify::RecommendedWatcher;
use notify::Watcher;
use shared_string::SharedString;
use tokio::net::TcpListener;

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

  let mut watcher = RecommendedWatcher::new(
    move |result: Result<notify::Event, notify::Error>| {
      let event = result.unwrap();

      if event.kind.is_modify() {
        println!("File updated {:?}", event);               
      }
    },
    notify::Config::default(),
  ).unwrap();

  watcher.watch(&config.serve_dir_abs, notify::RecursiveMode::Recursive).unwrap();

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
        let req_path = SharedString::from(req.uri().path())
          .get(1..)
          .unwrap()
          .to_string();

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
          res = res.header("Content-Type", "text/html");
          return Ok(res.body(Full::new(Bytes::from(output))).unwrap());
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
          println!("{} {}", "[200]".green().bold(), req.uri());
        }

        let resp = res.body(Full::new(Bytes::from(file))).unwrap();
        Ok(resp)
      }
    })
  })
}
