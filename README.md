# http-server: a simple static HTTP server 🚀🦀

Rewrite of the popular npm package [http-server](https://github.com/http-party/http-server/tree/master) in Rust with some extras included.

`http-server` is a simple, zero-configuration command-line static HTTP server. It is powerful enough for production usage, but it's simple and hackable enough to be used for testing, local development and learning.

https://github.com/user-attachments/assets/7b6bbee0-3428-4c9f-80a6-2804ddac6e01

## Installation

### NPM

```bash
# Rerun to update
npm install -g http-server-rs
```

### Binary Install Script

```bash
# Rerun to update
eval $(curl -sSf https://raw.githubusercontent.com/alshdavid/http-server-rs/refs/heads/main/install.sh | sh)
```

## Usage

```bash
# Use default configuration
http-server

# Arguments
# Enable CORS, reroute requests to index.html and automatically compress to brotli 
http-server --cors --spa -Z ./public

# Custom Headers
http-server -H X-Custom-Header:some-value
```

```
Usage: http-server [OPTIONS] [SERVE_DIR]

Arguments:
  [SERVE_DIR]  Target directory to serve [default: ./dist]

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
  -w, --watch
          Watch folder for changes and trigger a browser reload
      --watch-dir <WATCH_DIR>
          Watch for changes [default: SERVE_DIR]
      --no-watch-inject
          Don't automatically inject watch listener into html
      --stream-buffer-size <STREAM_BUFFER_SIZE>
          Configure the buffer size when streaming files [default: 4000]
  -h, --help
          Print help
```

## Watch Mode

`http-server` under `--watch` mode can watch the served directory for changes and emit an event to the client notifying of a change. By default the server will intercept html files and inject a JavaScript script which subscribes to change events and triggers a page reload.

```bash
http-server --watch ./dist
```

To customize the reload functionality, disable the auto-inject script, manually subscribe to change events and trigger the desired functionality.

```bash
http-server --watch --no-watch-inject ./dist
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

## Installation

### MacOS & Linux

Download the binary from the [latest GitHub release](https://github.com/alshdavid/http-server-rs/releases/latest) and add it to your `PATH`

```shell
# Linux AMD64
curl -L --url https://github.com/alshdavid/http-server-rs/releases/latest/download/http-server-linux-amd64.tar.gz | tar -xvzf - -C $HOME/.local/bin

# Linux ARM64
curl -L --url https://github.com/alshdavid/http-server-rs/releases/latest/download/http-server-linux-arm64.tar.gz | tar -xvzf - -C $HOME/.local/bin

# MacOS ARM64 (Apple Silicon)
curl -L --url https://github.com/alshdavid/http-server-rs/releases/latest/download/http-server-macos-arm64.tar.gz | tar -xvzf - -C $HOME/.local/bin 

# MacOS AMD64 (Intel)
curl -L --url https://github.com/alshdavid/http-server-rs/releases/latest/download/http-server-macos-amd64.tar.gz | tar -xvzf - -C $HOME/.local/bin

# Add to PATH if not already there:
echo "\nexport \PATH=\$PATH:\$HOME/.local/bin\n" >> $HOME/.zshrc
echo "\nexport \PATH=\$PATH:\$HOME/.local/bin\n" >> $HOME/.bashrc
```

### Windows

Download the binary from the [latest GitHub release](https://github.com/alshdavid/http-server-rs/releases/latest) and add it to your `PATH`
