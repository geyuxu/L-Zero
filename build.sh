#!/bin/bash
set -e

# L-0 Build Script
# Compiles toolchain and packages distribution files

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$PROJECT_DIR/out"

echo "=== L-0 Build ==="

# Clean previous build
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR/tools" "$OUT_DIR/examples"

# Compile (release mode)
echo "[1/3] Compiling..."
cargo build --release

# Copy binaries
echo "[2/3] Copying binaries..."
cp "$PROJECT_DIR/target/release/l0vm" "$OUT_DIR/"
cp "$PROJECT_DIR/target/release/l0asm" "$OUT_DIR/"

# Copy tools (plugins only, no source)
for f in "$PROJECT_DIR/tools/"*; do
    if [ -x "$f" ] && [ ! -f "$f.rs" ] || [ -x "$f" ] && [[ "$f" != *.rs ]]; then
        # Copy executable files (exclude .rs source files)
        [[ "$f" != *.rs ]] && [ -f "$f" ] && cp "$f" "$OUT_DIR/tools/"
    fi
done

# Copy config and docs
cp "$PROJECT_DIR/tools.json" "$OUT_DIR/"
cp "$PROJECT_DIR/BOOT.md" "$OUT_DIR/"

# Copy examples
cp -r "$PROJECT_DIR/examples/"* "$OUT_DIR/examples/" 2>/dev/null || true

# Copy example asm/l0 files from project root
for f in "$PROJECT_DIR/"*.asm "$PROJECT_DIR/"*.l0; do
    [ -f "$f" ] && cp "$f" "$OUT_DIR/examples/"
done

echo "[3/3] Done!"
echo ""
echo "Output: $OUT_DIR/"
ls -la "$OUT_DIR/"
