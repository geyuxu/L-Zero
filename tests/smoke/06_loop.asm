# Smoke test: Loop (sum 1 to 10)
# Verifies: Loop constructs with CMP/BLT/JMP

SET 1, 0          # sum = 0
SET 2, 1          # i = 1
SET 3, 11         # limit = 11

loop:
    CMP 2, 3
    BLT continue
    JMP done
continue:
    ADD 1, 1, 2   # sum += i
    SET 4, 1
    ADD 2, 2, 4   # i++
    JMP loop

done:
    SET 5, 55     # expected sum = 55
    CMP 1, 5
    BEQ sum_ok
    PANIC "Loop sum incorrect"
sum_ok:

SETS 255, "Loop test passed"
TEXEC 0x5000, 255, 0
HALT
