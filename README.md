# L-Zero (v0.3): The Native Language of AI Agents

L-Zero is a **low-level, structural instruction set** designed specifically for AI generation, not human writing. 

Unlike traditional languages that prioritize human readability (syntax sugar, complex parsers), L-Zero prioritizes **Machine Determinism**:
- **JSON-Native ISA**: Code is a JSON array of instruction objects. No parsing ambiguity.
- **Type-Safe**: Based on a strict Rust Enum definition, ensuring 100% valid structure if it compiles.
- **Zero-Compile**: The VM executes the JSON structure directly. No bytecode step required.
- **Agentic Extensibility**: Core logic is minimal (Primitives); infinite capability is added via dynamic Tools (Plugins).

---

## 🚀 Quick Start

### 1. Build
```bash
cargo build --release --bin l0vm --bin l0asm
mkdir -p bin
cp target/release/l0vm bin/
cp target/release/l0asm bin/
```

### 2. Run an Example
Create `hello.json`:
```json
[
  { "SETS": { "reg": 1, "val": "Hello, AI World!" } },
  { "TEXEC": { "tool": 20480, "arg": 1, "dest": 255 } },
  "HALT"
]
```
> Note: Tool ID `20480` (0x5000) corresponds to `PRINT`.

Run it:
```bash
./bin/l0vm hello.json
# Output: Hello, AI World!
```

---

## 🧠 Architecture (v0.3)

L-Zero v0.3 adopts a **Protocol-First** architecture.

### 1. The ISA (Protocol)
The language is defined entirely by the `Instruction` enum in `src/lib.rs`. It supports 7 categories of primitives:

| Category | Instructions | Description |
|----------|--------------|-------------|
| **System** | `NOP`, `HALT`, `PANIC`, `ASSERT`, `DUMP`, `GAS` | Lifecycle & Debugging |
| **Registers** | `SET` (Int), `SETS` (String), `MOV`, `SWAP` | 256 Registers (i64 / Ptr) |
| **Math** | `ADD`, `SUB`, `MUL`, `DIV`, `MOD` | Integer Arithmetic |
| **Logic** | `AND`, `OR`, `XOR`, `NOT` | Bitwise Operations |
| **Control** | `CMP`, `JMP`, `BEQ`, `BGT`, `BLT` | Flow Control (Line numbers) |
| **Memory** | `NEW`, `FREE`, `READ`, `WRITE` | Safe Managed Heap |
| **Extensions**| `REGEX`, `TEXEC` | AI & System Capabilities |

### 2. The Runtime (Direct Execution)
`l0vm` is a direct-execution engine. It:
- Loads the JSON program.
- deserializes it into the Rust `Instruction` memory structure.
- Executes the loop with a 256-register file and a managed heap.

### 3. The Tool Registry (Dynamic)
L-Zero is small but infinite. The `TEXEC` instruction allows invoking external capabilities defined in `tools.json`.

**Feature: Zero-Recompile Extension**
You can add new capabilities without recompiling the VM. Just edit `tools.json`:

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
Now `TEXEC 0x7000` will spawn `my_script.py` and communicate via JSON-RPC over stdin/stdout.

---

## 🛠️ Toolchain

| Binary | Status | Description |
|--------|--------|-------------|
| `l0vm` | **Core** | The Virtual Machine. Executes JSON programs. |
| `l0asm`| Utility | Validates and pretty-prints L-0 JSON. |

*Deprecated in v0.3: `l0c` (Compiler) and Bytecode formats. We are now pure JSON structure.*

---

## 📦 Directory Structure

```
.
├── bin/                # Compiled binaries
├── src/                # Rust Source
│   ├── lib.rs          # ISA Definition (The Protocol)
│   ├── vm.rs           # Virtual Machine Implementation
│   └── asm.rs          # Validator
├── tools/              # External Plugin Scripts
│   ├── ai_plugin/      # AI Embeddings/Generation
│   ├── db_plugin/      # Database Access
│   └── ...
├── tools.json          # Tool Registry Configuration
└── examples/           # Example Code (JSON)
```

## 📜 License
MIT
# L-Zero
