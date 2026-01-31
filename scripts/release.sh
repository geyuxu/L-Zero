#!/bin/bash
set -e

# L-0 Release Build Script
# Creates a production-ready distribution package

VERSION="${1:-1.0.0}"
DIST_DIR="dist/l0-lang"

echo "=== L-0 Release Build v${VERSION} ==="

# 1. Clean old builds
echo "[1/8] Cleaning old builds..."
cargo clean 2>/dev/null || true
rm -rf dist

# 2. Build all binaries (release mode)
echo "[2/8] Building workspace (release mode)..."
cargo build --release --workspace

# 3. Create distribution directory structure
echo "[3/8] Creating distribution structure..."
mkdir -p "${DIST_DIR}/bin"
mkdir -p "${DIST_DIR}/lib/l0/plugins"
mkdir -p "${DIST_DIR}/config"
mkdir -p "${DIST_DIR}/examples"

# 4. Copy core binaries
echo "[4/8] Copying core binaries..."
cp target/release/l0vm "${DIST_DIR}/bin/"
cp target/release/l0asm "${DIST_DIR}/bin/"
cp target/release/l0cc "${DIST_DIR}/bin/"

# 5. Copy plugin binaries
echo "[5/8] Copying plugins..."
cp target/release/file_plugin "${DIST_DIR}/lib/l0/plugins/"
cp target/release/data_plugin "${DIST_DIR}/lib/l0/plugins/"
cp target/release/http_plugin "${DIST_DIR}/lib/l0/plugins/"
cp target/release/db_plugin "${DIST_DIR}/lib/l0/plugins/"

# 6. Generate production tools.json (transform paths from source)
echo "[6/8] Generating production config..."
# Transform: "target/release/xxx_plugin" -> "lib/l0/plugins/xxx_plugin"
sed 's|"target/release/\([^"]*\)"|"lib/l0/plugins/\1"|g' tools.json > "${DIST_DIR}/config/tools.json"

# 7. Copy documentation and examples
echo "[7/8] Copying docs and examples..."
cp README.md "${DIST_DIR}/"
cp -r examples/*.asm "${DIST_DIR}/examples/" 2>/dev/null || true

# Create install script
cat > "${DIST_DIR}/install.sh" << 'INSTALL_EOF'
#!/bin/bash
set -e

PREFIX="${1:-$HOME/.l0}"
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

echo "Installing L-0 to: $PREFIX"

mkdir -p "$PREFIX"
cp -r "$SCRIPT_DIR/bin" "$PREFIX/"
cp -r "$SCRIPT_DIR/lib" "$PREFIX/"
cp -r "$SCRIPT_DIR/config" "$PREFIX/"
cp -r "$SCRIPT_DIR/examples" "$PREFIX/"

chmod +x "$PREFIX/bin/"*
chmod +x "$PREFIX/lib/l0/plugins/"*

echo ""
echo "Add to your shell profile:"
echo "  export L0_HOME=\"$PREFIX\""
echo "  export PATH=\"\$L0_HOME/bin:\$PATH\""
echo ""
echo "Then run: l0vm --info"
INSTALL_EOF
chmod +x "${DIST_DIR}/install.sh"

# 8. Create tarball
echo "[8/8] Creating release package..."
cd dist
tar -czf "l0-lang-v${VERSION}.tar.gz" l0-lang

echo ""
echo "=== Release Complete ==="
echo "Package: dist/l0-lang-v${VERSION}.tar.gz"
echo ""
echo "Contents:"
find l0-lang -type f | head -20
echo ""
echo "Install: tar xzf l0-lang-v${VERSION}.tar.gz && cd l0-lang && ./install.sh"
