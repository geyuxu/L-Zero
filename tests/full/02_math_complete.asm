# Full test: Complete math operations
# Verifies: ADD, SUB, MUL, DIV, MOD, ATOI, ITOA

# Basic arithmetic
SET 1, 100
SET 2, 25
ADD 10, 1, 2    # 125
SUB 11, 1, 2    # 75
MUL 12, 1, 2    # 2500
DIV 13, 1, 2    # 4
MOD 14, 1, 2    # 0

# Verify ADD
SET 20, 125
CMP 10, 20
BEQ add_ok
PANIC "ADD failed: 100+25 != 125"
add_ok:

# Verify SUB
SET 20, 75
CMP 11, 20
BEQ sub_ok
PANIC "SUB failed: 100-25 != 75"
sub_ok:

# Verify MUL
SET 20, 2500
CMP 12, 20
BEQ mul_ok
PANIC "MUL failed: 100*25 != 2500"
mul_ok:

# Verify DIV
SET 20, 4
CMP 13, 20
BEQ div_ok
PANIC "DIV failed: 100/25 != 4"
div_ok:

# Verify MOD
SET 20, 0
CMP 14, 20
BEQ mod_ok
PANIC "MOD failed: 100%25 != 0"
mod_ok:

# ATOI test: "42" -> 42
SETS 40, "42"
ATOI 41, 40
SET 42, 42
CMP 41, 42
BEQ atoi_ok
PANIC "ATOI failed"
atoi_ok:

# ITOA test: 123 -> "123"
SET 50, 123
ITOA 51, 50
SETS 52, "123"
SCMP 51, 52
BEQ itoa_ok
PANIC "ITOA failed"
itoa_ok:

SETS 255, "Math complete tests passed"
TEXEC 0x5000, 255, 0
HALT
