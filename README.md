# L-Zero (v0.3): The Native Language of AI Agents

L-Zero is a **low-level, structural instruction set** designed specifically for AI generation, not human writing.

Unlike traditional languages that prioritize human readability (syntax sugar, complex parsers), L-Zero prioritizes **Machine Determinism**:
- **Dual Format**: JSON for debugging/generation, Binary (Bincode) for execution
- **Type-Safe**: Based on a strict Rust Enum definition, ensuring 100% valid structure
- **Agentic Extensibility**: Core logic is minimal (29 Primitives); infinite capability via dynamic Tools (Plugins)

---

## Quick Start

### 1. Build
```bash
cargo build --release --bin l0vm --bin l0asm
mkdir -p bin
cp target/release/l0vm bin/
cp target/release/l0asm bin/
```

### 2. Run Examples
```bash
# JSON format (direct execution)
./bin/l0vm examples/hello.json

# Assembly format (compile then run)
./bin/l0asm examples/bubble_sort.asm > /tmp/out.l0
./bin/l0vm /tmp/out.l0

# Run test suite
./examples/run_tests.sh
```

---

## Architecture (v0.3)

L-Zero v0.3 adopts a **Protocol-First** architecture with Assembly support.

### 1. The ISA (29 Instructions)

The language is defined by the `Instruction` enum in `src/lib.rs`:

| Category | Instructions | Description |
|----------|--------------|-------------|
| **System** | `NOP`, `HALT`, `PANIC`, `ASSERT`, `DUMP`, `GAS` | Lifecycle & Debugging |
| **Registers** | `SET`, `SETS`, `MOV`, `SWAP` | 256 Registers (i64 / Heap Ptr) |
| **Math** | `ADD`, `SUB`, `MUL`, `DIV`, `MOD` | Integer Arithmetic |
| **Logic** | `AND`, `OR`, `XOR`, `NOT` | Bitwise Operations |
| **Control** | `CMP`, `JMP`, `BEQ`, `BGT`, `BLT` | Flow Control (Line numbers) |
| **Memory** | `NEW`, `FREE`, `READ`, `WRITE`, `READR`, `WRITER` | Safe Managed Heap |
| **Extensions** | `TEXEC`, `ITOA`, `ATOI`, `SCAT` | Tools & String Operations |

### 2. The Runtime

`l0vm` is a direct-execution engine that:
- Loads JSON programs directly, or Bincode (.l0) bytecode
- Executes with 256-register file and managed heap
- Invokes tools via `TEXEC` instruction

### 3. The Tool Registry

The `TEXEC` instruction invokes external capabilities defined in `tools.json`:

**Built-in Tools:**
| ID | Name | Description |
|----|------|-------------|
| `0x5000` | PRINT | Print with newline |
| `0x5001` | PRINTN | Print without newline |
| `0x5002` | INPUT | Read line from stdin |
| `0x5003` | RAND | Random number |
| `0x5004` | STRLEN | String length |
| `0x5005` | ABS | Absolute value |
| `0x5006` | MIN | Minimum of two values |
| `0x5007` | MAX | Maximum of two values |
| `0x5008` | TIME | Unix timestamp |
| `0x500C` | UPPER | Uppercase string |
| `0x500D` | LOWER | Lowercase string |
| `0x500E` | TRIM | Trim whitespace |
| `0x1004` | POW | Power function |

**Plugin Protocol:**
Add new capabilities without recompiling. Edit `tools.json`:
```json
"plugins": {
    "0x7000": {
        "name": "MY_TOOL",
        "type": "plugin",
        "binary": "tools/my_script.py",
        "method": "run"
    }
}
```
Plugins communicate via JSON-RPC over stdin/stdout.

---

## Toolchain

```
┌─────────────┐    l0asm     ┌─────────────┐    l0vm      ┌─────────────┐
│  .asm       │ ──────────►  │  .l0        │ ──────────►  │  Execute    │
│  Assembly   │              │  Bytecode   │              │  Result     │
└─────────────┘              └─────────────┘              └─────────────┘

                             ┌─────────────┐
                             │  .json      │ ──────────►  (l0vm direct)
                             │  JSON       │
                             └─────────────┘
```

| Binary | Description |
|--------|-------------|
| `l0vm` | Virtual Machine - executes .json or .l0 programs |
| `l0asm`| Assembler - compiles .asm to .l0 bytecode |

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

### Test Suite

Comprehensive test coverage in `examples/`:

| Test | Coverage |
|------|----------|
| `test_basic.asm` | SET, MOV, SWAP, SETS, ITOA, ATOI |
| `test_math.asm` | ADD, SUB, MUL, DIV, MOD, AND, OR, XOR, NOT |
| `test_memory.asm` | NEW, FREE, READ, WRITE, READR, WRITER |
| `test_control.asm` | JMP, BEQ, BGT, BLT, loops |
| `test_string.asm` | STRLEN, UPPER, LOWER, TRIM, SCAT |
| `test_tools.asm` | PRINT, ABS, RAND, TIME |
| `test_all.asm` | Comprehensive suite (17 tests) |
| `bubble_sort.asm` | Algorithm example |

Run all tests:
```bash
./examples/run_tests.sh
```

---

## Key Concepts

### TEXEC Returns Heap Pointer
All tool calls return strings stored on heap. Use `ATOI` to convert to integer:
```asm
SETS 1, -42
TEXEC 0x5005, 1, 2    # ABS returns string "42"
ATOI 3, 2             # Convert to integer 42
```

### Memory Values are u8
`WRITE`/`READ` store bytes (0-255). Values are truncated:
```asm
SET 50, 300           # 300 mod 256 = 44
WRITE 10, 0, 50       # Stores 44, not 300
```

### Comma Limitation in ASM
Commas are argument separators. For strings with commas, use JSON format:
```json
{ "SETS": { "reg": 1, "val": "2,10" } }
```

---

## Directory Structure

```
.
├── bin/                # Compiled binaries
├── src/                # Rust Source
│   ├── lib.rs          # ISA Definition (29 instructions)
│   ├── vm.rs           # Virtual Machine (~530 LOC)
│   └── asm.rs          # Assembler (~220 LOC)
├── tools/              # External Plugins
│   └── file_plugin     # File I/O plugin
├── tools.json          # Tool Registry (31 tools)
├── examples/           # Example & Test Programs
│   ├── hello.json      # Hello World (JSON)
│   ├── bubble_sort.asm # Sorting algorithm
│   ├── test_*.asm      # Test suites
│   └── run_tests.sh    # Test runner
├── BOOT.md             # Detailed documentation
└── README.md           # This file
```

## Introspection

Query the VM for ISA and tools:
```bash
./bin/l0vm --info
```

## License
MIT
