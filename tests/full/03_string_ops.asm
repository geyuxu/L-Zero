# Full test: String operations
# Verifies: SETS, SCAT, SLICE, TEXEC UPPER/LOWER/TRIM, HLEN, SCMP

# SCAT test (string concat)
SETS 1, "Hello"
SETS 2, "World"
SCAT 4, 1, 2
SETS 6, "HelloWorld"
SCMP 4, 6
BEQ scat_ok
PANIC "SCAT failed"
scat_ok:

# SLICE test: extract "World" from "HelloWorld"
SETS 10, "HelloWorld"
SET 11, 5          # offset register
SET 12, 5          # length register
SLICE 13, 10, 11, 12
SETS 14, "World"
SCMP 13, 14
BEQ slice_ok
PANIC "SLICE failed"
slice_ok:

# UPPER test (via TEXEC 0x500C)
SETS 20, "hello"
TEXEC 0x500C, 20, 21
SETS 22, "HELLO"
SCMP 21, 22
BEQ upper_ok
PANIC "UPPER failed"
upper_ok:

# LOWER test (via TEXEC 0x500D)
SETS 30, "HELLO"
TEXEC 0x500D, 30, 31
SETS 32, "hello"
SCMP 31, 32
BEQ lower_ok
PANIC "LOWER failed"
lower_ok:

# TRIM test (via TEXEC 0x500E)
SETS 40, "  trimmed  "
TEXEC 0x500E, 40, 41
SETS 42, "trimmed"
SCMP 41, 42
BEQ trim_ok
PANIC "TRIM failed"
trim_ok:

# HLEN on strings
SETS 50, "test"
HLEN 51, 50
SET 52, 4
CMP 51, 52
BEQ hlen_ok
PANIC "HLEN string failed"
hlen_ok:

SETS 255, "String operations tests passed"
TEXEC 0x5000, 255, 0
HALT
