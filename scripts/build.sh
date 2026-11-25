#!/usr/bin/env bash
set -euo pipefail
project_root="$(cd -- "$(dirname -- "$0")/.." && pwd)"
export RUSTUP_HOME="$project_root/build/.rustup-home" CARGO_HOME="$project_root/build/.cargo-home" CARGO_TARGET_DIR="$project_root/build"
export PATH="$CARGO_HOME/bin:$PATH"
if ! command -v cargo >/dev/null 2>&1; then
 command -v curl >/dev/null 2>&1 || { echo 'curl é necessário para instalar Rust.' >&2; exit 1; }
 curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | env RUSTUP_HOME="$RUSTUP_HOME" CARGO_HOME="$CARGO_HOME" sh -s -- -y --no-modify-path --profile minimal
 export PATH="$CARGO_HOME/bin:$PATH"
fi
exec cargo build --release --manifest-path "$project_root/Cargo.toml"
