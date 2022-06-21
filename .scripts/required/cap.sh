#!/bin/bash

CANISTER_CAP_ID=$(cd ./cap && dfx canister id cap-router)

if [ -z "$CANISTER_CAP_ID" ];
then
  printf "🤖 Oops! The Cap Service Canister (%s) is not running...\n\n" "$CANISTER_CAP_ID"

  exit 1
fi

printf "🌈 Cap Service running as canister id (%s)\n\n" "$CANISTER_CAP_ID"
