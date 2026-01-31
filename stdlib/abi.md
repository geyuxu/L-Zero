# L-0 Application Binary Interface (ABI) v1.0

## Register Convention

L-0 has 256 registers (R0-R255). This ABI defines their usage:

```
┌─────────────┬──────────────────────────────────────────────┐
│ Registers   │ Purpose                                      │
├─────────────┼──────────────────────────────────────────────┤
│ R0-R3       │ Arguments / Return value                     │
│ R4-R9       │ Temporaries (pattern-local)                  │
│ R10-R19     │ Preserved across patterns                    │
│ R20-R49     │ Local variables (main program workspace)     │
│ R50-R99     │ Scratch (pattern-local)                      │
│ R100-R109   │ Constants (R100=1, R101=8, etc.)             │
│ R200-R239   │ Reserved for future use                      │
│ R240-R243   │ Governance (target, state, threshold, sim)   │
│ R255        │ I/O buffer (for TEXEC)                       │
└─────────────┴──────────────────────────────────────────────┘
```

## Code Reuse Pattern

**IMPORTANT**: L-0 has NO indirect jumps or CALL/RET instructions.
All code is inlined. AI Agents must copy patterns and rename labels.

### Pattern: Inline Expansion

```asm
# Step 1: Set up inputs in registers
SET 0, 10              # R0 = input value

# Step 2: Inline the pattern body (copy from stdlib)
# ---- BEGIN fibonacci pattern ----
SET 4, 0
CMP 0, 4
BEQ fib_1_zero         # Use unique labels!
# ... rest of pattern ...
fib_1_done:
# ---- END fibonacci pattern ----

# Step 3: Use result in R0
ITOA 255, 0
TEXEC 0x5000, 255, 0   # Print result
```

### Caller-Save Responsibility

**CRITICAL**: Before pasting a pattern, move any live data from R0-R9 to R10-R49.

Patterns WILL overwrite R0-R9 without warning. If you have important data in these registers, save it first:

```asm
# Save important data before calling fibonacci
MOV 20, 5              # Save R5 (important) -> R20 (local)
MOV 21, 6              # Save R6 (important) -> R21 (local)

# Now safe to inline pattern
SET 0, 10              # Input for fibonacci
# ---- BEGIN fibonacci ----
# ... pattern will clobber R4-R9 ...
# ---- END fibonacci ----

# Restore if needed
MOV 5, 20
MOV 6, 21
```

### Label Hygiene (Collision Prevention)

When copying patterns, use **hash/UUID suffixes** instead of simple numbers.
This prevents collisions in long programs with thousands of lines:

```asm
# BAD - collision risk in long programs
fib_1_loop:
fib_2_loop:

# GOOD - unique hash suffix
fib_a7b9_loop:
fib_c3d2_loop:

# BEST - include context in suffix
calc_total_fib_loop:
print_result_fib_loop:
```

Recommended suffix formats:
- `pattern_<4-char-hash>_label` (e.g., `fib_a7b9_done`)
- `<context>_pattern_label` (e.g., `main_fib_done`)

### Argument Convention

| Register | Purpose                       |
|----------|-------------------------------|
| R0       | Primary input / output        |
| R1       | Secondary input               |
| R2       | Tertiary input                |
| R3       | Quaternary input              |
| R4-R9    | Pattern temporaries           |
| R50-R99  | Pattern scratch space         |

### Pattern Isolation

Patterns should only modify:
- R0 (output)
- R4-R9 (temporaries declared in pattern header)
- R50-R99 (scratch, always considered clobbered)

Patterns must NOT modify:
- R10-R19 (preserved registers)
- R20-R49 (caller's local variables)
- R100-R109 (constants)

## Standard Pattern Format

Each pattern in stdlib files follows this format:

```asm
# ==================================================================
# pattern_name: Brief description
# ==================================================================
# Input:  R0 = description, R1 = description
# Output: R0 = description
# Clobbers: R4, R5, R6 (list all modified temporaries)
# ------------------------------------------------------------------
pattern_name:
    # Implementation
pattern_name_done:
    # Pattern ends here (no return - just falls through)
```

## Memory Layout

```
Heap Pointer (i64):
  0        = NULL (invalid)
  1+       = Valid allocation ID

String: Raw bytes, length via HLEN instruction
Array:  i64 elements via STORE64/LOAD64 (8 bytes each)
```

## Error Convention

| Value     | Meaning                   |
|-----------|---------------------------|
| R0 = -1   | Error / not found         |
| R0 = 0    | Success / false           |
| R0 > 0    | Success with value / true |

### Standard Error Check Pattern (CHECK_ERR)

After any pattern that may fail, use this standard check:

```asm
# ---- CHECK_ERR pattern ----
# Input: R0 = result from previous operation
# Jumps to error handler if R0 < 0
SET 50, 0                      # R50 = 0 (zero constant)
CMP 0, 50                      # Compare R0 with 0
BLT handle_error_<suffix>      # If R0 < 0, jump to error
# ---- END CHECK_ERR ----
```

Example usage:

```asm
# Try to find element
SET 0, 5                       # needle
# ---- inline array_find ----
# ...
array_find_abc1_done:
# ---- end array_find ----

# Check for error
SET 50, 0
CMP 0, 50
BLT not_found_abc1             # R0 < 0 means not found

# Success path
ITOA 255, 0
TEXEC 0x5000, 255, 0           # Print index
JMP done_abc1

not_found_abc1:
SETS 255, "Element not found"
TEXEC 0x5000, 255, 0

done_abc1:
HALT
```

## Tool IDs (TEXEC)

| ID       | Name    | Description            |
|----------|---------|------------------------|
| 0x5000   | PRINT   | Print with newline     |
| 0x5001   | PRINTN  | Print without newline  |
| 0x5002   | INPUT   | Read line from stdin   |
| 0x5003   | RAND    | Random number          |
| 0x5005   | ABS     | Absolute value         |
| 0x5006   | MIN     | Minimum of two         |
| 0x5007   | MAX     | Maximum of two         |
| 0x5008   | TIME    | Unix timestamp         |

## AI Agent Workflow

1. **Parse user request** → Identify needed patterns
2. **Read stdlib files** → Copy relevant patterns
3. **Save live registers** → Move R0-R9 data to R10-R49 if needed
4. **Rename labels** → Use hash suffix (e.g., `_a7b9`) to prevent collisions
5. **Wire inputs** → Set R0-R3 with pattern arguments
6. **Inline pattern** → Copy pattern body with renamed labels
7. **Check errors** → Use CHECK_ERR pattern if operation may fail
8. **Generate .asm** → Self-contained, no external dependencies

### Quick Reference Card

```
Before Pattern:  MOV 20, 5         # Save R5 → R20 if needed
                 SET 0, <input>    # Set input in R0

Inline Pattern:  # ---- BEGIN pattern_<hash> ----
                 # (copy and rename all labels)
                 # ---- END pattern ----

After Pattern:   SET 50, 0         # CHECK_ERR
                 CMP 0, 50
                 BLT error_<hash>
```

---

*L-0 ABI v1.0 | For AI Agent code generation*
