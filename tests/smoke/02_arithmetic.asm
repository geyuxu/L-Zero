# Smoke test: Basic arithmetic
# Verifies: SET, ADD, SUB, MUL, DIV, MOD, CMP, BEQ, PANIC

# ADD test: 10 + 5 = 15
SET 1, 10
SET 2, 5
ADD 0, 1, 2
SET 3, 15
CMP 0, 3
BEQ add_ok
PANIC "ADD failed"
add_ok:

# SUB test: 10 - 3 = 7
SET 1, 10
SET 2, 3
SUB 0, 1, 2
SET 3, 7
CMP 0, 3
BEQ sub_ok
PANIC "SUB failed"
sub_ok:

# MUL test: 6 * 7 = 42
SET 1, 6
SET 2, 7
MUL 0, 1, 2
SET 3, 42
CMP 0, 3
BEQ mul_ok
PANIC "MUL failed"
mul_ok:

# DIV test: 20 / 4 = 5
SET 1, 20
SET 2, 4
DIV 0, 1, 2
SET 3, 5
CMP 0, 3
BEQ div_ok
PANIC "DIV failed"
div_ok:

# MOD test: 17 % 5 = 2
SET 1, 17
SET 2, 5
MOD 0, 1, 2
SET 3, 2
CMP 0, 3
BEQ mod_ok
PANIC "MOD failed"
mod_ok:

SETS 255, "Arithmetic tests passed"
TEXEC 0x5000, 255, 0
HALT
