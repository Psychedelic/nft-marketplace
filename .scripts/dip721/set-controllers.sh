#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../DIP721 || exit 1

# Args
CONTROLLER_MAIN=$1
DIP721_TOKEN_CONTRACT_ID=$2

echo "[debug] set-controllers.sh: DEFAULT_PRINCIPAL_ID $CONTROLLER_MAIN"

echo "[debug] set-controllers.sh: update_settings (
  record {
    canister_id=principal \"$DIP721_TOKEN_CONTRACT_ID\";
    settings=record {
      controllers=opt vec{
        principal\"$CONTROLLER_MAIN\";
        principal\"$DIP721_TOKEN_CONTRACT_ID\";
      }
    }
  }
)"

echo "pwd"
pwd

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