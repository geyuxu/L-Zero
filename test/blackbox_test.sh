#!/bin/bash
# L-0 Black-Box Test Suite
# Tests the system purely from README.md documented interfaces
# No internal knowledge required

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
    rm -f /tmp/test_*.asm /tmp/test_*.l0 /tmp/test_*.c /tmp/test_program
    rm -f .l0_http_config.json .l0_http_pending_*.json /tmp/l0_http_daemon.pid
    rm -f test_blackbox.db l0_database.json
}

trap cleanup EXIT

echo "========================================"
echo "L-0 Black-Box Test Suite"
echo "========================================"
echo ""

# =============================================================================
echo -e "${YELLOW}=== 1. Toolchain Tests ===${NC}"
# =============================================================================

test_case "l0vm --info returns valid JSON"
INFO=$(./target/release/l0vm --info 2>/dev/null || echo "")
if [[ "$INFO" == *"version"* ]] && [[ "$INFO" == *"isa"* ]]; then
    pass
else
    fail "Missing version or isa in output"
fi

test_case "l0asm compiles Hello World"
cat > /tmp/test_hello.asm << 'EOF'
SETS 1, Hello, L-0!
TEXEC 0x5000, 1, 255
HALT
EOF
./target/release/l0asm /tmp/test_hello.asm > /tmp/test_hello.l0 2>/dev/null
if [ -s /tmp/test_hello.l0 ]; then
    pass
else
    fail "No output bytecode"
fi

test_case "l0vm executes Hello World"
OUTPUT=$(./target/release/l0vm /tmp/test_hello.l0 2>/dev/null)
if [[ "$OUTPUT" == *"Hello"* ]]; then
    pass
else
    fail "Expected 'Hello' in output: $OUTPUT"
fi

test_case "l0cc generates C code"
./target/release/l0cc /tmp/test_hello.l0 -o /tmp/test_hello.c 2>/dev/null
if [ -s /tmp/test_hello.c ] && grep -q "main" /tmp/test_hello.c; then
    pass
else
    fail "No valid C code generated"
fi

echo ""

# =============================================================================
echo -e "${YELLOW}=== 2. ISA Core Instructions ===${NC}"
# =============================================================================

test_case "SET and ITOA work correctly"
cat > /tmp/test_set.asm << 'EOF'
SET 1, 42
ITOA 2, 1
TEXEC 0x5000, 2, 255
HALT
EOF
./target/release/l0asm /tmp/test_set.asm > /tmp/test_set.l0
OUTPUT=$(./target/release/l0vm /tmp/test_set.l0 2>/dev/null)
if [[ "$OUTPUT" == "42" ]]; then
    pass
else
    fail "Expected '42', got '$OUTPUT'"
fi

test_case "ADD instruction"
cat > /tmp/test_add.asm << 'EOF'
SET 1, 10
SET 2, 32
ADD 3, 1, 2
ITOA 4, 3
TEXEC 0x5000, 4, 255
HALT
EOF
./target/release/l0asm /tmp/test_add.asm > /tmp/test_add.l0
OUTPUT=$(./target/release/l0vm /tmp/test_add.l0 2>/dev/null)
if [[ "$OUTPUT" == "42" ]]; then
    pass
else
    fail "Expected '42', got '$OUTPUT'"
fi

test_case "CMP and BEQ (control flow)"
cat > /tmp/test_cmp.asm << 'EOF'
SET 1, 5
SET 2, 5
CMP 1, 2
BEQ Equal
SETS 3, "not equal"
JMP Done
Equal:
SETS 3, "equal"
Done:
TEXEC 0x5000, 3, 255
HALT
EOF
./target/release/l0asm /tmp/test_cmp.asm > /tmp/test_cmp.l0
OUTPUT=$(./target/release/l0vm /tmp/test_cmp.l0 2>/dev/null)
if [[ "$OUTPUT" == "equal" ]]; then
    pass
else
    fail "Expected 'equal', got '$OUTPUT'"
fi

