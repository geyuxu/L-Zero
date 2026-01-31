# L-0 Bootstrap (v1.0)

> Code as Documentation. The VM is self-describing.

## What is L-0?

L-0 is a **custom virtual machine** with its own instruction set (ISA). Programs are written in **L-0 Assembly** (`.asm`) or **JSON**, compiled to **bytecode** (`.l0`), and executed by `l0vm`. Optionally, programs can be **AOT compiled** to native C code via `l0cc`.

**IMPORTANT FOR AI AGENTS:**
- Do NOT use Python, JavaScript, or other languages
- **Only `.l0` or `.json` files should remain** - delete `.asm` after compiling

```bash
# Standard workflow
./l0asm program.asm > program.l0 && rm program.asm   # Compile & cleanup
./l0vm program.l0                                     # Run & verify

# AOT compilation (for maximum performance)
./l0cc program.l0 -o program.c                        # Generate C
gcc -O2 program.c -o program && ./program             # Compile & run native
```

**Debugging Tips:**
- Use `--debug` flag to trace execution: `./l0vm program.l0 --debug`
- Use `DUMP` instruction to print all register values
- Common bugs: wrong register reuse, off-by-one in loops, forgetting to initialize

---

## Three-Layer Architecture

```
+---------------------------------------------------------------+
| Layer 1: ISA Core (52 primitives)                             |
| ------------------------------------------------------------- |
| - Register-based operations, compiled into VM                 |
| - Fast, direct execution                                      |
| - Defined in lib.rs                                           |
+---------------------------------------------------------------+
                              |
                              | TEXEC 0x5xxx
                              v
+---------------------------------------------------------------+
| Layer 2: Builtins                                             |
| ------------------------------------------------------------- |
| - String-based I/O, no external process                       |
| - PRINT, INPUT, RAND, TIME, UPPER, LOWER, etc.               |
| - Implemented in vm.rs                                        |
+---------------------------------------------------------------+
                              |
                              | TEXEC 0x3xxx-0x9xxx
                              v
+---------------------------------------------------------------+
| Layer 3: Plugins                                              |
| ------------------------------------------------------------- |
| - External subprocess, JSON-RPC protocol                      |
| - DB, HTTP, File, JSON, AI                                    |
| - Cross-language (Rust, Python, Swift, etc.)                 |
+---------------------------------------------------------------+
```

**When to use each layer:**
| Task | Layer | Example |
|------|-------|---------|
| Math on registers | ISA | `ADD 1, 2, 3` |
| String length | ISA | `HLEN 1, 2` |
| Regex match | ISA | `REGEX 1, 2, 3` |
| Print to stdout | Builtin | `TEXEC 0x5000, reg, _` |
| Read user input | Builtin | `TEXEC 0x5002, _, reg` |
| Database query | Plugin | `TEXEC 0x9003, reg, dest` |
| HTTP request | Plugin | `TEXEC 0x8004, reg, dest` |

---

## Toolchain

```
+-----------+   l0asm   +-----------+   l0vm    +-----------+
|  .asm     | --------> |  .l0      | --------> |  Execute  |
|  Assembly |           |  Bytecode |           |  Result   |
+-----------+           +-----------+           +-----------+
                              |
                              | l0cc
                              v
                        +-----------+    gcc    +-----------+
                        |  .c       | --------> |  Native   |
                        |  C Code   |           |  Binary   |
                        +-----------+           +-----------+
```

| Binary | Description |
|--------|-------------|
| `l0vm` | Virtual Machine - executes .json or .l0 programs |
| `l0asm`| Assembler - compiles .asm to .l0 bytecode |
| `l0cc` | AOT Compiler - generates C code from .l0/.json |

| Format | Purpose | Human-Readable |
|--------|---------|----------------|
| `.asm` | Source code (labels, comments) | Yes |
| `.json`| Intermediate (debugging) | Yes |
| `.l0`  | Binary bytecode (execution) | No |
| `.c`   | AOT compiled (native performance) | Yes |

---

## Introspection

Query the VM to discover ISA and Tools:

```bash
l0vm --info
```

Returns: `version`, `architecture`, `isa.primitives`, `tools`.

---

## ISA Reference (52 Instructions)

