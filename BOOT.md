# L-0 Bootstrap

> Code as Documentation. The VM is self-describing.

## What is L-0?

L-0 is a **custom virtual machine** with its own instruction set (ISA). Programs are written in **L-0 Assembly** (`.asm`), compiled to **bytecode** (`.l0`), and executed by `l0vm`.

**IMPORTANT FOR AI AGENTS:**
- Do NOT use Python, JavaScript, or other languages
- **Only `.l0` file should remain** - delete `.asm` after compiling

```bash
# REQUIRED workflow
./l0asm program.asm > program.l0 && rm program.asm   # Compile & cleanup
./l0vm program.l0                                      # Run & verify
```

- Use `ITOA dest, src` to convert integer to string for printing.

**Debugging Tips:**
- Use `--debug` flag to trace execution: `./l0vm program.l0 --debug`
- Use `DUMP` instruction to print all register values
- Common bugs: wrong register reuse, off-by-one in loops, forgetting to initialize

---

## Toolchain

```
┌─────────────┐    l0asm     ┌─────────────┐    l0vm      ┌─────────────┐
│  .asm       │ ──────────►  │  .l0        │ ──────────►  │  Execute    │
│  Assembly   │              │  Bytecode   │              │  Result     │
└─────────────┘              └─────────────┘              └─────────────┘
```

| Format | Purpose | Human-Readable |
|--------|---------|----------------|
| `.asm` | Source code (labels, comments) | Yes |
| `.json`| Intermediate (debugging) | Yes |
| `.l0`  | Binary bytecode (execution) | No |

---

## Introspection

Query the VM to discover ISA and Tools:

```bash
l0vm --info
```

Returns: `version`, `architecture`, `isa.primitives`, `tools`.

---

## Quick Start

```bash
# Compile & cleanup (remove .asm intermediate)
./l0asm program.asm > program.l0 && rm program.asm

# Run bytecode
./l0vm program.l0

# Debug (trace execution)
./l0vm program.l0 --debug
```

---

## Assembly Language

### Syntax

```asm
# Comment (ignored)
SET 1, 42         # Inline comment

Label:            # Define jump target
JMP Label         # Jump to label (resolved to line number)
BEQ Label         # Branch to label

SETS 255, Hello   # String without quotes
TEXEC 0x5000, 1, 0  # Hex literals supported
```

### Instruction Forms

| ASM | JSON Equivalent |
|-----|-----------------|
| `SET 1, 42` | `{ "SET": { "reg": 1, "val": 42 } }` |
| `ADD 1, 2, 3` | `{ "ADD": { "dest": 1, "s1": 2, "s2": 3 } }` |
| `HALT` | `"HALT"` |
| `PANIC Error!` | `{ "PANIC": "Error!" }` |

### Example

```asm
# Hello World
SETS 1, Hello, L-0!
TEXEC 0x5000, 1, 255
HALT
```

Assemble and run:
```bash
./l0asm hello.asm > hello.l0 && rm hello.asm
./l0vm hello.l0
```

---

## ISA Reference

### JSON Format

Programs can be written as JSON arrays (useful for debugging/generation):

| Form | Example |
|------|---------|
| Unit (no args) | `"HALT"` |
| Tuple (single arg) | `{ "PANIC": "error message" }` |
| Struct (named args) | `{ "SET": { "reg": 1, "val": 42 } }` |

Use `l0vm --info` to see all instruction signatures and ordinals.

### Registers

256 registers (0-255), each holds `i64` or heap pointer.

| Range | Convention |
|-------|------------|
| R0-R15 | General purpose |
| R255 | Tool result (TEXEC writes here) |

### Instructions

**System**
| Op | Args | Description |
|----|------|-------------|
| NOP | - | No operation |
| HALT | - | Stop execution |
| PANIC | msg: String | Abort with message |
| DUMP | - | Dump VM state (regs/heap) |
| GAS | reg | R[reg] = current gas limit |
| ASSERT | reg | Abort if R[reg] == 0 |

