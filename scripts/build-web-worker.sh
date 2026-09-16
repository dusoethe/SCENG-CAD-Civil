#!/bin/sh
set -eu

target_dir="${CARGO_TARGET_DIR:-target}"
cargo build --locked --release --target wasm32-unknown-unknown --package ocs_web_worker
worker_out="${TRUNK_STAGING_DIR:?}/worker_pkg"
mkdir -p "$worker_out"
wasm-bindgen \
  --target web \
  --out-dir "$worker_out" \
  --out-name ocs_web_worker \
  "$target_dir/wasm32-unknown-unknown/release/ocs_web_worker.wasm"
