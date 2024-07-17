# HTTP Server

Rewrite of the popular npm package [http-server](https://github.com/http-party/http-server/tree/master) in Rust with some extras included

`http-server` is a simple, zero-configuration command-line static HTTP server. It is powerful enough for production usage, but it's simple and hackable enough to be used for testing, local development and learning.


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
curl -L --url https://github.com/alshdavid/procmon/releases/latest/download/linux-amd64.tar.gz | tar -xvzf - -C $HOME/.local/bin

# MacOS ARM64 (Apple Silicon)
curl -L --url https://github.com/alshdavid/procmon/releases/latest/download/macos-arm64.tar.gz | tar -xvzf - -C $HOME/.local/bin

# Add to PATH if not already there:
echo "\nexport \PATH=\$PATH:\$HOME/.local/bin\n" >> $HOME/.zshrc
echo "\nexport \PATH=\$PATH:\$HOME/.local/bin\n" >> $HOME/.bashrc
```

### Windows

Download the binary from the [latest GitHub release](https://github.com/alshdavid-labs/alshx/releases/latest) and add it to your `PATH`

## Usage

```bash
# Use default configuration
http-server

# Arguments
http-server --no-cache --cors ./public
```

```
Usage: http-server [OPTIONS] [SERVE_DIR]

Arguments:
  [SERVE_DIR]  Target directory to serve [default: ./dist]

Options:
  -a, --address <ADDRESS>        [default: 0.0.0.0]
  -p, --port <PORT>              [default: 8080]
  -c, --cache-time <CACHE_TIME>  Set the cache control time [default: 3600]
  -H, --header <HEADERS>         Add custom headers
      --cors                     Enable CORS header
      --no-cache                 Disable cache control header
  -Q, --quiet                    Don't print any logs to terminal
  -h, --help                     Print help
```