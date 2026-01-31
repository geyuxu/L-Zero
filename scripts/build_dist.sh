#!/bin/bash
set -e

# L-0 Distribution Build Script
# Builds a production-ready release package

VERSION="${1:-1.0.0}"
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Normalize architecture name
case "$ARCH" in
    x86_64) ARCH="x64" ;;
    aarch64|arm64) ARCH="arm64" ;;
esac

DIST_NAME="l0-lang-v${VERSION}-${PLATFORM}-${ARCH}"
DIST_DIR="dist/${DIST_NAME}"
ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)

echo "=== L-0 Distribution Build ==="
echo "Version: ${VERSION}"
echo "Platform: ${PLATFORM}-${ARCH}"
echo ""

# Step 1: Build all workspace crates
echo "[1/5] Building workspace..."
cd "$ROOT_DIR"
cargo build --release --workspace

# Step 2: Create distribution directory structure
echo "[2/5] Creating distribution structure..."
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR/bin"
mkdir -p "$DIST_DIR/lib/l0/plugins"
mkdir -p "$DIST_DIR/config"
mkdir -p "$DIST_DIR/examples"

# Step 3: Copy binaries
echo "[3/5] Copying binaries..."
cp target/release/l0vm "$DIST_DIR/bin/"
cp target/release/l0asm "$DIST_DIR/bin/"
cp target/release/l0cc "$DIST_DIR/bin/"

cp target/release/file_plugin "$DIST_DIR/lib/l0/plugins/"
cp target/release/data_plugin "$DIST_DIR/lib/l0/plugins/"
cp target/release/http_plugin "$DIST_DIR/lib/l0/plugins/"
cp target/release/db_plugin "$DIST_DIR/lib/l0/plugins/"

# Step 4: Generate production tools.json
echo "[4/5] Generating configuration..."
cat > "$DIST_DIR/config/tools.json" << 'TOOLS_EOF'
{
    "$schema": "L-0 Tool Registry (Production)",
    "$version": "1.0.0",

    "builtins": {
        "0x5000": { "name": "PRINT", "type": "builtin", "desc": "Print string with newline" },
        "0x5001": { "name": "PRINTN", "type": "builtin", "desc": "Print string without newline" },
        "0x5002": { "name": "INPUT", "type": "builtin", "desc": "Read line from stdin" },
        "0x1004": { "name": "POW", "type": "builtin", "desc": "Power function: base,exp" },
        "0x5003": { "name": "RAND", "type": "builtin", "desc": "Random number 0..max" },
        "0x5005": { "name": "ABS", "type": "builtin", "desc": "Absolute value" },
        "0x5006": { "name": "MIN", "type": "builtin", "desc": "Minimum of two values" },
        "0x5007": { "name": "MAX", "type": "builtin", "desc": "Maximum of two values" },
        "0x5008": { "name": "TIME", "type": "builtin", "desc": "Unix timestamp (seconds)" },
        "0x500A": { "name": "SUBSTR", "type": "builtin", "desc": "Substring: str,start,len" },
        "0x500B": { "name": "SPLIT", "type": "builtin", "desc": "Split and get element: str,delim,idx" },
        "0x500C": { "name": "UPPER", "type": "builtin", "desc": "Uppercase string" },
        "0x500D": { "name": "LOWER", "type": "builtin", "desc": "Lowercase string" },
        "0x500E": { "name": "TRIM", "type": "builtin", "desc": "Trim whitespace" }
    },

    "plugins": {
        "$note": "External plugins - paths relative to L0_HOME",

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
TOOLS_EOF

# Step 5: Copy examples and docs
echo "[5/5] Copying examples and documentation..."

# Hello World example
cat > "$DIST_DIR/examples/hello_world.asm" << 'ASM_EOF'
# Hello World - L-0 Assembly
SETS 1, Hello, L-0!
TEXEC 0x5000, 1, 255
HALT
ASM_EOF

# Loop example
cat > "$DIST_DIR/examples/count.asm" << 'ASM_EOF'
# Count 1 to 5
SET 1, 1       # counter
SET 2, 6       # limit

Loop:
CMP 1, 2
BEQ Done
ITOA 10, 1
TEXEC 0x5000, 10, 255
SET 3, 1
ADD 1, 1, 3
JMP Loop

Done:
HALT
ASM_EOF

# Install script
cat > "$DIST_DIR/install.sh" << 'INSTALL_EOF'
#!/bin/bash
set -e

# L-0 Installation Script

PREFIX="${1:-$HOME/.l0}"
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

echo "=== L-0 Installation ==="
echo "Installing to: $PREFIX"
echo ""

# Create installation directory
mkdir -p "$PREFIX"

# Copy all files
cp -r "$SCRIPT_DIR/bin" "$PREFIX/"
cp -r "$SCRIPT_DIR/lib" "$PREFIX/"
cp -r "$SCRIPT_DIR/config" "$PREFIX/"
cp -r "$SCRIPT_DIR/examples" "$PREFIX/"

# Make binaries executable
chmod +x "$PREFIX/bin/"*
chmod +x "$PREFIX/lib/l0/plugins/"*

# Generate shell profile additions
PROFILE_ADDITIONS="
# L-0 Language
export L0_HOME=\"$PREFIX\"
export PATH=\"\$L0_HOME/bin:\$PATH\"
"

echo "Installation complete!"
echo ""
echo "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
echo "----------------------------------------"
echo "$PROFILE_ADDITIONS"
echo "----------------------------------------"
echo ""
echo "Then reload your shell or run:"
echo "  source ~/.bashrc  # or ~/.zshrc"
echo ""
echo "Verify installation:"
echo "  l0vm --info"
INSTALL_EOF
chmod +x "$DIST_DIR/install.sh"

# README
cat > "$DIST_DIR/README.md" << 'README_EOF'
# L-0 Language

L-0 is a low-level instruction set designed for AI agents.

## Quick Install

```bash
./install.sh              # Install to ~/.l0 (default)
./install.sh /opt/l0      # Install to custom path
```

After installation, add to your shell profile:

```bash
export L0_HOME="$HOME/.l0"
export PATH="$L0_HOME/bin:$PATH"
```

## Usage

```bash
# Compile assembly to bytecode
l0asm examples/hello_world.asm > hello.l0

# Execute bytecode
l0vm hello.l0

# AOT compile to C (optional)
l0cc hello.l0 -o hello.c
gcc -O2 hello.c -o hello
./hello
```

## Included Tools

| Binary | Description |
|--------|-------------|
| `l0vm` | Virtual Machine - executes .l0 bytecode |
| `l0asm` | Assembler - compiles .asm to .l0 |
| `l0cc` | AOT Compiler - compiles .l0 to C |

## Plugins

Plugins are located in `lib/l0/plugins/`:
- `file_plugin` - File I/O operations
- `data_plugin` - JSON operations
- `http_plugin` - HTTP server
- `db_plugin` - SQLite database

## Documentation

For full documentation, visit: https://github.com/geyuxu/ai-programming-lang

## License

MIT
README_EOF

# Create tarball
echo ""
echo "Creating distribution package..."
cd dist
tar -czf "${DIST_NAME}.tar.gz" "$DIST_NAME"

echo ""
echo "=== Build Complete ==="
echo "Package: dist/${DIST_NAME}.tar.gz"
echo ""
echo "Contents:"
ls -la "${DIST_NAME}/"
echo ""
echo "To test locally:"
echo "  cd dist/${DIST_NAME} && ./install.sh /tmp/l0-test"
