# L-0 Math Operations Test Suite
# Tests: ADD, SUB, MUL, DIV, MOD, AND, OR, XOR, NOT

SETS 255, === Math Operations Test ===
TEXEC 0x5000, 255, 0

# --- Test ADD ---
SET 1, 25
SET 2, 17
ADD 3, 1, 2
SET 100, 42
CMP 3, 100
BEQ AddPass
SETS 255, [FAIL] ADD: 25 + 17 != 42
TEXEC 0x5000, 255, 0
JMP AddEnd
AddPass:
SETS 255, [PASS] ADD: 25 + 17 = 42
TEXEC 0x5000, 255, 0
AddEnd:
NOP

# --- Test SUB ---
SET 1, 100
SET 2, 37
SUB 3, 1, 2
SET 100, 63
CMP 3, 100
BEQ SubPass
SETS 255, [FAIL] SUB: 100 - 37 != 63
TEXEC 0x5000, 255, 0
JMP SubEnd
SubPass:
SETS 255, [PASS] SUB: 100 - 37 = 63
TEXEC 0x5000, 255, 0
SubEnd:
NOP

# --- Test MUL ---
SET 1, 7
SET 2, 8
MUL 3, 1, 2
SET 100, 56
CMP 3, 100
BEQ MulPass
SETS 255, [FAIL] MUL: 7 * 8 != 56
TEXEC 0x5000, 255, 0
JMP MulEnd
MulPass:
SETS 255, [PASS] MUL: 7 * 8 = 56
TEXEC 0x5000, 255, 0
MulEnd:
NOP

# --- Test DIV ---
SET 1, 100
SET 2, 7
DIV 3, 1, 2
SET 100, 14
CMP 3, 100
BEQ DivPass
SETS 255, [FAIL] DIV: 100 / 7 != 14
TEXEC 0x5000, 255, 0
JMP DivEnd
DivPass:
SETS 255, [PASS] DIV: 100 / 7 = 14
TEXEC 0x5000, 255, 0
DivEnd:
NOP

# --- Test MOD ---
SET 1, 100
SET 2, 7
MOD 3, 1, 2
SET 100, 2
CMP 3, 100
BEQ ModPass
SETS 255, [FAIL] MOD: 100 mod 7 != 2
TEXEC 0x5000, 255, 0
JMP ModEnd
ModPass:
SETS 255, [PASS] MOD: 100 mod 7 = 2
TEXEC 0x5000, 255, 0
ModEnd:
NOP

# --- Test AND (bitwise) ---
# 0b1100 (12) AND 0b1010 (10) = 0b1000 (8)
SET 1, 12
SET 2, 10
AND 3, 1, 2
SET 100, 8
CMP 3, 100
BEQ AndPass
SETS 255, [FAIL] AND: 12 AND 10 != 8
TEXEC 0x5000, 255, 0
JMP AndEnd
AndPass:
SETS 255, [PASS] AND: 12 AND 10 = 8
TEXEC 0x5000, 255, 0
AndEnd:
NOP

# --- Test OR (bitwise) ---
# 0b1100 (12) OR 0b1010 (10) = 0b1110 (14)
SET 1, 12
SET 2, 10
OR 3, 1, 2
SET 100, 14
CMP 3, 100
BEQ OrPass
SETS 255, [FAIL] OR: 12 OR 10 != 14
TEXEC 0x5000, 255, 0
JMP OrEnd
OrPass:
SETS 255, [PASS] OR: 12 OR 10 = 14
TEXEC 0x5000, 255, 0
OrEnd:
NOP

# --- Test XOR (bitwise) ---
# 0b1100 (12) XOR 0b1010 (10) = 0b0110 (6)
SET 1, 12
SET 2, 10
XOR 3, 1, 2
SET 100, 6
CMP 3, 100
BEQ XorPass
SETS 255, [FAIL] XOR: 12 XOR 10 != 6
TEXEC 0x5000, 255, 0
JMP XorEnd
XorPass:
SETS 255, [PASS] XOR: 12 XOR 10 = 6
TEXEC 0x5000, 255, 0
XorEnd:
NOP

# --- Test NOT (bitwise) ---
# NOT 0 = -1 (all bits set in two's complement)
SET 1, 0
NOT 2, 1
SET 100, -1
CMP 2, 100
BEQ NotPass
SETS 255, [FAIL] NOT: NOT 0 != -1
TEXEC 0x5000, 255, 0
JMP NotEnd
NotPass:
SETS 255, [PASS] NOT: NOT 0 = -1
TEXEC 0x5000, 255, 0
NotEnd:
NOP

SETS 255, === Math Tests Complete ===
TEXEC 0x5000, 255, 0
HALT
