# L-Zero (v1.0): The Native Language of AI Agents

L-Zero is a **low-level, structural instruction set** designed specifically for AI generation, not human writing.

Unlike traditional languages that prioritize human readability (syntax sugar, complex parsers), L-Zero prioritizes **Machine Determinism**:
- **ASM Source Format**: Assembly with labels - optimal for AI token efficiency and control flow
- **Type-Safe**: Based on a strict Rust Enum definition, ensuring 100% valid structure
- **Agentic Extensibility**: Core logic (52 Primitives) with semantic computing; infinite capability via dynamic Tools (Plugins)
- **AOT Compilation**: Compile to native C code for maximum performance

---

## Quick Start

### 1. Build
```bash
cargo build --release --workspace
```

### 2. Run Examples
```bash
# Standard workflow: ASM → Bytecode → Execute
./target/release/l0asm examples/bubble_sort.asm > /tmp/out.l0
./target/release/l0vm /tmp/out.l0

# AOT compile to native binary
./target/release/l0cc /tmp/out.l0 -o program.c
gcc -O2 program.c -o program && ./program
```

### 3. For AI Agents
```
AI generates .asm file (with labels, comments)
       ↓
l0asm compiles to .l0 (validates syntax)
       ↓
l0vm executes .l0 (or l0cc for AOT)
```

---

## Architecture (v1.0)

L-Zero v1.0 adopts a **Three-Layer Architecture**:

```
+---------------------------------------------------------------+
| Layer 1: ISA Core (52 primitives)                             |
| ------------------------------------------------------------- |
| - Register-based operations, compiled into VM                 |
| - ADD, SUB, MUL, DIV, MOD, SCAT, HLEN, REGEX, etc.           |
| - Defined in lib.rs, executed directly                        |
+---------------------------------------------------------------+
                              |
                              | TEXEC instruction
                              v
+---------------------------------------------------------------+
| Layer 2: Builtins (0x5xxx)                                    |
| ------------------------------------------------------------- |
| - String-based I/O, no external process                       |
| - PRINT, INPUT, RAND, TIME, UPPER, LOWER, etc.               |
| - Implemented in vm.rs execute_builtin()                      |
+---------------------------------------------------------------+
                              |
                              | TEXEC instruction
                              v
+---------------------------------------------------------------+
| Layer 3: Plugins (0x3xxx-0x9xxx)                              |
| ------------------------------------------------------------- |
| - External subprocess, JSON-RPC protocol                      |
| - DB, HTTP, File, JSON, AI                                    |
| - Cross-language support (Rust, Swift, Python, etc.)         |
+---------------------------------------------------------------+
```

### 1. The ISA (52 Instructions)

The language is defined by the `Instruction` enum in `crates/l0_core/src/lib.rs`:

| Category | Instructions | Description |
|----------|--------------|-------------|
| **System** | `NOP`, `HALT`, `PANIC`, `ASSERT`, `DUMP`, `GAS` | Lifecycle & Debugging |
| **Registers** | `SET`, `SETS`, `MOV`, `SWAP` | 256 Registers (i64 / Heap Ptr) |
| **Math** | `ADD`, `SUB`, `MUL`, `DIV`, `MOD` | Integer Arithmetic |
| **Logic** | `AND`, `OR`, `XOR`, `NOT` | Bitwise Operations |
| **Control** | `CMP`, `JMP`, `BEQ`, `BGT`, `BLT` | Flow Control (Line numbers) |
| **Memory** | `NEW`, `FREE`, `READ`, `WRITE`, `READR`, `WRITER` | Safe Managed Heap |
| **Extensions** | `TEXEC`, `ITOA`, `ATOI`, `SCAT`, `REGEX` | Tools & String Operations |
| **64-bit** | `STORE64`, `LOAD64` | Store/Load full i64 to heap |
| **Batch** | `MEMCPY`, `HLEN`, `SLICE`, `MEMSET` | Bulk memory operations |
| **Vector** | `VNEW`, `VSET`, `VGET`, `VDOT`, `VSIM`, `VMAG`, `VNORM` | Semantic Computing |
| **Governance** | `LATCH`, `GUARD`, `TRAP`, `YIELD` | Autonomous AI Drift Control |

### 2. The Toolchain

| Binary | Description |
|--------|-------------|
| `l0vm` | Virtual Machine - executes .l0 bytecode |
| `l0asm`| Assembler - compiles .asm to .l0 bytecode |
| `l0cc` | AOT Compiler - compiles .l0 to C code (with supervisor protocol) |

| Format | Role | AI Generates? |
|--------|------|---------------|
| `.asm` | **Source** (labels, comments) | **YES** |
| `.l0`  | Binary bytecode | No (compiled) |
| `.c`   | AOT output | No (generated) |

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

### 3. The Tool Registry

The `TEXEC` instruction invokes capabilities defined in `tools.json`:

**Built-in Tools (I/O):**
| ID | Name | Description |
|----|------|-------------|
| `0x5000` | PRINT | Print with newline |
| `0x5001` | PRINTN | Print without newline |
| `0x5002` | INPUT | Read line from stdin |
| `0x5003` | RAND | Random number |
| `0x5008` | TIME | Unix timestamp |

**Built-in Tools (String):**
| ID | Name | Description |
|----|------|-------------|
| `0x500C` | UPPER | Uppercase string |
| `0x500D` | LOWER | Lowercase string |
| `0x500E` | TRIM | Trim whitespace |
| `0x500A` | SUBSTR | Substring |
| `0x500B` | SPLIT | Split string |

