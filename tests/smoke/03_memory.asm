# Smoke test: Memory operations
# Verifies: NEW, NEWR, HLEN, STORE64, LOAD64

# NEW test: allocate 32 bytes
NEW 0, 32
HLEN 1, 0
SET 2, 32
CMP 1, 2
BEQ new_ok
PANIC "NEW size mismatch"
new_ok:

# NEWR test: dynamic allocation
SET 3, 64
NEWR 4, 3
HLEN 5, 4
SET 6, 64
CMP 5, 6
BEQ newr_ok
PANIC "NEWR size mismatch"
newr_ok:

# STORE64/LOAD64 test
NEW 10, 64
SET 11, 42
SET 12, 0
STORE64 10, 12, 11
LOAD64 13, 10, 12
CMP 11, 13
BEQ store_ok
PANIC "STORE64/LOAD64 failed"
store_ok:

SETS 255, "Memory tests passed"
TEXEC 0x5000, 255, 0
HALT
