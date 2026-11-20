#!/bin/sh
# Builds the plugin (wasm32, release) and writes the importable folder: dist/
# (figcraft-plugin.json + figcraft_plugin_test.wasm). That folder is the only place to import from.
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist"
cd "$ROOT"
cargo build --release --target wasm32-unknown-unknown
mkdir -p "$DIST"
/bin/cp -f "$ROOT/figcraft-plugin.json" "$DIST/figcraft-plugin.json"
/bin/cp -f "$ROOT/target/wasm32-unknown-unknown/release/figcraft_plugin_test.wasm" "$DIST/figcraft_plugin_test.wasm"
echo "Import this manifest: $DIST/figcraft-plugin.json"
wc -c "$DIST/figcraft-plugin.json" "$DIST/figcraft_plugin_test.wasm"
