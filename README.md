# http-server: a simple static HTTP server 🚀🦀

Rewrite of the popular npm package [http-server](https://github.com/http-party/http-server/tree/master) in Rust with some extras included.

`http-server` is a simple, zero-configuration command-line static HTTP server. It is powerful enough for production usage, but it's simple and hackable enough to be used for testing, local development and learning.

## Usage

```bash
# Use default configuration
http-server

# Arguments
http-server --no-cache --cors ./public

# Custom Headers
http-server -H Cross-Origin-Opener-Policy:same-origin
```

```
Usage: http-server [OPTIONS] [SERVE_DIR]

Arguments:
  [SERVE_DIR]  Target directory to serve [default: ./dist]

Options:
  -a, --address <ADDRESS>        [default: 0.0.0.0]
  -p, --port <PORT>              [default: 8080]
      --spa                      Redirect requests to /index.html for Single Page Applications
  -c, --cache-time <CACHE_TIME>  Cache control time [default: 0]
  -H, --header <HEADERS>         Custom headers (Format "key:value")
      --cors                     Enable CORS header
      --no-cache                 Disable cache control header
  -Q, --quiet                    Don't print any logs to terminal
  -w, --watch                    Watch folder for changes and trigger a browser reload
  -h, --help                     Print help
```

## Installation

### MacOS & Linux

#### Install & Update Script

This will prompt you for the install path, run it again to update the version

```shell
curl -s "https://raw.githubusercontent.com/alshdavid/http-server-rs/main/scripts/install.sh" | sh
```

#### Manual

Download the binary from the [latest GitHub release](https://github.com/alshdavid-labs/alshx/releases/latest) and add it to your `PATH`

```shell
# Linux AMD64
curl -L --url https://github.com/alshdavid/http-server-rs/releases/latest/download/linux-amd64.tar.gz | tar -xvzf - -C $HOME/.local/bin --strip-components=1

# Linux ARM64
curl -L --url https://github.com/alshdavid/http-server-rs/releases/latest/download/linux-arm64.tar.gz | tar -xvzf - -C $HOME/.local/bin --strip-components=1

# MacOS ARM64 (Apple Silicon)
curl -L --url https://github.com/alshdavid/http-server-rs/releases/latest/download/macos-arm64.tar.gz | tar -xvzf - -C $HOME/.local/bin --strip-components=1

# MacOS AMD64 (Intel)
curl -L --url https://github.com/alshdavid/http-server-rs/releases/latest/download/macos-amd64.tar.gz | tar -xvzf - -C $HOME/.local/bin --strip-components=1

# Add to PATH if not already there:
echo "\nexport \PATH=\$PATH:\$HOME/.local/bin\n" >> $HOME/.zshrc
echo "\nexport \PATH=\$PATH:\$HOME/.local/bin\n" >> $HOME/.bashrc
```

### Windows

Download the binary from the [latest GitHub release](https://github.com/alshdavid-labs/alshx/releases/latest) and add it to your `PATH`
