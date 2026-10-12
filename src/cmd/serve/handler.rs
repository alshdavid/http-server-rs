use std::convert::Infallible;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use colored::Colorize;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes as HyperBytes;
use hyper::body::Incoming;
use hyper::http::response::Builder as ResponseBuilder;
use hyper::Request;
use hyper::Response;
use normalize_path::NormalizePath;
use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncWriteExt;

use super::config::Config;
use super::constants::DEFAULT_CHARSET_SUFFIX;
use crate::platform::b64;
use crate::platform::compress;
use crate::platform::explorer::reload_script;
use crate::platform::explorer::render_directory_explorer;
use crate::platform::explorer::RenderDirectoryExplorerConfig;
use crate::platform::http1::ResponseBuilderExt;
use crate::platform::logger::Logger;
use crate::platform::proxy::match_proxy_route;
use crate::platform::proxy::proxy_request;
use crate::platform::proxy::ProxyClient;
use crate::platform::transpile;
use crate::platform::transpile::tsconfig;
use crate::platform::transpile::tsconfig::TsConfigCache;
use crate::platform::watcher::Watcher;

pub struct ServeState {
  pub config: Arc<Config>,
  pub logger: Arc<Logger>,
  pub watcher: Option<Watcher>,
  pub proxy_client: Option<Arc<ProxyClient>>,
  pub tsconfig_cache: Arc<TsConfigCache>,
}

