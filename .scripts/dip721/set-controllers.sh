#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

cd ../../DIP721 || exit 1

# Args
CONTROLLER_MAIN=$1

dfx canister --no-wallet \
call aaaaa-aa \
update_settings "(
  record {
    canister_id=principal \"$(dfx canister id nft)\";
    settings=record {
      controllers=opt vec{
        principal\"$CONTROLLER_MAIN\";
        principal\"$(dfx canister id nft)\";
      }
    }
  }
)"