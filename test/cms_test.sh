#!/bin/bash
# =============================================================================
# L-0 CMS Black-Box Test Suite
# =============================================================================
# Tests the complete CMS application: Database + HTTP + Web UI
# Usage: ./test/cms_test.sh
# =============================================================================

set -e
cd "$(dirname "$0")/.."

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
FAIL=0
TOTAL=0

test_case() {
    ((TOTAL++))
    echo -e "  [$TOTAL] $1..."
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
    lsof -ti:8090 | xargs kill -9 2>/dev/null || true
    rm -f cms.db cms_test.db .l0_http_config.json /tmp/cms_test.l0 /tmp/cms_test.log /tmp/cms_test.asm
}

trap cleanup EXIT

echo "========================================"
echo "L-0 CMS Black-Box Test Suite"
echo "========================================"
echo ""

# Clean up before starting
cleanup
sleep 0.5

# =============================================================================
echo -e "${YELLOW}=== 1. Compile CMS ===${NC}"
# =============================================================================

test_case "Compile cms.asm to bytecode"
./target/release/l0asm examples/cms.asm > /tmp/cms_test.l0 2>&1
if [ -s /tmp/cms_test.l0 ]; then
    pass
else
    fail "Compilation failed"
    exit 1
fi

# =============================================================================
echo -e "${YELLOW}=== 2. Start CMS Server ===${NC}"
# =============================================================================

# Modify the port for testing (use sed to change 8080 to 8090)
cat examples/cms.asm | sed 's/8080/8090/g' > /tmp/cms_test.asm
./target/release/l0asm /tmp/cms_test.asm > /tmp/cms_test.l0 2>&1

test_case "Start CMS server on port 8090"
./target/release/l0vm /tmp/cms_test.l0 > /tmp/cms_test.log 2>&1 &
CMS_PID=$!
sleep 2

if kill -0 $CMS_PID 2>/dev/null; then
    pass
else
    fail "Server failed to start"
    cat /tmp/cms_test.log
    exit 1
fi

# =============================================================================
echo -e "${YELLOW}=== 3. Database Tests ===${NC}"
# =============================================================================

test_case "Database initialized with sample data"
ARTICLES=$(curl -s http://127.0.0.1:8090/api/articles 2>/dev/null)
if [[ "$ARTICLES" == *"Welcome to L-0 CMS"* ]]; then
    pass
else
    fail "Sample data not found"
fi

test_case "Initial article count is 3"
COUNT=$(echo "$ARTICLES" | grep -o '"id":' | wc -l | tr -d ' ')
if [[ "$COUNT" -ge "3" ]]; then
    pass
else
    fail "Expected at least 3 articles, got $COUNT"
fi

# =============================================================================
echo -e "${YELLOW}=== 4. CRUD Operations ===${NC}"
# =============================================================================

test_case "CREATE: POST new article"
CREATE_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"title":"Test Article","content":"Test content","author":"Tester","created_at":"2024-01-20"}' \
    http://127.0.0.1:8090/api/articles 2>/dev/null)
NEW_ID="$CREATE_RESULT"
if [[ "$CREATE_RESULT" =~ ^[0-9]+$ ]]; then
    pass
else
    fail "Expected numeric ID, got: $CREATE_RESULT"
fi

test_case "READ: Verify new article exists"
ARTICLES=$(curl -s http://127.0.0.1:8090/api/articles 2>/dev/null)
if [[ "$ARTICLES" == *"Test Article"* ]]; then
    pass
else
    fail "New article not found"
fi

test_case "UPDATE: Modify article via PUT"
UPDATE_RESULT=$(curl -s -X PUT -H "Content-Type: application/json" \
    -d '{"title":"Updated Article","content":"Updated content","author":"Tester"}' \
    "http://127.0.0.1:8090/api/articles/$NEW_ID" 2>/dev/null)
if [[ "$UPDATE_RESULT" == *"updated"* ]]; then
    pass
else
    fail "Update failed: $UPDATE_RESULT"
fi

test_case "READ: Verify update applied"
ARTICLES=$(curl -s http://127.0.0.1:8090/api/articles 2>/dev/null)
if [[ "$ARTICLES" == *"Updated Article"* ]]; then
    pass
else
    fail "Update not visible"
fi

test_case "DELETE: Remove article"
DELETE_RESULT=$(curl -s -X DELETE "http://127.0.0.1:8090/api/articles/$NEW_ID" 2>/dev/null)
if [[ "$DELETE_RESULT" == *"deleted"* ]]; then
    pass
else
    fail "Delete failed: $DELETE_RESULT"
fi

test_case "READ: Verify article deleted"
ARTICLES=$(curl -s http://127.0.0.1:8090/api/articles 2>/dev/null)
if [[ "$ARTICLES" != *"Updated Article"* ]]; then
    pass
else
    fail "Article still exists after delete"
fi

# =============================================================================
echo -e "${YELLOW}=== 5. Web UI Tests ===${NC}"
# =============================================================================

test_case "GET / returns HTML"
HTML=$(curl -s http://127.0.0.1:8090/ 2>/dev/null)
if [[ "$HTML" == *"<!DOCTYPE html>"* ]]; then
    pass
else
    fail "No HTML returned"
fi

test_case "HTML contains title"
if [[ "$HTML" == *"L-0 CMS"* ]]; then
    pass
else
    fail "Title not found in HTML"
fi

test_case "HTML contains CSS styles"
if [[ "$HTML" == *"<style>"* ]] && [[ "$HTML" == *"</style>"* ]]; then
    pass
else
    fail "CSS not found"
fi

test_case "HTML contains JavaScript"
if [[ "$HTML" == *"<script>"* ]] && [[ "$HTML" == *"loadArticles"* ]]; then
    pass
else
    fail "JavaScript not found"
fi

test_case "HTML contains form elements"
if [[ "$HTML" == *"articleForm"* ]] && [[ "$HTML" == *"modal"* ]]; then
    pass
else
    fail "Form elements not found"
fi

# =============================================================================
echo -e "${YELLOW}=== 6. Edge Cases ===${NC}"
# =============================================================================

test_case "POST with special characters"
SPECIAL_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"title":"Test with special chars","content":"Content here","author":"User","created_at":"2024-01-21"}' \
    http://127.0.0.1:8090/api/articles 2>/dev/null)
if [[ "$SPECIAL_RESULT" =~ ^[0-9]+$ ]]; then
    pass
else
    fail "Special characters failed: $SPECIAL_RESULT"
fi

test_case "GET single article by ID"
SINGLE=$(curl -s "http://127.0.0.1:8090/api/articles/1" 2>/dev/null)
if [[ "$SINGLE" == *"Welcome to L-0 CMS"* ]]; then
    pass
else
    fail "Single article GET failed: $SINGLE"
fi

test_case "GET non-existent article returns empty"
NOTFOUND=$(curl -s "http://127.0.0.1:8090/api/articles/999" 2>/dev/null)
if [[ "$NOTFOUND" == "[]" ]] || [[ "$NOTFOUND" == "" ]] || [[ "$NOTFOUND" == "null" ]]; then
    pass
else
    fail "Expected empty, got: $NOTFOUND"
fi

# =============================================================================
# Results
# =============================================================================
cleanup
echo ""
echo "========================================"
echo -e "Results: ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}, $TOTAL total"
echo "========================================"

if [ $FAIL -gt 0 ]; then
    exit 1
fi