| Category | Count | Instructions |
|----------|-------|--------------|
| System | 6 | NOP, HALT, PANIC, DUMP, GAS, ASSERT |
| Registers | 4 | SET, SETS, MOV, SWAP |
| Math | 5 | ADD, SUB, MUL, DIV, MOD |
| Logic | 4 | AND, OR, XOR, NOT |
| Control | 5 | CMP, JMP, BEQ, BGT, BLT |
| Memory | 4 | NEW, FREE, READ, WRITE |
| Extensions | 9 | REGEX, TEXEC, ITOA, ATOI, READR, WRITER, SCAT, STORE64, LOAD64 |
| Batch | 4 | MEMCPY, HLEN, SLICE, MEMSET |
| Vector | 7 | VNEW, VSET, VGET, VDOT, VSIM, VMAG, VNORM |
| Governance | 4 | LATCH, GUARD, TRAP, YIELD |

### Registers

256 registers (0-255), each holds `i64` or heap pointer.

| Range | Convention |
|-------|------------|
| R0-R15 | General purpose |
| R50-R99 | Temporary values |
| R100-R109 | Constants (e.g., r100=1) |
| R240-R243 | Governance (target, state, threshold, similarity) |
| R255 | Tool result buffer |

### System
| Op | Args | Description |
|----|------|-------------|
| NOP | - | No operation |
| HALT | - | Stop execution |
| PANIC | msg: String | Abort with message |
| DUMP | - | Dump VM state (regs/heap) |
| GAS | reg | R[reg] = current gas limit |
| ASSERT | reg | Abort if R[reg] == 0 |

### Registers
| Op | Args | Description |
|----|------|-------------|
| SET | reg, val | R[reg] = val (integer) |
| SETS | reg, val | R[reg] = heap_alloc(val) (string) |
| MOV | dest, src | R[dest] = R[src] |
| SWAP | r1, r2 | Swap R[r1] and R[r2] |

### Math (all: dest, s1, s2)
| Op | Description |
|----|-------------|
| ADD | R[dest] = R[s1] + R[s2] |
| SUB | R[dest] = R[s1] - R[s2] |
| MUL | R[dest] = R[s1] * R[s2] |
| DIV | R[dest] = R[s1] / R[s2] |
| MOD | R[dest] = R[s1] % R[s2] |

### Logic (all: dest, s1, s2 except NOT)
| Op | Description |
|----|-------------|
| AND | R[dest] = R[s1] & R[s2] |
| OR | R[dest] = R[s1] \| R[s2] |
| XOR | R[dest] = R[s1] ^ R[s2] |
| NOT | R[dest] = ~R[src] |

### Control Flow
| Op | Args | Description |
|----|------|-------------|
| CMP | r1, r2 | Set flags: EQ (==), GT (>), LT (<) |
| JMP | target | Jump to line number |
| BEQ | target | Jump if EQ flag set |
| BGT | target | Jump if GT flag set |
| BLT | target | Jump if LT flag set |

### Memory (Basic)
| Op | Args | Description |
|----|------|-------------|
| NEW | dest, size | R[dest] = heap_alloc(size bytes) |
| FREE | ptr | Free heap[R[ptr]] |
| READ | dest, ptr, offset | R[dest] = heap[R[ptr]][offset] (u8) |
| WRITE | ptr, offset, val | heap[R[ptr]][offset] = R[val] (u8) |
| READR | dest, ptr, off | R[dest] = heap[R[ptr]][R[off]] (dynamic) |
| WRITER | ptr, off, val | heap[R[ptr]][R[off]] = R[val] (dynamic) |

### Memory (64-bit)
| Op | Args | Description |
|----|------|-------------|
| STORE64 | ptr, off, val | heap[R[ptr]][off*8..] = R[val] (full i64) |
| LOAD64 | dest, ptr, off | R[dest] = heap[R[ptr]][off*8..] (full i64) |

### Batch Memory Operations
| Op | Args | Description |
|----|------|-------------|
| MEMCPY | dst, doff, src, soff, len | Copy R[len] bytes between heap regions |
| HLEN | dest, ptr | R[dest] = length of heap[R[ptr]] |
| SLICE | dest, ptr, off, len | R[dest] = new heap with substring |
| MEMSET | ptr, off, len, val | Fill R[len] bytes with R[val] |

### Extensions
| Op | Args | Description |
|----|------|-------------|
| TEXEC | tool, arg, dest | Call tool with heap[R[arg]], store result in R[dest] |
| ITOA | dest, src | R[dest] = str(R[src]) - integer to string |
| ATOI | dest, src | R[dest] = int(R[src]) - string to integer |
| SCAT | dest, s1, s2 | R[dest] = R[s1] + R[s2] - string concatenation |
| REGEX | dest, pat, text | R[dest] = 1 if R[text] matches R[pat], else 0 |

