use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use hyper::body::Bytes as HyperBytes;
use hyper::body::Incoming;
use hyper::Request;
use hyper::Response;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use serde::Deserialize;

/// Shared HTTP client used when forwarding proxied requests upstream.
pub type ProxyClient = Client<HttpConnector, Incoming>;

/// Build the HTTP client used to forward proxied requests.
pub fn build_client() -> ProxyClient {
  Client::builder(TokioExecutor::new()).build_http()
}

/// Forward an incoming request to the upstream target defined by `route`.
///
/// Returns a response with the upstream status, headers and body.
pub async fn proxy_request(
  client: &Arc<ProxyClient>,
  route: &ProxyRoute,
  req: Request<Incoming>,
) -> anyhow::Result<Response<BoxBody<HyperBytes, Infallible>>> {
  let request_path = req.uri().path().to_string();
  let query = req.uri().query().map(|v| v.to_string());
  let method = req.method().clone();
  let version = req.version();

  let uri = build_target_uri(route, &request_path, query.as_deref())?;

  // Rebuild the request against the upstream URI, preserving method, headers
  // and body.
  let mut upstream_req = Request::builder()
    .method(method)
    .uri(uri.clone())
    .version(version);

  let target_host = uri
    .host()
    .map(|host| match uri.port() {
      Some(port) => format!("{}:{}", host, port),
      None => host.to_string(),
    })
    .unwrap_or_default();

  for (key, value) in req.headers() {
    // Skip hop-by-hop headers, they must not be forwarded
    match key.as_str() {
      "host" | "connection" | "keep-alive" | "transfer-encoding" | "upgrade"
      | "proxy-connection" => continue,
      _ => {}
    }
    upstream_req = upstream_req.header(key, value);
  }

  // Rewrite the Host header when `change_origin` is enabled, otherwise keep
  // the client supplied host (falling back to the target host when absent).
  if route.change_origin || !req.headers().contains_key("host") {
    upstream_req = upstream_req.header("host", &target_host);
  }

  let upstream_req = upstream_req.body(req.into_body())?;

  let upstream_res = client
    .request(upstream_req)
    .await
    .map_err(|err| anyhow::anyhow!("Proxy request to '{}' failed: {}", uri, err))?;

  let status = upstream_res.status();
  let headers = upstream_res.headers().clone();

  let mut res = Response::builder().status(status);

  for (key, value) in headers.iter() {
    match key.as_str() {
      "connection" | "keep-alive" | "transfer-encoding" | "upgrade" | "proxy-connection" => {
        continue
      }
      _ => {}
    }
    res = res.header(key, value);
  }

  // Collect the upstream body into memory so it can be re-framed with an
  // `Infallible` error type expected by the server response body.
  let body = upstream_res
    .into_body()
    .collect()
    .await
    .map_err(|err| anyhow::anyhow!("Unable to read proxied response body: {}", err))?
    .to_bytes();

  let body: BoxBody<HyperBytes, Infallible> = BoxBody::new(http_body_util::Full::new(body));

  let res = res.body(body)?;

  Ok(res)
}

/// A single reverse proxy route as provided by `--proxy`.
///
/// The flag takes a URL-encoded string, for example:
///
/// ```text
/// --proxy path=/api&target=http://localhost:3000&secure=false&change_origin=false&path_rewrite[key]=^/api&path_rewrite[value]=
/// ```
///
/// Which deserializes into:
///
/// ```json
/// {
///   "/api": {
///     "target": "http://localhost:3000",
///     "secure": false,
///     "changeOrigin": false,
///     "pathRewrite": { "^/api": "" }
///   }
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct ProxyRouteInput {
  /// The request path prefix that triggers this proxy route (e.g. `/api`).
  pub path: String,

  /// The upstream origin to forward matching requests to.
  pub target: String,

  /// Whether to verify TLS certificates. Defaults to `true`.
  #[serde(default = "default_true")]
  pub secure: bool,

  /// Whether to rewrite the `Host` header to the target origin.
  #[serde(default)]
  pub change_origin: bool,

  /// Path rewrites as `pattern -> replacement` pairs.
  ///
  /// Provided on the CLI as `path_rewrite[key]=<pattern>&path_rewrite[value]=<replacement>`.
  #[serde(default)]
  pub path_rewrite: Option<PathRewriteInput>,
}

/// The `path_rewrite[key]` / `path_rewrite[value]` pair.
#[derive(Debug, Clone, Deserialize)]
pub struct PathRewriteInput {
  /// The pattern (regex) to match against the request path.
  pub key: String,

