# Test SCMP instruction
SETS 1, "hello"
SETS 2, "hello"
SETS 3, "world"
SCMP 1, 2
BEQ equal_ok
PANIC "SCMP equal failed"
equal_ok:
SCMP 1, 3
BEQ should_not_equal
JMP not_equal_ok
should_not_equal:
PANIC "SCMP not-equal failed"
not_equal_ok:
SETS 255, "SCMP tests passed"
TEXEC 0x5000, 255, 0
HALT
