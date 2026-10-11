# http-server: a simple static HTTP server 🚀🦀

Rewrite of the popular npm package [http-server](https://github.com/http-party/http-server/tree/master) in Rust with some extras included.

`http-server` is a simple, zero-configuration command-line static HTTP server. It is powerful enough for production usage, but it's simple and hackable enough to be used for testing, local development and learning.

https://github.com/user-attachments/assets/7b6bbee0-3428-4c9f-80a6-2804ddac6e01

## Installation

### Cargo

```bash
# Installs globally
cargo install http-server-rs
```

### NPM

```bash
# Installs to ./node_modules
npm install http-server-rs
npx http-server-rs --help
```

If you want to override the binary used (for custom platforms)
```bash
npm install http-server-rs

export HTTP_SERVER_RS_BIN_OVERRIDE=/path/to/http-server-rs
npx http-server-rs --help # Uses the override
```

### Manual Install

Download the latest binary from [GitHub Releases](https://github.com/alshdavid-public/http-server-rs/releases/latest) and add it to your `PATH`.

Update with `http-server-rs update`

```bash
# Linux AMD64 (Change for your platform)
curl -L --url https://github.com/alshdavid/http-server-rs/releases/latest/download/http-server-rs-linux-amd64.tar.gz | tar -xvzf - -C $HOME/.local/bin

# Add to PATH if not already there:
echo "\nexport \PATH=\$PATH:\$HOME/.local/bin\n" >> $HOME/.zshrc
echo "\nexport \PATH=\$PATH:\$HOME/.local/bin\n" >> $HOME/.bashrc
```

## Example Usage

```bash
# Use default configuration
http-server-rs serve

# Enable CORS, reroute requests to index.html and automatically compress served files
http-server-rs serve --cors --spa -Z ./public

# Responds with headers required for SharedArrayBuffer
http-server-rs serve --shared-array-buffer ./public

# Automatically transpiles TypeScript to JavaScript on request
http-server-rs serve --transpile ./public

# Proxy /api to an external backend (quotes are required)
http-server-rs serve --proxy "path=/api&target=http://localhost:3000" ./public

# Custom Headers
http-server-rs serve -H X-Custom-Header:some-value

# Update the binary to the latest release
http-server-rs update
```

```
Usage: http-server-rs <COMMAND>

Commands:
  serve   Serve a target directory over HTTP
  update  Update the binary to the latest release
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

```
Usage: http-server-rs serve [OPTIONS] [SERVE_DIR]

Arguments:
  [SERVE_DIR]  Target directory to serve [default: .]

Options:
  -a, --address <ADDRESS>
          [default: 0.0.0.0]
  -p, --port <PORT>
          [default: 8080]
      --spa
          Redirect requests to /index.html for Single Page Applications
  -c, --cache-time <CACHE_TIME>
          Cache control time [default: 0]
  -Z, --compress
          Compress responses (JIT)
  -H, --header <HEADERS>
          Custom headers (Format "key:value")
      --auth <BASIC_AUTH>
          Put server behind basic auth (Format "username:password")
      --cors
          Enable CORS header
  -S, --shared-array-buffer
          Enable headers for SharedArrayBuffer
  -Q, --quiet
          Don't print any logs to terminal
  -t, --transpile
          Transpile served files
      --no-source-maps
          Don't generate source maps for transpiled files
  -w, --watch
          Watch folder for changes and trigger a browser reload
      --watch-dir <WATCH_DIR>
          Watch for changes [default: SERVE_DIR]
      --watch-debounce-delay <WATCH_DEBOUNCE_DELAY>
          Debounce delay in milliseconds before watched changes trigger a reload [default: 200]
      --no-watch-inject
          Don't automatically inject watch listener into html
      --skip-upgrade-check
          Skip checking for a newer release on startup
      --stream-buffer-size <STREAM_BUFFER_SIZE>
          Configure the buffer size when streaming files [default: 4000]
      --proxy <PROXY>
          Reverse proxy routes (URL-encoded, e.g. "path=/api&target=http://localhost:3000&secure=false&change_origin=false&path_rewrite[key]=^/api&path_rewrite[value]=")
  -h, --help
          Print help
```

## On-The-Fly Transpile TypeScript

Will on-the-fly transpile TypeScript to JavaScript on request. This allows serving TypeScript sources 
directly to the browser without a build step.

The idea is that you write your web application as if your browser natively supports TypeScript. It's equivalent to writing your application in module JavaScript with jsdoc, but using TypeScript.

This is to simplify local development for bundler-less configurations. Uses native ESM imports built into your browser allowing for a zero-build step development experience.

```bash
http-server-rs serve --transpile --port 8080 ./testing/transpile/react

# Returns transpiled JavaScript that runs directly in the browser
curl http://localhost:8080/app.tsx 
```

https://github.com/user-attachments/assets/3d1bc313-e898-42bb-8c8f-a8023c41ec35

Notes;
- Use builtin `import` syntax
- Libraries are fetched from a CDN (like [esm.sh](https://esm.sh/#tsx)) or vendored locally (no `npm`)
- [importmap](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/script/type/importmap) is used to map ESM specifier (e.g. `import 'react'`)  
- At the moment, Deno is used for type checking.
  - Unfortunately, [TypeScript does not support remote declaration file](https://github.com/microsoft/TypeScript/issues/28985). If you want to use tsc you need to install packages using `npm` and configure a `tsconfig.json`.
- For performance sensitive applications, you'll still probably want to use a bundler for production - but who know, this might be good enough for most use cases

## Watch Mode

`http-server` under `--watch` mode can watch the served directory for changes and emit an event to the client notifying of a change. By default the server will intercept html files and inject a JavaScript script which subscribes to change events and triggers a page reload.

```bash
http-server-rs serve --watch ./dist
```

To customize the reload functionality, disable the auto-inject script, manually subscribe to change events and trigger the desired functionality.

```bash
http-server-rs serve --watch --no-watch-inject ./dist
```

```html
<html>
  <head>
    <script>
      new EventSource("/.http-server-rs/reload")
        .onmessage = () => window.location.reload();
    </script>
  </head>
  <body>
    <script src="./app.js"></script>
  </body>
</html>
```
