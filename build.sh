#!/bin/bash

cargo build --target wasm32-unknown-unknown --release --package marketplace

ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/marketplace.wasm -o ./target/wasm32-unknown-unknown/release/marketplace-opt.wasm