### Vector Operations (Semantic Computing)
| Op | Args | Description |
|----|------|-------------|
| VNEW | dest, dims | R[dest] = allocate vector of R[dims] dimensions |
| VSET | vec, idx, val | vec[R[idx]] = f64::from_bits(R[val]) |
| VGET | dest, vec, idx | R[dest] = vec[R[idx]] as i64 bits |
| VDOT | dest, v1, v2 | R[dest] = dot(v1, v2) * 1,000,000 |
| VSIM | dest, v1, v2 | R[dest] = cosine_similarity(v1, v2) * 1,000,000 |
| VMAG | dest, vec | R[dest] = magnitude(vec) * 1,000,000 |
| VNORM | vec | Normalize vec in-place to unit length |

**Precision Model:** All scalar results scaled by 1,000,000 for integer precision (6 decimal places).

### Governance (Semantic Drift Control)
| Op | Args | Description |
|----|------|-------------|
| LATCH | target, threshold | R240 = R[target] (vec ptr), R242 = R[threshold] |
| GUARD | state | Compute VSIM(R240, R[state]); TRAP if below R242 |
| TRAP | code | Suspend VM, emit JSON context to supervisor |
| YIELD | query, dest | Send R[query] to supervisor, receive into R[dest] |

**Reserved Registers:**
- R240: Target vector pointer (set by LATCH)
- R242: Drift threshold (set by LATCH)
- R243: Last computed similarity (set by GUARD)

**Supervisor Protocol:** JSON over stdin/stdout for VM ↔ AI communication:
```json
{"action": "continue"}          // Resume execution
{"action": "jump", "target": N} // Jump to instruction N
{"action": "halt"}              // Stop execution
```

---

## Semantic Computing Philosophy

> The soul of L-0: enabling AI agents to sense and respond to semantic drift autonomously.

L-0 is a **semantic computing platform** designed for the era of autonomous AI agents. The key insight:

**For AI agents to operate without human intervention, they must be able to:**
1. Represent knowledge as vectors (embeddings)
2. Measure semantic similarity between states
3. Detect drift from target goals
4. Self-correct based on semantic distance

This is the "closed loop" that traditional computing lacks.

### Semantic Weight System

Every state in an AI agent's operation can be represented as a **semantic vector**:

```
                    Semantic Space
                         |
    Target Vector  *     |
                    \    |
                     \   |    Drift = 1 - cos_sim(current, target)
                      \  |
    Current Vector  *---+
                         |
```

### Vector Memory Format

```
Offset 0-7:   int64  dimensions
Offset 8+:    [f64]  values (little-endian IEEE 754)
```

---

## Semantic Computing Patterns

### Pattern 1: Goal-Directed Behavior

```
┌─────────────────────────────────────────────────────────────┐
│                    Agent Control Loop                        │
│                                                              │
│   ┌─────────┐    VSIM    ┌─────────┐    BLT    ┌─────────┐ │
│   │ Target  │──────────►│ Compare │──────────►│ Correct │ │
│   │ Vector  │            │ Drift   │            │ Course  │ │
│   └─────────┘            └─────────┘            └─────────┘ │
│        ▲                                             │       │
│        │                                             ▼       │
│   ┌─────────┐                                  ┌─────────┐  │
│   │ Update  │◄─────────────────────────────────│ Execute │  │
│   │ State   │                                  │ Action  │  │
│   └─────────┘                                  └─────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Pattern 2: Semantic Memory

Store and retrieve based on meaning, not exact match:

```asm
# Store memory with embedding
SETS 1, "The capital of France is Paris"
TEXEC 0x3000, 1, 2     # AI_EMBED -> R2 = embedding heap ptr

# Later: Query by meaning
SETS 10, "What city is France's capital?"
TEXEC 0x3000, 10, 11   # AI_EMBED -> R11 = query embedding

# Find most similar memory
VSIM 20, 2, 11         # Compare embeddings
# R20 = similarity score
```

### Pattern 3: Drift Monitoring with Governance

```asm
# Initialize drift monitor
SET 10, 768
VNEW 1, 10             # target vector
VNEW 2, 10             # state vector
# ... populate vectors ...

SET 100, 900000        # threshold = 0.9
LATCH 1, 100           # Set target and threshold

