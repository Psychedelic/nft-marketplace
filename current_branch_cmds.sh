#!/bin/bash

# yarn marketplace:deploy $(dfx identity get-principal) $(cd ./cap && dfx canister id ic-history-router)

dfx --identity marketplace.alice canister call --update qaa6y-5yaaa-aaaaa-aaafa-cai approve "( principal \"rdmx6-jaaaa-aaaaa-aaadq-cai\", 1_000_000_000:nat )"
dfx --identity marketplace.alice canister call --update rdmx6-jaaaa-aaaaa-aaadq-cai makeOffer "( principal \"rkp4c-7iaaa-aaaaa-aaaca-cai\", 0:nat, 1242:nat )"
dfx --identity marketplace.alice canister call --update rdmx6-jaaaa-aaaaa-aaadq-cai makeOffer "( principal \"rkp4c-7iaaa-aaaaa-aaaca-cai\", 1:nat, 1243:nat )"
dfx canister call --update rdmx6-jaaaa-aaaaa-aaadq-cai getTokenOffers "( principal \"rkp4c-7iaaa-aaaaa-aaaca-cai\", principal \"$(dfx identity get-principal)\")"