# HTTP Server

Rewrite of the popular npm package [http-server](https://github.com/http-party/http-server/tree/master) in Rust with some extras included

`http-server` is a simple, zero-configuration command-line static HTTP server. It is powerful enough for production usage, but it's simple and hackable enough to be used for testing, local development and learning.


## Installation

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