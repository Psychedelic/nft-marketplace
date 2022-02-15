#!/bin/bash

cd $(dirname $BASH_SOURCE) || exit 1

IC_HISTORY_ROUTER=$(cd ../../cap && dfx canister id ic-history-router)

cd ../../wicp || exit 1

dfx canister create wicp

cargo build --target wasm32-unknown-unknown --package rust --release \
	&& ic-cdk-optimizer target/wasm32-unknown-unknown/release/rust.wasm \
	-o target/wasm32-unknown-unknown/release/opt.wasm

OWNER=$1
FEES_TO=$1

yes yes | dfx canister install wicp \
	--argument="(
		\"data:image/jpeg;base64,$(base64 ./WICP-logo.png)\",
		\"wicp\",
		\"WICP\",
		8:nat8,
	  1000000000000:nat,
		principal \"$OWNER\",
		0,
		principal \"$FEES_TO\",
		principal \"$IC_HISTORY_ROUTER\"
	)" \
  --mode="reinstall"
