# L-0 Memory Operations Test Suite
# Tests: NEW, FREE, READ, WRITE, READR, WRITER

SETS 255, === Memory Operations Test ===
TEXEC 0x5000, 255, 0

# --- Test NEW and WRITE/READ ---
# Allocate 8 bytes and write value 42 at offset 0
NEW 10, 8
SET 50, 42
WRITE 10, 0, 50
READ 1, 10, 0
SET 100, 42
CMP 1, 100
BEQ NewWriteReadPass
SETS 255, [FAIL] NEW/WRITE/READ: value not stored correctly
TEXEC 0x5000, 255, 0
JMP NewWriteReadEnd
NewWriteReadPass:
SETS 255, [PASS] NEW/WRITE/READ: stored and read 42
TEXEC 0x5000, 255, 0
NewWriteReadEnd:
NOP

# --- Test multiple offsets ---
SET 50, 10
WRITE 10, 0, 50
SET 50, 20
WRITE 10, 1, 50
SET 50, 30
WRITE 10, 2, 50

READ 1, 10, 0
READ 2, 10, 1
READ 3, 10, 2

SET 100, 10
CMP 1, 100
BEQ OffsetCheck1
SETS 255, [FAIL] Multiple offsets: offset 0 wrong
TEXEC 0x5000, 255, 0
JMP OffsetEnd
OffsetCheck1:
SET 100, 20
CMP 2, 100
BEQ OffsetCheck2
SETS 255, [FAIL] Multiple offsets: offset 1 wrong
TEXEC 0x5000, 255, 0
JMP OffsetEnd
OffsetCheck2:
SET 100, 30
CMP 3, 100
BEQ OffsetPass
SETS 255, [FAIL] Multiple offsets: offset 2 wrong
TEXEC 0x5000, 255, 0
JMP OffsetEnd
OffsetPass:
SETS 255, [PASS] Multiple offsets: [10,20,30] stored correctly
TEXEC 0x5000, 255, 0
OffsetEnd:
NOP

# --- Test READR (dynamic offset read) ---
# Note: WRITE/READ store u8 values (0-255)
NEW 11, 4
SET 50, 10
WRITE 11, 0, 50
SET 50, 20
WRITE 11, 1, 50
SET 50, 30
WRITE 11, 2, 50
SET 50, 40
WRITE 11, 3, 50

# Read at offset stored in r20
SET 20, 2
READR 1, 11, 20
SET 100, 30
CMP 1, 100
BEQ ReadrPass
SETS 255, [FAIL] READR: dynamic offset read failed
TEXEC 0x5000, 255, 0
JMP ReadrEnd
ReadrPass:
SETS 255, [PASS] READR: read value 30 at dynamic offset 2
TEXEC 0x5000, 255, 0
ReadrEnd:
NOP

# --- Test WRITER (dynamic offset write) ---
SET 20, 1
SET 50, 99
WRITER 11, 20, 50
READ 1, 11, 1
SET 100, 99
CMP 1, 100
BEQ WriterPass
SETS 255, [FAIL] WRITER: dynamic offset write failed
TEXEC 0x5000, 255, 0
JMP WriterEnd
WriterPass:
SETS 255, [PASS] WRITER: wrote 99 at dynamic offset 1
TEXEC 0x5000, 255, 0
WriterEnd:
NOP

# --- Test FREE ---
FREE 10
FREE 11
SETS 255, [PASS] FREE: memory freed (no crash)
TEXEC 0x5000, 255, 0

SETS 255, === Memory Tests Complete ===
TEXEC 0x5000, 255, 0
HALT
