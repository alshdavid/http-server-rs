#!/bin/sh

set -e

DEFAULT_INSTALL_PATH="$HOME/.local/bin"

read -p "Where to install binary? [default: \$HOME/.local/bin] " INSTALL_PATH

if [ "$INSTALL_PATH" = "" ]; then
  INSTALL_PATH="$DEFAULT_INSTALL_PATH"
fi

echo Installing to: $INSTALL_PATH

if ! [ -d "$INSTALL_PATH" ]; then
  echo "  ERROR: Install path does not exist"
  exit 1
fi


HAS_INSTALL_IN_PATH="false"
for SEG in $(echo $PATH | tr ":" "\n")
do
  if [ "$SEG" = "$INSTALL_PATH" ]; then
    HAS_INSTALL_IN_PATH="true"
  fi
done

if [ "$HAS_INSTALL_IN_PATH" = "false" ]; then
  echo "  WARN: Not in \$PATH: $INSTALL_PATH"
fi

ARCH=""
OS=""
URL=""

case $(uname -m) in
  x86_64 | x86-64 | x64 | amd64)
    ARCH="amd64"
  ;;
  aarch64 | arm64)
    ARCH="arm64"
  ;;
esac

case $(uname -s) in
  Darwin)
    OS="macos"
  ;;
  Linux)
    OS="linux"
  ;;
esac

case "$OS-$ARCH" in
  linux-amd64)
    URL=https://github.com/alshdavid/http-server-rs/releases/latest/download/http-server-linux-amd64.tar.gz
  ;;
  linux-arm64)
    URL=https://github.com/alshdavid/http-server-rs/releases/latest/download/http-server-linux-arm64.tar.gz
  ;;
  macos-amd64)
    URL=https://github.com/alshdavid/http-server-rs/releases/latest/download/http-server-macos-amd64.tar.gz
  ;;
  macos-arm64)
    URL=https://github.com/alshdavid/http-server-rs/releases/latest/download/http-server-macos-arm64.tar.gz
  ;;
esac

rm -rf $INSTALL_PATH/http-server
curl -L --url $URL | tar -xzf - -C $INSTALL_PATH --strip-components=1
chmod +x $INSTALL_PATH/http-server
