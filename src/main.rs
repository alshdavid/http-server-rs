mod cli;
mod config;

use std::convert::Infallible;
use std::fs;
use std::fs::Permissions;
use std::future::Future;
use std::os::unix::fs::PermissionsExt;
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
use serde_json::json;
use shared_string::SharedString;
use tokio::net::TcpListener;
use handlebars::Handlebars;
use chrono::{DateTime, Utc}; 
use unix_mode;

use crate::config::Config;

const DIR_PAGE: &str = include_str!("./html/dir.hbs");

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
          .unwrap();

        // Guess the file path of the file to serve
        let mut file_path = config.serve_dir_abs.join(req_path.to_string());

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
          let dir = fs::read_dir(&file_path).unwrap();
          let mut files = Vec::<(String, String, String)>::new();
          let mut folders = Vec::<(String, String, String)>::new();
          
          for item in dir {
            let item = item.unwrap();
            let meta = item.metadata().unwrap();
            let meta_mode = self::unix_mode::to_string(meta.permissions().mode());
            let last_modified: DateTime<Utc> =  meta.modified().unwrap().into();

            let rel_path = pathdiff::diff_paths(item.path(), &config.serve_dir_abs).unwrap();
            let rel_path_str = rel_path.to_str().unwrap();
            if item.file_type().unwrap().is_dir() {
              folders.push((format!("{}", meta_mode), format!("{}", last_modified.format("%d %b %Y %H:%M")), rel_path_str.to_string()));
            } else {
              files.push((format!("{}", meta_mode), format!("{}", last_modified.format("%d %b %Y %H:%M")), rel_path_str.to_string()));
            };
          }

          let handlebars = Handlebars::new();
          let output = handlebars.render_template(DIR_PAGE, &json!({
            "path": req_uri.clone(),
            "files": files,
            "folders": folders,
            "address": config.address.clone(),
            "port": config.port.clone(),
          })).unwrap();
          
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
