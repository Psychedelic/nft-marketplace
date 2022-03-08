#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../wicp || exit 1

DEFAULT_USER_WALLET=$(dfx identity get-wallet)

# args
account="$1"

dfx canister --wallet "$DEFAULT_USER_WALLET" \
  call --query wicp \
  balanceOf "(
    principal \"$account\"
  )"