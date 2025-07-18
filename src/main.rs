#![deny(unused_crate_dependencies)]
#![allow(clippy::module_inception)]

mod auth;
mod b64;
mod cli;
mod compress;
mod config;
mod explorer;
mod http1;
mod logger;
mod utils;
mod watcher;

use std::net::UdpSocket;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
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
use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncWriteExt;
use watcher::Watcher;
use watcher::WatcherOptions;

use crate::config::Config;

const DEFAULT_CHARSET_SUFFIX: &str = "charset=UTF-8";

// copy from https://github.com/egmkang/local_ipaddress/blob/master/src/lib.rs
// Todo: need all ips use https://crates.io/crates/local-ip-address
fn get_intranet_ip() -> Option<String> {
  let socket = match UdpSocket::bind("0.0.0.0:0") {
    Ok(s) => s,
    Err(_) => return None,
  };

  match socket.connect("8.8.8.8:80") {
    Ok(()) => (),
    Err(_) => return None,
  };

  match socket.local_addr() {
    Ok(addr) => Some(addr.ip().to_string()),
    Err(_) => None,
  }
}

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
  logger.print_config("Compress (JIT)", &config.compress);
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

  // print intranet ip domain
  // Todo address bind to local ip 127.0.0.1 skip print?
  let intranet_domain = get_intranet_ip();
  if intranet_domain.is_some() {
    let Some(intranet_domain_str) = intranet_domain.as_ref() else {
      return Err(anyhow::anyhow!("Unable to get intranet domain str"));
    };
    if intranet_domain_str != &config.domain_pretty && intranet_domain_str != &config.domain {
      logger.println(format!("🔗 http://{}:{}", intranet_domain_str, config.port));
    }
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
        // Basic Auth
        if !config.basic_auth.is_empty() {
          let Some(header) = req.headers().get("authorization") else {
            return Ok(
              res
                .header(
                  "WWW-Authenticate",
                  "Basic realm=\"http-server-rs\"".to_string(),
                )
                .status(401)
                .body_from("")?,
            );
          };

          let Some((_, token)) = header.to_str()?.split_once(" ") else {
            return Ok(res.status(500).body_from("Invalid basic auth header")?);
          };

          let decoded = b64::decode_string(token)?;

          let Some((username, password)) = decoded.split_once(":") else {
            return Ok(res.status(500).body_from("Invalid basic auth header")?);
          };

          let Some(creds) = config.basic_auth.get(username) else {
            return Ok(res.status(403).body_from("")?);
          };

          if creds != password {
            return Ok(res.status(403).body_from("")?);
          }
        }

        // Remove the leading slash
        let req_path = req.uri().path().to_string().replacen("/", "", 1);
        let req_path = urlencoding::decode(&req_path)?.to_string();

        // Guess the file path of the file to serve
        let mut file_path = config.serve_dir_abs.join(req_path.clone());

        // If the watcher is enabled, return an event stream to the client to notify changes
        if req_path == ".http-server-rs/reload.js" {
          if !config.watch {
            return Ok(res.status(404).body_from("Watcher not running")?);
          };

          return Ok(
            res
              .header(
                "Content-Type",
                format!("application/javascript; {}", DEFAULT_CHARSET_SUFFIX),
              )
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
            .header(
              "Content-Type",
              format!("text/event-stream; {}", DEFAULT_CHARSET_SUFFIX),
            )
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .status(hyper::StatusCode::OK)
            .body_stream(config.stream_buffer_size)?;

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

          // Todo check file charset
          return Ok(
            res
              .header(
                "Content-Type",
                format!("text/html;{}", DEFAULT_CHARSET_SUFFIX),
              )
              .status(200)
              .body_from(output)?,
          );
        }

        // If SPA and file doesn't exist, route to root index
        if config.spa && !file_path.exists() {
          file_path = config.serve_dir_abs.join("index.html");
        }

        // If not SPA an file doesn't exist, route to 404.html
        if !config.spa && !file_path.exists() {
          file_path = config.serve_dir_abs.join("404.html");
        }

        // 404 if no file exists
        if !file_path.exists() {
          logger.println(format!("{} {}", "[404]".red().bold(), req.uri()));
          return Ok(res.status(404).body_from("File not found")?);
        }

        // Apply mime type
        let mime = self::mime_guess::from_path(&file_path)
          .first()
          .map(|v| v.to_string())
          .unwrap_or_default();
        if !mime.is_empty() {
          let mut content_type = mime.clone();
          // mime starts with "text/" or "application/"
          if content_type.starts_with("text/")
            || content_type.starts_with("application/javascript")
            || content_type.starts_with("application/json")
          {
            // Todo check file charset
            content_type = format!("{}; {}", content_type, DEFAULT_CHARSET_SUFFIX);
          }
          res = res.header("Content-Type", &content_type);
        }

        // If a .br or .gz file is found next to the target, serve that file
        if !config.compress {
          let brotli_path = PathBuf::from(format!("{}.br", file_path.to_str().unwrap()));
          let gzip_path = PathBuf::from(format!("{}.gz", file_path.to_str().unwrap()));

          if brotli_path.exists() {
            file_path = brotli_path;
            res = res.header("Content-Encoding", "br");
          } else if gzip_path.exists() {
            file_path = gzip_path;
            res = res.header("Content-Encoding", "gzip");
          }
        }

        let mut file = File::open(&file_path).await?;

        #[cfg(unix)]
        let content_length = file.metadata().await?.size();
        #[cfg(windows)]
        let content_length = file.metadata().await?.file_size();

        logger.println(format!("{} {}", "[200]".green().bold(), req.uri()));

        // Read file
        // Stream file if it's larger than 5mb
        if !mime.starts_with("text/html") && content_length > 500_000 {
          let (res, mut writer) = res
            .header("Connection", "keep-alive")
            .header("Content-Length", content_length)
            .status(hyper::StatusCode::OK)
            .body_stream(config.stream_buffer_size)?;

          tokio::task::spawn(async move {
            io::copy(&mut file, &mut writer).await.ok();
          });

          return Ok(res);
        }

        let Ok(mut contents) = tokio::fs::read(&file_path).await else {
          return Ok(res.status(500).body_from("Unable to open file")?);
        };

        // If using watch mode and automatically injecting the reload script
        // and file is html, mutate response to inject script
        if config.watch
          && !config.no_watch_inject
          && res
            .headers_ref()
            .unwrap()
            .get("Content-Type")
            .is_some_and(|h| h.to_str().unwrap_or("").starts_with("text/html"))
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

        if config.compress {
          res = res.header("Content-Encoding", "br");
          Ok(res.status(200).body_from(compress::brotli(&contents))?)
        } else {
          Ok(res.status(200).body_from(contents)?)
        }
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
