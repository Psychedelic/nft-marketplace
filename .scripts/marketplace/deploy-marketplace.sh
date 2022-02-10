#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

source "../dfx-identity.sh"

cd ../../ || exit 1

# Args
IC_HISTORY_ROUTER=$1
OWNER_ID=$2

dfx deploy --no-wallet --mode reinstall marketplace --argument "(
  principal \"$IC_HISTORY_ROUTER\",
  principal \"$OWNER_ID\"
)"
