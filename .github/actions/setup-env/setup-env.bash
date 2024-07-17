#!/bin/bash
set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
ROOT_DIR=$(dirname $(dirname $(dirname $SCRIPT_DIR)))

cd $ROOT_DIR

# Just
export PATH="${HOME}/.local/just:$PATH"

# Rust
export RUSTUP_HOME="${HOME}/.local/rust/rustup"
export CARGO_HOME="${HOME}/.local/rust/cargo"
export PATH="${HOME}/.local/rust/cargo/bin:$PATH"

ENV_PATH="${ROOT_DIR}/.env"

if [ -f "$ENV_PATH" ]; then
  echo "Reading $ENV_PATH"
  while read -r LINE; do
    # Remove leading and trailing whitespaces, and carriage return
    CLEANED_LINE=$(echo "$LINE" | awk '{$1=$1};1' | tr -d '\r')

    if [[ $CLEANED_LINE != '#'* ]] && [[ $CLEANED_LINE == *'='* ]]; then
      echo "Setting: $CLEANED_LINE"
      export "$CLEANED_LINE"
    fi
  done < "$ENV_PATH"
fi