  /// The replacement to substitute for matched patterns.
  #[serde(default)]
  pub value: String,
}

fn default_true() -> bool {
  true
}

/// A resolved reverse proxy route keyed by its path prefix.
#[derive(Debug, Clone)]
pub struct ProxyRoute {
  pub target: String,

  /// Whether TLS certificates should be verified when the target is HTTPS.
  ///
  /// Parsed and stored for now, but not yet enforced by the client.
  #[allow(dead_code)]
  pub secure: bool,

  pub change_origin: bool,
  pub path_rewrite: HashMap<String, String>,
}

/// Reverse proxy routes keyed by their path prefix.
pub type ProxyConfig = HashMap<String, ProxyRoute>;

impl ProxyRouteInput {
  /// Parse a single URL-encoded `--proxy` value into a [`ProxyRouteInput`].
  pub fn parse(value: &str) -> anyhow::Result<Self> {
    let route: ProxyRouteInput = serde_qs::from_str(value)
      .map_err(|err| anyhow::anyhow!("Unable to parse proxy route '{}': {}", value, err))?;

    if route.path.is_empty() {
      return Err(anyhow::anyhow!("Proxy route is missing a 'path'"));
    }

    if route.target.is_empty() {
      return Err(anyhow::anyhow!(
        "Proxy route '{}' is missing a 'target'",
        route.path
      ));
    }

    Ok(route)
  }
}

impl From<ProxyRouteInput> for ProxyRoute {
  fn from(value: ProxyRouteInput) -> Self {
    let mut path_rewrite = HashMap::new();

    if let Some(rewrite) = value.path_rewrite {
      path_rewrite.insert(rewrite.key, rewrite.value);
    }

    ProxyRoute {
      target: value.target,
      secure: value.secure,
      change_origin: value.change_origin,
      path_rewrite,
    }
  }
}

/// Find a proxy route whose `path` prefix matches the request path.
///
/// Paths are matched on segment boundaries so `/api` matches `/api` and
/// `/api/users` but not `/apiary`.
pub fn match_proxy_route<'a>(
  config: &'a ProxyConfig,
  request_path: &str,
) -> Option<(&'a str, &'a ProxyRoute)> {
  let mut best: Option<(&str, &ProxyRoute)> = None;

  for (path, route) in config.iter() {
    if !path_matches(path, request_path) {
      continue;
    }

    // Prefer the longest matching prefix for more specific routes
    let is_better = match best {
      Some((best_path, _)) => path.len() > best_path.len(),
      None => true,
    };

    if is_better {
      best = Some((path.as_str(), route));
    }
  }

  best
}

/// Does `prefix` match `request_path` on a segment boundary?
fn path_matches(
  prefix: &str,
  request_path: &str,
) -> bool {
  if prefix == "/" {
    return true;
  }

  let prefix = prefix.trim_end_matches('/');

  if !request_path.starts_with(prefix) {
    return false;
  }

  // Ensure the match ends at a path segment boundary
  matches!(request_path.as_bytes().get(prefix.len()), None | Some(b'/'))
}

/// Apply the route's path rewrites to a request path.
fn apply_path_rewrite(
  route: &ProxyRoute,
  path: &str,
) -> String {
  let mut path = path.to_string();

  for (pattern, replacement) in route.path_rewrite.iter() {
    if let Ok(re) = regex::Regex::new(pattern) {
      path = re.replace_all(&path, replacement.as_str()).to_string();
    } else if path.starts_with(pattern.as_str()) {
      // Fall back to a plain string prefix replacement
      path = format!("{}{}", replacement, &path[pattern.len()..]);
    }
  }

  path
}

/// Build the upstream request URI from the route target and the incoming path/query.
fn build_target_uri(
  route: &ProxyRoute,
  request_path: &str,
  query: Option<&str>,
) -> anyhow::Result<hyper::Uri> {
  let target = route.target.trim_end_matches('/');
  let path = apply_path_rewrite(route, request_path);

  let mut uri = format!("{}{}", target, path);

  if let Some(query) = query {
    if !query.is_empty() {
      uri.push('?');
      uri.push_str(query);
    }
  }

  uri
    .parse::<hyper::Uri>()
    .map_err(|err| anyhow::anyhow!("Unable to build target uri '{}': {}", uri, err))
}