test_case "Loop with JMP"
cat > /tmp/test_loop.asm << 'EOF'
SET 1, 0
SET 2, 3
SET 10, 1
Loop:
CMP 1, 2
BEQ Done
ADD 1, 1, 10
JMP Loop
Done:
ITOA 3, 1
TEXEC 0x5000, 3, 255
HALT
EOF
./target/release/l0asm /tmp/test_loop.asm > /tmp/test_loop.l0
OUTPUT=$(./target/release/l0vm /tmp/test_loop.l0 2>/dev/null)
if [[ "$OUTPUT" == "3" ]]; then
    pass
else
    fail "Expected '3', got '$OUTPUT'"
fi

test_case "SCAT (string concatenation)"
cat > /tmp/test_scat.asm << 'EOF'
SETS 1, "Hello"
SETS 2, "World"
SCAT 3, 1, 2
TEXEC 0x5000, 3, 255
HALT
EOF
./target/release/l0asm /tmp/test_scat.asm > /tmp/test_scat.l0
OUTPUT=$(./target/release/l0vm /tmp/test_scat.l0 2>/dev/null)
if [[ "$OUTPUT" == "HelloWorld" ]]; then
    pass
else
    fail "Expected 'HelloWorld', got '$OUTPUT'"
fi

test_case "HLEN (string length)"
cat > /tmp/test_hlen.asm << 'EOF'
SETS 1, "Hello"
HLEN 2, 1
ITOA 3, 2
TEXEC 0x5000, 3, 255
HALT
EOF
./target/release/l0asm /tmp/test_hlen.asm > /tmp/test_hlen.l0
OUTPUT=$(./target/release/l0vm /tmp/test_hlen.l0 2>/dev/null)
if [[ "$OUTPUT" == "5" ]]; then
    pass
else
    fail "Expected '5', got '$OUTPUT'"
fi

test_case "REGEX pattern matching"
cat > /tmp/test_regex.asm << 'EOF'
SETS 1, "[0-9]+"
SETS 2, "test123abc"
REGEX 3, 1, 2
ITOA 4, 3
TEXEC 0x5000, 4, 255
HALT
EOF
./target/release/l0asm /tmp/test_regex.asm > /tmp/test_regex.l0
OUTPUT=$(./target/release/l0vm /tmp/test_regex.l0 2>/dev/null)
if [[ "$OUTPUT" == "1" ]]; then
    pass
else
    fail "Expected '1' (match), got '$OUTPUT'"
fi

echo ""

# =============================================================================
echo -e "${YELLOW}=== 3. Builtin Tools ===${NC}"
# =============================================================================

test_case "TIME returns Unix timestamp"
cat > /tmp/test_time.asm << 'EOF'
TEXEC 0x5008, 255, 1
ATOI 2, 1
SET 3, 1000000000
CMP 2, 3
BGT Valid
SETS 4, "invalid"
JMP Done
Valid:
SETS 4, "valid"
Done:
TEXEC 0x5000, 4, 255
HALT
EOF
./target/release/l0asm /tmp/test_time.asm > /tmp/test_time.l0
OUTPUT=$(./target/release/l0vm /tmp/test_time.l0 2>/dev/null)
if [[ "$OUTPUT" == "valid" ]]; then
    pass
else
    fail "Timestamp should be > 1000000000"
fi

test_case "ABS (absolute value)"
cat > /tmp/test_abs.asm << 'EOF'
SETS 1, "-42"
TEXEC 0x5005, 1, 2
TEXEC 0x5000, 2, 255
HALT
EOF
./target/release/l0asm /tmp/test_abs.asm > /tmp/test_abs.l0
OUTPUT=$(./target/release/l0vm /tmp/test_abs.l0 2>/dev/null)
if [[ "$OUTPUT" == "42" ]]; then
    pass
else
    fail "Expected '42', got '$OUTPUT'"
fi

