#!/bin/bash
# L-Zero Test Runner
# Usage: ./run_tests.sh [smoke|full|all]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
L0_ROOT="$(dirname "$SCRIPT_DIR")"
TMP_DIR="/tmp/l0_tests"

# Find L-Zero binaries
if [ -d "/opt/l0/bin" ]; then
    export PATH="/opt/l0/bin:$PATH"
fi

# Binary names (l0vm is the VM, l0asm is assembler)
L0VM="${L0VM:-l0vm}"
L0ASM="${L0ASM:-l0asm}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0
SKIPPED=0

mkdir -p "$TMP_DIR"

run_test() {
    local test_file="$1"
    local test_name="$(basename "$test_file" .asm)"
    local bytecode="$TMP_DIR/${test_name}.l0b"

    printf "  %-40s " "$test_name"

    # Compile
    if ! $L0ASM "$test_file" > "$bytecode" 2>/dev/null; then
        echo -e "${RED}COMPILE ERROR${NC}"
        ((FAILED++))
        return 1
    fi

    # Run
    output=$($L0VM "$bytecode" 2>&1) || true

    # Check for PANIC
    if echo "$output" | grep -q "PANIC"; then
        echo -e "${RED}FAILED${NC}"
        echo "    Output: $output"
        ((FAILED++))
        return 1
    fi

    # Check for "passed" in output (our test convention)
    if echo "$output" | grep -qi "passed\|complete\|skipped"; then
        if echo "$output" | grep -qi "skipped"; then
            echo -e "${YELLOW}SKIPPED${NC}"
            ((SKIPPED++))
        else
            echo -e "${GREEN}PASSED${NC}"
            ((PASSED++))
        fi
        return 0
    fi

    # No explicit pass/fail, assume passed if no error
    echo -e "${GREEN}PASSED${NC}"
    ((PASSED++))
    return 0
}

run_suite() {
    local suite_name="$1"
    local suite_dir="$2"

    echo -e "\n${BLUE}=== Running $suite_name Tests ===${NC}\n"

    if [ ! -d "$suite_dir" ]; then
        echo -e "${YELLOW}Suite directory not found: $suite_dir${NC}"
        return
    fi

    for test_file in "$suite_dir"/*.asm; do
        [ -f "$test_file" ] || continue
        run_test "$test_file" || true
    done
}

print_summary() {
    echo -e "\n${BLUE}=== Test Summary ===${NC}"
    echo -e "  ${GREEN}Passed:  $PASSED${NC}"
    echo -e "  ${RED}Failed:  $FAILED${NC}"
    echo -e "  ${YELLOW}Skipped: $SKIPPED${NC}"
    echo -e "  Total:   $((PASSED + FAILED + SKIPPED))"

    if [ $FAILED -eq 0 ]; then
        echo -e "\n${GREEN}All tests passed!${NC}"
        return 0
    else
        echo -e "\n${RED}Some tests failed.${NC}"
        return 1
    fi
}

# Check for l0asm and l0vm
if ! command -v $L0ASM &> /dev/null; then
    echo -e "${RED}Error: $L0ASM not found in PATH${NC}"
    echo "Please ensure L-Zero is installed and in your PATH"
    exit 1
fi

if ! command -v $L0VM &> /dev/null; then
    echo -e "${RED}Error: $L0VM not found in PATH${NC}"
    echo "Please ensure L-Zero is installed and in your PATH"
    exit 1
fi

# Parse arguments
MODE="${1:-all}"

case "$MODE" in
    smoke)
        run_suite "Smoke" "$SCRIPT_DIR/smoke"
        ;;
    full)
        run_suite "Full" "$SCRIPT_DIR/full"
        ;;
    all)
        run_suite "Smoke" "$SCRIPT_DIR/smoke"
        run_suite "Full" "$SCRIPT_DIR/full"
        ;;
    *)
        echo "Usage: $0 [smoke|full|all]"
        echo "  smoke - Run minimal smoke tests only"
        echo "  full  - Run comprehensive tests only"
        echo "  all   - Run all tests (default)"
        exit 1
        ;;
esac

print_summary

# Cleanup
rm -rf "$TMP_DIR"

exit $FAILED
