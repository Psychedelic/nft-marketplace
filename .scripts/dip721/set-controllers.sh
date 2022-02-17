#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../DIP721 || exit 1

echo "[debug] pwd"
pwd

# Args
CONTROLLER_MAIN=$1
DIP721_TOKEN_CONTRACT_ID=$2

dfx canister --no-wallet \
call aaaaa-aa \
update_settings "(
  record {
    canister_id=principal \"$DIP721_TOKEN_CONTRACT_ID\";
    settings=record {
      controllers=opt vec{
        principal \"$CONTROLLER_MAIN\";
        principal \"$DIP721_TOKEN_CONTRACT_ID\";
      }
    }
  }
)"