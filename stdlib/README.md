# L-0 Standard Library (Prompt Resource Pack)

This directory contains **inline code patterns** for AI agents generating L-0 assembly.

## Quick Start

```
stdlib/
├── abi.md           # ABI spec (register convention, calling rules)
├── string.asm.txt   # String operations (strlen, strcat, substr, etc.)
├── math.asm.txt     # Math operations (abs, min, max, gcd, fibonacci, etc.)
├── array.asm.txt    # Array operations (new, get, set, sort, etc.)
└── README.md        # This file
```

## Key Concept: Inline Patterns (No Function Calls)

**L-0 has NO indirect jumps or CALL/RET instructions.**

All code reuse is done by **copying and pasting** pattern bodies:

```asm
# WRONG: L-0 cannot do this
SET 250, return_addr    # No indirect jumps!
JMP my_function

# CORRECT: Copy the pattern inline
SET 0, 10               # Input
# ---- BEGIN fibonacci_1 ----
    SET 4, 0
    CMP 0, 4
    BEQ fibonacci_1_zero
    ...
fibonacci_1_done:
# ---- END fibonacci_1 ----
# R0 now contains result
```

## AI Agent Workflow

1. **Read user request** → Identify needed operations
2. **Read stdlib files** → Copy relevant pattern bodies
3. **Rename labels** → Add unique suffix (`_1`, `_2`, etc.)
4. **Wire inputs** → Set R0, R1, R2, R3 before pattern
5. **Use outputs** → R0 contains result after `_done` label
6. **Generate .asm** → Self-contained, no external dependencies

## Example: Using fibonacci pattern

```asm
# Compute F(10)
SET 0, 10              # Input in R0

# ---- BEGIN fibonacci_1 (copied from stdlib/math.asm.txt) ----
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
# ---- END fibonacci_1 ----

# R0 = 55 (Fibonacci of 10)
ITOA 1, 0
TEXEC 0x5000, 1, 255    # Print "55"
HALT
```

## Register Convention

See [abi.md](abi.md) for full details.

| Registers | Purpose |
|-----------|---------|
| R0-R3 | Arguments / Return value |
| R4-R9 | Pattern temporaries |
| R10-R19 | Preserved across patterns |
| R50-R99 | Scratch (always clobbered) |
| R255 | I/O buffer |

## Pattern Documentation Format

Each pattern in the `.asm.txt` files follows this format:

```asm
# ==================================================================
# pattern_name: Brief description
# ==================================================================
# Input:  R0 = description, R1 = description
# Output: R0 = description
# Clobbers: R4, R5, R6 (list all modified registers)
# ------------------------------------------------------------------
pattern_name:
    # Implementation
pattern_name_done:
    # Falls through (no return instruction)
```

## Label Uniqueness Rule

When using a pattern multiple times, rename ALL internal labels:

```asm
# First usage
fibonacci_1_loop:
fibonacci_1_zero:
fibonacci_1_done:

# Second usage
fibonacci_2_loop:
fibonacci_2_zero:
fibonacci_2_done:
```

## Error Convention

| Value | Meaning |
|-------|---------|
| R0 = -1 | Error / not found |
| R0 = 0 | Success / false |
| R0 > 0 | Success with value / true |

---

*L-0 Standard Library v1.0 | For AI Agent code generation*