pub async fn handle(
  req: Request<Incoming>,
  mut res: ResponseBuilder,
  state: Arc<ServeState>,
) -> anyhow::Result<Response<BoxBody<HyperBytes, Infallible>>> {
  let config = state.config.clone();
  let logger = state.logger.clone();
  let watcher = state.watcher.clone();
  let proxy_client = state.proxy_client.clone();
  let tsconfig_cache = state.tsconfig_cache.clone();

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

  // Reverse proxy routes
  if let Some((_proxy_path, route)) = match_proxy_route(&config.proxy, req.uri().path()) {
    let Some(proxy_client) = proxy_client else {
      return Ok(res.status(500).body_from("Proxy client not available")?);
    };

    let target = route.target.clone();
    let request_uri = req.uri().clone();

    logger.println(format!(
      "{} {} -> {}",
      "[proxy]".blue().bold(),
      request_uri,
      target
    ));

    let response = proxy_request(&proxy_client, route, req).await?;

    logger.println(format!(
      "{} {} -> {} {}",
      "[proxy]".blue().bold(),
      request_uri,
      target,
      response.status()
    ));

    return Ok(response);
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
      while rx.changed().await.is_ok() {
        let msg = format!(
          "data:{}\n\n",
          rx.borrow_and_update()
            .iter()
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

  if let Some(file) = req_path.strip_prefix(".http-server-rs/swc/") {
    let source = file
      .strip_suffix(".js")
      .and_then(transpile::helpers::module_source);

    let Some(source) = source else {
      logger.println(format!("{} {}", "[404]".red().bold(), req.uri()));
      return Ok(res.status(404).body_from("Unknown helper")?);
    };

    logger.println(format!("{} {}", "[200]".green().bold(), req.uri()));

    return Ok(
      res
        .header(
          "Content-Type",
          format!("application/javascript; {}", DEFAULT_CHARSET_SUFFIX),
        )
        .status(200)
        .body_from(source)?,
    );
  }

  // hyper handles preventing access to parent directories via "../../"
  // but this is an extra layer of protection
  if !file_path.normalize().starts_with(&config.serve_dir_abs) {
    logger.println(format!("{} {}", "[403]".red().bold(), req.uri()));
    return Ok(res.status(403).body_from("Not allowed")?);
  }

  // Source maps are requested as the `.map` sibling of a transpiled file
  if config.transpile && config.source_maps {
    if let Some(map) = transpile_source_map(&config, &tsconfig_cache, &req_path).await? {
      logger.println(format!("{} {}", "[200]".green().bold(), req.uri()));

      return Ok(
        res
          .header(
            "Content-Type",
            format!("application/json; {}", DEFAULT_CHARSET_SUFFIX),
          )
          .status(200)
          .body_from(map)?,
      );
    }
  }

  // Redirect directory requests to a trailing slash so that relative
  // URLs within (e.g. "./main.tsx") resolve against the directory
  // rather than the parent.
  if file_path.is_dir() && !req.uri().path().ends_with('/') {
    let mut location = format!("{}/", req.uri().path());

    if let Some(query) = req.uri().query() {
      location.push('?');
      location.push_str(query);
    }

    logger.println(format!("{} {}", "[301]".yellow().bold(), req.uri()));

    return Ok(res.header("Location", location).status(301).body_from("")?);
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

  // Serve folder explorer
  if file_path.is_dir() {
    let mut output = render_directory_explorer(
      &RenderDirectoryExplorerConfig {
        serve_dir_abs: config.serve_dir_abs.clone(),
        address: config.address.clone(),
        port: config.port,
      },
      &req_path,
      &file_path,
    )?;

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

  let ext = file_path
    .extension()
    .and_then(|v| v.to_str())
    .unwrap_or_default()
    .to_string();

  if config.transpile && (ext == "scss" || ext == "sass") {
    let css = transpile_stylesheet(&config, &req_path, &file_path).await?;

    logger.println(format!("{} {}", "[200]".green().bold(), req.uri()));

    let content_type = format!("text/css; {}", DEFAULT_CHARSET_SUFFIX);

    if config.compress {
      res = res.header("Content-Encoding", "br");
      return Ok(
        res
          .header("Content-Type", content_type)
          .status(200)
          .body_from(compress::brotli(css.as_bytes()))?,
      );
    }

    return Ok(
      res
        .header("Content-Type", content_type)
        .status(200)
        .body_from(css)?,
    );
  }

  if config.transpile && (ext == "ts" || ext == "tsx") {
    let result = transpile_file(&config, &tsconfig_cache, &file_path, &req_path, &ext).await?;

    let map_name = file_path
      .file_name()
      .and_then(|value| value.to_str())
      .unwrap_or_default();

    let code = match result.map {
      Some(_) => format!("{}\n//# sourceMappingURL={}.map\n", result.code, map_name),
      None => result.code,
    };

    logger.println(format!("{} {}", "[200]".green().bold(), req.uri()));

    let content_type = format!("application/javascript; {}", DEFAULT_CHARSET_SUFFIX);

    if config.compress {
      res = res.header("Content-Encoding", "br");
      return Ok(
        res
          .header("Content-Type", content_type)
          .status(200)
          .body_from(compress::brotli(code.as_bytes()))?,
      );
    }

    return Ok(
      res
        .header("Content-Type", content_type)
        .status(200)
        .body_from(code)?,
    );
  }

  // Apply mime type
  let mime = mime_guess::from_path(&file_path)
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

async fn transpile_file(
  config: &Config,
  tsconfig_cache: &TsConfigCache,
  file_path: &Path,
  url_path: &str,
  kind: &str,
) -> anyhow::Result<transpile::TransformerResult> {
  let content = tokio::fs::read(file_path).await?;

  let tsconfig = tsconfig_cache.resolve(file_path, &config.serve_dir_abs);

  transpile::typescript::transpile(transpile::TransformerContext {
    content,
    path: file_path.to_path_buf(),
    source_url: format!("/{}", url_path),
    source_maps: config.source_maps,
    kind: kind.to_string(),
    root_dir: config.serve_dir_abs.clone(),
    tsconfig,
  })
}

/// Compile a scss file to css.
async fn transpile_stylesheet(
  config: &Config,
  req_path: &str,
  file_path: &Path,
) -> anyhow::Result<String> {
  let content = tokio::fs::read(file_path).await?;

  let result = transpile::scss::transpile(transpile::TransformerContext {
    content,
    path: file_path.to_path_buf(),
    source_url: format!("/{}", req_path),
    source_maps: false,
    kind: "scss".to_string(),
    root_dir: config.serve_dir_abs.clone(),
    tsconfig: tsconfig::TsConfig::defaults(),
  })?;

  // The browser only knows the `.css` URL, so any import that names a
  // stylesheet without its extension has to be rewritten to one it can
  // request from this server.
  let candidates = transpile::scss::candidates(&result.code, file_path, &config.serve_dir_abs);

  let mut code = result.code;

  for (specifier, replacement) in candidates {
    let rewrites = if specifier.contains('\'') {
      vec![(format!("\"{specifier}\""), format!("\"{replacement}\""))]
    } else if specifier.contains('"') {
      vec![(format!("'{specifier}'"), format!("'{replacement}'"))]
    } else {
      vec![
        (format!("\"{specifier}\""), format!("\"{replacement}\"")),
        (format!("'{specifier}'"), format!("'{replacement}'")),
      ]
    };

    for (target, value) in rewrites {
      code = code.replace(&target, &value);
    }
  }

  Ok(code)
}

/// Resolve a `.map` request to the source map of the transpiled file it names.
async fn transpile_source_map(
  config: &Config,
  tsconfig_cache: &TsConfigCache,
  req_path: &str,
) -> anyhow::Result<Option<String>> {
  let Some(source_path) = req_path.strip_suffix(".map") else {
    return Ok(None);
  };

  let file_path = config.serve_dir_abs.join(source_path);

  let kind = file_path
    .extension()
    .and_then(|value| value.to_str())
    .unwrap_or_default();

  if !matches!(kind, "ts" | "tsx") || !file_path.is_file() {
    return Ok(None);
  }

  let result = transpile_file(config, tsconfig_cache, &file_path, source_path, kind).await?;

  Ok(result.map)
}
