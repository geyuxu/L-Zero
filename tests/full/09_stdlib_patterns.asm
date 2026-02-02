# Full test: Standard library patterns
# Verifies: StringBuilder pattern, Array pattern using NEWR, STORE64/LOAD64

# ===== StringBuilder Pattern =====
# sb_init: allocate initial buffer
SET 1, 256
NEWR 100, 1               # sb.buf
SET 101, 0                # sb.len (byte offset)

# sb_append: append "Hello" using READR/WRITER
SETS 10, "Hello"
HLEN 11, 10
SET 12, 0
sb_append1:
    CMP 12, 11
    BLT sb_cont1
    JMP sb_append1_done
sb_cont1:
    READR 13, 10, 12      # read byte from source
    WRITER 100, 101, 13   # write byte to buffer
    SET 14, 1
    ADD 101, 101, 14
    ADD 12, 12, 14
    JMP sb_append1
sb_append1_done:

# Verify length is 5 ("Hello")
SET 40, 5
CMP 101, 40
BEQ sb_len_ok
PANIC "StringBuilder length mismatch"
sb_len_ok:

# ===== Array Pattern =====
# array_new: create array for 10 elements (8 bytes each for i64)
SET 50, 10
SET 51, 8
MUL 52, 50, 51            # 10 * 8 = 80 bytes
NEWR 200, 52              # array buffer

# array_set: fill array with squares using STORE64
SET 60, 0                 # i
SET 61, 10                # limit
array_fill:
    CMP 60, 61
    BLT array_cont
    JMP array_fill_done
array_cont:
    MUL 62, 60, 60        # i*i
    STORE64 200, 60, 62   # array[i] = i*i
    SET 63, 1
    ADD 60, 60, 63
    JMP array_fill
array_fill_done:

# array_get: verify array[5] = 25 using LOAD64
SET 70, 5
LOAD64 71, 200, 70
SET 72, 25
CMP 71, 72
BEQ array_ok
PANIC "Array pattern failed"
array_ok:

SETS 255, "Stdlib patterns tests passed"
TEXEC 0x5000, 255, 0
HALT