test_case "MIN/MAX"
cat > /tmp/test_minmax.asm << 'EOF'
SETS 1, "10,3"
TEXEC 0x5006, 1, 2
TEXEC 0x5000, 2, 255
SETS 1, "10,3"
TEXEC 0x5007, 1, 2
TEXEC 0x5000, 2, 255
HALT
EOF
./target/release/l0asm /tmp/test_minmax.asm > /tmp/test_minmax.l0
OUTPUT=$(./target/release/l0vm /tmp/test_minmax.l0 2>/dev/null)
if [[ "$OUTPUT" == *"3"* ]] && [[ "$OUTPUT" == *"10"* ]]; then
    pass
else
    fail "Expected '3' and '10' in output"
fi

test_case "UPPER/LOWER"
cat > /tmp/test_case.asm << 'EOF'
SETS 1, "Hello"
TEXEC 0x500C, 1, 2
TEXEC 0x5000, 2, 255
TEXEC 0x500D, 1, 3
TEXEC 0x5000, 3, 255
HALT
EOF
./target/release/l0asm /tmp/test_case.asm > /tmp/test_case.l0
OUTPUT=$(./target/release/l0vm /tmp/test_case.l0 2>/dev/null)
if [[ "$OUTPUT" == *"HELLO"* ]] && [[ "$OUTPUT" == *"hello"* ]]; then
    pass
else
    fail "Expected 'HELLO' and 'hello'"
fi

echo ""

# =============================================================================
echo -e "${YELLOW}=== 4. DB Plugin ===${NC}"
# =============================================================================

test_case "DB_INIT and DB_CREATE_TABLE"
cat > /tmp/test_db.asm << 'EOF'
SETS 255, "test_blackbox.db"
TEXEC 0x9000, 255, 0
SETS 255, "users|id INTEGER PRIMARY KEY,name TEXT"
TEXEC 0x9001, 255, 0
TEXEC 0x5000, 0, 255
HALT
EOF
./target/release/l0asm /tmp/test_db.asm > /tmp/test_db.l0
OUTPUT=$(./target/release/l0vm /tmp/test_db.l0 2>/dev/null)
if [[ "$OUTPUT" == *"ok"* ]] || [[ "$OUTPUT" == *"created"* ]] || [[ "$OUTPUT" == *"exists"* ]]; then
    pass
else
    fail "Expected success: $OUTPUT"
fi

test_case "DB_INSERT with JSON format"
cat > /tmp/test_db_insert.asm << 'EOF'
SETS 255, "test_blackbox.db"
TEXEC 0x9000, 255, 0
SETS 255, "users|{\"name\":\"Alice\"}"
TEXEC 0x9002, 255, 0
TEXEC 0x5000, 0, 255
HALT
EOF
./target/release/l0asm /tmp/test_db_insert.asm > /tmp/test_db_insert.l0
OUTPUT=$(./target/release/l0vm /tmp/test_db_insert.l0 2>/dev/null)
if [[ "$OUTPUT" == *"ok"* ]] || [[ "$OUTPUT" == *"1"* ]]; then
    pass
else
    fail "Insert failed: $OUTPUT"
fi

test_case "DB_SELECT returns data"
cat > /tmp/test_db_select.asm << 'EOF'
SETS 255, "test_blackbox.db"
TEXEC 0x9000, 255, 0
SETS 255, "users"
TEXEC 0x9003, 255, 0
TEXEC 0x5000, 0, 255
HALT
EOF
./target/release/l0asm /tmp/test_db_select.asm > /tmp/test_db_select.l0
OUTPUT=$(./target/release/l0vm /tmp/test_db_select.l0 2>/dev/null)
if [[ "$OUTPUT" == *"Alice"* ]]; then
    pass
else
    fail "Expected 'Alice' in data: $OUTPUT"
fi

