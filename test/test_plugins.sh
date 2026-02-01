#!/bin/bash
# L-0 Plugin Edge Case Tests
# Run: ./test/test_plugins.sh

set -e
cd "$(dirname "$0")/.."

PASS=0
FAIL=0
TOTAL=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Test helper
test_case() {
    TOTAL=$((TOTAL + 1))
    echo -n "  [$TOTAL] $1... "
}

pass() {
    PASS=$((PASS + 1))
    echo -e "${GREEN}PASS${NC}"
}

fail() {
    FAIL=$((FAIL + 1))
    echo -e "${RED}FAIL${NC}: $1"
}

cleanup() {
    rm -f .l0_http_config.json .l0_db_config.json test_*.db /tmp/test_*.asm /tmp/test_*.l0
}

trap cleanup EXIT

echo "========================================"
echo "L-0 Plugin Edge Case Tests"
echo "========================================"
echo ""

# Build first
echo "Building..."
cargo build --release --workspace -q 2>/dev/null || { echo "Build failed"; exit 1; }
echo ""

# =============================================================================
echo -e "${YELLOW}=== HTTP Plugin Tests ===${NC}"
# =============================================================================

test_case "JSON array in route content"
cleanup
echo '{"method":"init","args":["18100"]}' | ./target/release/http_plugin > /dev/null
RESULT=$(echo '{"method":"route","args":["GET /test|[{\"id\":1},{\"id\":2}]"]}' | ./target/release/http_plugin)
if [[ "$RESULT" == *"registered"* ]]; then
    pass
else
    fail "Route with JSON array failed: $RESULT"
fi

test_case "Nested brackets in route"
RESULT=$(echo '{"method":"route","args":["GET /nested|[[1,2],[3,4]]"]}' | ./target/release/http_plugin)
if [[ "$RESULT" == *"registered"* ]]; then
    pass
else
    fail "Nested brackets failed: $RESULT"
fi

test_case "Route update preserves after init"
echo '{"method":"route","args":["GET /data|{\"v\":1}"]}' | ./target/release/http_plugin > /dev/null
echo '{"method":"init","args":["18101"]}' | ./target/release/http_plugin > /dev/null
CONFIG=$(cat .l0_http_config.json)
if [[ "$CONFIG" == *"/data"* ]]; then
    pass
else
    fail "Route lost after init"
fi

test_case "Route content update"
echo '{"method":"route","args":["GET /update|old"]}' | ./target/release/http_plugin > /dev/null
echo '{"method":"route","args":["GET /update|new"]}' | ./target/release/http_plugin > /dev/null
CONFIG=$(cat .l0_http_config.json)
if [[ "$CONFIG" == *'"new"'* ]] && [[ "$CONFIG" != *'"old"'* ]]; then
    pass
else
    fail "Route update failed"
fi

test_case "Large JSON payload"
LARGE_JSON='[{"id":1,"title":"Post 1","content":"Lorem ipsum dolor sit amet"},{"id":2,"title":"Post 2","content":"Consectetur adipiscing elit"},{"id":3,"title":"Post 3","content":"Sed do eiusmod tempor"}]'
RESULT=$(echo "{\"method\":\"route\",\"args\":[\"GET /large|$(echo "$LARGE_JSON" | sed 's/"/\\"/g')\"]}" | ./target/release/http_plugin)
if [[ "$RESULT" == *"registered"* ]]; then
    pass
else
    fail "Large JSON failed"
fi

test_case "Special characters in route content"
RESULT=$(echo '{"method":"route","args":["GET /special|<html>&amp;\"test\""]}' | ./target/release/http_plugin)
if [[ "$RESULT" == *"registered"* ]]; then
    pass
else
    fail "Special chars failed"
fi

cleanup
echo ""

# =============================================================================
echo -e "${YELLOW}=== DB Plugin Tests ===${NC}"
# =============================================================================

test_case "Basic CRUD cycle"
echo '{"method":"init","args":["test_crud.db"]}' | ./target/release/db_plugin > /dev/null
echo '{"method":"create_table","args":["items|id INTEGER PRIMARY KEY,name TEXT"]}' | ./target/release/db_plugin > /dev/null
echo '{"method":"insert","args":["items|name|Test"]}' | ./target/release/db_plugin > /dev/null
RESULT=$(echo '{"method":"select","args":["items"]}' | ./target/release/db_plugin)
if [[ "$RESULT" == *"Test"* ]]; then
    pass
else
    fail "CRUD failed: $RESULT"
fi

test_case "Delete operation"
echo '{"method":"delete","args":["items|id=1"]}' | ./target/release/db_plugin > /dev/null
RESULT=$(echo '{"method":"select","args":["items"]}' | ./target/release/db_plugin)
if [[ "$RESULT" == *"[]"* ]]; then
    pass
else
    fail "Delete failed: $RESULT"
fi

test_case "Update operation"
# Get the ID of the inserted row first
INSERT_RESULT=$(echo '{"method":"insert","args":["items|name|Original"]}' | ./target/release/db_plugin)
# Extract ID (last inserted row)
LAST_ID=$(echo "$INSERT_RESULT" | grep -o '"value":[0-9]*' | grep -o '[0-9]*')
echo "{\"method\":\"update\",\"args\":[\"items|id=$LAST_ID|name=Updated\"]}" | ./target/release/db_plugin > /dev/null
RESULT=$(echo '{"method":"select","args":["items"]}' | ./target/release/db_plugin)
if [[ "$RESULT" == *"Updated"* ]]; then
    pass
