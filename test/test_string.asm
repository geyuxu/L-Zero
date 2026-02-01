# L-0 String Operations Test Suite
# Tests: HLEN (string length), UPPER, LOWER, TRIM, SCAT
# Note: HLEN is a native instruction that returns length as integer

SETS 255, === String Operations Test ===
TEXEC 0x5000, 255, 0

# --- Test HLEN (string length) ---
SETS 1, Hello World
HLEN 3, 1
SET 100, 11
CMP 3, 100
BEQ StrlenPass
SETS 255, [FAIL] HLEN: 'Hello World' should be 11
TEXEC 0x5000, 255, 0
JMP StrlenEnd
StrlenPass:
SETS 255, [PASS] HLEN: 'Hello World' = 11 chars
TEXEC 0x5000, 255, 0
StrlenEnd:
NOP

# --- Test HLEN with short string ---
SETS 1, X
HLEN 3, 1
SET 100, 1
CMP 3, 100
BEQ StrlenShortPass
SETS 255, [FAIL] HLEN: 'X' should be 1
TEXEC 0x5000, 255, 0
JMP StrlenShortEnd
StrlenShortPass:
SETS 255, [PASS] HLEN: 'X' = 1 char
TEXEC 0x5000, 255, 0
StrlenShortEnd:
NOP

# --- Test UPPER ---
SETS 1, hello
TEXEC 0x500C, 1, 2
HLEN 4, 2
SET 100, 5
CMP 4, 100
BEQ UpperPass
SETS 255, [FAIL] UPPER: result length wrong
TEXEC 0x5000, 255, 0
JMP UpperEnd
UpperPass:
SETS 255, [PASS] UPPER: 'hello' -> uppercase (5 chars)
TEXEC 0x5000, 255, 0
UpperEnd:
NOP

# --- Test LOWER ---
SETS 1, WORLD
TEXEC 0x500D, 1, 2
HLEN 4, 2
SET 100, 5
CMP 4, 100
BEQ LowerPass
SETS 255, [FAIL] LOWER: result length wrong
TEXEC 0x5000, 255, 0
JMP LowerEnd
LowerPass:
SETS 255, [PASS] LOWER: 'WORLD' -> lowercase (5 chars)
TEXEC 0x5000, 255, 0
LowerEnd:
NOP

# --- Test TRIM ---
SETS 1, "  hello  "
TEXEC 0x500E, 1, 2
HLEN 4, 2
SET 100, 5
CMP 4, 100
BEQ TrimPass
SETS 255, [FAIL] TRIM: result length should be 5
TEXEC 0x5000, 255, 0
JMP TrimEnd
TrimPass:
SETS 255, [PASS] TRIM: whitespace removed correctly
TEXEC 0x5000, 255, 0
TrimEnd:
NOP

# --- Test SCAT (instruction) ---
SETS 1, Foo
SETS 2, Bar
SCAT 3, 1, 2
HLEN 5, 3
SET 100, 6
CMP 5, 100
BEQ ScatPass
SETS 255, [FAIL] SCAT: 'Foo' + 'Bar' should be 6 chars
TEXEC 0x5000, 255, 0
JMP ScatEnd
ScatPass:
SETS 255, [PASS] SCAT: 'Foo' + 'Bar' = 6 chars
TEXEC 0x5000, 255, 0
# Print the concatenated result
SETS 255, [INFO] Result:
TEXEC 0x5001, 255, 0
TEXEC 0x5000, 3, 0
ScatEnd:
NOP

SETS 255, === String Tests Complete ===
TEXEC 0x5000, 255, 0
HALT
