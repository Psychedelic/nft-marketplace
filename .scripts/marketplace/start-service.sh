#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

source "../dfx-identity.sh"

cd ../../ || exit 1

dfx deploy --no-wallet marketplace --argument "(
  principal \"$(cd ./cap && dfx canister id ic-history-router)\",
  principal \"$(dfx identity get-principal)\"
)"
