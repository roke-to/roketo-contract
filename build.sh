#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-target}"

cargo fmt

mkdir -p ./res
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release -p streaming
cp $TARGET/wasm32-unknown-unknown/release/streaming.wasm ./res/streaming.wasm
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release -p finance
cp $TARGET/wasm32-unknown-unknown/release/finance.wasm ./res/finance.wasm
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release -p locked-token
cp $TARGET/wasm32-unknown-unknown/release/locked_token.wasm ./res/locked_token.wasm
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release -p roke-token
cp $TARGET/wasm32-unknown-unknown/release/roke_token.wasm ./res/roke_token.wasm
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release -p nft-roketo
cp $TARGET/wasm32-unknown-unknown/release/nft_roketo.wasm ./res/nft_roketo.wasm