**Built-in Tools (Math):**
| ID | Name | Description |
|----|------|-------------|
| `0x5005` | ABS | Absolute value |
| `0x5006` | MIN | Minimum of two |
| `0x5007` | MAX | Maximum of two |
| `0x1004` | POW | Power function |

**Plugins:**
| Prefix | Category | Examples |
|--------|----------|----------|
| `0x4xxx` | File I/O | FILE_READ, FILE_WRITE, FILE_EXISTS |
| `0x6xxx` | JSON | JSON_LOAD, JSON_SAVE, JSON_GET |
| `0x8xxx` | HTTP | HTTP_SERVE, HTTP_REQUEST, HTTP_ROUTE |
| `0x9xxx` | Database | DB_CREATE_TABLE, DB_INSERT, DB_SELECT |
| `0x3xxx` | AI | AI_EMBED, AI_GENERATE |

---

## Examples

### Hello World
```json
[
  { "SETS": { "reg": 1, "val": "Hello, L-0!" } },
  { "TEXEC": { "tool": 20480, "arg": 1, "dest": 255 } },
  "HALT"
]
```

### Assembly Format
```asm
# Hello World
SETS 1, Hello, L-0!
TEXEC 0x5000, 1, 255
HALT
```

### REGEX Example
```json
[
  { "SETS": { "reg": 1, "val": "[0-9]+" } },
  { "SETS": { "reg": 2, "val": "test123abc" } },
  { "REGEX": { "dest": 3, "pat": 1, "text": 2 } },
  { "ITOA": { "dest": 4, "src": 3 } },
  { "TEXEC": { "tool": 20480, "arg": 4, "dest": 0 } },
  "HALT"
]
```

### Database Example
```json
[
  { "SETS": { "reg": 1, "val": "users" } },
  { "TEXEC": { "tool": 36865, "arg": 1, "dest": 2 } },
  { "SETS": { "reg": 1, "val": "users|{\"name\":\"Alice\"}" } },
  { "TEXEC": { "tool": 36866, "arg": 1, "dest": 2 } },
  { "SETS": { "reg": 1, "val": "users" } },
  { "TEXEC": { "tool": 36867, "arg": 1, "dest": 2 } },
  { "TEXEC": { "tool": 20480, "arg": 2, "dest": 0 } },
  "HALT"
]
```

### Blog Web Server (Full-Stack Example)
A complete web application using DB + HTTP plugins together:
```bash
./target/release/l0vm examples/blog_web.json
# Server starts at http://127.0.0.1:8080
# Routes: GET / (styled HTML), GET /api (JSON data)
```

The example demonstrates:
- **Database Plugin** (`0x9xxx`): Initialize DB, create table, insert posts
- **HTTP Plugin** (`0x8xxx`): Configure server, register routes, serve requests
- **String Operations**: `SCAT` for building HTML from fragments
- **Dynamic Content**: Posts fetched from DB rendered into HTML via JavaScript

---

## Key Concepts

### ISA vs Builtins vs Plugins

| Layer | Example | Interface | Use Case |
|-------|---------|-----------|----------|
| ISA | `ADD 1, 2, 3` | Register-based | Fast computation |
| Builtin | `TEXEC 0x5000` | String via TEXEC | I/O operations |
| Plugin | `TEXEC 0x9003` | JSON-RPC subprocess | External services |

### TEXEC Returns Heap Pointer
All tool calls return strings stored on heap. Use `ATOI` to convert to integer:
```asm
SETS 1, -42
TEXEC 0x5005, 1, 2    # ABS returns string "42"
ATOI 3, 2             # Convert to integer 42
```

### Batch Memory Operations
```asm
SETS 1, Hello World
HLEN 2, 1             # R2 = 11 (length)
SET 3, 0              # offset
SET 4, 5              # length
SLICE 5, 1, 3, 4      # R5 = "Hello" (new heap)
```

### Semantic Computing (Vector Operations)
L-0 provides native vector operations for AI agents to monitor semantic drift:
```asm
SET 10, 768            # embedding dimension
VNEW 1, 10             # target vector
VNEW 2, 10             # current state vector
VSIM 3, 1, 2           # R3 = cosine_similarity * 1,000,000
```

### Governance (Autonomous AI Control)
Enable AI agents to self-monitor and request human intervention:
```asm
SET 30, 800000         # threshold = 0.8
LATCH 1, 30            # set target vector and threshold
GUARD 2                # check state; TRAP if drift detected
```

When drift is detected, GUARD triggers TRAP which:
1. Suspends execution (VM or AOT)
2. Emits JSON context to stdout: `{"trap":"DRIFT","code":1,"pc":N,...}`
3. Waits for supervisor response on stdin: `{"action":"continue"}`

Both VM and AOT-compiled binaries support the same supervisor protocol for governance.

---

## Directory Structure

```
.
├── Cargo.toml              # Workspace configuration
├── crates/                 # Core components
│   ├── l0_core/            # ISA Definition (52 instructions)
│   ├── l0_vm/              # Virtual Machine
│   ├── l0_asm/             # Assembler
│   └── l0_compiler/        # AOT Compiler
├── plugins/                # External plugins
│   ├── file_plugin/        # File I/O operations
│   ├── data_plugin/        # JSON operations
│   ├── http_plugin/        # HTTP server
│   └── db_plugin/          # SQLite database
├── tools.json              # Tool Registry (40+ tools)
├── examples/               # Example programs
├── stdlib/                 # Standard library patterns
├── scripts/                # Build scripts
├── BOOT.md                 # Detailed documentation
└── README.md               # This file
```

## Introspection

Query the VM for ISA and tools:
```bash
./target/release/l0vm --info
```

## License
MIT
