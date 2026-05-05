#!/usr/bin/env bash
# Cloudflare Pages build script for the family-tree-app crate.
# Bootstraps Rust toolchain + trunk, then produces dist/ for Pages to serve.
# Mirrors the local toolchain pinned in rust-toolchain.toml.
set -euo pipefail

RUST_VERSION="1.95.0"
TRUNK_VERSION="0.21.14"

if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain "${RUST_VERSION}" --profile minimal
fi
# shellcheck disable=SC1091
. "${HOME}/.cargo/env"
rustup target add wasm32-unknown-unknown

TRUNK_URL="https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz"
mkdir -p "${HOME}/bin"
curl -sSfL "${TRUNK_URL}" | tar -xz -C "${HOME}/bin"
chmod +x "${HOME}/bin/trunk"
export PATH="${HOME}/bin:${PATH}"

cd crates/family-tree-app
trunk build
