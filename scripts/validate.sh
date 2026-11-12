#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo test --release --locked --workspace
(cd ui && npm ci && npm run build)
cargo build --release --locked -p prophet-render
mkdir -p evidence/wavs evidence/probes
for rate in 48000 96000; do target/release/prophet-render --demo-all evidence/wavs --sample-rate "$rate" --seconds 5; done
target/release/prophet-probes evidence/probes
MPLCONFIGDIR="${TMPDIR:-/tmp}/prophet-matplotlib" python3 scripts/analyze.py
target/release/prophet-bench > evidence/benchmarks.csv
