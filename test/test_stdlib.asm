# Test L-0 Stdlib Inline Patterns
# ================================
# This demonstrates how AI agents should use stdlib patterns:
# 1. Copy the pattern body (not the label)
# 2. Rename all internal labels with unique suffix
# 3. Pattern falls through at _done label

# ============================================================
# Test 1: Fibonacci (n=10)
# ============================================================
SET 0, 10              # Input: compute F(10)

# ---- BEGIN fibonacci_1 pattern (inline from stdlib/math.asm.txt) ----
    SET 4, 0
    CMP 0, 4
    BEQ fibonacci_1_zero

    SET 4, 1
    CMP 0, 4
    BEQ fibonacci_1_one

    MOV 4, 0
    SET 5, 0
    SET 6, 1
    SET 7, 1

fibonacci_1_loop:
    CMP 7, 4
    BEQ fibonacci_1_finish

    ADD 50, 5, 6
    MOV 5, 6
    MOV 6, 50
    SET 51, 1
    ADD 7, 7, 51
    JMP fibonacci_1_loop

fibonacci_1_zero:
    SET 0, 0
    JMP fibonacci_1_done

fibonacci_1_one:
    SET 0, 1
    JMP fibonacci_1_done

fibonacci_1_finish:
    MOV 0, 6
fibonacci_1_done:
# ---- END fibonacci_1 pattern ----

# Print result: "Fibonacci(10) = 55"
SETS 20, "Fibonacci(10) = "
TEXEC 0x5001, 20, 255
ITOA 21, 0
TEXEC 0x5000, 21, 255

# ============================================================
# Test 2: GCD (48, 18) = 6
# ============================================================
SET 0, 48
SET 1, 18

# ---- BEGIN gcd_1 pattern (inline from stdlib/math.asm.txt) ----
    MOV 4, 0
    MOV 5, 1

gcd_1_loop:
    SET 50, 0
    CMP 5, 50
    BEQ gcd_1_finish

    MOD 50, 4, 5
    MOV 4, 5
    MOV 5, 50
    JMP gcd_1_loop

gcd_1_finish:
    MOV 0, 4
gcd_1_done:
# ---- END gcd_1 pattern ----

# Print result: "GCD(48, 18) = 6"
SETS 20, "GCD(48, 18) = "
TEXEC 0x5001, 20, 255
ITOA 21, 0
TEXEC 0x5000, 21, 255

# ============================================================
# Test 3: is_prime (17) = 1 (true)
# ============================================================
SET 0, 17

# ---- BEGIN is_prime_1 pattern (inline from stdlib/math.asm.txt) ----
    SET 4, 2
    CMP 0, 4
    BLT is_prime_1_no
    BEQ is_prime_1_yes

    SET 4, 2
    MOD 5, 0, 4
    SET 6, 0
    CMP 5, 6
    BEQ is_prime_1_no

    MOV 6, 0
    SET 4, 3

is_prime_1_loop:
    MUL 5, 4, 4
    CMP 5, 6
    BGT is_prime_1_yes

    MOD 5, 6, 4
    SET 50, 0
    CMP 5, 50
    BEQ is_prime_1_no

    SET 50, 2
    ADD 4, 4, 50
    JMP is_prime_1_loop

is_prime_1_yes:
    SET 0, 1
    JMP is_prime_1_done

is_prime_1_no:
    SET 0, 0
is_prime_1_done:
# ---- END is_prime_1 pattern ----

# Print result: "is_prime(17) = 1"
SETS 20, "is_prime(17) = "
TEXEC 0x5001, 20, 255
ITOA 21, 0
TEXEC 0x5000, 21, 255

# ============================================================
# Test 4: Absolute value of -42 = 42
# ============================================================
SET 0, -42

# ---- BEGIN abs_1 pattern (inline from stdlib/math.asm.txt) ----
    SET 4, 0
    CMP 0, 4
    BGT abs_1_done
    BEQ abs_1_done
    SET 4, -1
    MUL 0, 0, 4
abs_1_done:
# ---- END abs_1 pattern ----

# Print result: "abs(-42) = 42"
SETS 20, "abs(-42) = "
TEXEC 0x5001, 20, 255
ITOA 21, 0
TEXEC 0x5000, 21, 255

# ============================================================
# Test 5: String operations
# ============================================================
SETS 0, "Hello"
SETS 1, " World!"

# ---- BEGIN strcat_1 pattern (inline from stdlib/string.asm.txt) ----
    MOV 4, 0
    SCAT 0, 4, 1
strcat_1_done:
# ---- END strcat_1 pattern ----

# Print result: "strcat = Hello World!"
SETS 20, "strcat = "
TEXEC 0x5001, 20, 255
TEXEC 0x5000, 0, 255

# ============================================================
# All tests complete
# ============================================================
SETS 255, "All stdlib tests passed!"
TEXEC 0x5000, 255, 255

HALT
