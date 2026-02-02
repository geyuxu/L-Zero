# Full test: Governance layer
# Verifies: GAS, LATCH, GUARD, TRAP, YIELD

# GAS test: check remaining gas
GAS 1
SET 2, 0
CMP 1, 2
BGT gas_ok
PANIC "GAS should be positive"
gas_ok:

# LATCH test: mark immutable state (target, threshold)
# LATCH { target: u8, threshold: u8 } - both are register indices
SET 10, 0
SET 11, 0
LATCH 10, 11

# GUARD test: check drift against latched target
# GUARD { state: u8 } - state is register index
SET 20, 0
GUARD 20

# TRAP test: emit trap with code
# TRAP { code: u8 } - code is register index
SET 30, 0
TRAP 30

# YIELD test: query supervisor
# YIELD { query: u8, dest: u8 } - query reg, dest reg
SETS 40, "status"
YIELD 40, 41

SETS 255, "Governance tests passed"
TEXEC 0x5000, 255, 0
HALT
