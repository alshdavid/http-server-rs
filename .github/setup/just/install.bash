#!/bin/bash
set -e

# Default to current latest
target_version="$JUST_VERSION"
if [ "$target_version" = "" ]; then
  target_version=$(curl --silent "https://api.github.com/repos/casey/just/releases/latest" | jq -r '.tag_name')
fi 

# Default to home directory
if [ "$out_dir" = "" ]; then
  out_dir="$HOME/.local/just"
fi 

# Detect environment
url=""
arch=""
platform=""

case $(uname -m) in
  x86_64 | x86-64 | x64 | amd64)
    arch="amd64"
  ;;
  aarch64 | arm64)
    arch="arm64"
  ;;
esac

case $(uname -s) in
  Darwin)
    platform="macos"
  ;;
  Linux)
    platform="linux"
  ;;
  MSYS_NT*)
    platform="windows"
  ;;
esac

echo "Installing $platform-$arch"

# Pick URL
case "$platform-$arch" in
  linux-amd64)
    url=https://github.com/casey/just/releases/download/${target_version}/just-${target_version}-x86_64-unknown-linux-musl.tar.gz
  ;;
  linux-arm64)
    url=https://github.com/casey/just/releases/download/${target_version}/just-${target_version}-aarch64-unknown-linux-musl.tar.gz
  ;;
  macos-amd64)
    url=https://github.com/casey/just/releases/download/${target_version}/just-${target_version}-x86_64-apple-darwin.tar.gz
  ;;
  macos-arm64)
    url=https://github.com/casey/just/releases/download/${target_version}/just-${target_version}-aarch64-apple-darwin.tar.gz
  ;;
  windows-amd64)
    url=https://github.com/casey/just/releases/download/${target_version}/just-${target_version}-x86_64-pc-windows-msvc.zip
  ;;
  windows-arm64)
    url=https://github.com/casey/just/releases/download/${target_version}/just-${target_version}-aarch64-pc-windows-msvc.zip
  ;;
esac

if [ "$url" == "" ]; then
  echo "Cannot find installer for Just"
  exit 1
fi

# Install into directory
echo $url

test -d $out_dir && rm -rf $out_dir
mkdir -p $out_dir

curl -s -L --url $url | tar -xzf - -C $out_dir

export PATH="${out_dir}:$PATH"

if [ "$GITHUB_PATH" != "" ]; then
  echo "${out_dir}" >> $GITHUB_PATH
fi

# Debug
just --version