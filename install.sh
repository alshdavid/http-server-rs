#!/bin/bash
set -e

# Default to current latest
VERSION="$HTTP_SERVER_RS_VERSION"
if [ "$VERSION" = "" ]; then
  VERSION=$(curl --silent "https://api.github.com/repos/alshdavid/http-server-rs/releases/latest" | jq -r '.tag_name')
fi

if [ "$VERSION" = "" ]; then
  echo "Unable to fetch version"
  exit 1
fi

# Default to home directory
if [ "$OUT_DIR" = "" ]; then
  OUT_DIR="$HOME/.local/http-server-rs"
fi

>&2 echo VERSION: $VERSION
>&2 echo OUT_DIR: $OUT_DIR

ARCH=""
case "$(uname -m)" in
  x86_64 | x86-64 | x64 | amd64) ARCH="amd64";;
  aarch64 | arm64) ARCH="arm64";;
  *) ARCH="";;
esac

OS=""
case "$(uname -s)" in
  Darwin) OS="macos";;
  Linux) OS="linux";;
  MINGW64_NT* | Windows_NT) OS="windows";;
  *) OS="";;
esac

>&2 echo ARCH: $ARCH
>&2 echo OS: $OS

URL=""
case "$OS-$ARCH" in
  linux-amd64) URL="https://github.com/alshdavid/http-server-rs/releases/download/${VERSION}/http-server-linux-amd64.tar.gz";;
  linux-arm64) URL="https://github.com/alshdavid/http-server-rs/releases/download/${VERSION}/http-server-linux-arm64.tar.gz";;
  macos-amd64) URL="https://github.com/alshdavid/http-server-rs/releases/download/${VERSION}/http-server-macos-amd64.tar.gz";;
  macos-arm64) URL="https://github.com/alshdavid/http-server-rs/releases/download/${VERSION}/http-server-macos-arm64.tar.gz";;
  windows-amd64) URL="https://github.com/alshdavid/http-server-rs/releases/download/${VERSION}/http-server-windows-amd64.tar.gz";;
  windows-arm64) URL="https://github.com/alshdavid/http-server-rs/releases/download/${VERSION}/http-server-windows-arm64.tar.gz";;
esac

if [ "$URL" = "" ]; then
  echo "Cannot find archive"
  exit 1
fi

>&2 echo URL: $URL

test -d $OUT_DIR && rm -rf $OUT_DIR
mkdir -p $OUT_DIR

if [ -z "${URL##*.tar.gz}" ]; then
  curl -s -L --url $URL | tar -xzf - -C $OUT_DIR
  chmod +x $OUT_DIR/http-server*
fi

echo "export PATH=\"${OUT_DIR}:\$PATH\""
>&2 echo "======="
>&2 echo "Add this to \$PATH"
>&2 echo "  export PATH=\"${OUT_DIR}:\$PATH\""

if [ "$GITHUB_PATH" != "" ]; then
  echo "${OUT_DIR}" >> $GITHUB_PATH
fi