#![deny(unused_crate_dependencies)]

mod cli;
mod config;
mod explorer;
mod fmt;
mod http1;
mod utils;

use std::convert::Infallible;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use colored::Colorize;
use explorer::render_directory_explorer;
use fmt::Logger;
use futures::TryStreamExt;
use http1::http1_server;
use http1::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::StreamBody;
use hyper::body::Bytes as HyperBytes;
use mime_guess;
use normalize_path::NormalizePath;
use notify::RecommendedWatcher;
use notify::Watcher;
use tokio::io::AsyncWriteExt;
use utils::broadcast::BroadcastChannel;

use crate::config::Config;

async fn main_async() -> anyhow::Result<()> {
  let config = Arc::new(Config::from_cli()?);
  let logger = Arc::new(Logger::new(&config));

  logger.println(format!(
    "{}",
    "🚀 HTTP Server 🌏".green().bold().to_string()
  ));
  logger.br();
  logger.println(format!("📁 {}", config.serve_dir_fmt).bold().to_string());
  logger.print_config("Directory Listings", &true); // TODO
  logger.print_config("Watch", &config.watch); // TODO
  logger.print_config("GZIP", &false); // TODO
  logger.print_config("Brotli", &false); // TODO

  for (key, values) in config.headers.iter() {
    logger.print_config_str(key, &values.join(", "));
  }
  logger.br();
  logger.println(
    format!("🔗 http://{}", config.domain)
      .bold()
      .bright_white()
      .to_string(),
  );

  if config.domain != config.domain_pretty {
    logger.println(
      format!("🔗 http://{}", config.domain_pretty)
        .bold()
        .bright_white()
        .to_string(),
    );
  }
  logger.br();
  logger.println("📜 LOGS 📜".blue().bold().to_string());

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
      )
      .unwrap();

      watcher
        .watch(&config.serve_dir_abs, notify::RecursiveMode::Recursive)
        .unwrap();
      Some(watcher)
    } else {
      None
    }
  };

  http1_server(&config.domain, {
    let config = config.clone();
    let logger = logger.clone();
    let trx_watch = trx_watch.clone();

    move |req, mut res| {
      let config = config.clone();
      let logger = logger.clone();
      let trx_watch = trx_watch.clone();

      async move {
        // Remove the leading slash
        let req_path = req.uri().path()[1..].to_string();

        // Guess the file path of the file to serve
        let mut file_path = config.serve_dir_abs.join(req_path.clone());

        // If the watcher is enabled, return an event stream to the client to notify changes
        if req_path == ".http-server-rs/reload" {
          let trx_watch = trx_watch.clone();

          let (mut writer, reader) = tokio::io::duplex(512);

          let reader_stream = tokio_util::io::ReaderStream::new(reader)
            .map_ok(hyper::body::Frame::data)
            .map_err(|_item| panic!());

          let stream_body = StreamBody::new(reader_stream);
          let boxed_body = BoxBody::<HyperBytes, Infallible>::new(stream_body); // = stream_body.boxed().into();

          tokio::task::spawn(async move {
            let mut rx = trx_watch.subscribe();
            while let Some(changes) = rx.recv().await {
              let msg = format!(
                "event:changed\ndata:{}\n\n",
                changes
                  .into_iter()
                  .map(|v| v.to_str().unwrap().to_string())
                  .collect::<Vec<String>>()
                  .join(",")
              );
              if writer.write_all(msg.as_bytes()).await.is_err() {
                break;
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
              .expect("not to fail"),
          );
        }

        // hyper handles preventing access to parent directories via "../../"
        // but this is an extra layer of protection
        if !file_path.normalize().starts_with(&config.serve_dir_abs) {
          logger.println(format!("{} {}", "[403]".red().bold(), req.uri()));
          return Ok(res.status(403).body(Bytes::from("Not allowed").into())?);
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
          let output = render_directory_explorer(&config, &req_path, &file_path)?;

          return Ok(
            res
              .header("Content-Type", "text/html")
              .status(200)
              .body(Bytes::from(output).into())?,
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
          logger.println(format!("{} {}", "[404]".red().bold(), req.uri()));
          return Ok(res.status(404).body(Bytes::from("File not found").into())?);
        }

        // Apply mime type
        if let Some(mime) = self::mime_guess::from_path(&file_path).first() {
          res = res.header("Content-Type", mime.to_string());
        }

        // Read file
        // TODO not sure why tokio file read doesn't work here
        let Ok(contents) = fs::read(&file_path) else {
          return Ok(
            res
              .status(500)
              .body(Bytes::from("Unable to open file").into())?,
          );
        };

        logger.println(format!("{} {}", "[200]".green().bold(), req.uri()));
        Ok(res.status(200).body(Bytes::from(contents).into())?)
      }
    }
  })
  .await
}

fn main() -> anyhow::Result<()> {
  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .worker_threads(num_cpus::get_physical())
    .build()
    .unwrap()
    .block_on(main_async())
}
