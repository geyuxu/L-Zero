# Test MARK/RESET watermark pattern
# This test verifies that heap allocations are properly reset

# Initial message
SETS 1, "Testing MARK/RESET watermark..."
TEXEC 0x5000, 1, 0

# Test 1: Basic MARK/RESET
SETS 1, "[Test 1] Basic MARK/RESET"
TEXEC 0x5000, 1, 0

MARK 100                    # Save watermark to R100

# Allocate some strings (these should be freed after RESET)
SETS 10, "String A"
SETS 11, "String B"
SETS 12, "String C"
SCAT 13, 10, 11             # "String AString B"
SCAT 14, 13, 12             # "String AString BString C"

# Print concatenated result before reset
TEXEC 0x5000, 14, 0

RESET 100                   # Reset heap to watermark

SETS 1, "[Test 1] PASS - Heap reset completed"
TEXEC 0x5000, 1, 0

# Test 2: Multiple MARK/RESET cycles (simulating request loop)
SETS 1, "[Test 2] Multiple cycles"
TEXEC 0x5000, 1, 0

SET 20, 0                   # Counter
SET 21, 3                   # Limit

CycleLoop:
    CMP 20, 21
    BEQ CycleDone

    MARK 100                # Mark before each "request"

    # Simulate request processing with allocations
    SETS 30, "Request "
    ITOA 31, 20
    SCAT 32, 30, 31         # "Request N"
    TEXEC 0x5000, 32, 0

    RESET 100               # Reset after each "request"

    SET 50, 1
    ADD 20, 20, 50
    JMP CycleLoop

CycleDone:
SETS 1, "[Test 2] PASS - Multiple cycles completed"
TEXEC 0x5000, 1, 0

# Final summary
SETS 1, "All MARK/RESET tests passed!"
TEXEC 0x5000, 1, 0

HALT
