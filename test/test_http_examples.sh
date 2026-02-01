#!/bin/bash
# Test HTTP examples from README
# Tests: Static server, Dynamic server, REST API

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
    # Kill any background processes
    jobs -p | xargs kill 2>/dev/null || true
    # Clean up files
    rm -f .l0_http_config.json .l0_http_pending_*.json /tmp/l0_http_daemon.pid
    rm -f test/test_http_static.l0 test/test_http_dynamic.l0 test/test_rest_api.l0
    rm -f test_rest_api.db
    # Kill any processes on test ports
    lsof -ti:18600 | xargs kill -9 2>/dev/null || true
    lsof -ti:18601 | xargs kill -9 2>/dev/null || true
    lsof -ti:18602 | xargs kill -9 2>/dev/null || true
}

trap cleanup EXIT

echo "========================================"
echo "L-0 HTTP Examples Test Suite"
echo "========================================"
echo ""

# Build if needed
if [ ! -f target/release/l0vm ]; then
    echo "Building..."
    cargo build --release --workspace
fi

cleanup
sleep 0.5

# =============================================================================
echo -e "${YELLOW}=== 1. Static HTTP Server (README Example) ===${NC}"
# =============================================================================

test_case "Compile test_http_static.asm"
./target/release/l0asm test/test_http_static.asm > test/test_http_static.l0 2>/dev/null
if [ -s test/test_http_static.l0 ]; then
    pass
else
    fail "Compilation failed"
fi

test_case "Static server serves HTML route"
# Start server in background
./target/release/l0vm test/test_http_static.l0 >/dev/null 2>&1 &
VM_PID=$!
sleep 2

# Make request
RESPONSE=$(curl -s --connect-timeout 3 http://127.0.0.1:18600/ 2>/dev/null || echo "")
kill $VM_PID 2>/dev/null || true
wait $VM_PID 2>/dev/null || true

if [[ "$RESPONSE" == *"Hello World"* ]]; then
    pass
else
    fail "Expected 'Hello World' in response: $RESPONSE"
fi

cleanup
sleep 0.5

# =============================================================================
echo -e "${YELLOW}=== 2. Dynamic HTTP Server (README Example) ===${NC}"
# =============================================================================

test_case "Compile test_http_dynamic.asm"
./target/release/l0asm test/test_http_dynamic.asm > test/test_http_dynamic.l0 2>/dev/null
if [ -s test/test_http_dynamic.l0 ]; then
    pass
else
    fail "Compilation failed"
fi

test_case "Dynamic server returns JSON with timestamp"
# Start server in background
./target/release/l0vm test/test_http_dynamic.l0 >/dev/null 2>&1 &
VM_PID=$!
sleep 2

# Make request
RESPONSE=$(curl -s --connect-timeout 3 http://127.0.0.1:18601/api/test 2>/dev/null || echo "")
kill $VM_PID 2>/dev/null || true
wait $VM_PID 2>/dev/null || true

if [[ "$RESPONSE" == *"timestamp"* ]] && [[ "$RESPONSE" == *"ok"* ]]; then
    pass
else
    fail "Expected JSON with timestamp: $RESPONSE"
fi

cleanup
sleep 0.5

# =============================================================================
echo -e "${YELLOW}=== 3. RESTful API Server (README Example) ===${NC}"
# =============================================================================

test_case "Compile test_rest_api.asm"
./target/release/l0asm test/test_rest_api.asm > test/test_rest_api.l0 2>/dev/null
if [ -s test/test_rest_api.l0 ]; then
    pass
else
    fail "Compilation failed"
fi

test_case "REST API returns posts from database"
# Start server in background
./target/release/l0vm test/test_rest_api.l0 >/dev/null 2>&1 &
VM_PID=$!
sleep 2

# Make request
RESPONSE=$(curl -s --connect-timeout 3 http://127.0.0.1:18602/api/posts 2>/dev/null || echo "")
kill $VM_PID 2>/dev/null || true
wait $VM_PID 2>/dev/null || true

if [[ "$RESPONSE" == *"Hello World"* ]] || [[ "$RESPONSE" == *"id"* ]]; then
    pass
else
    fail "Expected posts data: $RESPONSE"
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
