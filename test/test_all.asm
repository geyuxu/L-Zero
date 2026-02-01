# L-0 Comprehensive Test Suite
# A combined test that covers all major functionality

SETS 255, ========================================
TEXEC 0x5000, 255, 0
SETS 255, L-0 Virtual Machine Test Suite v0.3
TEXEC 0x5000, 255, 0
SETS 255, ========================================
TEXEC 0x5000, 255, 0

# Initialize pass/fail counters
SET 200, 0
SET 201, 0
SET 100, 1

# === SECTION 1: Basic Operations ===
SETS 255, [Section 1] Basic Operations
TEXEC 0x5000, 255, 0

# Test SET/MOV
SET 1, 123
MOV 2, 1
SET 50, 123
CMP 2, 50
BEQ T1Pass
ADD 201, 201, 100
JMP T1End
T1Pass:
ADD 200, 200, 100
T1End:
NOP

# Test SWAP
SET 10, 100
SET 11, 200
SWAP 10, 11
SET 50, 200
CMP 10, 50
BEQ T2Pass
ADD 201, 201, 100
JMP T2End
T2Pass:
ADD 200, 200, 100
T2End:
NOP

# === SECTION 2: Arithmetic ===
SETS 255, [Section 2] Arithmetic
TEXEC 0x5000, 255, 0

# Test ADD
SET 1, 15
SET 2, 27
ADD 3, 1, 2
SET 50, 42
CMP 3, 50
BEQ T3Pass
ADD 201, 201, 100
JMP T3End
T3Pass:
ADD 200, 200, 100
T3End:
NOP

# Test SUB
SET 1, 100
SET 2, 37
SUB 3, 1, 2
SET 50, 63
CMP 3, 50
BEQ T4Pass
ADD 201, 201, 100
JMP T4End
T4Pass:
ADD 200, 200, 100
T4End:
NOP

# Test MUL
SET 1, 6
SET 2, 7
MUL 3, 1, 2
SET 50, 42
CMP 3, 50
BEQ T5Pass
ADD 201, 201, 100
JMP T5End
T5Pass:
ADD 200, 200, 100
T5End:
NOP

# Test DIV
SET 1, 84
SET 2, 2
DIV 3, 1, 2
SET 50, 42
CMP 3, 50
BEQ T6Pass
ADD 201, 201, 100
JMP T6End
T6Pass:
ADD 200, 200, 100
T6End:
NOP

# Test MOD
SET 1, 47
SET 2, 5
MOD 3, 1, 2
SET 50, 2
CMP 3, 50
BEQ T7Pass
ADD 201, 201, 100
JMP T7End
T7Pass:
ADD 200, 200, 100
T7End:
NOP

# === SECTION 3: Logic ===
SETS 255, [Section 3] Logic
TEXEC 0x5000, 255, 0

# Test AND
SET 1, 15
SET 2, 7
AND 3, 1, 2
SET 50, 7
CMP 3, 50
BEQ T8Pass
ADD 201, 201, 100
JMP T8End
T8Pass:
ADD 200, 200, 100
T8End:
NOP

# Test OR
SET 1, 8
SET 2, 3
OR 3, 1, 2
SET 50, 11
CMP 3, 50
BEQ T9Pass
ADD 201, 201, 100
JMP T9End
T9Pass:
ADD 200, 200, 100
T9End:
NOP

# Test XOR
SET 1, 15
SET 2, 6
XOR 3, 1, 2
SET 50, 9
CMP 3, 50
BEQ T10Pass
ADD 201, 201, 100
JMP T10End
T10Pass:
ADD 200, 200, 100
T10End:
NOP

# === SECTION 4: Control Flow ===
SETS 255, [Section 4] Control Flow
TEXEC 0x5000, 255, 0

# Test loop 0..5
SET 1, 0
SET 2, 5
SET 3, 0
LoopTest:
CMP 1, 2
BEQ LoopDone
ADD 3, 3, 100
ADD 1, 1, 100
JMP LoopTest
LoopDone:
SET 50, 5
CMP 3, 50
BEQ T11Pass
ADD 201, 201, 100
JMP T11End
T11Pass:
ADD 200, 200, 100
T11End:
NOP

# === SECTION 5: Memory ===
SETS 255, [Section 5] Memory
TEXEC 0x5000, 255, 0

# Test NEW/WRITE/READ
NEW 10, 4
SET 50, 42
WRITE 10, 0, 50
SET 50, 99
WRITE 10, 1, 50
READ 1, 10, 0
READ 2, 10, 1
SET 50, 42
CMP 1, 50
BEQ MemCheck1
ADD 201, 201, 100
JMP MemEnd
MemCheck1:
SET 50, 99
CMP 2, 50
BEQ T12Pass
ADD 201, 201, 100
JMP MemEnd
T12Pass:
ADD 200, 200, 100
MemEnd:
FREE 10

# Test READR/WRITER (values truncated to u8: 0-255)
NEW 10, 8
SET 20, 3
SET 50, 77
WRITER 10, 20, 50
READR 1, 10, 20
SET 50, 77
CMP 1, 50
BEQ T13Pass
ADD 201, 201, 100
JMP T13End
T13Pass:
ADD 200, 200, 100
T13End:
FREE 10

# === SECTION 6: Strings ===
SETS 255, [Section 6] Strings
TEXEC 0x5000, 255, 0

# Test ITOA/ATOI roundtrip
SET 1, 12345
ITOA 2, 1
ATOI 3, 2
CMP 3, 1
BEQ T14Pass
ADD 201, 201, 100
JMP T14End
T14Pass:
ADD 200, 200, 100
T14End:
NOP

# Test SCAT with HLEN
SETS 1, AB
SETS 2, CD
SCAT 3, 1, 2
HLEN 5, 3
SET 50, 4
CMP 5, 50
BEQ T15Pass
ADD 201, 201, 100
JMP T15End
T15Pass:
ADD 200, 200, 100
T15End:
NOP

# === SECTION 7: Tools ===
SETS 255, [Section 7] Tools
TEXEC 0x5000, 255, 0

# Test ABS (needs string input, returns string)
SETS 1, -50
TEXEC 0x5005, 1, 2
ATOI 3, 2
SET 50, 50
CMP 3, 50
BEQ T16Pass
ADD 201, 201, 100
JMP T16End
T16Pass:
ADD 200, 200, 100
T16End:
NOP

# Test TIME (returns string, need ATOI)
SETS 1, 0
TEXEC 0x5008, 1, 2
ATOI 3, 2
SET 50, 1000000000
CMP 3, 50
BGT T17Pass
ADD 201, 201, 100
JMP T17End
T17Pass:
ADD 200, 200, 100
T17End:
NOP

# === RESULTS ===
SETS 255, ========================================
TEXEC 0x5000, 255, 0
SETS 255, Test Results:
TEXEC 0x5000, 255, 0

# Print passed count
SETS 255, Passed:
TEXEC 0x5001, 255, 0
ITOA 255, 200
TEXEC 0x5000, 255, 0

# Print failed count
SETS 255, Failed:
TEXEC 0x5001, 255, 0
ITOA 255, 201
TEXEC 0x5000, 255, 0

SETS 255, ========================================
TEXEC 0x5000, 255, 0

# Final status
SET 50, 0
CMP 201, 50
BEQ AllPassed
SETS 255, [SOME TESTS FAILED]
TEXEC 0x5000, 255, 0
HALT

AllPassed:
SETS 255, [ALL TESTS PASSED]
TEXEC 0x5000, 255, 0
HALT
