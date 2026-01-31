#!/bin/bash
set -e

# L-0 Release Build Script
# Builds all workspace crates

echo "=== L-0 Workspace Build ==="
cargo build --release --workspace
echo ""
echo "Build complete! Binaries in target/release/"
echo "  - l0vm"
echo "  - l0asm"
echo "  - l0cc"
echo "  - file_plugin"
echo "  - data_plugin"
echo "  - http_plugin"
echo "  - db_plugin"
