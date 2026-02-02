# Smoke test: Control flow
# Verifies: JMP, CMP, BEQ, BLT, BGT

# JMP test
JMP skip_panic
PANIC "JMP failed"
skip_panic:

# BEQ test
SET 1, 5
SET 2, 5
CMP 1, 2
BEQ beq_ok
PANIC "BEQ failed"
beq_ok:

# Test BNE pattern (not equal): if equal, skip to fail
SET 1, 5
SET 2, 10
CMP 1, 2
BEQ bne_fail
JMP bne_ok
bne_fail:
PANIC "BNE pattern failed"
bne_ok:

# BLT test
SET 1, 3
SET 2, 10
CMP 1, 2
BLT blt_ok
PANIC "BLT failed"
blt_ok:

# BGT test
SET 1, 20
SET 2, 5
CMP 1, 2
BGT bgt_ok
PANIC "BGT failed"
bgt_ok:

SETS 255, "Control flow tests passed"
TEXEC 0x5000, 255, 0
HALT