MonitorLoop:
    # Update state vector from current context
    # ...

    # Check drift (auto-TRAP if below threshold)
    GUARD 2

    # If we reach here, similarity >= threshold
    # Continue normal operation
    JMP MonitorLoop
```

### Pattern 4: Closed-Loop AI Generation

```asm
# Goal: Write code that passes tests
SETS 1, "Write a function that sorts a list"
TEXEC 0x3000, 1, goal_vec     # Goal embedding

WriteLoop:
    # Generate code
    TEXEC 0x3004, prompt, code_ptr

    # Get embedding of generated code
    TEXEC 0x3000, code_ptr, code_vec

    # Check semantic alignment
    VSIM 50, goal_vec, code_vec

    # If good enough, done
    SET 51, 800000    # 0.8 threshold
    CMP 50, 51
    BGT AcceptCode

    # Otherwise, refine prompt and retry
    SETS prompt, "Improve the sorting function..."
    JMP WriteLoop

AcceptCode:
    TEXEC 0x5000, code_ptr, 0
    HALT
```

---

## Design Rationale

### Why ISA-Level Vector Ops?

1. **Performance**: Avoids tool call overhead for frequent operations
2. **Determinism**: No external API variability
3. **Autonomy**: Agent can compute similarity without network
4. **AOT Compatibility**: Compiles to native C with math.h

### Why Integer Scaling (×1,000,000)?

1. **Register Uniformity**: All registers are i64
2. **Determinism**: Exact bit-level reproducibility
3. **Simplicity**: No floating-point mode flags
4. **Precision**: 6 decimals sufficient for similarity thresholds

### Why Cosine Similarity?

1. **Scale Invariance**: Works regardless of vector magnitude
2. **Bounded Output**: Always in [-1, 1] range
3. **Intuitive**: 1.0 = identical, 0.0 = orthogonal, -1.0 = opposite
4. **Industry Standard**: Compatible with all embedding models

---

## Tool Reference

### Builtins (Layer 2)

**I/O Tools**
| ID | Name | Description |
|----|------|-------------|
| `0x5000` | PRINT | Print string with newline |
| `0x5001` | PRINTN | Print string without newline |
| `0x5002` | INPUT | Read line from stdin |

**Math Tools**
| ID | Name | Args | Description |
|----|------|------|-------------|
| `0x1004` | POW | "base,exp" | Power function |
| `0x5005` | ABS | "n" | Absolute value |
| `0x5006` | MIN | "a,b" | Minimum |
| `0x5007` | MAX | "a,b" | Maximum |
| `0x5003` | RAND | "max" | Random 0..max |

**String Tools**
| ID | Name | Args | Description |
|----|------|------|-------------|
| `0x500A` | SUBSTR | "str,start,len" | Substring |
| `0x500B` | SPLIT | "str,delim,idx" | Split and get element |
| `0x500C` | UPPER | "str" | Uppercase |
| `0x500D` | LOWER | "str" | Lowercase |
| `0x500E` | TRIM | "str" | Trim whitespace |

**System Tools**
| ID | Name | Description |
|----|------|-------------|
| `0x5008` | TIME | Unix timestamp (seconds) |

**Deprecated** (use ISA equivalents):
| ID | Name | Use Instead |
|----|------|-------------|
| `0x5004` | STRLEN | `HLEN` instruction |
| `0x5009` | CONCAT | `SCAT` instruction |

### Plugins (Layer 3)

**File I/O (0x4xxx)**
| ID | Name | Description |
|----|------|-------------|
| `0x4000` | FILE_READ | Read file contents |
| `0x4001` | FILE_WRITE | Write to file |
| `0x4003` | FILE_DELETE | Delete file |
| `0x4004` | FILE_LIST | List directory |
| `0x4005` | FILE_EXISTS | Check if file exists |

**JSON (0x6xxx)**
| ID | Name | Description |
|----|------|-------------|
| `0x6000` | JSON_LOAD | Load JSON from file |
| `0x6001` | JSON_SAVE | Save JSON to file |
| `0x6002` | JSON_GET | Get value from JSON |
| `0x6003` | JSON_SET | Set value in JSON |
| `0x6004` | JSON_PARSE | Parse JSON string |

**Database (0x9xxx)**
| ID | Name | Args | Description |
|----|------|------|-------------|
| `0x9000` | DB_INIT | - | Initialize database |
| `0x9001` | DB_CREATE_TABLE | "table_name" | Create table |
| `0x9002` | DB_INSERT | "table\|{json}" | Insert row |
| `0x9003` | DB_SELECT | "table[\|condition]" | Query rows |
| `0x9004` | DB_UPDATE | "table\|condition\|{json}" | Update rows |
| `0x9005` | DB_DELETE | "table\|condition" | Delete rows |
| `0x9006` | DB_DROP_TABLE | "table_name" | Drop table |
| `0x9007` | DB_LIST_TABLES | - | List all tables |

**HTTP (0x8xxx)**
| ID | Name | Description |
|----|------|-------------|
| `0x8000` | HTTP_INIT | Initialize HTTP server |
| `0x8001` | HTTP_ROUTE | Register route |
| `0x8002` | HTTP_SERVE | Start server (blocking) |
| `0x8003` | HTTP_SERVE_ONCE | Handle one request |
| `0x8004` | HTTP_REQUEST | Make HTTP request |
| `0x8005` | HTTP_LIST_ROUTES | List registered routes |

**AI (0x3xxx)** - Optional
| ID | Name | Description |
|----|------|-------------|
| `0x3000` | AI_EMBED | Generate embeddings |
| `0x3004` | AI_GENERATE | Generate text |

---

## Plugin Protocol

Plugins communicate via JSON-RPC over stdin/stdout:

**Request** (VM -> Plugin):
```json
{"method": "select", "args": ["users|id=1"]}
```

**Response** (Plugin -> VM):
```json
{"ok": true, "value": [{"id": 1, "name": "Alice"}]}
```

Or on error:
```json
{"ok": false, "error": "table not found"}
```

**Argument Convention:**
- Single arg: plain string
- Multiple args: pipe-separated (`table|condition|value`)

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
| `REGEX 1, 2, 3` | `{ "REGEX": { "dest": 1, "pat": 2, "text": 3 } }` |

---

## Examples

### Hello World
```asm
SETS 1, Hello, L-0!
TEXEC 0x5000, 1, 255
HALT
```

### REGEX Pattern Match
```json
[
  { "SETS": { "reg": 1, "val": "[0-9]+" } },
  { "SETS": { "reg": 2, "val": "test123abc" } },
  { "REGEX": { "dest": 3, "pat": 1, "text": 2 } },
  { "SETS": { "reg": 255, "val": "Match result: " } },
  { "ITOA": { "dest": 4, "src": 3 } },
  { "SCAT": { "dest": 5, "s1": 255, "s2": 4 } },
  { "TEXEC": { "tool": 20480, "arg": 5, "dest": 0 } },
  "HALT"
]
```

### Batch Memory Operations
```json
[
  { "SETS": { "reg": 1, "val": "Hello World" } },
  { "HLEN": { "dest": 2, "ptr": 1 } },
  { "ITOA": { "dest": 3, "src": 2 } },
  { "SETS": { "reg": 4, "val": "Length: " } },
  { "SCAT": { "dest": 5, "s1": 4, "s2": 3 } },
  { "TEXEC": { "tool": 20480, "arg": 5, "dest": 0 } },

  { "SET": { "reg": 10, "val": 0 } },
  { "SET": { "reg": 11, "val": 5 } },
  { "SLICE": { "dest": 12, "ptr": 1, "off": 10, "len": 11 } },
  { "SETS": { "reg": 13, "val": "Slice: " } },
  { "SCAT": { "dest": 14, "s1": 13, "s2": 12 } },
  { "TEXEC": { "tool": 20480, "arg": 14, "dest": 0 } },
  "HALT"
]
```

### Database Operations
```json
[
  { "SETS": { "reg": 1, "val": "users" } },
  { "TEXEC": { "tool": 36865, "arg": 1, "dest": 2 } },

  { "SETS": { "reg": 1, "val": "users|{\"name\":\"Alice\",\"age\":30}" } },
  { "TEXEC": { "tool": 36866, "arg": 1, "dest": 2 } },

  { "SETS": { "reg": 1, "val": "users" } },
  { "TEXEC": { "tool": 36867, "arg": 1, "dest": 2 } },
  { "TEXEC": { "tool": 20480, "arg": 2, "dest": 0 } },
  "HALT"
]
```

### Blog Web Server (Full-Stack Example)

A complete web application combining Database and HTTP plugins:

```bash
./target/release/l0vm examples/blog_web.json
# Server ready at http://127.0.0.1:8080
# Routes: GET /, GET /api
```

**Architecture:**
```
┌─────────────────────────────────────────────────────────────┐
│                    blog_web.json                             │
│                                                              │
│   1. DB_INIT (0x9000)      Initialize database               │
│   2. DB_CREATE_TABLE       Create "posts" table              │
│   3. DB_INSERT (×3)        Add sample blog posts             │
│   4. DB_SELECT             Fetch posts JSON -> R40           │
│   5. HTTP_INIT             Configure port 8080               │
│   6. SCAT (×5)             Build HTML with CSS + R40 data    │
│   7. HTTP_ROUTE (×2)       Register GET / and GET /api       │
│   8. HTTP_SERVE            Serve 10 requests                 │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**Key Techniques:**

