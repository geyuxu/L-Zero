#!/bin/bash
# Test additional README examples
# Tests: File I/O, HTTP Client

set -e
cd "$(dirname "$0")/.."

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
FAIL=0

test_case() {
    echo -e "  $1..."
}

pass() {
    ((PASS++))
    echo -e " ${GREEN}PASS${NC}"
}

fail() {
    ((FAIL++))
    echo -e " ${RED}FAIL${NC}: $1"
}

cleanup() {
    rm -f test/test_file_io.l0 test/test_http_client.l0
    rm -f test_input.txt test_output.txt
}

trap cleanup EXIT

echo "========================================"
echo "L-0 Additional Examples Test Suite"
echo "========================================"
echo ""

# Build if needed
if [ ! -f target/release/l0vm ]; then
    echo "Building..."
    cargo build --release --workspace
fi

cleanup

# =============================================================================
echo -e "${YELLOW}=== 1. File I/O Example ===${NC}"
# =============================================================================

test_case "Compile test_file_io.asm"
./target/release/l0asm test/test_file_io.asm > test/test_file_io.l0 2>/dev/null
if [ -s test/test_file_io.l0 ]; then
    pass
else
    fail "Compilation failed"
fi

test_case "File I/O operations work"
OUTPUT=$(./target/release/l0vm test/test_file_io.l0 2>&1)

if [[ "$OUTPUT" == *"HELLO FROM L-0 VM!"* ]] && [[ "$OUTPUT" == *"File I/O test complete"* ]]; then
    pass
else
    fail "Expected uppercase transformation: $OUTPUT"
fi

# =============================================================================
echo -e "${YELLOW}=== 2. HTTP Client Example ===${NC}"
# =============================================================================

test_case "Compile test_http_client.asm"
./target/release/l0asm test/test_http_client.asm > test/test_http_client.l0 2>/dev/null
if [ -s test/test_http_client.l0 ]; then
    pass
else
    fail "Compilation failed"
fi

test_case "HTTP Client requests work"
# This test requires network access (uses HTTP, not HTTPS)
./target/release/l0vm test/test_http_client.l0 > /tmp/http_client_output.txt 2>&1 &
VM_PID=$!
sleep 20
if ps -p $VM_PID > /dev/null 2>&1; then
    kill $VM_PID 2>/dev/null
    wait $VM_PID 2>/dev/null || true
    OUTPUT="timeout"
else
    wait $VM_PID 2>/dev/null || true
    OUTPUT=$(cat /tmp/http_client_output.txt)
fi

if [[ "$OUTPUT" == *"GET request succeeded"* ]] && [[ "$OUTPUT" == *"POST request succeeded"* ]]; then
    pass
elif [[ "$OUTPUT" == *"timeout"* ]]; then
    fail "Request timed out (network issue?)"
else
    fail "HTTP requests failed: $OUTPUT"
fi

cleanup

# =============================================================================
echo ""
echo "========================================"
echo "Results: $PASS passed, $FAIL failed"
echo "========================================"

if [ $FAIL -gt 0 ]; then
    exit 1
fi
