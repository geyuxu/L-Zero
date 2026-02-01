# L-0 Basic Instructions Test Suite
# Tests: SET, SETS, MOV, SWAP, ITOA, ATOI, NOP, DUMP

SETS 255, === Basic Instructions Test ===
TEXEC 0x5000, 255, 0

# --- Test SET and MOV ---
SET 1, 42
MOV 2, 1
SET 100, 42
CMP 2, 100
BEQ SetMovPass
SETS 255, [FAIL] SET/MOV: value not copied correctly
TEXEC 0x5000, 255, 0
JMP SetMovEnd
SetMovPass:
SETS 255, [PASS] SET/MOV: r2 = copy of r1 = 42
TEXEC 0x5000, 255, 0
SetMovEnd:
NOP

# --- Test SWAP ---
SET 10, 100
SET 11, 200
SWAP 10, 11
SET 100, 200
CMP 10, 100
BEQ SwapCheck2
SETS 255, [FAIL] SWAP: r10 should be 200
TEXEC 0x5000, 255, 0
JMP SwapEnd
SwapCheck2:
SET 100, 100
CMP 11, 100
BEQ SwapPass
SETS 255, [FAIL] SWAP: r11 should be 100
TEXEC 0x5000, 255, 0
JMP SwapEnd
SwapPass:
SETS 255, [PASS] SWAP: exchanged r10=200, r11=100
TEXEC 0x5000, 255, 0
SwapEnd:
NOP

# --- Test SETS (string) ---
SETS 1, Hello
HLEN 3, 1
SET 100, 5
CMP 3, 100
BEQ SetsPass
SETS 255, [FAIL] SETS: string length should be 5
TEXEC 0x5000, 255, 0
JMP SetsEnd
SetsPass:
SETS 255, [PASS] SETS: string Hello has length 5
TEXEC 0x5000, 255, 0
SetsEnd:
NOP

# --- Test ITOA (integer to string) ---
SET 1, 12345
ITOA 2, 1
HLEN 4, 2
SET 100, 5
CMP 4, 100
BEQ ItoaPass
SETS 255, [FAIL] ITOA: 12345 should produce 5-char string
TEXEC 0x5000, 255, 0
JMP ItoaEnd
ItoaPass:
SETS 255, [PASS] ITOA: 12345 -> 5-char string
TEXEC 0x5000, 255, 0
ItoaEnd:
NOP

# --- Test ATOI (string to integer) ---
SETS 1, 9876
ATOI 2, 1
SET 100, 9876
CMP 2, 100
BEQ AtoiPass
SETS 255, [FAIL] ATOI: should parse 9876
TEXEC 0x5000, 255, 0
JMP AtoiEnd
AtoiPass:
SETS 255, [PASS] ATOI: parsed 9876 correctly
TEXEC 0x5000, 255, 0
AtoiEnd:
NOP

# --- Test ITOA/ATOI roundtrip with negative ---
SET 1, -999
ITOA 2, 1
ATOI 3, 2
SET 100, -999
CMP 3, 100
BEQ NegRoundtripPass
SETS 255, [FAIL] ITOA/ATOI negative roundtrip failed
TEXEC 0x5000, 255, 0
JMP NegRoundtripEnd
NegRoundtripPass:
SETS 255, [PASS] ITOA/ATOI: -999 roundtrip works
TEXEC 0x5000, 255, 0
NegRoundtripEnd:
NOP

SETS 255, === Basic Tests Complete ===
TEXEC 0x5000, 255, 0
HALT