1. **String Concatenation for HTML:**
```json
{ "SETS": { "reg": 50, "val": "<!DOCTYPE html><html>..." } },
{ "SETS": { "reg": 51, "val": "<div class='card'>..." } },
{ "SCAT": { "dest": 60, "s1": 50, "s2": 51 } }
```

2. **Dynamic Data Injection:**
```json
{ "TEXEC": { "tool": 36867, "arg": 1, "dest": 40 } },  // DB_SELECT -> R40
{ "SCAT": { "dest": 62, "s1": 61, "s2": 40 } }         // Inject into HTML
```

3. **Route Registration:**
```json
{ "SETS": { "reg": 70, "val": "GET /|" } },
{ "SCAT": { "dest": 71, "s1": 70, "s2": 64 } },        // "GET /|<html>..."
{ "TEXEC": { "tool": 32769, "arg": 71, "dest": 0 } }   // HTTP_ROUTE
```

**Features:**
- Modern CSS with gradient backgrounds, card hover effects
- Client-side JavaScript for dynamic JSON rendering
- Two endpoints: styled HTML (`/`) and raw JSON API (`/api`)
- Data persisted in `l0_database.json`, routes in `.l0_http_config.json`

### Loop: Count 1 to 5
```asm
SET 1, 1       # counter
SET 2, 6       # limit

Loop:
CMP 1, 2
BEQ Done
ITOA 10, 1
TEXEC 0x5000, 10, 255
SET 3, 1
ADD 1, 1, 3
JMP Loop

Done:
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

**Print Integer:**
```asm
SET 1, 42             # r1 = 42
ITOA 2, 1             # r2 = "42" (string)
TEXEC 0x5000, 2, 0    # prints "42"
```

**TEXEC Returns String (use ATOI for numbers):**
```asm
SETS 1, -42
TEXEC 0x5005, 1, 2    # ABS returns string "42" in r2
ATOI 3, 2             # r3 = 42 (integer)
```

**Use ISA over Builtins when possible:**
```asm
# Bad: Use TEXEC for string length
TEXEC 0x5004, 1, 2    # Deprecated STRLEN

