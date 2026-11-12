#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Requires Rust, Node >=18, pkg-config and ALSA development headers on Linux.
cargo build --release --locked --workspace
(cd ui && npm ci && npm run build)
./scripts/bundle-linux.sh
