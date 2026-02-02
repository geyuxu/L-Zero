# Test NEWR instruction
SET 1, 64
NEWR 0, 1              # Allocate 64 bytes from register
HLEN 2, 0              # Should be 64
SET 3, 64
CMP 2, 3
BEQ size_ok
PANIC "NEWR size mismatch"
size_ok:
SETS 255, "NEWR tests passed"
TEXEC 0x5000, 255, 0
HALT
