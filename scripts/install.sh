#!/bin/sh

set -e

resolve_relative_path() (
    if [ -d "$1" ]; then
        cd "$1" || return 1
        pwd
    elif [ -e "$1" ]; then
        if [ ! "${1%/*}" = "$1" ]; then
            cd "${1%/*}" || return 1
        fi
        echo "$(pwd)/${1##*/}"
    else
        return 1
    fi
)

DEFAULT_INSTALL_PATH="$HOME/.local/bin"

read -p "Where to install binary? [default: $DEFAULT_INSTALL_PATH] " INSTALL_PATH

if [ "$INSTALL_PATH" = "" ]; then
  INSTALL_PATH="$DEFAULT_INSTALL_PATH"
fi

case $INSTALL_PATH in
/*)
    INSTALL_PATH=$INSTALL_PATH
    ;;
\$*)
    INSTALL_PATH=$(eval "echo \"$INSTALL_PATH\"")
    ;;
\~)
    INSTALL_PATH=$(eval "echo \"$HOME\"")
    ;;
\~\/*)
    INSTALL_PATH=$(eval "echo \"$HOME${INSTALL_PATH:1}\"")
    ;;
*)
    echo "  ERROR: Invalid install path, must be absolute path"
    echo "         $INSTALL_PATH"
    exit 1
    ;;
esac

echo Installing to: $INSTALL_PATH

exit 0
if ! [ -d "$INSTALL_PATH" ]; then
  echo "  WARN: Install path does not exist, creating"
  mkdir -p $INSTALL_PATH
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

if [ "$HAS_INSTALL_IN_PATH" = "false" ]; then
  echo
  echo "WARN $INSTALL_PATH is not in your \$PATH"
  echo
  echo "You can add it with:"
  echo "  echo \"\\\\nexport PATH=\\\"\\\$PATH:${INSTALL_PATH}\\\"\\\\n\" >> \$HOME/.zshrc"
  echo "  echo \"\\\\nexport PATH=\\\"\\\$PATH:${INSTALL_PATH}\\\"\\\\n\" >> \$HOME/.bashrc"
  echo "  export PATH=\$PATH:\$HOME/.local/bin"
fi


