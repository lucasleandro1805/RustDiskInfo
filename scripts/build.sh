#!/usr/bin/env bash
set -euo pipefail
project_root="$(cd -- "$(dirname -- "$0")/.." && pwd)"
archive="$project_root/vendor/smartmontools-7.5.tar.gz"
archive_sha256=690b83ca331378da9ea0d9d61008c4b22dde391387b9bbad7f29387f2595f76e
export RUSTUP_HOME="$project_root/build/.rustup-home"
export CARGO_HOME="$project_root/build/.cargo-home"
export CARGO_TARGET_DIR="$project_root/build"
export PATH="$CARGO_HOME/bin:$PATH"
for tool in tar sha256sum make g++; do
 command -v "$tool" >/dev/null 2>&1 || { printf 'Ferramenta de compilação ausente: %s\n' "$tool" >&2; exit 1; }
done
if ! command -v cargo >/dev/null 2>&1; then
 command -v curl >/dev/null 2>&1 || { printf 'curl é necessário para instalar a toolchain Rust local.\n' >&2; exit 1; }
 curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | env RUSTUP_HOME="$RUSTUP_HOME" CARGO_HOME="$CARGO_HOME" sh -s -- -y --no-modify-path --profile minimal
 export PATH="$CARGO_HOME/bin:$PATH"
fi
printf '%s  %s\n' "$archive_sha256" "$archive" | sha256sum --check --status || { printf 'Arquivo smartmontools falhou na verificação SHA-256.\n' >&2; exit 1; }
cargo build --release --manifest-path "$project_root/Cargo.toml"
smartctl_source="$project_root/build/smartmontools-7.5"
if [[ ! -d "$smartctl_source" ]]; then
 mkdir -p "$smartctl_source"
 tar -xzf "$archive" -C "$smartctl_source" --strip-components=1
fi
if [[ ! -f "$smartctl_source/Makefile" ]]; then
 (cd "$smartctl_source" && ./configure --with-libcap-ng=no --with-libsystemd=no)
fi
make -C "$smartctl_source" -j2 smartctl
install -m 0755 "$smartctl_source/smartctl" "$project_root/build/release/smartctl"
install -m 0644 "$smartctl_source/drivedb.h" "$project_root/build/release/drivedb.h"
printf 'Aplicativo e smartctl empacotados em %s\n' "$project_root/build/release"
