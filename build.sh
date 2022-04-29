#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-target}"
set -e

cd "$(dirname $0)"

perl -i -pe 's/\["cdylib", "rlib"\]/\["cdylib"\]/' streaming/Cargo.toml
perl -i -pe 's/\["cdylib", "rlib"\]/\["cdylib"\]/' finance/Cargo.toml
perl -i -pe 's/\["cdylib", "rlib"\]/\["cdylib"\]/' examples/nft/Cargo.toml

cat .git/HEAD > VERSION.md
echo $'\ncommits count:' >> VERSION.md
git rev-list --count HEAD >> VERSION.md

RUSTFLAGS='-C link-arg=-s' cargo build --all --target wasm32-unknown-unknown --release
cp $TARGET/wasm32-unknown-unknown/release/streaming.wasm ./res/streaming.wasm
cp $TARGET/wasm32-unknown-unknown/release/finance.wasm ./res/finance.wasm
cp $TARGET/wasm32-unknown-unknown/release/non_fungible_token_w_roketo.wasm ./res/nft_roketo.wasm

perl -i -pe 's/\["cdylib"\]/\["cdylib", "rlib"\]/' streaming/Cargo.toml
perl -i -pe 's/\["cdylib"\]/\["cdylib", "rlib"\]/' finance/Cargo.toml
perl -i -pe 's/\["cdylib"\]/\["cdylib", "rlib"\]/' examples/nft/Cargo.toml
