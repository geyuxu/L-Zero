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

### Label Uniqueness Rule

When copying a pattern multiple times, append a unique suffix:

```asm
# First usage of fibonacci
fib_1_loop:
fib_1_zero:
fib_1_done:

# Second usage of fibonacci
fib_2_loop:
fib_2_zero:
fib_2_done:
```

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
3. **Rename labels** → Add unique suffix (_1, _2, etc.)
4. **Wire together** → Set inputs, inline pattern, use outputs
5. **Generate .asm** → Self-contained, no external dependencies

---

*L-0 ABI v1.0 | For AI Agent code generation*
