#![deny(unused_crate_dependencies)]
#![allow(clippy::module_inception)]

mod cli;
mod config;
mod explorer;
mod http1;
mod logger;
mod utils;
mod watcher;

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::Arc;

use colored::Colorize;
use explorer::reload_script;
use explorer::render_directory_explorer;
use http1::http1_server;
use http1::ResponseBuilderExt;
use logger::Logger;
use mime_guess;
use normalize_path::NormalizePath;
use tokio::io::AsyncWriteExt;
use watcher::Watcher;
use watcher::WatcherOptions;

use crate::config::Config;

async fn main_async() -> anyhow::Result<()> {
  let config = Arc::new(Config::from_cli()?);
  let logger: Arc<Logger> = match config.quiet {
    true => Arc::new(Logger::Quiet),
    false => Arc::new(Logger::Default),
  };

  logger.println("🚀 HTTP Server 🌏".green().bold().to_string());
  logger.br();

  logger.print_folder(&config.serve_dir_fmt);
  logger.print_config("Directory Listings", &true);
  logger.print_config("CORS", &config.cors);
  logger.print_config("SharedArrayBuffer", &config.sab);
  logger.print_config("SPA", &config.spa);
  logger.print_config("Watch", &config.watch);
  logger.br();

  logger.print_headers(&config.headers);
  logger.br();

  logger.println(format!("🔗 http://{}", config.domain));
  if config.domain != config.domain_pretty {
    logger.println(format!("🔗 http://{}", config.domain_pretty));
  }
  logger.br();

  logger.println("📜 LOGS 📜".bold().blue().to_string());

  let watcher = match config.watch {
    true => Some(Watcher::new(WatcherOptions {
      target_dir: config.watch_dir.clone(),
      logger: logger.clone(),
    })?),
    false => None,
  };

  http1_server(&config.domain, {
    let config = config.clone();
    let logger = logger.clone();
    let watcher = watcher.clone();

    move |req, mut res| {
      let config = config.clone();
      let logger = logger.clone();
      let watcher = watcher.clone();

      async move {
        // Remove the leading slash
        let req_path = req.uri().path().to_string().replacen("/", "", 1);

        // Guess the file path of the file to serve
        let mut file_path = config.serve_dir_abs.join(req_path.clone());

        // If the watcher is enabled, return an event stream to the client to notify changes
        if req_path == ".http-server-rs/reload.js" {
          if !config.watch {
            return Ok(res.status(404).body_from("Watcher not running")?);
          };

          return Ok(
            res
              .header("Content-Type", "application/javascript")
              .status(200)
              .body_from(reload_script())?,
          );
        }

        // Endpoint for filesystem change event stream
        if req_path == ".http-server-rs/reload" {
          let Some(watcher) = watcher else {
            return Ok(res.status(404).body_from("Watcher not running")?);
          };

          let (res, mut writer) = res
            .header("X-Accel-Buffering", "no")
            .header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .status(hyper::StatusCode::OK)
            .body_stream()?;

          let mut rx = watcher.subscribe();

          tokio::task::spawn(async move {
            while let Some(changes) = rx.recv().await {
              let msg = format!(
                "data:{}\n\n",
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

          return Ok(res);
        }

        // hyper handles preventing access to parent directories via "../../"
        // but this is an extra layer of protection
        if !file_path.normalize().starts_with(&config.serve_dir_abs) {
          logger.println(format!("{} {}", "[403]".red().bold(), req.uri()));
          return Ok(res.status(403).body_from("Not allowed")?);
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
          let mut output = render_directory_explorer(&config, &req_path, &file_path)?;

          if config.watch {
            output = format!("{}\n<script>{}</script>", output, reload_script());
          }

          return Ok(
            res
              .header("Content-Type", "text/html")
              .status(200)
              .body_from(output)?,
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
          return Ok(res.status(404).body_from("File not found")?);
        }

        // Apply mime type
        if let Some(mime) = self::mime_guess::from_path(&file_path).first() {
          res = res.header("Content-Type", mime.to_string());
        }

        let brotli_path = PathBuf::from(format!("{}.br", file_path.to_str().unwrap()));
        let gzip_path = PathBuf::from(format!("{}.gz", file_path.to_str().unwrap()));

        if brotli_path.exists() {
          file_path = brotli_path;
          res = res.header("Content-Encoding", "br");
        } else if gzip_path.exists() {
          file_path = gzip_path;
          res = res.header("Content-Encoding", "gzip");
        }

        // Read file
        // TODO not sure why tokio file read doesn't work here
        let Ok(mut contents) = fs::read(&file_path) else {
          return Ok(res.status(500).body_from("Unable to open file")?);
        };

        logger.println(format!("{} {}", "[200]".green().bold(), req.uri()));

        if config.watch
          && !config.no_watch_inject
          && res
            .headers_ref()
            .unwrap()
            .get("Content-Type")
            .is_some_and(|h| h == "text/html")
        {
          let html = String::from_utf8(contents.clone())?;
          if html.contains("<head>") {
            contents = html
              .replacen(
                "<head>",
                &format!("<head>\n<script>{}</script>\n", reload_script(),),
                1,
              )
              .as_bytes()
              .to_vec();
          } else if html.contains("<body>") {
            contents = html
              .replacen(
                "<body>",
                &format!("<body>\n<script>{}</script>\n", reload_script()),
                1,
              )
              .as_bytes()
              .to_vec();
          } else {
            contents.extend(format!("<script>{}</script>", reload_script()).as_bytes());
          }
        }

        Ok(res.status(200).body_from(contents)?)
      }
    }
  })
  .await
}

fn main() -> anyhow::Result<()> {
  let (tx, rx) = channel::<anyhow::Result<()>>();

  std::thread::spawn(move || {
    tx.send(
      tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(num_cpus::get_physical())
        .build()
        .unwrap()
        .block_on(main_async()),
    )
    .unwrap();
  });

  rx.recv()?
}
