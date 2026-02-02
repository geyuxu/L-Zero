# Full test: Control flow
# Verifies: JMP, CMP, BEQ, BLT, BGT (no BNE/BLE/BGE/CALL/RET in ISA)

SET 100, 0   # test counter

# JMP test
JMP test_beq
PANIC "JMP failed"

# BEQ test
test_beq:
SET 1, 42
SET 2, 42
CMP 1, 2
BEQ test_bne_pattern
PANIC "BEQ failed"

# BNE pattern (no native BNE): if equal, skip to fail
test_bne_pattern:
SET 1, 10
SET 2, 20
CMP 1, 2
BEQ bne_fail
JMP test_blt
bne_fail:
PANIC "BNE pattern failed"

# BLT test
test_blt:
SET 1, 5
SET 2, 10
CMP 1, 2
BLT test_bgt
PANIC "BLT failed"

# BGT test
test_bgt:
SET 1, 20
SET 2, 10
CMP 1, 2
BGT test_ble_pattern
PANIC "BGT failed"

# BLE pattern (using BLT + BEQ)
test_ble_pattern:
SET 1, 5
SET 2, 10
CMP 1, 2
BLT test_ble2
PANIC "BLE less failed"

test_ble2:
SET 1, 10
SET 2, 10
CMP 1, 2
BEQ test_bge_pattern
PANIC "BLE equal failed"

# BGE pattern (using BGT + BEQ)
test_bge_pattern:
SET 1, 20
SET 2, 10
CMP 1, 2
BGT test_bge2
PANIC "BGE greater failed"

test_bge2:
SET 1, 10
SET 2, 10
CMP 1, 2
BEQ tests_done
PANIC "BGE equal failed"

tests_done:
SETS 255, "Control flow tests passed"
TEXEC 0x5000, 255, 0
HALT
