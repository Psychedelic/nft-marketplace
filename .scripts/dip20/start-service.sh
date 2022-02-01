#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

source "../dfx-identity.sh"

cd ../../DIP20/motoko || exit 1

BASE64_LOGO="No logo"
NAME="Test coin"
SYMBOL="TCI"
DECIMALS=9
TOTAL_SUPPLY=1000000
OWNER=$(dfx identity get-principal)
FEE=10000

dfx deploy --no-wallet token --argument "(
  \"$BASE64_LOGO\",
  \"$NAME\",
  \"$SYMBOL\",
  $DECIMALS,
  $TOTAL_SUPPLY,
  principal \"$OWNER\",
  $FEE
)"
