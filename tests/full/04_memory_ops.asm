# Full test: Memory operations
# Verifies: NEW, NEWR, STORE64, LOAD64, HLEN

# NEW with STORE64/LOAD64 pattern (array simulation)
NEW 10, 80                    # allocate 80 bytes (10 i64 slots)

# Store values 0-9 using STORE64
SET 1, 0
SET 2, 10
fill_loop:
    CMP 1, 2
    BLT fill_continue
    JMP fill_done
fill_continue:
    STORE64 10, 1, 1          # array[i] = i (using i64)
    SET 3, 1
    ADD 1, 1, 3
    JMP fill_loop
fill_done:

# Verify values using LOAD64
SET 1, 5
LOAD64 20, 10, 1
SET 21, 5
CMP 20, 21
BEQ verify_ok
PANIC "Array LOAD64 failed"
verify_ok:

# Verify another value
SET 1, 7
LOAD64 22, 10, 1
SET 23, 7
CMP 22, 23
BEQ verify2_ok
PANIC "Array LOAD64 failed at index 7"
verify2_ok:

# NEWR dynamic allocation
SET 30, 128
NEWR 31, 30
HLEN 32, 31
SET 33, 128
CMP 32, 33
BEQ newr_ok
PANIC "NEWR size failed"
newr_ok:

SETS 255, "Memory operations tests passed"
TEXEC 0x5000, 255, 0
HALT
