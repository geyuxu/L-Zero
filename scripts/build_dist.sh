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
mkdir -p "$DIST_DIR/stdlib"

# Step 3: Copy binaries (dynamically from workspace)
echo "[3/5] Copying binaries..."

# Core binaries from crates/ (read binary name from Cargo.toml)
for crate in crates/*/; do
    toml="${crate}Cargo.toml"
    name=$(grep -A1 '^\[\[bin\]\]' "$toml" 2>/dev/null | grep '^name' | sed 's/.*"\([^"]*\)".*/\1/')
    if [ -n "$name" ] && [ -f "target/release/$name" ]; then
        cp "target/release/$name" "$DIST_DIR/bin/"
        echo "  bin: $name"
    fi
done

# Plugins from plugins/
for plugin in plugins/*/; do
    name=$(basename "$plugin")
    if [ -f "target/release/$name" ]; then
        cp "target/release/$name" "$DIST_DIR/lib/l0/plugins/"
        echo "  plugin: $name"
    fi
done

# Step 4: Generate production tools.json (transform paths from source)
echo "[4/5] Generating configuration..."
# Transform: "target/release/xxx_plugin" -> "lib/l0/plugins/xxx_plugin"
sed 's|"target/release/\([^"]*\)"|"lib/l0/plugins/\1"|g' tools.json > "$DIST_DIR/config/tools.json"

# Step 5: Copy examples, stdlib, and docs
echo "[5/5] Copying examples, stdlib, and documentation..."

# Copy all .asm examples from examples/ directory
for asm in examples/*.asm; do
    if [ -f "$asm" ]; then
        cp "$asm" "$DIST_DIR/examples/"
        echo "  example: $(basename "$asm")"
    fi
done

# Copy stdlib (pattern library for AI agents)
if [ -d "stdlib" ]; then
    cp -r stdlib/* "$DIST_DIR/stdlib/"
    echo "  stdlib: $(ls stdlib | wc -l | tr -d ' ') files"
fi

# Install script
cat > "$DIST_DIR/install.sh" << 'INSTALL_EOF'
#!/bin/bash
set -e

# L-0 Installation Script
# Usage:
#   ./install.sh              # User install to ~/.l0
#   ./install.sh /custom/path # Install to custom path
#   ./install.sh --system     # System install to /usr/local/l0

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)

if [ "$1" = "--system" ] || [ "$1" = "-s" ]; then
    PREFIX="/usr/local/l0"
    SYSTEM_INSTALL=true
    SUDO="sudo"
    echo "=== L-0 System Installation ==="
else
    PREFIX="${1:-$HOME/.l0}"
    SYSTEM_INSTALL=false
    SUDO=""
    echo "=== L-0 User Installation ==="
fi

echo "Installing to: $PREFIX"

# Create directories and copy files
$SUDO mkdir -p "$PREFIX"
$SUDO cp -r "$SCRIPT_DIR/bin" "$PREFIX/"
$SUDO cp -r "$SCRIPT_DIR/lib" "$PREFIX/"
$SUDO cp -r "$SCRIPT_DIR/config" "$PREFIX/"
$SUDO cp -r "$SCRIPT_DIR/examples" "$PREFIX/"
$SUDO cp -r "$SCRIPT_DIR/stdlib" "$PREFIX/"

$SUDO chmod +x "$PREFIX/bin/"*
$SUDO chmod +x "$PREFIX/lib/l0/plugins/"*

if [ "$SYSTEM_INSTALL" = true ]; then
    # Create symlinks in /usr/local/bin
    echo "Creating symlinks in /usr/local/bin..."
    for bin in "$PREFIX/bin/"*; do
        name=$(basename "$bin")
        $SUDO ln -sf "$bin" "/usr/local/bin/$name"
        echo "  /usr/local/bin/$name -> $bin"
    done

    echo ""
    echo "Installation complete!"
    echo "Set L0_HOME in your shell profile:"
    echo "  export L0_HOME=$PREFIX"
    echo ""
    echo "Binaries are globally available. Try: l0vm --info"
else
    echo ""
    echo "Installation complete!"
    echo "Add to your shell profile (~/.bashrc or ~/.zshrc):"
    echo "  export L0_HOME=\"$PREFIX\""
    echo "  export PATH=\"\$L0_HOME/bin:\$PATH\""
    echo ""
    echo "Then run: l0vm --info"
fi
INSTALL_EOF
chmod +x "$DIST_DIR/install.sh"

# README
cat > "$DIST_DIR/README.md" << 'README_EOF'
# L-0 Language

L-0 is a low-level instruction set designed for AI agents.

## Quick Install

```bash
# User installation (recommended)
./install.sh              # Install to ~/.l0

# System installation (requires sudo)
./install.sh --system     # Install to /usr/local/l0

# Custom path
./install.sh /opt/l0
```

### User Installation
Add to your shell profile (~/.bashrc or ~/.zshrc):
```bash
export L0_HOME="$HOME/.l0"
export PATH="$L0_HOME/bin:$PATH"
```

### System Installation
Binaries are symlinked to /usr/local/bin. Just set:
```bash
export L0_HOME="/usr/local/l0"
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
