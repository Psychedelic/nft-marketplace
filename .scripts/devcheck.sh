#!/bin/bash

set -e
# regenerate did
printf "\nGenerating marketplace.did...\n\n"
cargo run > marketplace/marketplace.did
cat marketplace/marketplace.did
printf "\nReinstalling canister...\n\n"
# reinstall canister (upgrade breaks cap?)
yarn marketplace:deploy $(dfx identity get-principal) $(cd cap && dfx canister id ic-history-router) "-m reinstall" <<<yes
# run healthcheck
#yarn marketplace:healthcheck