**Registers**
| Op | Args | Description |
|----|------|-------------|
| SET | reg, val | R[reg] = val (integer) |
| SETS | reg, val | R[reg] = heap_alloc(val) (string) |
| MOV | dest, src | R[dest] = R[src] |
| SWAP | r1, r2 | Swap R[r1] and R[r2] |

**Math** (all: dest, s1, s2)
| Op | Description |
|----|-------------|
| ADD | R[dest] = R[s1] + R[s2] |
| SUB | R[dest] = R[s1] - R[s2] |
| MUL | R[dest] = R[s1] * R[s2] |
| DIV | R[dest] = R[s1] / R[s2] |
| MOD | R[dest] = R[s1] % R[s2] |

**Logic** (all: dest, s1, s2 except NOT)
| Op | Description |
|----|-------------|
| AND | R[dest] = R[s1] & R[s2] |
| OR | R[dest] = R[s1] \| R[s2] |
| XOR | R[dest] = R[s1] ^ R[s2] |
| NOT | R[dest] = ~R[src] |

**Control Flow**
| Op | Args | Description |
|----|------|-------------|
| CMP | r1, r2 | Set flags: EQ (==), GT (>), LT (<) |
| JMP | target | Jump to line number |
| BEQ | target | Jump if EQ flag set |
| BGT | target | Jump if GT flag set |
| BLT | target | Jump if LT flag set |

> Line numbers are 0-indexed.

**Memory**
| Op | Args | Description |
|----|------|-------------|
| NEW | dest, size | R[dest] = heap_alloc(size bytes) |
| FREE | ptr | Free heap[R[ptr]] |
| READ | dest, ptr, offset | R[dest] = heap[R[ptr]][offset] |
| WRITE | ptr, offset, val | heap[R[ptr]][offset] = R[val] |
| READR | dest, ptr, off | R[dest] = heap[R[ptr]][R[off]] (dynamic offset) |
| WRITER | ptr, off, val | heap[R[ptr]][R[off]] = R[val] (dynamic offset) |

**Tool Execution**
| Op | Args | Description |
|----|------|-------------|
| TEXEC | tool, arg, dest | Call tool with heap[R[arg]], store result ptr in R[dest] |
| ITOA | dest, src | R[dest] = str(R[src]) - integer to string |
| ATOI | dest, src | R[dest] = int(R[src]) - string to integer (0 on error) |

### TEXEC Workflow

1. Store argument string: `SETS` → R[arg] holds heap pointer
2. Call tool: `TEXEC { tool: ID, arg: argReg, dest: destReg }`
3. Result string pointer stored in R[dest]

Tool IDs from `--info`:
- `0x5000` (20480): PRINT - prints string with newline
- `0x5001` (20481): PRINTN - prints string without newline (for compact output)
- `0x5002` (20482): INPUT - reads line from stdin, returns trimmed string
- `0x1004` (4100): POW - computes power, returns result string
- Plugins: FILE_READ, DB_SELECT, etc.

---

## Examples

### Hello World (ASM)
```asm
SETS 1, Hello, L-0!
TEXEC 0x5000, 1, 255
HALT
```

### Loop: Count 1 to 5 (ASM)
```asm
SET 1, 1       # counter
SET 2, 6       # limit

Loop:
CMP 1, 2
BEQ Done
SETS 10, Loop iteration
TEXEC 0x5000, 10, 255
SET 3, 1
ADD 1, 1, 3
JMP Loop

Done:
HALT
```

### Bubble Sort (ASM)

