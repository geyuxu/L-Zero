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

# 6. Generate production tools.json (paths relative to L0_HOME)
echo "[6/8] Generating production config..."
cat > "${DIST_DIR}/config/tools.json" << 'EOF'
{
    "$schema": "L-0 Tool Registry (Production)",
    "$version": "1.0.0",

    "builtins": {
        "0x5000": { "name": "PRINT", "type": "builtin" },
        "0x5001": { "name": "PRINTN", "type": "builtin" },
        "0x5002": { "name": "INPUT", "type": "builtin" },
        "0x1004": { "name": "POW", "type": "builtin" },
        "0x5003": { "name": "RAND", "type": "builtin" },
        "0x5005": { "name": "ABS", "type": "builtin" },
        "0x5006": { "name": "MIN", "type": "builtin" },
        "0x5007": { "name": "MAX", "type": "builtin" },
        "0x5008": { "name": "TIME", "type": "builtin" },
        "0x500A": { "name": "SUBSTR", "type": "builtin" },
        "0x500B": { "name": "SPLIT", "type": "builtin" },
        "0x500C": { "name": "UPPER", "type": "builtin" },
        "0x500D": { "name": "LOWER", "type": "builtin" },
        "0x500E": { "name": "TRIM", "type": "builtin" }
    },

    "plugins": {
        "0x4000": { "name": "FILE_READ", "type": "plugin", "binary": "lib/l0/plugins/file_plugin", "method": "read" },
        "0x4001": { "name": "FILE_WRITE", "type": "plugin", "binary": "lib/l0/plugins/file_plugin", "method": "write" },
        "0x4003": { "name": "FILE_DELETE", "type": "plugin", "binary": "lib/l0/plugins/file_plugin", "method": "delete" },
        "0x4004": { "name": "FILE_LIST", "type": "plugin", "binary": "lib/l0/plugins/file_plugin", "method": "list" },
        "0x4005": { "name": "FILE_EXISTS", "type": "plugin", "binary": "lib/l0/plugins/file_plugin", "method": "exists" },

        "0x6000": { "name": "JSON_LOAD", "type": "plugin", "binary": "lib/l0/plugins/data_plugin", "method": "json_load" },
        "0x6001": { "name": "JSON_SAVE", "type": "plugin", "binary": "lib/l0/plugins/data_plugin", "method": "json_save" },
        "0x6002": { "name": "JSON_GET", "type": "plugin", "binary": "lib/l0/plugins/data_plugin", "method": "json_get" },
        "0x6003": { "name": "JSON_SET", "type": "plugin", "binary": "lib/l0/plugins/data_plugin", "method": "json_set" },
        "0x6004": { "name": "JSON_PARSE", "type": "plugin", "binary": "lib/l0/plugins/data_plugin", "method": "json_parse" },

        "0x8000": { "name": "HTTP_INIT", "type": "plugin", "binary": "lib/l0/plugins/http_plugin", "method": "init" },
        "0x8001": { "name": "HTTP_ROUTE", "type": "plugin", "binary": "lib/l0/plugins/http_plugin", "method": "route" },
        "0x8002": { "name": "HTTP_SERVE", "type": "plugin", "binary": "lib/l0/plugins/http_plugin", "method": "serve" },
        "0x8003": { "name": "HTTP_SERVE_ONCE", "type": "plugin", "binary": "lib/l0/plugins/http_plugin", "method": "serve_once" },
        "0x8004": { "name": "HTTP_REQUEST", "type": "plugin", "binary": "lib/l0/plugins/http_plugin", "method": "request" },
        "0x8005": { "name": "HTTP_LIST_ROUTES", "type": "plugin", "binary": "lib/l0/plugins/http_plugin", "method": "list_routes" },

        "0x9000": { "name": "DB_INIT", "type": "plugin", "binary": "lib/l0/plugins/db_plugin", "method": "init" },
        "0x9001": { "name": "DB_CREATE_TABLE", "type": "plugin", "binary": "lib/l0/plugins/db_plugin", "method": "create_table" },
        "0x9002": { "name": "DB_INSERT", "type": "plugin", "binary": "lib/l0/plugins/db_plugin", "method": "insert" },
        "0x9003": { "name": "DB_SELECT", "type": "plugin", "binary": "lib/l0/plugins/db_plugin", "method": "select" },
        "0x9004": { "name": "DB_UPDATE", "type": "plugin", "binary": "lib/l0/plugins/db_plugin", "method": "update" },
        "0x9005": { "name": "DB_DELETE", "type": "plugin", "binary": "lib/l0/plugins/db_plugin", "method": "delete" },
        "0x9006": { "name": "DB_DROP_TABLE", "type": "plugin", "binary": "lib/l0/plugins/db_plugin", "method": "drop_table" },
        "0x9007": { "name": "DB_LIST_TABLES", "type": "plugin", "binary": "lib/l0/plugins/db_plugin", "method": "list_tables" },
        "0x9008": { "name": "DB_CONNECT", "type": "plugin", "binary": "lib/l0/plugins/db_plugin", "method": "connect" },
        "0x9009": { "name": "DB_EXEC", "type": "plugin", "binary": "lib/l0/plugins/db_plugin", "method": "exec" },
        "0x900A": { "name": "DB_QUERY", "type": "plugin", "binary": "lib/l0/plugins/db_plugin", "method": "query" }
    }
}
EOF

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
