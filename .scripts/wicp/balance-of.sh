#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../wicp || exit 1

# args
account="$1"

dfx canister --no-wallet \
  call --query wicp \
  balanceOf "(
    principal \"$account\"
  )"