```asm
# Bubble Sort - sorts array [5,1,4,2,8] → [1,2,4,5,8]
# Registers: r1=base, r2=n, r3=i, r4=j, r10/r11=ptrs, r12/r13=vals, r100=const1

SET 100, 1          # constant 1
SET 2, 5            # n = 5

# Allocate array [5,1,4,2,8]
NEW 1, 1
SET 50, 5
WRITE 1, 0, 50
NEW 0, 1
ADD 10, 1, 100
SET 50, 1
WRITE 10, 0, 50
NEW 0, 1
ADD 10, 10, 100
SET 50, 4
WRITE 10, 0, 50
NEW 0, 1
ADD 10, 10, 100
SET 50, 2
WRITE 10, 0, 50
NEW 0, 1
ADD 10, 10, 100
SET 50, 8
WRITE 10, 0, 50

# Sort
SET 3, 0            # i = 0
OuterLoop:
CMP 3, 2
BEQ Done
SUB 5, 2, 100       # j_limit = n-1-i
SUB 5, 5, 3
SET 4, 0            # j = 0

InnerLoop:
CMP 4, 5
BGT NextI
BEQ NextI
ADD 10, 1, 4        # ptr_j
READ 12, 10, 0
ADD 11, 10, 100     # ptr_j+1
READ 13, 11, 0
CMP 12, 13
BLT NoSwap
BEQ NoSwap
WRITE 10, 0, 13     # swap
WRITE 11, 0, 12

NoSwap:
ADD 4, 4, 100
JMP InnerLoop

NextI:
ADD 3, 3, 100
JMP OuterLoop

Done:
SETS 255, Sorted!
TEXEC 0x5000, 255, 0
HALT
```

---

## Best Practices

**Register Convention:**
```
r0-r9     : Local variables, loop counters
r10-r19   : Pointers, addresses
r50-r99   : Temporary values
r100-r109 : Constants (e.g., r100=1, r101=8)
r255      : Tool I/O buffer
```

**Memory Pattern (Array):**
```asm
# Allocate contiguous array of N elements
NEW 1, 1              # r1 = base pointer
SET 100, 1            # constant 1
# Element i at: base + i (use ADD to compute pointer)
ADD 10, 1, i          # r10 = &arr[i]
READ 11, 10, 0        # r11 = arr[i]
WRITE 10, 0, val_reg  # arr[i] = val_reg
```

**Print Integer:**
```asm
SET 1, 42             # r1 = 42
ITOA 2, 1             # r2 = "42" (string)
TEXEC 0x5000, 2, 0    # prints "42"
```

**Compact Output (single line):**
```asm
# Use PRINTN (0x5001) for no newline, PRINT (0x5000) for final newline
SETS 1, [1,2,3]
TEXEC 0x5001, 1, 0    # prints "[1,2,3]" without newline
SETS 1, done
TEXEC 0x5000, 1, 0    # prints "done" with newline
# Output: [1,2,3]done
```

**Read & Parse User Input:**
```asm
SETS 1, _
TEXEC 0x5002, 1, 2    # r2 = user input string
ATOI 3, 2             # r3 = parsed integer
```

**Dynamic Array Access:**
```asm
NEW 1, 10             # r1 = array base
SET 2, 5              # r2 = index
SET 3, 42             # r3 = value
WRITER 1, 2, 3        # arr[5] = 42
READR 4, 1, 2         # r4 = arr[5]
```

---

## Extend

Add tools without recompiling. Edit `tools.json`:

```json
{
  "plugins": {
    "0x7000": { "name": "MY_TOOL", "type": "plugin", "binary": "tools/my_plugin", "method": "run" }
  }
}
```

Plugin Protocol (stdin/stdout JSON-RPC):
- Request: `{"method": "run", "args": ["..."]}`
- Response: `{"ok": true, "value": "..."}` or `{"ok": false, "error": "..."}`

---

## Bytecode Format (.l0)

Binary format details from `l0vm --info`:

| Property | Value |
|----------|-------|
| Format | Bincode (Rust Binary Serialization) |
| Endianness | Little Endian |
| ISA Definition | Enum (Untagged) |

Instructions are serialized sequentially. Each instruction:
1. Opcode (ordinal from ISA)
2. Arguments (type-specific encoding)

Strings are length-prefixed (u64 length + UTF-8 bytes).

---

## Build

```bash
./build.sh      # Compile and package to out/
```

Output structure:
```
out/
├── l0asm           # Assembler
├── l0vm            # Virtual Machine
├── tools.json      # Tool Registry
├── tools/          # Plugin Binaries
├── examples/       # Example Programs
└── BOOT.md         # This file
```

## Project

```
src/            # Rust source
build.sh        # Build script
Cargo.toml      # Rust manifest
tools.json      # Tool Registry
tools/          # Plugin source & binaries
examples/       # Example Programs
```

---

*L-0 v0.3.1 | Assembly + Bytecode | Protocol-First Architecture*
