# Smoke test: String operations
# Verifies: SETS, SCMP, SCAT, HLEN, TEXEC UPPER/LOWER

# SCMP test: equal strings
SETS 1, "hello"
SETS 2, "hello"
SCMP 1, 2
BEQ scmp_eq_ok
PANIC "SCMP equal failed"
scmp_eq_ok:

# SCMP test: not equal (use BEQ to skip panic if equal)
SETS 3, "hello"
SETS 4, "world"
SCMP 3, 4
BEQ scmp_ne_fail
JMP scmp_ne_ok
scmp_ne_fail:
PANIC "SCMP not-equal failed"
scmp_ne_ok:

# SCAT test (string concat)
SETS 10, "Hello"
SETS 11, "World"
SCAT 12, 10, 11
SETS 13, "HelloWorld"
SCMP 12, 13
BEQ scat_ok
PANIC "SCAT failed"
scat_ok:

# HLEN on string
SETS 20, "test"
HLEN 21, 20
SET 22, 4
CMP 21, 22
BEQ hlen_ok
PANIC "HLEN on string failed"
hlen_ok:

# UPPER test (via TEXEC 0x500C)
SETS 30, "hello"
TEXEC 0x500C, 30, 31
SETS 32, "HELLO"
SCMP 31, 32
BEQ upper_ok
PANIC "UPPER failed"
upper_ok:

# LOWER test (via TEXEC 0x500D)
SETS 40, "HELLO"
TEXEC 0x500D, 40, 41
SETS 42, "hello"
SCMP 41, 42
BEQ lower_ok
PANIC "LOWER failed"
lower_ok:

SETS 255, "String tests passed"
TEXEC 0x5000, 255, 0
HALT
