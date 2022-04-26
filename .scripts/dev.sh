#!/bin/bash

set -e
# regenerate did
printf "\n > Generating marketplace.did...\n\n"
cargo run > marketplace/marketplace.did
cat marketplace/marketplace.did
printf "\n\n > Reinstalling canister...\n\n"
# reinstall canister (upgrade breaks cap?)
yarn marketplace:deploy $(dfx identity get-principal) $(cd cap && dfx canister id ic-history-router) "-m reinstall" <<<yes
printf "\n ✅ Success\n\n\a"