test_case "DB_DELETE removes row"
cat > /tmp/test_db_delete.asm << 'EOF'
SETS 255, "test_blackbox.db"
TEXEC 0x9000, 255, 0
SETS 255, "users|name='Alice'"
TEXEC 0x9005, 255, 0
SETS 255, "users"
TEXEC 0x9003, 255, 0
TEXEC 0x5000, 0, 255
HALT
EOF
./target/release/l0asm /tmp/test_db_delete.asm > /tmp/test_db_delete.l0
OUTPUT=$(./target/release/l0vm /tmp/test_db_delete.l0 2>/dev/null)
if [[ "$OUTPUT" != *"Alice"* ]]; then
    pass
else
    fail "Alice should be deleted: $OUTPUT"
fi

echo ""

# =============================================================================
echo -e "${YELLOW}=== 5. HTTP Plugin ===${NC}"
# =============================================================================

cleanup

test_case "HTTP_INIT configures port"
echo '{"method":"init","args":["18500"]}' | ./target/release/http_plugin > /tmp/http_init.out
if grep -q "ok" /tmp/http_init.out; then
    pass
else
    fail "HTTP_INIT failed"
fi

test_case "HTTP_ROUTE registers static route"
echo '{"method":"route","args":["GET /test|Hello from test"]}' | ./target/release/http_plugin > /tmp/http_route.out
if grep -q "ok" /tmp/http_route.out; then
    pass
else
    fail "HTTP_ROUTE failed"
fi

test_case "HTTP_ROUTE with JSON array content"
echo '{"method":"route","args":["GET /api/items|[{\"id\":1},{\"id\":2}]"]}' | ./target/release/http_plugin > /tmp/http_route2.out
if grep -q "ok" /tmp/http_route2.out; then
    pass
else
    fail "HTTP_ROUTE with JSON failed"
fi

test_case "HTTP_LIST_ROUTES shows registered routes"
echo '{"method":"list_routes","args":[]}' | ./target/release/http_plugin > /tmp/http_list.out
if grep -q "/test" /tmp/http_list.out && grep -q "/api/items" /tmp/http_list.out; then
    pass
else
    fail "Routes not listed: $(cat /tmp/http_list.out)"
fi

# Test HTTP_SERVE_ONCE
test_case "HTTP_SERVE_ONCE handles request"
cleanup
echo '{"method":"init","args":["18501"]}' | ./target/release/http_plugin > /dev/null
echo '{"method":"route","args":["GET /hello|World"]}' | ./target/release/http_plugin > /dev/null

# Start serve_once in background
(echo '{"method":"serve_once","args":[]}' | ./target/release/http_plugin > /tmp/serve_once.out) &
SERVE_PID=$!
sleep 0.5

