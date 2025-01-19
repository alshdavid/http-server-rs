#!/bin/bash
set -e

# Default to current latest
if [ "$JUST_VERSION" = "" ]; then
  echo No Just Version Specified
  exit 1
fi 

# Default to home directory
if [ "$OUT_DIR" = "" ]; then
  OUT_DIR="$HOME/.local/just"
fi 

URL=""
ARCH=""
OS=""

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
    URL=https://github.com/casey/just/releases/download/${JUST_VERSION}/just-${JUST_VERSION}-x86_64-unknown-linux-musl.tar.gz
  ;;
  linux-arm64)
    URL=https://github.com/casey/just/releases/download/${JUST_VERSION}/just-${JUST_VERSION}-aarch64-unknown-linux-musl.tar.gz
  ;;
  macos-amd64)
    URL=https://github.com/casey/just/releases/download/${JUST_VERSION}/just-${JUST_VERSION}-x86_64-apple-darwin.tar.gz
  ;;
  macos-arm64)
    URL=https://github.com/casey/just/releases/download/${JUST_VERSION}/just-${JUST_VERSION}-aarch64-apple-darwin.tar.gz
  ;;
esac

if [ "$URL" == "" ]; then
  echo "Cannot find installer for Just"
  exit 1
fi

echo $URL

test -d $OUT_DIR && rm -rf $OUT_DIR
mkdir -p $OUT_DIR
curl -s -L --url $URL | tar -xzf - -C $OUT_DIR

export PATH="${OUT_DIR}:$PATH"
echo "${OUT_DIR}" >> $GITHUB_PATH