# Good: Use HLEN instruction
HLEN 2, 1             # Direct ISA, faster
```

---

## Extend

Add tools without recompiling. Edit `tools.json`:

```json
{
  "plugins": {
    "0x7000": {
      "name": "MY_TOOL",
      "type": "plugin",
      "binary": "tools/my_plugin",
      "method": "run"
    }
  }
}
```

Plugin Protocol (stdin/stdout JSON-RPC):
- Request: `{"method": "run", "args": ["..."]}\n`
- Response: `{"ok": true, "value": "..."}` or `{"ok": false, "error": "..."}`

---

## Build

```bash
cargo build --release
```

Binaries output to `target/release/`:
- `l0vm` - Virtual Machine
- `l0asm` - Assembler
- `l0cc` - AOT Compiler

Build plugins:
```bash
cd tools && cargo build --release
```

---

**Important Notes:**
- TEXEC always returns a heap pointer to a string. Use ATOI to convert numeric results.
- WRITE/READ store u8 values (0-255). Use STORE64/LOAD64 for full i64.
- ASM format uses comma as argument separator. For strings containing commas, use JSON format.
- Use ISA instructions (HLEN, SCAT) over deprecated builtins (STRLEN, CONCAT).

---

## Future Directions

1. **VADD/VSUB**: Vector arithmetic for interpolation
2. **VSCALE**: Scalar multiplication for weighting
3. **VPROJ**: Vector projection for decomposition
4. **Built-in Embedding**: Native transformer inference

---

*L-0 v1.0 | 52 ISA Primitives | Semantic Computing | The Language of Autonomous AI*