/// Build a [`ProxyConfig`] from the raw `--proxy` flag values.
///
/// Each flag maps to a single route keyed by its `path`.
pub fn parse_proxy_config(values: Vec<String>) -> anyhow::Result<ProxyConfig> {
  let mut routes = ProxyConfig::new();

  for value in values {
    let input = ProxyRouteInput::parse(&value)?;
    let path = input.path.clone();

    if routes.contains_key(&path) {
      return Err(anyhow::anyhow!("Duplicate proxy route for path '{}'", path));
    }

    routes.insert(path, input.into());
  }

  Ok(routes)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_proxy_route() {
    let input = ProxyRouteInput::parse(
      "path=/api&target=http://localhost:3000&secure=false&change_origin=false&path_rewrite[key]=^/api&path_rewrite[value]=",
    )
    .unwrap();

    assert_eq!(input.path, "/api");
    assert_eq!(input.target, "http://localhost:3000");
    assert!(!input.secure);
    assert!(!input.change_origin);
    assert_eq!(input.path_rewrite.as_ref().unwrap().key, "^/api");
    assert_eq!(input.path_rewrite.as_ref().unwrap().value, "");
  }

  #[test]
  fn defaults_secure_and_change_origin() {
    let input = ProxyRouteInput::parse("path=/api&target=http://localhost:3000").unwrap();

    assert!(input.secure);
    assert!(!input.change_origin);
    assert!(input.path_rewrite.is_none());
  }

  #[test]
  fn builds_config_keyed_by_path() {
    let config = parse_proxy_config(vec![
      "path=/api&target=http://localhost:3000&change_origin=true".to_string(),
      "path=/other&target=http://localhost:4000".to_string(),
    ])
    .unwrap();

    assert_eq!(config.len(), 2);
    assert_eq!(config.get("/api").unwrap().target, "http://localhost:3000");
    assert!(config.get("/api").unwrap().change_origin);
    assert_eq!(
      config.get("/other").unwrap().target,
      "http://localhost:4000"
    );
  }

  #[test]
  fn rejects_missing_path() {
    assert!(ProxyRouteInput::parse("target=http://localhost:3000").is_err());
  }

  #[test]
  fn rejects_missing_target() {
    assert!(ProxyRouteInput::parse("path=/api").is_err());
  }

  #[test]
  fn rejects_duplicate_paths() {
    let result = parse_proxy_config(vec![
      "path=/api&target=http://localhost:3000".to_string(),
      "path=/api&target=http://localhost:4000".to_string(),
    ]);

    assert!(result.is_err());
  }

  fn route(target: &str) -> ProxyRoute {
    ProxyRoute {
      target: target.to_string(),
      secure: true,
      change_origin: false,
      path_rewrite: HashMap::new(),
    }
  }

  #[test]
  fn matches_path_on_segment_boundary() {
    let mut config = ProxyConfig::new();
    config.insert("/api".to_string(), route("http://localhost:3000"));

    assert!(match_proxy_route(&config, "/api").is_some());
    assert!(match_proxy_route(&config, "/api/").is_some());
    assert!(match_proxy_route(&config, "/api/test").is_some());
    // Must not match a sibling path sharing the prefix
    assert!(match_proxy_route(&config, "/apiary").is_none());
    assert!(match_proxy_route(&config, "/").is_none());
  }

  #[test]
  fn matches_longest_prefix_first() {
    let mut config = ProxyConfig::new();
    config.insert("/api".to_string(), route("http://localhost:3000"));
    config.insert("/api/v2".to_string(), route("http://localhost:4000"));

    let (path, matched) = match_proxy_route(&config, "/api/v2/users").unwrap();

    assert_eq!(path, "/api/v2");
    assert_eq!(matched.target, "http://localhost:4000");
  }

  #[test]
  fn builds_target_uri_with_query() {
    let route = route("http://localhost:3000");

    let uri = build_target_uri(&route, "/api/test", Some("foo=bar")).unwrap();

    assert_eq!(uri.to_string(), "http://localhost:3000/api/test?foo=bar");
  }

  #[test]
  fn builds_target_uri_without_trailing_slash_on_target() {
    let route = route("http://localhost:3000/");

    let uri = build_target_uri(&route, "/api/test", None).unwrap();

    assert_eq!(uri.to_string(), "http://localhost:3000/api/test");
  }

  #[test]
  fn applies_regex_path_rewrite() {
    let mut route = route("http://localhost:3000");
    route
      .path_rewrite
      .insert("^/api".to_string(), String::new());

    assert_eq!(apply_path_rewrite(&route, "/api/test"), "/test");
  }

  #[test]
  fn applies_string_path_rewrite_fallback() {
    let mut route = route("http://localhost:3000");
    route
      .path_rewrite
      .insert("/api".to_string(), "/v2".to_string());

    assert_eq!(apply_path_rewrite(&route, "/api/test"), "/v2/test");
  }
}
