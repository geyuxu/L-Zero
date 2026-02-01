#!/bin/bash
# L-0 Test Runner Script
# Runs all test suites and reports results

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
L0VM="$PROJECT_DIR/bin/l0vm"
L0ASM="$PROJECT_DIR/bin/l0asm"

echo "========================================"
echo "L-0 Test Runner"
echo "========================================"
echo ""

# Check if binaries exist
if [ ! -f "$L0VM" ]; then
    echo "Error: l0vm not found at $L0VM"
    echo "Run: cargo build --release"
    exit 1
fi

if [ ! -f "$L0ASM" ]; then
    echo "Error: l0asm not found at $L0ASM"
    echo "Run: cargo build --release"
    exit 1
fi

# Create temp directory for compiled tests
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

run_test() {
    local name=$1
    local file=$2
    local ext="${file##*.}"

    echo "----------------------------------------"
    echo "Running: $name"
    echo "----------------------------------------"

    if [ "$ext" == "asm" ]; then
        # Compile assembly to bytecode
        "$L0ASM" "$file" > "$TEMP_DIR/$(basename "$file" .asm).l0"
        "$L0VM" "$TEMP_DIR/$(basename "$file" .asm).l0"
    elif [ "$ext" == "json" ]; then
        "$L0VM" "$file"
    elif [ "$ext" == "l0" ]; then
        "$L0VM" "$file"
    else
        echo "Unknown file type: $ext"
        return 1
    fi

    echo ""
}

# Run individual test suites
echo "Running test suites..."
echo ""

# Run JSON example
run_test "Hello World (JSON)" "$SCRIPT_DIR/hello.json"

# Run ASM tests
for test_file in "$SCRIPT_DIR"/test_*.asm; do
    if [ -f "$test_file" ]; then
        name=$(basename "$test_file" .asm)
        run_test "$name" "$test_file"
    fi
done

# Run JSON tests
for test_file in "$SCRIPT_DIR"/test_*.json; do
    if [ -f "$test_file" ]; then
        name=$(basename "$test_file" .json)
        run_test "$name (JSON)" "$test_file"
    fi
done

# Run bubble sort
run_test "Bubble Sort" "$SCRIPT_DIR/bubble_sort.asm"

echo "========================================"
echo "All tests completed!"
echo "========================================"