else
    fail "Update failed: $RESULT"
fi

test_case "JSON insert format"
RESULT=$(echo '{"method":"insert","args":["items|{\"name\":\"JsonInsert\"}"]}' | ./target/release/db_plugin)
if [[ "$RESULT" == *"ok\":true"* ]]; then
    pass
else
    fail "JSON insert failed: $RESULT"
fi

test_case "Select with condition"
RESULT=$(echo '{"method":"select","args":["items|name=JsonInsert"]}' | ./target/release/db_plugin)
if [[ "$RESULT" == *"JsonInsert"* ]]; then
    pass
else
    fail "Conditional select failed: $RESULT"
fi

cleanup
echo ""

# =============================================================================
echo -e "${YELLOW}=== Data Plugin Tests ===${NC}"
# =============================================================================

test_case "JSON with brackets in value"
RESULT=$(echo '{"method":"json_set","args":["key","[1,2,3]"]}' | ./target/release/data_plugin)
if [[ "$RESULT" == *"ok\":true"* ]]; then
    pass
else
    fail "JSON set with brackets failed: $RESULT"
fi

test_case "Nested JSON objects"
RESULT=$(echo '{"method":"json_set","args":["nested","{\"a\":{\"b\":1}}"]}' | ./target/release/data_plugin)
if [[ "$RESULT" == *"ok\":true"* ]]; then
    pass
else
    fail "Nested JSON failed: $RESULT"
fi

cleanup
echo ""

# =============================================================================
echo -e "${YELLOW}=== VM Integration Tests ===${NC}"
# =============================================================================

test_case "TEXEC with all DB operations"
cat > /tmp/test_db.asm << 'EOF'
SETS 255, "test_vm.db"
TEXEC 0x9000, 255, 0
SETS 255, "t|id INTEGER PRIMARY KEY,v TEXT"
TEXEC 0x9001, 255, 0
SETS 255, "t|v|hello"
TEXEC 0x9002, 255, 0
SETS 255, "t"
TEXEC 0x9003, 255, 0
MOV 10, 0
SETS 255, "t|id=1|v=updated"
TEXEC 0x9004, 255, 0
SETS 255, "t|id=1"
TEXEC 0x9005, 255, 0
HALT
EOF
./target/release/l0asm /tmp/test_db.asm > /tmp/test_db.l0 2>/dev/null
RESULT=$(./target/release/l0vm /tmp/test_db.l0 2>&1)
if [[ "$RESULT" != *"error"* ]] && [[ "$RESULT" != *"PANIC"* ]]; then
    pass
else
    fail "VM DB operations failed"
fi

test_case "HTTP route registration via VM"
cat > /tmp/test_http.asm << 'EOF'
SETS 255, "18102"
TEXEC 0x8000, 255, 0
SETS 255, "GET /vm-test|{\"from\":\"vm\"}"
TEXEC 0x8001, 255, 0
HALT
EOF
./target/release/l0asm /tmp/test_http.asm > /tmp/test_http.l0 2>/dev/null
./target/release/l0vm /tmp/test_http.l0 2>/dev/null
CONFIG=$(cat .l0_http_config.json 2>/dev/null || echo "")
if [[ "$CONFIG" == *"vm-test"* ]]; then
    pass
else
    fail "VM HTTP route failed"
fi

cleanup
echo ""

# =============================================================================
echo -e "${YELLOW}=== Dynamic HTTP Mode Tests ===${NC}"
# =============================================================================

test_case "HTTP_LISTEN starts daemon"
cleanup
lsof -ti:18400 | xargs kill -9 2>/dev/null || true
rm -f /tmp/l0_http_daemon.pid .l0_http_pending_*.json
echo '{"method":"init","args":["18400"]}' | ./target/release/http_plugin > /dev/null
(echo '{"method":"listen","args":[]}' | ./target/release/http_plugin > /dev/null) &
sleep 1
if [ -f /tmp/l0_http_daemon.pid ]; then
    pass
else
    fail "Daemon PID file not created"
fi

test_case "Dynamic request/response cycle"
# Make a request in background
(curl -s -X POST -d '{"test":true}' http://127.0.0.1:18400/api/test > /tmp/dynamic_result.txt) &
sleep 0.5
# Send response
echo '{"method":"send","args":["{\"status\":\"ok\",\"data\":123}"]}' | ./target/release/http_plugin > /dev/null
sleep 1
RESULT=$(cat /tmp/dynamic_result.txt 2>/dev/null || echo "")
if [[ "$RESULT" == *"status"* ]] && [[ "$RESULT" == *"ok"* ]]; then
    pass
else
    fail "Dynamic response not received: $RESULT"
fi

test_case "Stop dynamic daemon"
RESULT=$(echo '{"method":"stop_dynamic","args":[]}' | ./target/release/http_plugin)
if [[ "$RESULT" == *"stopped"* ]]; then
    pass
else
    fail "Stop daemon failed: $RESULT"
fi

# Clean up daemon files
rm -f /tmp/l0_http_daemon.pid /tmp/dynamic_result.txt .l0_http_pending_*.json
lsof -ti:18400 | xargs kill -9 2>/dev/null || true

cleanup
echo ""

# =============================================================================
echo "========================================"
echo -e "Results: ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}, $TOTAL total"
echo "========================================"

if [ $FAIL -gt 0 ]; then
    exit 1
fi
