#!/bin/bash

(cd "$(dirname $BASH_SOURCE)" && cd ../..) || exit 1

printf "🤖 Check if ICX Cli is available\n"

ICX_TEMP_DIR=$(mktemp -d 2>/dev/null || mktemp -d -t bob-temp)

if ! command -v icx --version /dev/null
then
  OS_ICX_DIRNAME="macos"

  # Override if linux
  if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS_ICX_DIRNAME="stable-x86_64-unknown-linux-gnu"
  fi

  cd ./.bin/$OS_ICX_DIRNAME || exit 1
  gzip -d < icx.gz > "$ICX_TEMP_DIR"/icx
  chmod +x "$ICX_TEMP_DIR"/icx
  PATH=$PATH:$ICX_TEMP_DIR

  icx --version
fi