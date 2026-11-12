#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
out=binaries/linux-x86_64
mkdir -p "$out/PROPHET-XII.vst3/Contents/x86_64-linux"
for name in prophet-standalone prophet-render prophet-probes prophet-bench prophet-presets; do cp "target/release/$name" "$out/$name"; done
cp target/release/libprophet_plugin.so "$out/PROPHET-XII.clap"
cp target/release/libprophet_plugin.so "$out/PROPHET-XII.vst3/Contents/x86_64-linux/PROPHET-XII.so"
