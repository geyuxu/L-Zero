# L-0 Control Flow Test Suite
# Tests: CMP, JMP, BEQ, BGT, BLT

SETS 255, === Control Flow Test ===
TEXEC 0x5000, 255, 0

# --- Test JMP (unconditional) ---
JMP JmpTarget
SETS 255, [FAIL] JMP: should not execute this
TEXEC 0x5000, 255, 0
JmpTarget:
SETS 255, [PASS] JMP: jumped to target
TEXEC 0x5000, 255, 0

# --- Test BEQ (branch if equal) ---
SET 1, 50
SET 2, 50
CMP 1, 2
BEQ BeqPass
SETS 255, [FAIL] BEQ: 50 == 50 should branch
TEXEC 0x5000, 255, 0
JMP BeqEnd
BeqPass:
SETS 255, [PASS] BEQ: 50 == 50 branched correctly
TEXEC 0x5000, 255, 0
BeqEnd:
NOP

# --- Test BEQ (should not branch) ---
SET 1, 50
SET 2, 60
CMP 1, 2
BEQ BeqNotPass
SETS 255, [PASS] BEQ: 50 != 60, correctly did not branch
TEXEC 0x5000, 255, 0
JMP BeqNotEnd
BeqNotPass:
SETS 255, [FAIL] BEQ: 50 != 60 should not branch
TEXEC 0x5000, 255, 0
BeqNotEnd:
NOP

# --- Test BGT (branch if greater) ---
SET 1, 100
SET 2, 50
CMP 1, 2
BGT BgtPass
SETS 255, [FAIL] BGT: 100 > 50 should branch
TEXEC 0x5000, 255, 0
JMP BgtEnd
BgtPass:
SETS 255, [PASS] BGT: 100 > 50 branched correctly
TEXEC 0x5000, 255, 0
BgtEnd:
NOP

# --- Test BGT (should not branch when equal) ---
SET 1, 50
SET 2, 50
CMP 1, 2
BGT BgtEqFail
SETS 255, [PASS] BGT: 50 == 50, correctly did not branch
TEXEC 0x5000, 255, 0
JMP BgtEqEnd
BgtEqFail:
SETS 255, [FAIL] BGT: equal values should not branch
TEXEC 0x5000, 255, 0
BgtEqEnd:
NOP

# --- Test BGT (should not branch when less) ---
SET 1, 30
SET 2, 50
CMP 1, 2
BGT BgtLessFail
SETS 255, [PASS] BGT: 30 < 50, correctly did not branch
TEXEC 0x5000, 255, 0
JMP BgtLessEnd
BgtLessFail:
SETS 255, [FAIL] BGT: 30 < 50 should not branch
TEXEC 0x5000, 255, 0
BgtLessEnd:
NOP

# --- Test BLT (branch if less) ---
SET 1, 30
SET 2, 50
CMP 1, 2
BLT BltPass
SETS 255, [FAIL] BLT: 30 < 50 should branch
TEXEC 0x5000, 255, 0
JMP BltEnd
BltPass:
SETS 255, [PASS] BLT: 30 < 50 branched correctly
TEXEC 0x5000, 255, 0
BltEnd:
NOP

# --- Test BLT (should not branch when equal) ---
SET 1, 50
SET 2, 50
CMP 1, 2
BLT BltEqFail
SETS 255, [PASS] BLT: 50 == 50, correctly did not branch
TEXEC 0x5000, 255, 0
JMP BltEqEnd
BltEqFail:
SETS 255, [FAIL] BLT: equal values should not branch
TEXEC 0x5000, 255, 0
BltEqEnd:
NOP

# --- Test BLT (should not branch when greater) ---
SET 1, 100
SET 2, 50
CMP 1, 2
BLT BltGreaterFail
SETS 255, [PASS] BLT: 100 > 50, correctly did not branch
TEXEC 0x5000, 255, 0
JMP BltGreaterEnd
BltGreaterFail:
SETS 255, [FAIL] BLT: 100 > 50 should not branch
TEXEC 0x5000, 255, 0
BltGreaterEnd:
NOP

# --- Test loop with counter ---
SET 1, 0
SET 2, 5
SET 100, 1
LoopStart:
CMP 1, 2
BEQ LoopDone
ADD 1, 1, 100
JMP LoopStart
LoopDone:
SET 100, 5
CMP 1, 100
BEQ LoopPass
SETS 255, [FAIL] Loop: counter should be 5
TEXEC 0x5000, 255, 0
JMP LoopEnd
LoopPass:
SETS 255, [PASS] Loop: counted to 5 correctly
TEXEC 0x5000, 255, 0
LoopEnd:
NOP

SETS 255, === Control Flow Tests Complete ===
TEXEC 0x5000, 255, 0
HALT