# Make request
RESPONSE=$(curl -s http://127.0.0.1:18501/hello 2>/dev/null || echo "")
wait $SERVE_PID 2>/dev/null || true

if [[ "$RESPONSE" == *"World"* ]]; then
    pass
else
    fail "Expected 'World', got '$RESPONSE'"
fi

echo ""

# =============================================================================
echo -e "${YELLOW}=== 6. Dynamic HTTP Mode ===${NC}"
# =============================================================================

cleanup
lsof -ti:18502 | xargs kill -9 2>/dev/null || true
sleep 0.5

test_case "HTTP_LISTEN starts daemon"
echo '{"method":"init","args":["18502"]}' | ./target/release/http_plugin > /dev/null
(echo '{"method":"listen","args":[]}' | ./target/release/http_plugin > /tmp/listen.out) &
LISTEN_PID=$!
sleep 1

if [ -f /tmp/l0_http_daemon.pid ]; then
    pass
else
    fail "Daemon not started"
fi

test_case "Dynamic request/response flow"
# Send request in background
(curl -s -X POST -d '{"test":"data"}' http://127.0.0.1:18502/api/dynamic > /tmp/curl_dynamic.out) &
CURL_PID=$!
sleep 0.5

# Send response
echo '{"method":"send","args":["{\"status\":\"success\",\"echo\":\"received\"}"]}' | ./target/release/http_plugin > /dev/null
sleep 1

wait $CURL_PID 2>/dev/null || true
wait $LISTEN_PID 2>/dev/null || true

RESULT=$(cat /tmp/curl_dynamic.out 2>/dev/null || echo "")
if [[ "$RESULT" == *"success"* ]]; then
    pass
else
    fail "Dynamic response failed: $RESULT"
fi

test_case "Stop dynamic daemon"
STOP_RESULT=$(echo '{"method":"stop_dynamic","args":[]}' | ./target/release/http_plugin)
if [[ "$STOP_RESULT" == *"stopped"* ]]; then
    pass
else
    fail "Stop failed: $STOP_RESULT"
fi

echo ""

# =============================================================================
echo -e "${YELLOW}=== 7. StdLib Patterns (via ASM) ===${NC}"
# =============================================================================

test_case "Fibonacci pattern (F_10 = 55)"
cat > /tmp/test_fib.asm << 'EOF'
# Test fibonacci pattern from README
SET 0, 10              # Input n=10

# ---- BEGIN fibonacci_test ----
SET 4, 0
CMP 0, 4
BEQ fibonacci_test_zero

SET 4, 1
CMP 0, 4
BEQ fibonacci_test_one

MOV 4, 0               # R4 = n
SET 5, 0               # R5 = F_(i-2) = 0
SET 6, 1               # R6 = F_(i-1) = 1
SET 7, 1               # R7 = current i

fibonacci_test_loop:
CMP 7, 4
BEQ fibonacci_test_finish

ADD 50, 5, 6           # R50 = F_(i-2) + F_(i-1)
MOV 5, 6               # F_(i-2) = F_(i-1)
MOV 6, 50              # F_(i-1) = new value
SET 51, 1
ADD 7, 7, 51           # i++
JMP fibonacci_test_loop

fibonacci_test_zero:
SET 0, 0
JMP fibonacci_test_done

fibonacci_test_one:
SET 0, 1
JMP fibonacci_test_done

fibonacci_test_finish:
MOV 0, 6               # R0 = F_n
fibonacci_test_done:
# ---- END fibonacci_test ----

ITOA 1, 0
TEXEC 0x5000, 1, 255
HALT
EOF
./target/release/l0asm /tmp/test_fib.asm > /tmp/test_fib.l0
OUTPUT=$(./target/release/l0vm /tmp/test_fib.l0 2>/dev/null)
if [[ "$OUTPUT" == "55" ]]; then
    pass
else
    fail "Expected '55', got '$OUTPUT'"
fi

test_case "GCD pattern (gcd(48, 18) = 6)"
cat > /tmp/test_gcd.asm << 'EOF'
SET 0, 48              # a
SET 1, 18              # b

# ---- BEGIN gcd_test ----
MOV 4, 0               # R4 = a
MOV 5, 1               # R5 = b

gcd_test_loop:
SET 50, 0
CMP 5, 50
BEQ gcd_test_finish

MOD 50, 4, 5           # R50 = a % b
MOV 4, 5               # a = b
MOV 5, 50              # b = a % b
JMP gcd_test_loop

gcd_test_finish:
MOV 0, 4               # R0 = gcd
# ---- END gcd_test ----

ITOA 1, 0
TEXEC 0x5000, 1, 255
HALT
EOF
./target/release/l0asm /tmp/test_gcd.asm > /tmp/test_gcd.l0
OUTPUT=$(./target/release/l0vm /tmp/test_gcd.l0 2>/dev/null)
if [[ "$OUTPUT" == "6" ]]; then
    pass
else
    fail "Expected '6', got '$OUTPUT'"
fi

test_case "is_prime pattern (17 is prime)"
cat > /tmp/test_prime.asm << 'EOF'
SET 0, 17              # Test if 17 is prime

# ---- BEGIN is_prime_test ----
SET 4, 2
CMP 0, 4
BLT is_prime_test_no
BEQ is_prime_test_yes

SET 4, 2
MOD 5, 0, 4
SET 6, 0
CMP 5, 6
BEQ is_prime_test_no

MOV 6, 0
SET 4, 3

is_prime_test_loop:
MUL 5, 4, 4
CMP 5, 6
BGT is_prime_test_yes

MOD 5, 6, 4
SET 50, 0
CMP 5, 50
BEQ is_prime_test_no

SET 50, 2
ADD 4, 4, 50
JMP is_prime_test_loop

is_prime_test_yes:
SET 0, 1
JMP is_prime_test_done

is_prime_test_no:
SET 0, 0
is_prime_test_done:
# ---- END is_prime_test ----

ITOA 1, 0
TEXEC 0x5000, 1, 255
HALT
EOF
./target/release/l0asm /tmp/test_prime.asm > /tmp/test_prime.l0
OUTPUT=$(./target/release/l0vm /tmp/test_prime.l0 2>/dev/null)
if [[ "$OUTPUT" == "1" ]]; then
    pass
else
    fail "Expected '1' (prime), got '$OUTPUT'"
fi

echo ""

# =============================================================================
echo -e "${YELLOW}=== 8. Edge Cases ===${NC}"
# =============================================================================

test_case "Quoted string with comma"
cat > /tmp/test_comma.asm << 'EOF'
SETS 1, "Hello, World!"
TEXEC 0x5000, 1, 255
HALT
EOF
./target/release/l0asm /tmp/test_comma.asm > /tmp/test_comma.l0
OUTPUT=$(./target/release/l0vm /tmp/test_comma.l0 2>/dev/null)
if [[ "$OUTPUT" == "Hello, World!" ]]; then
    pass
else
    fail "Expected 'Hello, World!', got '$OUTPUT'"
fi

test_case "Escape sequences in SETS"
cat > /tmp/test_escape.asm << 'EOF'
SETS 1, "Line1\nLine2"
TEXEC 0x5000, 1, 255
HALT
EOF
./target/release/l0asm /tmp/test_escape.asm > /tmp/test_escape.l0
OUTPUT=$(./target/release/l0vm /tmp/test_escape.l0 2>/dev/null)
if [[ "$OUTPUT" == *"Line1"* ]] && [[ "$OUTPUT" == *"Line2"* ]]; then
    pass
else
    fail "Escape sequence not working: $OUTPUT"
fi

test_case "STORE64/LOAD64 (64-bit memory)"
cat > /tmp/test_64bit.asm << 'EOF'
NEW 2, 80              # Allocate 80 bytes (literal size, not register!)
SET 3, 0               # index (used as R[off] in STORE64/LOAD64)
SET 4, 12345678        # value
STORE64 2, 3, 4        # arr[R3] = R4, offset = R[3]*8 = 0
LOAD64 5, 2, 3         # R5 = arr[R3]
ITOA 6, 5
TEXEC 0x5000, 6, 255
HALT
EOF
./target/release/l0asm /tmp/test_64bit.asm > /tmp/test_64bit.l0
OUTPUT=$(./target/release/l0vm /tmp/test_64bit.l0 2>/dev/null)
if [[ "$OUTPUT" == "12345678" ]]; then
    pass
else
    fail "Expected '12345678', got '$OUTPUT'"
fi

test_case "Multiple TEXEC calls in sequence"
cat > /tmp/test_multi.asm << 'EOF'
SETS 1, "One"
TEXEC 0x5000, 1, 255
SETS 2, "Two"
TEXEC 0x5000, 2, 255
SETS 3, "Three"
TEXEC 0x5000, 3, 255
HALT
EOF
./target/release/l0asm /tmp/test_multi.asm > /tmp/test_multi.l0
OUTPUT=$(./target/release/l0vm /tmp/test_multi.l0 2>/dev/null)
if [[ "$OUTPUT" == *"One"* ]] && [[ "$OUTPUT" == *"Two"* ]] && [[ "$OUTPUT" == *"Three"* ]]; then
    pass
else
    fail "Expected 'One', 'Two', 'Three'"
fi

echo ""

# =============================================================================
# Results
# =============================================================================
cleanup
echo "========================================"
echo -e "Results: ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}, $TOTAL total"
echo "========================================"

if [ $FAIL -gt 0 ]; then
    exit 1
fi
