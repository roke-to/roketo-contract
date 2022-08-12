#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-target}"
set -e

cd "$(dirname $0)"

perl -i -pe 's/\["cdylib", "rlib"\]/\["cdylib"\]/' streaming/Cargo.toml
perl -i -pe 's/\["cdylib", "rlib"\]/\["cdylib"\]/' finance/Cargo.toml
perl -i -pe 's/\["cdylib", "rlib"\]/\["cdylib"\]/' roke_token/Cargo.toml
perl -i -pe 's/\["cdylib", "rlib"\]/\["cdylib"\]/' locked_token/Cargo.toml
perl -i -pe 's/\["cdylib", "rlib"\]/\["cdylib"\]/' examples/nft/Cargo.toml

cargo fmt

RUSTFLAGS='-C link-arg=-s' cargo build --all --target wasm32-unknown-unknown --release
mkdir -p ./res
cp $TARGET/wasm32-unknown-unknown/release/streaming.wasm ./res/streaming.wasm
cp $TARGET/wasm32-unknown-unknown/release/finance.wasm ./res/finance.wasm
cp $TARGET/wasm32-unknown-unknown/release/locked_token.wasm ./res/locked_token.wasm
cp $TARGET/wasm32-unknown-unknown/release/roke_token.wasm ./res/roke_token.wasm
cp $TARGET/wasm32-unknown-unknown/release/nft_roketo.wasm ./res/nft_roketo.wasm

perl -i -pe 's/\["cdylib"\]/\["cdylib", "rlib"\]/' streaming/Cargo.toml
perl -i -pe 's/\["cdylib"\]/\["cdylib", "rlib"\]/' finance/Cargo.toml
perl -i -pe 's/\["cdylib"\]/\["cdylib", "rlib"\]/' roke_token/Cargo.toml
perl -i -pe 's/\["cdylib"\]/\["cdylib", "rlib"\]/' locked_token/Cargo.toml
perl -i -pe 's/\["cdylib"\]/\["cdylib", "rlib"\]/' examples/nft/Cargo.toml
