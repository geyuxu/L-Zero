#!/bin/bash
set -e

# L-0 Build Script
# Compiles toolchain, plugins, and packages distribution files

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="$PROJECT_DIR/out"
BIN_DIR="$PROJECT_DIR/bin"

echo "=== L-0 Build ==="

# Clean previous build
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR/tools" "$OUT_DIR/examples" "$BIN_DIR"

# Compile main toolchain (release mode)
echo "[1/4] Compiling toolchain..."
cargo build --release

# Copy toolchain binaries to bin/
cp "$PROJECT_DIR/target/release/l0vm" "$BIN_DIR/"
cp "$PROJECT_DIR/target/release/l0asm" "$BIN_DIR/"
cp "$PROJECT_DIR/target/release/l0cc" "$BIN_DIR/"

# Compile plugins
echo "[2/4] Compiling plugins..."
for rs in "$PROJECT_DIR/tools/"*.rs; do
    if [ -f "$rs" ]; then
        name=$(basename "$rs" .rs)
        echo "  - $name"
        rustc -O "$rs" -o "$PROJECT_DIR/tools/$name" 2>/dev/null || {
            echo "    Warning: Failed to compile $name (may need dependencies)"
        }
    fi
done

# Copy binaries to out/
echo "[3/4] Packaging distribution..."
cp "$BIN_DIR/l0vm" "$OUT_DIR/"
cp "$BIN_DIR/l0asm" "$OUT_DIR/"
cp "$BIN_DIR/l0cc" "$OUT_DIR/"

# Copy plugins (compiled binaries only)
for f in "$PROJECT_DIR/tools/"*; do
    if [ -x "$f" ] && [[ "$f" != *.rs ]]; then
        cp "$f" "$OUT_DIR/tools/"
    fi
done

# Copy config and docs
cp "$PROJECT_DIR/tools.json" "$OUT_DIR/"
cp "$PROJECT_DIR/BOOT.md" "$OUT_DIR/"
cp "$PROJECT_DIR/README.md" "$OUT_DIR/"

# Copy examples (exclude temporary test files)
for f in "$PROJECT_DIR/examples/"*.json "$PROJECT_DIR/examples/"*.asm "$PROJECT_DIR/examples/"*.l0; do
    [ -f "$f" ] && cp "$f" "$OUT_DIR/examples/"
done

echo "[4/4] Done!"
echo ""
echo "Binaries: $BIN_DIR/"
ls "$BIN_DIR/"
echo ""
echo "Distribution: $OUT_DIR/"
ls "$OUT_DIR/"
