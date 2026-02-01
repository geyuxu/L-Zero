# L-0 Tools Test Suite (ASM)
# Tests: PRINT, PRINTN, ABS, RAND, TIME
# Note: POW/MIN/MAX require commas in strings - see test_tools_advanced.json

SETS 255, === Tools Test Suite (ASM) ===
TEXEC 0x5000, 255, 0

# --- Test PRINT (0x5000) ---
SETS 1, Testing PRINT...
TEXEC 0x5000, 1, 0
SETS 255, [PASS] PRINT: message displayed above
TEXEC 0x5000, 255, 0

# --- Test PRINTN (0x5001) - no newline ---
SETS 1, Hello
SETS 2, World
TEXEC 0x5001, 1, 0
SETS 3,
TEXEC 0x5001, 3, 0
TEXEC 0x5000, 2, 0
SETS 255, [PASS] PRINTN: HelloWorld on one line
TEXEC 0x5000, 255, 0

# --- Test ABS (0x5005) ---
SETS 1, -42
TEXEC 0x5005, 1, 2
ATOI 3, 2
SET 100, 42
CMP 3, 100
BEQ AbsNegPass
SETS 255, [FAIL] ABS: |-42| should be 42
TEXEC 0x5000, 255, 0
JMP AbsNegEnd
AbsNegPass:
SETS 255, [PASS] ABS: |-42| = 42
TEXEC 0x5000, 255, 0
AbsNegEnd:
NOP

# Test ABS with positive
SETS 1, 99
TEXEC 0x5005, 1, 2
ATOI 3, 2
SET 100, 99
CMP 3, 100
BEQ AbsPosPass
SETS 255, [FAIL] ABS: |99| should be 99
TEXEC 0x5000, 255, 0
JMP AbsPosEnd
AbsPosPass:
SETS 255, [PASS] ABS: |99| = 99
TEXEC 0x5000, 255, 0
AbsPosEnd:
NOP

# --- Test RAND (0x5003) ---
SETS 1, 1000
TEXEC 0x5003, 1, 2
ATOI 3, 2
SET 100, 1000
CMP 3, 100
BLT RandPass
SETS 255, [FAIL] RAND: random(1000) should be < 1000
TEXEC 0x5000, 255, 0
JMP RandEnd
RandPass:
SETS 255, [PASS] RAND: random(1000) < 1000
TEXEC 0x5000, 255, 0
RandEnd:
NOP

# --- Test TIME (0x5008) ---
SETS 1, 0
TEXEC 0x5008, 1, 2
ATOI 3, 2
SET 100, 1000000000
CMP 3, 100
BGT TimePass
SETS 255, [FAIL] TIME: should be > 1000000000 (epoch)
TEXEC 0x5000, 255, 0
JMP TimeEnd
TimePass:
SETS 255, [PASS] TIME: > 1000000000 (valid epoch timestamp)
TEXEC 0x5000, 255, 0
TimeEnd:
NOP

SETS 255, === Tools Tests (ASM) Complete ===
TEXEC 0x5000, 255, 0
SETS 255, Run test_tools_advanced.json for POW/MIN/MAX
TEXEC 0x5000, 255, 0
HALT
