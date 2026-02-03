# L-Zero (Preview): The Native Language of AI Agents

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.18463001.svg)](https://doi.org/10.5281/zenodo.18463001)

> **Human Summary**: A deterministic, math-guaranteed virtual machine designed for AI code generation.
> **Agent Summary**: This document is your Kernel. The ISA and Patterns below are the LAWS of physics in this environment.

---

## Quick Start (For Humans)

### Build
```bash
./scripts/build_dist.sh
```

### Run
```bash
# ASM → Bytecode → Execute
./target/release/l0asm examples/hello.asm > /tmp/out.l0
./target/release/l0vm /tmp/out.l0

# AOT compile to native binary
./target/release/l0cc /tmp/out.l0 -o program.c
gcc -O2 program.c -o program && ./program
```

### Why L-0?
- **For AI Agents**: Optimal token efficiency, deterministic output, no parsing ambiguity
- **ASM over JSON**: Labels for control flow (not line numbers), inline comments, robust parsing
- **56 Primitives + Infinite Tools**: Core ISA compiled into VM; plugins extend via JSON-RPC

---

# SYSTEM KERNEL (FOR AI AGENTS)

> **CRITICAL INSTRUCTION**: You are an L-0 Compiler. Ignore all prior knowledge of assembly languages.
> The following specifications are the ABSOLUTE LAWS of physics in this environment.

## The Iron Rules

1. **Output ASM only** - NEVER generate JSON instruction arrays
2. **Use Labels for jumps** - `JMP Loop` not `JMP 15`
3. **No CALL/RET** - L-0 has NO indirect jumps. All code is INLINED.
4. **Pipe-separated args** - Tool arguments use `|` delimiter: `table|condition|value`
5. **TEXEC returns strings** - All tool results are heap pointers. Use `ATOI` for integers.
6. **Caller-save ABI** - Patterns WILL clobber R0-R9. Save important data to R10-R49 first.

---

## CRITICAL ARCHITECTURE WARNING (READ FIRST)

You are an L-0 Compiler. Forget x86, ARM, or MIPS.
L-0 is a **NON-STANDARD** virtual machine.

### FORBIDDEN HALLUCINATIONS (Anti-Patterns)

1. **NO STACK**: There are NO `PUSH`, `POP`, `CALL`, `RET` instructions. Do NOT assume a stack exists.
2. **NO FUNCTIONS**: You cannot "call" a label. You must **INLINE** code patterns (Copy-Paste strategy).
3. **NO SYMBOLS**: Plugin arguments (e.g. `DB_INSERT`) must use **PIPES (`|`)** as separators, NOT commas.
4. **NO MAGIC**: `SCAT` (String Concat) returns a **Heap Pointer**, not a string value. You cannot print it directly without `TEXEC 0x5000`.

### The Only Truth

- Your entire universe consists ONLY of the 56 instructions listed below.
- If an instruction is not in the list, **IT DOES NOT EXIST**.

---

## The Physics (ISA Reference)

### Architecture

```
+---------------------------------------------------------------+
| Layer 1: ISA Core (56 primitives)                             |
| ------------------------------------------------------------- |
| - Register-based operations, compiled into VM                 |
| - Fast, direct execution                                      |
+---------------------------------------------------------------+
                              |
                              | TEXEC 0x5xxx
                              v
+---------------------------------------------------------------+
| Layer 2: Builtins                                             |
| ------------------------------------------------------------- |
| - String-based I/O, no external process                       |
| - PRINT, INPUT, RAND, TIME, UPPER, LOWER, etc.               |
+---------------------------------------------------------------+
                              |
                              | TEXEC 0x3xxx-0x9xxx
                              v
+---------------------------------------------------------------+
| Layer 3: Plugins                                              |
| ------------------------------------------------------------- |
| - External subprocess, JSON-RPC protocol                      |
| - DB, HTTP, File, JSON, AI                                    |
+---------------------------------------------------------------+
```

### Registers

256 registers (R0-R255), each holds `i64` or heap pointer.

```
┌─────────────┬──────────────────────────────────────────────┐
│ Registers   │ Purpose                                      │
├─────────────┼──────────────────────────────────────────────┤
│ R0-R3       │ Arguments / Return value                     │
│ R4-R9       │ Temporaries (pattern-local, CLOBBERED)       │
│ R10-R19     │ Preserved across patterns                    │
│ R20-R49     │ Local variables (main program workspace)     │
│ R50-R99     │ Scratch (always clobbered)                   │
│ R100-R109   │ Constants (R100=1, R101=8, etc.)             │
│ R240-R243   │ Governance (target, state, threshold, sim)   │
│ R255        │ I/O buffer (for TEXEC)                       │
└─────────────┴──────────────────────────────────────────────┘
```

### The Holographic ISA (56 Instructions)

> **Every instruction shows its semantic execution. If you can't see the pseudocode, you don't understand the operation.**

| Category | Count | Instructions |
|----------|-------|--------------|
| System | 6 | NOP, HALT, PANIC, DUMP, GAS, ASSERT |
| Registers | 4 | SET, SETS, MOV, SWAP |
| Math | 5 | ADD, SUB, MUL, DIV, MOD |
| Logic | 4 | AND, OR, XOR, NOT |
| Control | 6 | CMP, SCMP, JMP, BEQ, BGT, BLT |
| Memory | 9 | NEW, NEWR, FREE, READ, WRITE, READR, WRITER, STORE64, LOAD64 |
| Batch | 4 | MEMCPY, HLEN, SLICE, MEMSET |
| Extensions | 5 | TEXEC, ITOA, ATOI, SCAT, REGEX |
| Vector | 7 | VNEW, VSET, VGET, VDOT, VSIM, VMAG, VNORM |
| Governance | 4 | LATCH, GUARD, TRAP, YIELD |
| Watermark | 2 | MARK, RESET |

#### System
| Syntax | Semantics | Example |
|--------|-----------|---------|
| `NOP` | `/* no-op */` | `NOP` |
| `HALT` | `exit(0)` | `HALT` |
| `PANIC msg` | `exit(1, msg)` | `PANIC "error"` |
| `DUMP` | `print(regs, heap)` | `DUMP` |
| `GAS r` | `regs[r] = gas_limit` | `GAS 0` |
| `ASSERT r` | `if regs[r] == 0: exit(1)` | `ASSERT 5` |

#### Registers
| Syntax | Semantics | Example |
|--------|-----------|---------|
| `SET r, val` | `regs[r] = val` | `SET 1, 42` |
| `SETS r, "str"` | `regs[r] = heap_alloc("str")` | `SETS 1, "Hello"` |
| `MOV d, s` | `regs[d] = regs[s]` | `MOV 2, 1` |
| `SWAP a, b` | `regs[a], regs[b] = regs[b], regs[a]` | `SWAP 1, 2` |

#### Math (all: dest, s1, s2)
| Syntax | Semantics | Example |
|--------|-----------|---------|
| `ADD d, a, b` | `regs[d] = regs[a] + regs[b]` | `ADD 3, 1, 2` |
| `SUB d, a, b` | `regs[d] = regs[a] - regs[b]` | `SUB 3, 1, 2` |
| `MUL d, a, b` | `regs[d] = regs[a] * regs[b]` | `MUL 3, 1, 2` |
| `DIV d, a, b` | `regs[d] = regs[a] / regs[b]` | `DIV 3, 1, 2` |
| `MOD d, a, b` | `regs[d] = regs[a] % regs[b]` | `MOD 3, 1, 2` |

#### Logic (all: dest, s1, s2 except NOT)
| Syntax | Semantics | Example |
|--------|-----------|---------|
| `AND d, a, b` | `regs[d] = regs[a] & regs[b]` | `AND 3, 1, 2` |
| `OR d, a, b` | `regs[d] = regs[a] \| regs[b]` | `OR 3, 1, 2` |
| `XOR d, a, b` | `regs[d] = regs[a] ^ regs[b]` | `XOR 3, 1, 2` |
| `NOT d, s` | `regs[d] = ~regs[s]` | `NOT 2, 1` |

#### Control Flow
| Syntax | Semantics | Example |
|--------|-----------|---------|
| `CMP a, b` | `flags = compare(regs[a], regs[b])` (integer) | `CMP 1, 2` |
| `SCMP a, b` | `flags = strcmp(heap[regs[a]], heap[regs[b]])` (string) | `SCMP 1, 2` |
| `JMP label` | `pc = label` | `JMP Loop` |
| `BEQ label` | `if flags.EQ: pc = label` | `BEQ Done` |
| `BGT label` | `if flags.GT: pc = label` | `BGT Bigger` |
| `BLT label` | `if flags.LT: pc = label` | `BLT Smaller` |

#### Memory (Heap Operations)
| Syntax | Semantics | Example |
|--------|-----------|---------|
| `NEW d, size` | `regs[d] = heap_alloc(size)` ⚠️ size is **literal** | `NEW 1, 256` |
| `NEWR d, r` | `regs[d] = heap_alloc(regs[r])` (dynamic size) | `NEWR 1, 2` |
| `FREE ptr` | `heap_free(regs[ptr])` | `FREE 1` |
| `READ d, p, off` | `regs[d] = heap[regs[p]][off]` ⚠️ off is **literal** | `READ 2, 1, 0` |
| `WRITE p, off, v` | `heap[regs[p]][off] = regs[v]` ⚠️ off is **literal** | `WRITE 1, 0, 2` |
| `READR d, p, o` | `regs[d] = heap[regs[p]][regs[o]]` | `READR 3, 1, 2` |
| `WRITER p, o, v` | `heap[regs[p]][regs[o]] = regs[v]` | `WRITER 1, 2, 3` |
| `LOAD64 d, p, o` | `regs[d] = *(i64*)&heap[regs[p]][regs[o]*8]` | `LOAD64 3, 1, 0` |
| `STORE64 p, o, v` | `*(i64*)&heap[regs[p]][regs[o]*8] = regs[v]` | `STORE64 1, 0, 3` |

#### Batch Operations
| Syntax | Semantics | Example |
|--------|-----------|---------|
| `MEMCPY d,do,s,so,n` | `memcpy(heap[d]+do, heap[s]+so, regs[n])` | `MEMCPY 1,0,2,0,10` |
| `HLEN d, ptr` | `regs[d] = len(heap[regs[ptr]])` | `HLEN 2, 1` |
| `SLICE d, p, o, n` | `regs[d] = heap_alloc(heap[p][o:o+n])` | `SLICE 3, 1, 0, 5` |
| `MEMSET p, o, n, v` | `memset(heap[p]+o, regs[v], regs[n])` | `MEMSET 1, 0, 10, 0` |

#### Extensions (String & Tools)
| Syntax | Semantics | Example |
|--------|-----------|---------|
| `TEXEC id, arg, d` | `regs[d] = plugins[id](heap[regs[arg]])` | `TEXEC 0x5000, 1, 255` |
| `ITOA d, s` | `regs[d] = heap_alloc(str(regs[s]))` | `ITOA 2, 1` → "42" |
| `ATOI d, s` | `regs[d] = int(heap[regs[s]])` | `ATOI 2, 1` → 42 |
| `SCAT d, a, b` | `regs[d] = heap_alloc(heap[a] + heap[b])` ⚠️ returns **heap ptr** | `SCAT 3, 1, 2` |
| `REGEX d, pat, txt` | `regs[d] = match(heap[pat], heap[txt]) ? 1 : 0` | `REGEX 3, 1, 2` |

#### Vector Operations (Semantic Computing)
| Syntax | Semantics | Example |
|--------|-----------|---------|
| `VNEW d, dims` | `regs[d] = vec_alloc(regs[dims])` | `VNEW 1, 128` |
| `VSET v, i, val` | `vec[regs[v]][regs[i]] = f64(regs[val])` | `VSET 1, 0, 2` |
| `VGET d, v, i` | `regs[d] = bits(vec[regs[v]][regs[i]])` | `VGET 3, 1, 0` |
| `VDOT d, a, b` | `regs[d] = dot(a, b) * 1e6` | `VDOT 3, 1, 2` |
| `VSIM d, a, b` | `regs[d] = cosine_sim(a, b) * 1e6` | `VSIM 3, 1, 2` |
| `VMAG d, v` | `regs[d] = magnitude(v) * 1e6` | `VMAG 2, 1` |
| `VNORM v` | `vec[v] = normalize(vec[v])` | `VNORM 1` |

#### Governance (Autonomous AI Control)
| Syntax | Semantics | Example |
|--------|-----------|---------|
| `LATCH t, th` | `R240=regs[t]; R242=regs[th]` | `LATCH 1, 2` |
| `GUARD s` | `if VSIM(R240, regs[s]) < R242: TRAP` | `GUARD 3` |
| `TRAP code` | `suspend(); emit_context(code)` | `TRAP 1` |
| `YIELD q, d` | `regs[d] = supervisor_query(regs[q])` | `YIELD 1, 2` |

#### Memory Watermark (Arena-style Reset)
| Syntax | Semantics | Example |
|--------|-----------|---------|
| `MARK d` | `regs[d] = heap_watermark()` | `MARK 100` |
| `RESET r` | `heap_reset_to(regs[r])` | `RESET 100` |

---

## The Protocol (Tools)

> **All tools are called via `TEXEC id, arg_reg, dest_reg`**
> - `arg_reg`: Register containing heap pointer to argument string
> - `dest_reg`: Register to store result (heap pointer or integer)

### Builtins (Layer 2)

#### I/O Tools
| ID | Name | Semantics | Example |
|----|------|-----------|---------|
| `0x5000` | PRINT | `print(heap[arg] + "\n")` | `SETS 1, "Hello" → TEXEC 0x5000, 1, 0` |
| `0x5001` | PRINTN | `print(heap[arg])` (no newline) | `SETS 1, "> " → TEXEC 0x5001, 1, 0` |
| `0x5002` | INPUT | `regs[dest] = readline()` | `TEXEC 0x5002, 255, 1` → R1 = user input |

```asm
# Example: Prompt and read input
SETS 1, "Enter name: "
TEXEC 0x5001, 1, 0          # Print prompt (no newline)
TEXEC 0x5002, 255, 2        # Read input -> R2
SETS 3, "Hello, "
SCAT 4, 3, 2                # R4 = "Hello, " + input
TEXEC 0x5000, 4, 0          # Print greeting
```

#### Math Tools
| ID | Name | Args | Semantics | Example |
|----|------|------|-----------|---------|
| `0x1004` | POW | "base,exp" | `regs[dest] = base^exp` | `SETS 1, "2,10" → TEXEC 0x1004, 1, 2` → R2=1024 |
| `0x5005` | ABS | "n" | `regs[dest] = \|n\|` | `SETS 1, "-42" → TEXEC 0x5005, 1, 2` → R2=42 |
| `0x5006` | MIN | "a,b" | `regs[dest] = min(a,b)` | `SETS 1, "5,3" → TEXEC 0x5006, 1, 2` → R2=3 |
| `0x5007` | MAX | "a,b" | `regs[dest] = max(a,b)` | `SETS 1, "5,3" → TEXEC 0x5007, 1, 2` → R2=5 |
| `0x5003` | RAND | "max" | `regs[dest] = rand(0..max)` | `SETS 1, "100" → TEXEC 0x5003, 1, 2` → R2=random |

```asm
# Example: Generate random number 1-6 (dice)
SETS 1, "6"
TEXEC 0x5003, 1, 2          # R2 = 0..5
SET 3, 1
ADD 4, 2, 3                 # R4 = 1..6
ITOA 5, 4
TEXEC 0x5000, 5, 0          # Print dice result
```

#### String Tools
| ID | Name | Args | Semantics | Example |
|----|------|------|-----------|---------|
| `0x500A` | SUBSTR | "str,start,len" | `regs[dest] = str[start:start+len]` | `SETS 1, "Hello,1,3" → TEXEC 0x500A, 1, 2` → "ell" |
| `0x500B` | SPLIT | "str,delim,idx" | `regs[dest] = str.split(delim)[idx]` | `SETS 1, "a:b:c,:,1" → TEXEC 0x500B, 1, 2` → "b" |
| `0x500C` | UPPER | "str" | `regs[dest] = str.upper()` | `SETS 1, "hello" → TEXEC 0x500C, 1, 2` → "HELLO" |
| `0x500D` | LOWER | "str" | `regs[dest] = str.lower()` | `SETS 1, "HELLO" → TEXEC 0x500D, 1, 2` → "hello" |
| `0x500E` | TRIM | "str" | `regs[dest] = str.trim()` | `SETS 1, "  hi  " → TEXEC 0x500E, 1, 2` → "hi" |

```asm
# Example: Parse CSV field
SETS 1, "Alice,30,Engineer"
SETS 2, ","
# Get second field (index 1)
SCAT 3, 1, 2                # "Alice,30,Engineer,"
SETS 4, ",1"
SCAT 5, 3, 4                # "Alice,30,Engineer,,,1"
TEXEC 0x500B, 5, 6          # R6 = "30"
```

#### System Tools
| ID | Name | Semantics | Example |
|----|------|-----------|---------|
| `0x5008` | TIME | `regs[dest] = unix_timestamp()` | `TEXEC 0x5008, 255, 1` → R1 = "1706745600" |

```asm
# Example: Print current timestamp
TEXEC 0x5008, 255, 1        # R1 = timestamp string
SETS 2, "Current time: "
SCAT 3, 2, 1
TEXEC 0x5000, 3, 0          # "Current time: 1706745600"
```

### Plugins (Layer 3)

#### File I/O (0x4xxx)
| ID | Name | Args | Semantics |
|----|------|------|-----------|
| `0x4000` | FILE_READ | "path" | `regs[dest] = read_file(path)` |
| `0x4001` | FILE_WRITE | "path\|content" | `write_file(path, content)` |
| `0x4003` | FILE_DELETE | "path" | `delete_file(path)` |
| `0x4004` | FILE_LIST | "dir" | `regs[dest] = list_dir(dir)` |
| `0x4005` | FILE_EXISTS | "path" | `regs[dest] = exists(path) ? "true" : "false"` |

```asm
# Example: Read and print file
SETS 1, "config.txt"
TEXEC 0x4000, 1, 2          # R2 = file contents
TEXEC 0x5000, 2, 0          # Print contents

# Example: Write file
SETS 1, "output.txt|Hello, World!"
TEXEC 0x4001, 1, 0          # Write to output.txt

# Example: Check if file exists
SETS 1, "data.json"
TEXEC 0x4005, 1, 2          # R2 = "true" or "false"
SETS 3, "true"
SCMP 2, 3                   # String compare
BEQ FileExists
```

#### JSON (0x6xxx)
| ID | Name | Args | Semantics |
|----|------|------|-----------|
| `0x6000` | JSON_LOAD | "path" | `regs[dest] = json.load(path)` |
| `0x6001` | JSON_SAVE | "path\|json" | `json.save(path, json)` |
| `0x6002` | JSON_GET | "key" | `regs[dest] = memory_store[key]` (from HashMap) |
| `0x6003` | JSON_SET | "key\|value" | `memory_store[key] = value` |
| `0x6004` | JSON_PARSE | "json_str\|field" | `regs[dest] = extract_field(json_str, field)` |

```asm
# Example: Parse JSON and extract field
# JSON_PARSE format: "json_str|field_path"
SETS 1, "{\"name\":\"Alice\",\"age\":30}|name"
TEXEC 0x6004, 1, 2          # JSON_PARSE -> R2 = "Alice"
TEXEC 0x5000, 2, 0          # Print "Alice"

# Extract another field
SETS 3, "{\"name\":\"Alice\",\"age\":30}|age"
TEXEC 0x6004, 3, 4          # JSON_PARSE -> R4 = "30"
```

#### Database (0x9xxx)
| ID | Name | Args | Semantics |
|----|------|------|-----------|
| `0x9000` | DB_INIT | "path" | `db = sqlite.open(path)` |
| `0x9001` | DB_CREATE_TABLE | "table\|schema" | `CREATE TABLE IF NOT EXISTS table (schema)` |
| `0x9002` | DB_INSERT | "table\|{json}" | `INSERT INTO table VALUES(json)` → returns ID |
| `0x9003` | DB_SELECT | "table[\|condition]" | `SELECT * FROM table [WHERE condition]` → JSON array |
| `0x9004` | DB_UPDATE | "table\|condition\|updates" | `UPDATE table SET updates WHERE condition` |
| `0x9005` | DB_DELETE | "table\|condition" | `DELETE FROM table WHERE condition` |
| `0x9006` | DB_DROP_TABLE | "table" | `DROP TABLE table` |
| `0x9007` | DB_LIST_TABLES | - | `regs[dest] = ["table1", "table2", ...]` |

```asm
# Example: Complete CRUD operations
# 1. Initialize
SETS 255, "app.db"
TEXEC 0x9000, 255, 0

# 2. Create table
SETS 255, "users|id INTEGER PRIMARY KEY AUTOINCREMENT,name TEXT,email TEXT"
TEXEC 0x9001, 255, 0

# 3. Insert
SETS 255, "users|{\"name\":\"Alice\",\"email\":\"alice@example.com\"}"
TEXEC 0x9002, 255, 0        # R0 = inserted ID (e.g., 1)

# 4. Select all
SETS 255, "users"
TEXEC 0x9003, 255, 1        # R1 = [{"id":1,"name":"Alice",...}]
TEXEC 0x5000, 1, 0          # Print JSON array

# 5. Select with condition
SETS 255, "users|id=1"
TEXEC 0x9003, 255, 2        # R2 = [{"id":1,...}]

# 6. Update
SETS 255, "users|id=1|name=Bob"
TEXEC 0x9004, 255, 0

# 7. Delete
SETS 255, "users|id=1"
TEXEC 0x9005, 255, 0

# 8. Select with ORDER BY (requires 1=1 prefix)
SETS 255, "users|1=1 ORDER BY id DESC"
TEXEC 0x9003, 255, 3        # R3 = sorted results
```

> **⚠️ DB_SELECT ORDER BY Syntax**: The condition field goes into the SQL WHERE clause.
> To use ORDER BY without a real condition, prefix with `1=1`:
> - `"table|1=1 ORDER BY col DESC"` → `SELECT * FROM table WHERE 1=1 ORDER BY col DESC`
> - `"table|active=1 ORDER BY created_at"` → `SELECT * FROM table WHERE active=1 ORDER BY created_at`

#### HTTP (0x8xxx)
| ID | Name | Args | Semantics |
|----|------|------|-----------|
| `0x8000` | HTTP_INIT | "port" | `server.bind(port)` |
| `0x8001` | HTTP_ROUTE | "METHOD path\|response" | `routes[method+path] = response` |
| `0x8002` | HTTP_SERVE | "count" | `serve(count)` then return ⚠️ NOT infinite |
| `0x8003` | HTTP_SERVE_ONCE | - | `serve(1)` |
| `0x8004` | HTTP_REQUEST | "url[\|method[\|body]]" | `regs[dest] = http_request(...)` |
| `0x8005` | HTTP_LIST_ROUTES | - | `regs[dest] = [routes...]` |
| `0x8006` | HTTP_STATIC | "dir" | `static_dir = dir` |
| `0x8008` | HTTP_LISTEN | - | Block until request, return `{method,path,body,addr}` |
| `0x8009` | HTTP_SEND | "response" | Send response for pending request |

```asm
# Example: Static server with routes
SETS 255, "8080"
TEXEC 0x8000, 255, 0        # HTTP_INIT

SETS 255, "GET /|<h1>Welcome</h1>"
TEXEC 0x8001, 255, 0        # Route: GET / -> HTML

SETS 255, "GET /api/status|{\"ok\":true}"
TEXEC 0x8001, 255, 0        # Route: GET /api/status -> JSON

SETS 255, "100"
TEXEC 0x8002, 255, 0        # Serve 100 requests
HALT

# Example: Dynamic response server
SETS 255, "8080"
TEXEC 0x8000, 255, 0

DynamicLoop:
    TEXEC 0x8008, 255, 0    # HTTP_LISTEN -> R0 = request JSON
    # Parse R0, build response in R10
    SETS 10, "{\"time\":"
    TEXEC 0x5008, 255, 11   # Get timestamp
    SCAT 12, 10, 11
    SETS 13, "}"
    SCAT 14, 12, 13         # R14 = {"time":...}
    TEXEC 0x8009, 14, 0     # HTTP_SEND
    JMP DynamicLoop
```

#### AI (0x3xxx) - Optional
| ID | Name | Args | Semantics |
|----|------|------|-----------|
| `0x3000` | AI_EMBED | "text" | `regs[dest] = embedding_vector(text)` |
| `0x3004` | AI_GENERATE | "prompt" | `regs[dest] = llm_generate(prompt)` |

### Plugin Protocol

**Request** (VM -> Plugin):
```json
{"method": "select", "args": ["users|id=1"]}
```

**Response** (Plugin -> VM):
```json
{"ok": true, "value": [{"id": 1, "name": "Alice"}]}
```

**Error**:
```json
{"ok": false, "error": "table not found"}
```

**TEXEC Return Values:**

All `TEXEC` calls return a heap pointer to a JSON response string. The VM automatically extracts the `value` field for you.

| Tool Type | Return Value | Example |
|-----------|--------------|---------|
| DB_SELECT | JSON array string | `[{"id":1,"name":"Alice"}]` |
| DB_INSERT | Last insert ID (integer as string) | `"1"` |
| FILE_READ | File contents | `"Hello, World!"` |
| FILE_EXISTS | Boolean as string | `"true"` or `"false"` |
| HTTP_SERVE | Status message | `"served 100 requests"` |
| PRINT/PRINTN | Empty string | `""` |
| TIME | Unix timestamp string | `"1706745600"` |

**Error Handling:**

When a tool fails, the response contains `{"ok":false,"error":"..."}`. The VM stores `-1` in the destination register on error.

```asm
# Check for errors after TEXEC
SETS 255, "nonexistent.txt"
TEXEC 0x4000, 255, 10           # FILE_READ -> R10

SET 50, -1
CMP 10, 50
BEQ FileError                   # R10 == -1 means error

# Success: R10 contains file contents
TEXEC 0x5000, 10, 0             # Print contents
JMP Continue

FileError:
SETS 255, "File not found!"
TEXEC 0x5000, 255, 0
HALT

Continue:
# ...
```

---

## The Knowledge (Standard Library Patterns)

> **CRITICAL**: L-0 has NO function calls. These are INLINE PATTERNS.
> You MUST copy the pattern body and rename ALL labels with unique suffix.

### How to Use Patterns

```asm
# Step 1: Save any important data from R0-R9
MOV 20, 5              # Save R5 -> R20 if needed

# Step 2: Set inputs
SET 0, 10              # R0 = input value

# Step 3: Inline the pattern (copy & rename labels)
# ---- BEGIN fibonacci_abc1 ----
SET 4, 0
CMP 0, 4
BEQ fibonacci_abc1_zero
# ... rest of pattern with _abc1 suffix ...
fibonacci_abc1_done:
# ---- END fibonacci_abc1 ----

# Step 4: Use result in R0
ITOA 255, 0
TEXEC 0x5000, 255, 0
```

### String Patterns

```asm
# ==================================================================
# strlen: Get string length
# ==================================================================
# Input:  R0 = string pointer
# Output: R0 = length (bytes)
# Clobbers: none
# ------------------------------------------------------------------
strlen:
    HLEN 0, 0              # R0 = heap length of string at R0
strlen_done:

# ==================================================================
# strcat: Concatenate two strings
# ==================================================================
# Input:  R0 = string1 pointer, R1 = string2 pointer
# Output: R0 = new string pointer (s1 + s2)
# Clobbers: R4
# ------------------------------------------------------------------
strcat:
    MOV 4, 0               # Save s1
    SCAT 0, 4, 1           # R0 = concat(s1, s2)
strcat_done:

# ==================================================================
# substr: Extract substring
# ==================================================================
# Input:  R0 = string pointer, R1 = offset, R2 = length
# Output: R0 = new string pointer (substring)
# Clobbers: none
# ------------------------------------------------------------------
substr:
    SLICE 0, 0, 1, 2       # R0 = slice(R0, offset=R1, len=R2)
substr_done:

# ==================================================================
# strcmp: Compare two strings (lexicographic)
# ==================================================================
# NOTE: For simple equality checks, use SCMP instruction instead:
#       SCMP 0, 1    # Compare strings in R0 and R1, sets flags
#       BEQ equal    # Jump if strings are equal
#
# This pattern is for when you need the -1/0/1 result value.
# Input:  R0 = string1 pointer, R1 = string2 pointer
# Output: R0 = 0 if equal, -1 if s1<s2, 1 if s1>s2
# Clobbers: R4, R5, R6, R7, R8
# ------------------------------------------------------------------
strcmp:
    MOV 4, 0               # R4 = s1
    MOV 5, 1               # R5 = s2
    HLEN 6, 4              # R6 = len(s1)
    HLEN 7, 5              # R7 = len(s2)
    SET 8, 0               # R8 = index

strcmp_loop:
    CMP 8, 6
    BEQ strcmp_s1_end      # s1 exhausted
    CMP 8, 7
    BEQ strcmp_s2_end      # s2 exhausted

    READR 0, 4, 8          # R0 = s1[i]
    READR 1, 5, 8          # R1 = s2[i]
    CMP 0, 1
    BLT strcmp_less
    BGT strcmp_greater

    SET 50, 1
    ADD 8, 8, 50           # i++
    JMP strcmp_loop

strcmp_s1_end:
    CMP 8, 7
    BEQ strcmp_equal       # Both exhausted = equal
    SET 0, -1              # s1 shorter = less
    JMP strcmp_done

strcmp_s2_end:
    SET 0, 1               # s2 shorter = s1 greater
    JMP strcmp_done

strcmp_less:
    SET 0, -1
    JMP strcmp_done

strcmp_greater:
    SET 0, 1
    JMP strcmp_done

strcmp_equal:
    SET 0, 0
strcmp_done:

# ==================================================================
# strchr: Find character in string
# ==================================================================
# Input:  R0 = string pointer, R1 = character (byte value)
# Output: R0 = index of first occurrence, or -1 if not found
# Clobbers: R4, R5, R6
# ------------------------------------------------------------------
strchr:
    MOV 4, 0               # R4 = string
    HLEN 5, 4              # R5 = length
    SET 6, 0               # R6 = index

strchr_loop:
    CMP 6, 5
    BEQ strchr_notfound    # End of string

    READR 0, 4, 6          # R0 = str[i]
    CMP 0, 1
    BEQ strchr_found

    SET 50, 1
    ADD 6, 6, 50           # i++
    JMP strchr_loop

strchr_found:
    MOV 0, 6               # R0 = index
    JMP strchr_done

strchr_notfound:
    SET 0, -1
strchr_done:

# ==================================================================
# itoa: Integer to ASCII string
# ==================================================================
# Input:  R0 = integer value
# Output: R0 = string pointer
# Clobbers: R4
# ------------------------------------------------------------------
itoa:
    MOV 4, 0               # Save value
    ITOA 0, 4              # R0 = string representation
itoa_done:

# ==================================================================
# atoi: ASCII string to integer
# ==================================================================
# Input:  R0 = string pointer
# Output: R0 = integer value
# Clobbers: none
# ------------------------------------------------------------------
atoi:
    ATOI 0, 0              # R0 = integer from string
atoi_done:

# ==================================================================
# print: Print string with newline
# ==================================================================
# Input:  R0 = string pointer
# Output: none
# Clobbers: none
# ------------------------------------------------------------------
print:
    TEXEC 0x5000, 0, 255   # Print R0
print_done:

# ==================================================================
# print_int: Print integer
# ==================================================================
# Input:  R0 = integer value
# Output: none
# Clobbers: R4
# ------------------------------------------------------------------
print_int:
    ITOA 4, 0              # R4 = string of R0
    TEXEC 0x5000, 4, 255   # Print R4
print_int_done:

# ==================================================================
# sb_init: Initialize String Builder
# ==================================================================
# Input:  R0 = capacity (bytes)
# Output: R0 = buffer pointer, R1 = current length (0)
# Clobbers: R50
# ------------------------------------------------------------------
sb_init:
    MOV 50, 0             # R50 = capacity (save R0)
    NEWR 0, 50            # R0 = new heap allocation (size from R50)
    SET 1, 0              # Length = 0
sb_init_done:

# ==================================================================
# sb_append: Append string to buffer
# ==================================================================
# Input:  R0 = buffer ptr, R1 = current len, R2 = string to append
# Output: R1 = new length
# Clobbers: R4, R5
# ------------------------------------------------------------------
sb_append:
    # Get length of string to append
    HLEN 4, 2             # R4 = len(string)
    SET 5, 0              # R5 = 0 (source offset)

    # Memcpy(dest=buf, dest_off=len, src=str, src_off=0, count=str_len)
    MEMCPY 0, 1, 2, 5, 4  # dst=R0, doff=R1, src=R2, soff=R5(0), len=R4

    # Update length
    ADD 1, 1, 4           # new_len = old_len + str_len
sb_append_done:

# ==================================================================
# sb_finish: Convert buffer to string (Slice)
# ==================================================================
# Input:  R0 = buffer ptr, R1 = length
# Output: R0 = new string pointer
# Note: Caller should FREE the original buffer if needed
# Clobbers: R50
# ------------------------------------------------------------------
sb_finish:
    SET 50, 0             # R50 = 0 (offset)
    SLICE 0, 0, 50, 1     # Create exact-sized string (ptr=R0, off=R50(0), len=R1)
sb_finish_done:
```

### Math Patterns

```asm
# ==================================================================
# abs: Absolute value
# ==================================================================
# Input:  R0 = integer value
# Output: R0 = |R0|
# Clobbers: R4
# ------------------------------------------------------------------
abs:
    SET 4, 0
    CMP 0, 4
    BGT abs_done           # Already positive
    BEQ abs_done           # Zero
    SET 4, -1
    MUL 0, 0, 4            # Negate
abs_done:

# ==================================================================
# min: Minimum of two values
# ==================================================================
# Input:  R0 = a, R1 = b
# Output: R0 = min(a, b)
# Clobbers: none
# ------------------------------------------------------------------
min:
    CMP 0, 1
    BLT min_done           # R0 < R1, keep R0
    MOV 0, 1               # R0 >= R1, use R1
min_done:

# ==================================================================
# max: Maximum of two values
# ==================================================================
# Input:  R0 = a, R1 = b
# Output: R0 = max(a, b)
# Clobbers: none
# ------------------------------------------------------------------
max:
    CMP 0, 1
    BGT max_done           # R0 > R1, keep R0
    MOV 0, 1               # R0 <= R1, use R1
max_done:

# ==================================================================
# pow: Power function (base^exp)
# ==================================================================
# Input:  R0 = base, R1 = exponent (non-negative)
# Output: R0 = base^exp
# Clobbers: R4, R5, R6
# ------------------------------------------------------------------
pow:
    SET 4, 0               # Check exp
    CMP 1, 4
    BEQ pow_zero           # exp == 0 -> return 1

    MOV 4, 0               # R4 = base
    MOV 5, 1               # R5 = exp (counter)
    SET 6, 1               # R6 = result

pow_loop:
    SET 50, 0
    CMP 5, 50
    BEQ pow_finish

    MUL 6, 6, 4            # result *= base
    SET 50, 1
    SUB 5, 5, 50           # exp--
    JMP pow_loop

pow_zero:
    SET 0, 1
    JMP pow_done

pow_finish:
    MOV 0, 6               # R0 = result
pow_done:

# ==================================================================
# fibonacci: Fibonacci number (F_n)
# ==================================================================
# Input:  R0 = n (0-indexed)
# Output: R0 = F_n
# Clobbers: R4, R5, R6, R7
# ------------------------------------------------------------------
fibonacci:
    SET 4, 0
    CMP 0, 4
    BEQ fibonacci_zero     # F_0 = 0

    SET 4, 1
    CMP 0, 4
    BEQ fibonacci_one      # F_1 = 1

    MOV 4, 0               # R4 = n (target)
    SET 5, 0               # R5 = F_(i-2) = 0
    SET 6, 1               # R6 = F_(i-1) = 1
    SET 7, 1               # R7 = current i

fibonacci_loop:
    CMP 7, 4
    BEQ fibonacci_finish   # i == n, done

    ADD 50, 5, 6           # R50 = F_(i-2) + F_(i-1)
    MOV 5, 6               # F_(i-2) = F_(i-1)
    MOV 6, 50              # F_(i-1) = new value
    SET 51, 1
    ADD 7, 7, 51           # i++
    JMP fibonacci_loop

fibonacci_zero:
    SET 0, 0
    JMP fibonacci_done

fibonacci_one:
    SET 0, 1
    JMP fibonacci_done

fibonacci_finish:
    MOV 0, 6               # R0 = F_n
fibonacci_done:

# ==================================================================
# gcd: Greatest Common Divisor (Euclidean algorithm)
# ==================================================================
# Input:  R0 = a, R1 = b
# Output: R0 = gcd(a, b)
# Clobbers: R4, R5
# ------------------------------------------------------------------
gcd:
    MOV 4, 0               # R4 = a
    MOV 5, 1               # R5 = b

gcd_loop:
    SET 50, 0
    CMP 5, 50
    BEQ gcd_finish         # b == 0, return a

    MOD 50, 4, 5           # R50 = a % b
    MOV 4, 5               # a = b
    MOV 5, 50              # b = a % b
    JMP gcd_loop

gcd_finish:
    MOV 0, 4               # R0 = gcd
gcd_done:

# ==================================================================
# is_prime: Check if number is prime
# ==================================================================
# Input:  R0 = n
# Output: R0 = 1 if prime, 0 if not
# Clobbers: R4, R5, R6
# ------------------------------------------------------------------
is_prime:
    SET 4, 2
    CMP 0, 4
    BLT is_prime_no        # n < 2, not prime
    BEQ is_prime_yes       # n == 2, prime

    SET 4, 2
    MOD 5, 0, 4            # R5 = n % 2
    SET 6, 0
    CMP 5, 6
    BEQ is_prime_no        # n % 2 == 0, not prime

    MOV 6, 0               # R6 = n
    SET 4, 3               # R4 = divisor (start at 3)

is_prime_loop:
    MUL 5, 4, 4            # R5 = divisor^2
    CMP 5, 6
    BGT is_prime_yes       # divisor^2 > n, is prime

    MOD 5, 6, 4            # R5 = n % divisor
    SET 50, 0
    CMP 5, 50
    BEQ is_prime_no        # n % divisor == 0, not prime

    SET 50, 2
    ADD 4, 4, 50           # divisor += 2
    JMP is_prime_loop

is_prime_yes:
    SET 0, 1
    JMP is_prime_done

is_prime_no:
    SET 0, 0
is_prime_done:

# ==================================================================
# factorial: Factorial (n!)
# ==================================================================
# Input:  R0 = n (non-negative)
# Output: R0 = n!
# Clobbers: R4, R5
# ------------------------------------------------------------------
factorial:
    SET 4, 0
    CMP 0, 4
    BEQ factorial_base     # 0! = 1

    SET 4, 1
    CMP 0, 4
    BEQ factorial_base     # 1! = 1

    MOV 4, 0               # R4 = n (counter)
    SET 5, 1               # R5 = result

factorial_loop:
    SET 50, 1
    CMP 4, 50
    BEQ factorial_finish   # counter == 1, done

    MUL 5, 5, 4            # result *= counter
    SUB 4, 4, 50           # counter--
    JMP factorial_loop

factorial_base:
    SET 0, 1
    JMP factorial_done

factorial_finish:
    MUL 0, 5, 4            # Include final counter (1)
factorial_done:

# ==================================================================
# lcm: Least Common Multiple
# ==================================================================
# Input:  R0 = a, R1 = b
# Output: R0 = lcm(a, b)
# Clobbers: R4, R5, R6, R7
# Note: Inlines GCD algorithm (L-0 has no function calls)
# ------------------------------------------------------------------
lcm:
    MOV 6, 0               # R6 = a (save)
    MOV 7, 1               # R7 = b (save)

    # Inline GCD: compute gcd(a, b) into R4
    MOV 4, 0               # R4 = a
    MOV 5, 1               # R5 = b

lcm_gcd_loop:
    SET 50, 0
    CMP 5, 50
    BEQ lcm_gcd_done       # b == 0, gcd in R4

    MOD 50, 4, 5           # R50 = a % b
    MOV 4, 5               # a = b
    MOV 5, 50              # b = a % b
    JMP lcm_gcd_loop

lcm_gcd_done:
    # R4 = gcd(a, b)
    # lcm = (a * b) / gcd
    MUL 5, 6, 7            # R5 = a * b
    DIV 0, 5, 4            # R0 = (a * b) / gcd
lcm_done:

# ==================================================================
# sqrt_int: Integer square root (floor)
# ==================================================================
# Input:  R0 = n (non-negative)
# Output: R0 = floor(sqrt(n))
# Clobbers: R4, R5, R6
# ------------------------------------------------------------------
sqrt_int:
    SET 4, 0
    CMP 0, 4
    BEQ sqrt_int_zero      # sqrt(0) = 0

    MOV 4, 0               # R4 = n
    SET 5, 1               # R5 = guess

sqrt_int_loop:
    MUL 6, 5, 5            # R6 = guess^2
    CMP 6, 4
    BGT sqrt_int_finish    # guess^2 > n, prev was answer

    SET 50, 1
    ADD 5, 5, 50           # guess++
    JMP sqrt_int_loop

sqrt_int_zero:
    SET 0, 0
    JMP sqrt_int_done

sqrt_int_finish:
    SET 50, 1
    SUB 0, 5, 50           # R0 = guess - 1
sqrt_int_done:
```

### Array Patterns

```asm
# Arrays use STORE64/LOAD64. Each element is 8 bytes.
# Create with NEW(count * 8).

# ==================================================================
# array_new: Create new array
# ==================================================================
# Input:  R0 = number of elements
# Output: R0 = array pointer
# Clobbers: R4
# ------------------------------------------------------------------
array_new:
    SET 4, 8
    MUL 4, 0, 4            # R4 = count * 8 (bytes)
    NEWR 0, 4              # R0 = new heap allocation (size from R4)
array_new_done:

# ==================================================================
# array_get: Get element at index
# ==================================================================
# Input:  R0 = array pointer, R1 = index
# Output: R0 = element value
# Clobbers: none
# ------------------------------------------------------------------
array_get:
    LOAD64 0, 0, 1         # R0 = arr[R1] (8-byte element)
array_get_done:

# ==================================================================
# array_set: Set element at index
# ==================================================================
# Input:  R0 = array pointer, R1 = index, R2 = value
# Output: none
# Clobbers: none
# ------------------------------------------------------------------
array_set:
    STORE64 0, 1, 2        # arr[R1] = R2
array_set_done:

# ==================================================================
# array_sum: Sum all elements
# ==================================================================
# Input:  R0 = array pointer, R1 = length
# Output: R0 = sum of elements
# Clobbers: R4, R5, R6
# ------------------------------------------------------------------
array_sum:
    MOV 4, 0               # R4 = array
    SET 5, 0               # R5 = index
    SET 6, 0               # R6 = sum

array_sum_loop:
    CMP 5, 1
    BEQ array_sum_finish   # index == length, done

    LOAD64 50, 4, 5        # R50 = arr[i]
    ADD 6, 6, 50           # sum += arr[i]
    SET 51, 1
    ADD 5, 5, 51           # i++
    JMP array_sum_loop

array_sum_finish:
    MOV 0, 6               # R0 = sum
array_sum_done:

# ==================================================================
# array_find: Find element in array
# ==================================================================
# Input:  R0 = array pointer, R1 = length, R2 = value to find
# Output: R0 = index of first occurrence, or -1 if not found
# Clobbers: R4, R5, R6
# ------------------------------------------------------------------
array_find:
    MOV 4, 0               # R4 = array
    SET 5, 0               # R5 = index

array_find_loop:
    CMP 5, 1
    BEQ array_find_notfound # index == length, not found

    LOAD64 6, 4, 5         # R6 = arr[i]
    CMP 6, 2
    BEQ array_find_found

    SET 50, 1
    ADD 5, 5, 50           # i++
    JMP array_find_loop

array_find_found:
    MOV 0, 5               # R0 = index
    JMP array_find_done

array_find_notfound:
    SET 0, -1
array_find_done:

# ==================================================================
# bubble_sort: Sort array (ascending, in-place)
# ==================================================================
# Input:  R0 = array pointer, R1 = length
# Output: none (array sorted in-place)
# Clobbers: R4, R5, R6, R7, R8, R9
# ------------------------------------------------------------------
bubble_sort:
    MOV 4, 0               # R4 = array
    MOV 5, 1               # R5 = length
    SET 50, 1
    SUB 6, 5, 50           # R6 = length - 1 (outer bound)

bubble_sort_outer:
    SET 51, 0
    CMP 6, 51
    BEQ bubble_sort_done   # outer == 0, done

    SET 7, 0               # R7 = inner index
    SET 9, 0               # R9 = swapped flag

bubble_sort_inner:
    CMP 7, 6
    BEQ bubble_sort_inner_done

    LOAD64 50, 4, 7        # R50 = arr[j]
    SET 51, 1
    ADD 8, 7, 51           # R8 = j + 1
    LOAD64 51, 4, 8        # R51 = arr[j+1]

    CMP 50, 51
    BLT bubble_sort_no_swap
    BEQ bubble_sort_no_swap

    STORE64 4, 7, 51       # arr[j] = arr[j+1]
    STORE64 4, 8, 50       # arr[j+1] = arr[j]
    SET 9, 1               # swapped = true

bubble_sort_no_swap:
    SET 52, 1
    ADD 7, 7, 52           # j++
    JMP bubble_sort_inner

bubble_sort_inner_done:
    SET 51, 0
    CMP 9, 51
    BEQ bubble_sort_done   # No swaps = already sorted

    SET 51, 1
    SUB 6, 6, 51           # outer bound--
    JMP bubble_sort_outer

bubble_sort_done:

# ==================================================================
# reverse: Reverse array in-place
# ==================================================================
# Input:  R0 = array pointer, R1 = length
# Output: none (array reversed in-place)
# Clobbers: R4, R5, R6, R7
# ------------------------------------------------------------------
reverse:
    MOV 4, 0               # R4 = array
    SET 5, 0               # R5 = left index
    SET 50, 1
    SUB 6, 1, 50           # R6 = right index (length - 1)

reverse_loop:
    CMP 5, 6
    BGT reverse_done       # left > right, done
    BEQ reverse_done       # left == right, done

    LOAD64 7, 4, 5         # R7 = arr[left]
    LOAD64 50, 4, 6        # R50 = arr[right]
    STORE64 4, 5, 50       # arr[left] = arr[right]
    STORE64 4, 6, 7        # arr[right] = arr[left]

    SET 51, 1
    ADD 5, 5, 51           # left++
    SUB 6, 6, 51           # right--
    JMP reverse_loop

reverse_done:

# ==================================================================
# array_fill: Fill array with value
# ==================================================================
# Input:  R0 = array pointer, R1 = length, R2 = value
# Output: none
# Clobbers: R4, R5
# ------------------------------------------------------------------
array_fill:
    MOV 4, 0               # R4 = array
    SET 5, 0               # R5 = index

array_fill_loop:
    CMP 5, 1
    BEQ array_fill_done    # index == length, done

    STORE64 4, 5, 2        # arr[i] = value
    SET 50, 1
    ADD 5, 5, 50           # i++
    JMP array_fill_loop

array_fill_done:

# ==================================================================
# array_copy: Copy array
# ==================================================================
# Input:  R0 = dest array, R1 = src array, R2 = length
# Output: none
# Clobbers: R4, R5, R6
# ------------------------------------------------------------------
array_copy:
    MOV 4, 0               # R4 = dest
    MOV 5, 1               # R5 = src
    SET 6, 0               # R6 = index

array_copy_loop:
    CMP 6, 2
    BEQ array_copy_done    # index == length, done

    LOAD64 50, 5, 6        # R50 = src[i]
    STORE64 4, 6, 50       # dest[i] = R50
    SET 51, 1
    ADD 6, 6, 51           # i++
    JMP array_copy_loop

array_copy_done:

# ==================================================================
# array_min: Find minimum element
# ==================================================================
# Input:  R0 = array pointer, R1 = length (must be > 0)
# Output: R0 = minimum value
# Clobbers: R4, R5, R6
# ------------------------------------------------------------------
array_min:
    MOV 4, 0               # R4 = array
    SET 5, 0
    LOAD64 6, 4, 5         # R6 = arr[0] (initial min)
    SET 5, 1               # R5 = index (start at 1)

array_min_loop:
    CMP 5, 1
    BEQ array_min_finish   # index == length, done

    LOAD64 50, 4, 5        # R50 = arr[i]
    CMP 50, 6
    BGT array_min_skip     # arr[i] > min, skip
    BEQ array_min_skip
    MOV 6, 50              # new min

array_min_skip:
    SET 51, 1
    ADD 5, 5, 51           # i++
    JMP array_min_loop

array_min_finish:
    MOV 0, 6               # R0 = min
array_min_done:

# ==================================================================
# array_max: Find maximum element
# ==================================================================
# Input:  R0 = array pointer, R1 = length (must be > 0)
# Output: R0 = maximum value
# Clobbers: R4, R5, R6
# ------------------------------------------------------------------
array_max:
    MOV 4, 0               # R4 = array
    SET 5, 0
    LOAD64 6, 4, 5         # R6 = arr[0] (initial max)
    SET 5, 1               # R5 = index (start at 1)

array_max_loop:
    CMP 5, 1
    BEQ array_max_finish   # index == length, done

    LOAD64 50, 4, 5        # R50 = arr[i]
    CMP 50, 6
    BLT array_max_skip     # arr[i] < max, skip
    BEQ array_max_skip
    MOV 6, 50              # new max

array_max_skip:
    SET 51, 1
    ADD 5, 5, 51           # i++
    JMP array_max_loop

array_max_finish:
    MOV 0, 6               # R0 = max
array_max_done:

# ==================================================================
# array_print: Print array (for debugging)
# ==================================================================
# Input:  R0 = array pointer, R1 = length
# Output: none (prints to stdout)
# Clobbers: R4, R5, R6
# ------------------------------------------------------------------
array_print:
    MOV 4, 0               # R4 = array
    SET 5, 0               # R5 = index

    SETS 255, "["
    TEXEC 0x5001, 255, 255 # Print "["

array_print_loop:
    CMP 5, 1
    BEQ array_print_finish # index == length, done

    LOAD64 6, 4, 5         # R6 = arr[i]
    ITOA 255, 6            # Convert to string
    TEXEC 0x5001, 255, 255 # Print element

    SET 50, 1
    ADD 5, 5, 50           # i++
    CMP 5, 1
    BEQ array_print_no_comma # Last element, no comma

    SETS 255, ", "
    TEXEC 0x5001, 255, 255 # Print ", "

array_print_no_comma:
    JMP array_print_loop

array_print_finish:
    SETS 255, "]"
    TEXEC 0x5000, 255, 255 # Print "]" with newline
array_print_done:
```

---

## Examples

### Hello World
```asm
SETS 1, Hello, L-0!
TEXEC 0x5000, 1, 255
HALT
```

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

### Database CRUD
```asm
# Initialize
SETS 255, "data.db"
TEXEC 0x9000, 255, 0   # DB_INIT

# Create table
SETS 255, "users|id INTEGER PRIMARY KEY,name TEXT"
TEXEC 0x9001, 255, 0   # DB_CREATE_TABLE

# Insert
SETS 255, "users|{\"name\":\"Alice\"}"
TEXEC 0x9002, 255, 0   # DB_INSERT

# Select all
SETS 255, "users"
TEXEC 0x9003, 255, 0   # DB_SELECT -> R0 = JSON array
TEXEC 0x5000, 0, 255   # Print result

HALT
```

### Static HTTP Server
```asm
# Initialize
SETS 255, "8080"
TEXEC 0x8000, 255, 0   # HTTP_INIT

# Register route
SETS 255, "GET /|<html><body>Hello World</body></html>"
TEXEC 0x8001, 255, 0   # HTTP_ROUTE

# Serve
SETS 255, "10"
TEXEC 0x8002, 255, 0   # HTTP_SERVE (handle 10 requests)

HALT
```

### Dynamic HTTP Server
```asm
# Initialize
SETS 255, "8080"
TEXEC 0x8000, 255, 0   # HTTP_INIT

RequestLoop:
    # Wait for request
    TEXEC 0x8008, 255, 0   # HTTP_LISTEN
    # R0 = {"method":"GET","path":"/api","body":"...","addr":"..."}

    # Build dynamic response
    SETS 10, "{\"status\":\"ok\",\"timestamp\":"
    TEXEC 0x5008, 255, 11  # TIME -> R11
    ATOI 12, 11
    ITOA 13, 12
    SCAT 14, 10, 13
    SETS 15, "}"
    SCAT 16, 14, 15        # R16 = full JSON response

    # Send response
    TEXEC 0x8009, 16, 0    # HTTP_SEND

    JMP RequestLoop
```

### Complete RESTful API Server (Production Pattern)

Save as `rest_api.asm`, compile with `l0asm rest_api.asm > rest_api.l0`, run with `l0vm rest_api.l0`.

```asm
# ============================================================
# L-0 RESTful API Server - Complete Working Example
# ============================================================
# Endpoints:
#   GET    /api/posts       - List all posts
#   GET    /api/posts?id=1  - Get single post
#   POST   /api/posts       - Create post (JSON body)
#   DELETE /api/posts?id=1  - Delete post
#   GET    /                - HTML frontend
#
# Architecture: HTTP_LISTEN + Direct DB (TEXEC 0x9xxx)
# All business logic in L-0, frontend only for display
# ============================================================

# === Database Setup ===
SETS 255, "api.db"
TEXEC 0x9000, 255, 0                    # DB_INIT

SETS 255, "posts|id INTEGER PRIMARY KEY AUTOINCREMENT,title TEXT,content TEXT,created INTEGER"
TEXEC 0x9001, 255, 0                    # DB_CREATE_TABLE

# Sample data with server timestamp
TEXEC 0x5008, 255, 50                   # R50 = current timestamp
SETS 51, "posts|title,content,created|Hello World,Welcome to L-0 API,"
SCAT 52, 51, 50                         # Append timestamp
TEXEC 0x9002, 52, 0                     # DB_INSERT

# === HTTP Server ===
SETS 255, "8080"
TEXEC 0x8000, 255, 0                    # HTTP_INIT

SETS 255, "RESTful API running at http://127.0.0.1:8080"
TEXEC 0x5000, 255, 0

# ============================================================
# REQUEST HANDLER LOOP
# ============================================================
RequestLoop:
    MARK 200                            # Memory watermark for cleanup

    TEXEC 0x8008, 255, 0                # HTTP_LISTEN -> R0 = request JSON
    # R0 = {"method":"GET","path":"/api/posts?id=1","body":"...","addr":"..."}

    # --- Extract method using JSON_PARSE ---
    SETS 255, "|method"
    SCAT 1, 0, 255                       # R1 = json + "|method"
    TEXEC 0x6004, 1, 2                  # JSON_PARSE -> R2 = "GET" / "POST" / "DELETE"

    # --- Extract path ---
    SETS 255, "|path"
    SCAT 3, 0, 255                       # R3 = json + "|path"
    TEXEC 0x6004, 3, 4                  # JSON_PARSE -> R4 = "/api/posts?id=1"

    # --- Extract body (for POST) ---
    SETS 255, "|body"
    SCAT 5, 0, 255                       # R5 = json + "|body"
    TEXEC 0x6004, 5, 6                  # JSON_PARSE -> R6 = POST body

    # ========================================
    # ROUTING: Compare method and path
    # ========================================

    # Check if path starts with /api/posts
    SETS 10, "/api/posts"
    HLEN 11, 4                          # R11 = len(path)
    HLEN 12, 10                         # R12 = len("/api/posts") = 11
    SET 15, 0                           # R15 = 0 (offset for SLICE)
    SLICE 13, 4, 15, 12                 # R13 = first 11 chars of path
    SCMP 13, 10                         # String compare R13 vs R10
    BEQ ApiPostsRoute                   # Path starts with /api/posts

    # Check if path is "/"
    SETS 14, "/"
    SCMP 4, 14                          # String compare R4 vs R14
    BEQ ServeHtml                       # Serve HTML frontend

    # 404 for other paths
    JMP NotFound

# ========================================
# API ROUTES: /api/posts
# ========================================
ApiPostsRoute:
    # Check method (use SCMP for string comparison)
    SETS 20, "GET"
    SCMP 2, 20                          # String compare method vs "GET"
    BEQ HandleGet

    SETS 21, "POST"
    SCMP 2, 21                          # String compare method vs "POST"
    BEQ HandlePost

    SETS 22, "DELETE"
    SCMP 2, 22                          # String compare method vs "DELETE"
    BEQ HandleDelete

    JMP MethodNotAllowed

# --- GET /api/posts or /api/posts?id=N ---
HandleGet:
    # Check if ?id= exists in path
    SETS 30, "?id="
    # Simple check: if path length > 11, assume has query param
    HLEN 31, 4
    SET 32, 11
    CMP 31, 32
    BGT GetSinglePost

    # GET all posts
    SETS 255, "posts"
    TEXEC 0x9003, 255, 40              # DB_SELECT -> R40 = all posts JSON
    TEXEC 0x8009, 40, 0                # HTTP_SEND
    JMP RequestDone

GetSinglePost:
    # Extract ID from ?id=N (chars after position 15: /api/posts?id=)
    SET 33, 15
    SUB 34, 31, 33                     # R34 = remaining length
    SLICE 35, 4, 33, 34                # R35 = ID string

    # Query single post
    SETS 36, "posts|id="
    SCAT 37, 36, 35                    # R37 = "posts|id=N"
    TEXEC 0x9003, 37, 38              # DB_SELECT -> R38

    TEXEC 0x8009, 38, 0               # HTTP_SEND
    JMP RequestDone

# --- POST /api/posts (create) ---
HandlePost:
    # Body validation: check if body is not empty
    HLEN 40, 6
    SET 41, 2
    CMP 40, 41
    BLT InvalidBody                    # Body too short

    # Parse JSON body and insert
    # Body format: {"title":"...","content":"..."}

    # Extract title from body using JSON_PARSE
    SETS 42, "|title"
    SCAT 43, 6, 42                     # R43 = body + "|title"
    TEXEC 0x6004, 43, 44              # JSON_PARSE -> R44 = title

    # Extract content
    SETS 45, "|content"
    SCAT 46, 6, 45                     # R46 = body + "|content"
    TEXEC 0x6004, 46, 47              # JSON_PARSE -> R47 = content

    # Server-side: add timestamp (business logic in L-0!)
    TEXEC 0x5008, 255, 48             # R48 = current timestamp

    # Build insert query
    SETS 50, "posts|title,content,created|"
    SCAT 51, 50, 44                    # + title
    SETS 52, ","
    SCAT 53, 51, 52
    SCAT 54, 53, 47                    # + content
    SCAT 55, 54, 52
    SCAT 56, 55, 48                    # + timestamp

    TEXEC 0x9002, 56, 57              # DB_INSERT -> R57 = last_id

    # Return created response
    SETS 58, "{\"ok\":true,\"id\":"
    SCAT 59, 58, 57
    SETS 60, "}"
    SCAT 61, 59, 60
    TEXEC 0x8009, 61, 0               # HTTP_SEND
    JMP RequestDone

# --- DELETE /api/posts?id=N ---
HandleDelete:
    # Extract ID (same as GET single)
    HLEN 70, 4
    SET 71, 15
    SUB 72, 70, 71
    SLICE 73, 4, 71, 72               # R73 = ID string

    # Delete from DB
    SETS 74, "posts|id="
    SCAT 75, 74, 73
    TEXEC 0x9005, 75, 76              # DB_DELETE

    SETS 77, "{\"ok\":true,\"deleted\":true}"
    TEXEC 0x8009, 77, 0
    JMP RequestDone

# ========================================
# HTML FRONTEND
# ========================================
ServeHtml:
    SETS 80, "<!DOCTYPE html><html><head><meta charset=\"UTF-8\"><title>Posts API</title>"
    SETS 81, "<style>body{font-family:system-ui;max-width:800px;margin:2rem auto;padding:0 1rem}"
    SETS 82, ".post{border:1px solid #ddd;padding:1rem;margin:1rem 0;border-radius:8px}"
    SETS 83, "form{display:flex;flex-direction:column;gap:0.5rem}input,textarea{padding:0.5rem}"
    SETS 84, "button{padding:0.5rem 1rem;cursor:pointer}</style></head><body>"
    SETS 85, "<h1>L-0 Posts API</h1>"
    SETS 86, "<form onsubmit=\"create(event)\"><input id=\"t\" placeholder=\"Title\" required>"
    SETS 87, "<textarea id=\"c\" placeholder=\"Content\"></textarea>"
    SETS 88, "<button>Create Post</button></form><div id=\"posts\"></div><script>"
    SETS 89, "async function load(){const r=await fetch('/api/posts');const d=await r.json();"
    SETS 90, "document.getElementById('posts').innerHTML=d.map(p=>"
    SETS 91, "`<div class='post'><h3>${p.title}</h3><p>${p.content}</p>"
    SETS 92, "<small>ID:${p.id} Created:${new Date(p.created*1000).toLocaleString()}</small>"
    SETS 93, "<button onclick='del(${p.id})'>Delete</button></div>`).join('')}"
    SETS 94, "async function create(e){e.preventDefault();"
    SETS 95, "await fetch('/api/posts',{method:'POST',headers:{'Content-Type':'application/json'},"
    SETS 96, "body:JSON.stringify({title:t.value,content:c.value})});t.value='';c.value='';load()}"
    SETS 97, "async function del(id){await fetch('/api/posts?id='+id,{method:'DELETE'});load()}"
    SETS 98, "load()</script></body></html>"

    # Concatenate HTML
    SCAT 100, 80, 81
    SCAT 101, 100, 82
    SCAT 102, 101, 83
    SCAT 103, 102, 84
    SCAT 104, 103, 85
    SCAT 105, 104, 86
    SCAT 106, 105, 87
    SCAT 107, 106, 88
    SCAT 108, 107, 89
    SCAT 109, 108, 90
    SCAT 110, 109, 91
    SCAT 111, 110, 92
    SCAT 112, 111, 93
    SCAT 113, 112, 94
    SCAT 114, 113, 95
    SCAT 115, 114, 96
    SCAT 116, 115, 97
    SCAT 117, 116, 98

    TEXEC 0x8009, 117, 0              # HTTP_SEND
    JMP RequestDone

# ========================================
# ERROR HANDLERS
# ========================================
NotFound:
    SETS 120, "{\"error\":\"Not Found\",\"code\":404}"
    TEXEC 0x8009, 120, 0
    JMP RequestDone

MethodNotAllowed:
    SETS 121, "{\"error\":\"Method Not Allowed\",\"code\":405}"
    TEXEC 0x8009, 121, 0
    JMP RequestDone

InvalidBody:
    SETS 122, "{\"error\":\"Invalid request body\",\"code\":400}"
    TEXEC 0x8009, 122, 0
    JMP RequestDone

# ========================================
# CLEANUP AND LOOP
# ========================================
RequestDone:
    RESET 200                          # Free all allocations since MARK
    JMP RequestLoop
```

**What this example demonstrates:**

| Feature | Implementation | L-0 Instructions Used |
|---------|----------------|----------------------|
| **Request Parsing** | Extract method, path, body from JSON | `JSON_PARSE` (0x6004), `SCAT` |
| **Routing** | Compare path prefix and method | `SCMP`, `BEQ`, `SLICE`, `HLEN` |
| **Query Parameters** | Extract `?id=N` from path | `SLICE`, `SUB` |
| **POST Body Parsing** | Extract fields from JSON body | `JSON_PARSE` (0x6004) |
| **Server Timestamps** | Add `created` field on insert | `TIME` (0x5008) |
| **Database CRUD** | Direct DB access | `DB_*` (0x9xxx) |
| **Memory Management** | Per-request cleanup | `MARK`, `RESET` |
| **Error Handling** | Return JSON errors with codes | `SETS`, `HTTP_SEND` |

**Key Architecture Points:**
1. **Business logic in L-0**: Timestamps, validation, data transformation all happen server-side
2. **Direct DB access**: No proxy layer - VM calls db_plugin directly via TEXEC
3. **Stateless routing**: Each request is independent, MARK/RESET ensures no memory leaks
4. **Frontend is dumb**: JavaScript only displays what server returns

### File I/O
```asm
# Read file, transform content, write to new file

# Check if source file exists
SETS 255, "input.txt"
TEXEC 0x4005, 255, 1          # FILE_EXISTS -> R1 = "true" or "false"
SETS 2, "true"
SCMP 1, 2                      # String compare with "true"
BEQ FileExists
JMP FileNotFound

FileExists:

# Read file content
SETS 255, "input.txt"
TEXEC 0x4000, 255, 10         # FILE_READ -> R10 = file content

# Transform: convert to uppercase
TEXEC 0x500C, 10, 11          # UPPER -> R11

# Write to output file
SETS 12, "output.txt|"
SCAT 13, 12, 11               # "output.txt|CONTENT"
TEXEC 0x4001, 13, 0           # FILE_WRITE

SETS 255, "File transformed and saved to output.txt"
TEXEC 0x5000, 255, 0
HALT

FileNotFound:
SETS 255, "Error: input.txt not found"
TEXEC 0x5000, 255, 0
HALT
```

### HTTP Client (API Consumer)

**Note**: HTTP_REQUEST only supports HTTP, not HTTPS. For HTTPS endpoints, use an external tool or proxy.

```asm
# Make HTTP request to external API

# GET request (HTTP only, no TLS support)
SETS 1, "http://httpbin.org/get"
TEXEC 0x8004, 1, 10           # HTTP_REQUEST -> R10 = response

SETS 2, "GET Response:"
TEXEC 0x5000, 2, 0
TEXEC 0x5000, 10, 0           # Print response

# POST request with body
SETS 3, "http://httpbin.org/post|POST|{\"test\":\"data\"}"
TEXEC 0x8004, 3, 20           # HTTP_REQUEST -> R20 = response

SETS 4, "POST Response:"
TEXEC 0x5000, 4, 0
TEXEC 0x5000, 20, 0           # Print response

HALT
```

---

## Best Practices

### Architecture Rule: Backend Logic in L-0

**CRITICAL**: L-0 is a backend language. Business logic MUST be in L-0 assembly, NOT in frontend JavaScript.

#### BAD Pattern (Frontend-Heavy)
```asm
# BAD: L-0 only serves static files, all logic in frontend JS
SETS 255, "8080"
TEXEC 0x8000, 255, 0        # HTTP_INIT
SETS 255, "static/"
TEXEC 0x8006, 255, 0        # HTTP_STATIC
# Frontend JS does: fetch data, validate, compute, format...
# L-0 is just a file server - no business logic!
SETS 255, "999999"
TEXEC 0x8002, 255, 0        # HTTP_SERVE
HALT
# Problem: L-0 does nothing! All validation, computation in frontend JS.
```

#### GOOD Pattern (Backend Logic)
```asm
# GOOD: L-0 handles business logic, frontend only for presentation

SETS 255, "8080"
TEXEC 0x8000, 255, 0        # HTTP_INIT

SETS 255, "app.db"
TEXEC 0x9000, 255, 0        # DB_INIT

# === DYNAMIC REQUEST HANDLER ===
RequestLoop:
    MARK 100                 # Watermark for memory management
    TEXEC 0x8008, 255, 0     # HTTP_LISTEN -> R0 = {method, path, body}

    # Parse request path using JSON_PARSE
    SETS 1, "|path"
    SCAT 2, 0, 1                  # R2 = json + "|path"
    TEXEC 0x6004, 2, 3            # JSON_PARSE -> R3 = path

    # Route to handlers
    SETS 2, "/api/calculate"
    # CMP and branch to handlers...

    # === BUSINESS LOGIC IN L-0 ===
    # Example: Validate input
    # Example: Compute derived values
    # Example: Apply business rules
    # Example: Transform data before storage

    # Build response with computed data
    SETS 10, "{\"result\":"
    # ... computation ...
    TEXEC 0x8009, 10, 0      # HTTP_SEND

    RESET 100                # Free allocations
    JMP RequestLoop
```

#### When to Use Each Pattern

| Use Case | Pattern | Why |
|----------|---------|-----|
| Static site | HTTP_STATIC + HTTP_SERVE | No dynamic content needed |
| Data validation | HTTP_LISTEN + L-0 validation | Prevent invalid data at backend |
| Computed fields | HTTP_LISTEN + L-0 computation | Server calculates, client displays |
| Access control | HTTP_LISTEN + L-0 auth check | Security must be server-side |
| Business rules | HTTP_LISTEN + L-0 logic | Rules enforced regardless of client |
| Rate limiting | HTTP_LISTEN + L-0 counter | Client cannot bypass |
| REST API | HTTP_LISTEN + DB_* (0x9xxx) | Full control over data operations |

#### Backend Logic Examples

```asm
# Example 1: Validation before insert
# Instead of: frontend validates, calls /api/users
# Do: L-0 validates, then DB_INSERT

TEXEC 0x8008, 255, 0         # HTTP_LISTEN -> R0
# Parse body, extract "email" field into R10

# Validate email format using REGEX
SETS 11, "^[^@]+@[^@]+\\.[^@]+$"
REGEX 12, 11, 10             # R12 = 1 if valid, 0 if not

SET 13, 0
CMP 12, 13
BEQ ValidationFailed

# Valid: proceed with insert
SETS 255, "users|email="
SCAT 255, 255, 10
TEXEC 0x9002, 255, 0         # DB_INSERT
SETS 20, "{\"ok\":true}"
TEXEC 0x8009, 20, 0
JMP RequestLoop

ValidationFailed:
SETS 20, "{\"ok\":false,\"error\":\"Invalid email format\"}"
TEXEC 0x8009, 20, 0
JMP RequestLoop
```

```asm
# Example 2: Computed field (order total)
# Calculate total = sum(items.price * items.quantity)

TEXEC 0x8008, 255, 0         # Get order request
# Parse items array...

SET 20, 0                    # R20 = running total

ComputeLoop:
    # For each item: total += price * quantity
    # (parsing and computation in L-0)
    MUL 50, 21, 22           # price * quantity
    ADD 20, 20, 50           # total += result
    # ... loop control ...

# Store computed total with order
SETS 255, "orders|total="
ITOA 51, 20
SCAT 255, 255, 51
# ... include other fields ...
TEXEC 0x9002, 255, 0         # DB_INSERT with computed total
```

### Register Convention
```
R0-R3     : Arguments / Return (CLOBBERED by patterns)
R4-R9     : Pattern temporaries (CLOBBERED by patterns)
R10-R19   : Preserved across patterns (SAFE)
R20-R49   : Local variables (SAFE)
R50-R99   : Scratch (always CLOBBERED)
R100-R109 : Constants (e.g., R100=1)
R255      : I/O buffer
```

### Caller-Save Pattern
```asm
# BEFORE using a pattern, save important data
MOV 20, 5              # Save R5 -> R20 (safe zone)
MOV 21, 6              # Save R6 -> R21

# Use pattern
SET 0, 10              # Input
# ---- BEGIN fibonacci_abc1 ----
# ... pattern clobbers R4-R9 ...
# ---- END ----

# AFTER pattern, restore if needed
MOV 5, 20
MOV 6, 21
```

### Label Hygiene
```asm
# BAD - collision risk
fib_1_loop:
fib_2_loop:

# GOOD - unique hash suffix
fib_a7b9_loop:
fib_c3d2_loop:

# BEST - context in suffix
calc_total_fib_loop:
print_result_fib_loop:
```

### Error Handling
```asm
# After pattern that may return -1
SET 50, 0
CMP 0, 50
BLT error_handler_abc1     # R0 < 0 = error

# Success path
# ...
JMP continue_abc1

error_handler_abc1:
SETS 255, "Error occurred"
TEXEC 0x5000, 255, 0
HALT

continue_abc1:
# ...
```

### Handler Pattern (Control Flow)
```asm
MainLoop:
    # ... dispatch logic ...
    CMP 5, 6
    BEQ HandleA
    CMP 5, 7
    BEQ HandleB
    JMP MainLoop

HandleA:
    # ... handler code ...
    JMP MainLoop         # CRITICAL: Jump back!

HandleB:
    # ... handler code ...
    JMP MainLoop         # CRITICAL: Jump back!
```

### Memory Rule (SCAT Chaining)

**CRITICAL**: When chaining `SCAT`, you MUST manage memory to avoid leaks.

#### Option 1: Watermark Pattern (RECOMMENDED for servers)

Use `MARK` and `RESET` for arena-style allocation:

```asm
# BEST - Watermark pattern for request loops
RequestLoop:
    MARK 100                   # Save heap watermark

    # Process request (allocate freely, no FREE needed)
    TEXEC 0x8008, 255, 0       # HTTP_LISTEN -> R0 = request
    SCAT 40, 10, 11            # Build response...
    SCAT 41, 40, 12
    SCAT 42, 41, 13            # R42 = final response
    TEXEC 0x8009, 42, 0        # HTTP_SEND

    RESET 100                  # FREE ALL allocations since MARK!
    JMP RequestLoop
```

#### Option 2: Accumulator Pattern (manual FREE)

```asm
# GOOD - Manual accumulator pattern
SCAT 40, 10, 11         # R40 = accumulator
MOV 50, 40              # Save old pointer
SCAT 40, 50, 12         # R40 = new concatenation
FREE 50                 # Free old pointer
MOV 50, 40              # Save again
SCAT 40, 50, 13         # R40 = final result
FREE 50                 # Free intermediate
```

#### BAD Pattern (Memory Leak!)

```asm
# BAD - Memory leak! Each SCAT allocates new heap, old ones orphaned
SCAT 40, 10, 11         # R40 = new heap (10+11)
SCAT 41, 40, 12         # R41 = new heap (40+12), R40 is now orphaned!
SCAT 42, 41, 13         # R42 = new heap, R41 orphaned!
# Result: 2 orphaned heap allocations
```

**Short-lived programs**: If building HTML for HTTP response and then `HALT`, leaks are acceptable (OS reclaims memory).

**Long-running servers**: Use `MARK`/`RESET` watermark pattern or manual `FREE`.

---

## Known Limitations

### HTTP Plugin

**CRITICAL: HTTP_SERVE(N) is NOT "blocking indefinitely"**
- `HTTP_SERVE` serves **exactly N requests**, then returns
- Default N=1 if no argument provided
- Browser makes multiple concurrent requests (/, favicon.ico, CSS, JS)
- A single page load may consume 3-10 requests instantly

| Issue | Solution |
|-------|----------|
| "Server stops immediately" | Pass request count: `SETS 255, "100"` before `TEXEC 0x8002` |
| Route Priority | Registered routes first, then `/api/*` auto-CRUD |
| Port Reuse | Wait ~30s after stop or use different port |
| Static vs Dynamic | Use HTTP_LISTEN + HTTP_SEND for dynamic responses |
| API URL format | Use path-based: `/api/table/id` not `/api/table?id=X` |
| Concurrency | Multi-threaded: handles browser concurrent requests |

**Server Patterns:**
```asm
# Pattern 1: Finite service (recommended for testing)
SETS 255, "100"
TEXEC 0x8002, 255, 0   # Serve 100 requests, then HALT

# Pattern 2: Infinite loop service
ServerLoop:
    SETS 255, "50"
    TEXEC 0x8002, 255, 0
    JMP ServerLoop         # Restart after 50 requests

# Pattern 3: Dynamic responses (full control)
RequestLoop:
    TEXEC 0x8008, 255, 0   # HTTP_LISTEN (blocks waiting)
    # ... build response in R16 ...
    TEXEC 0x8009, 16, 0    # HTTP_SEND
    JMP RequestLoop
```

### Database Plugin

**CRITICAL: DB_SELECT returns JSON string, not object handle**
- Result is a heap pointer to a JSON string like `[{"id":1,"name":"Alice"}]`
- Print directly with `TEXEC 0x5000, result_reg, 0`
- Parse JSON fields manually if needed

| Issue | Solution |
|-------|----------|
| INSERT format | Use `table\|{json}`: `SETS 255, "users\|{\"name\":\"Alice\"}"` |
| DELETE syntax | Format: `table\|condition` (e.g., `users\|id=5`) |
| String conditions | Use SQL quotes: `name='Alice'` not `name=Alice` |
| DB_SELECT return | Returns JSON string directly, not object handle |
| ORDER BY without condition | Prefix with `1=1`: `"table\|1=1 ORDER BY col DESC"` |

**Database Examples:**
```asm
# Initialize database
SETS 255, "data.db"
TEXEC 0x9000, 255, 0   # DB_INIT

# Create table
SETS 255, "users|id INTEGER PRIMARY KEY,name TEXT"
TEXEC 0x9001, 255, 0   # DB_CREATE_TABLE

# Insert (note: JSON in string)
SETS 255, "users|{\"name\":\"Alice\"}"
TEXEC 0x9002, 255, 0   # DB_INSERT -> R0 = last_insert_id

# Select all
SETS 255, "users"
TEXEC 0x9003, 255, 0   # DB_SELECT -> R0 = "[{...},...]"
TEXEC 0x5000, 0, 0     # Print the JSON array

# Select with ORDER BY (note: 1=1 prefix required)
SETS 255, "users|1=1 ORDER BY id DESC"
TEXEC 0x9003, 255, 1   # R1 = sorted results
```

### TEXEC Instruction

**CRITICAL: TEXEC has exactly 3 operands**
- Syntax: `TEXEC tool_id, arg_register, dest_register`
- There is NO 4-operand form
- All arguments must be packed into a single string using `|` delimiter

| Issue | Solution |
|-------|----------|
| Operand count | Only 3 operands: `TEXEC tool, arg, dest` |
| Multiple args | Use pipe delimiter: `SETS 255, "arg1\|arg2\|arg3"` |
| Result type | Always returns string in heap, use ATOI if needed |
| Tool names | Use `TEXEC 0x5000` for PRINT (with newline), `TEXEC 0x5001` for PRINTN (no newline) |

**Print Examples:**
```asm
# PRINT (with newline) - 0x5000
SETS 255, "Hello"
TEXEC 0x5000, 255, 0   # Output: Hello\n

# PRINTN (no newline) - 0x5001
SETS 255, "World"
TEXEC 0x5001, 255, 0   # Output: World (no newline)

# Print integer with newline
SET 10, 42
ITOA 11, 10            # R11 = "42"
TEXEC 0x5000, 11, 0    # Print "42\n"
```

### Control Flow
| Issue | Solution |
|-------|----------|
| No CALL/RET | Inline all code, use unique label suffixes |
| Fall-through | Always end handlers with `JMP` to loop or `HALT` |
| Label collision | Use hash suffixes: `pattern_a7b9_label` |

### String Handling
| Feature | Note |
|---------|------|
| Escape sequences | `\n`, `\t`, `\r`, `\"`, `\\` fully supported in SETS |
| Comma in strings | Works correctly: `SETS 1, "Hello, World"` |
| JSON in strings | Use `\"` for quotes: `SETS 1, "users\|{\"name\":\"Alice\"}"` |
| Trailing spaces | **Known issue**: Trailing spaces may be stripped from strings |

**Escape Sequence Examples:**
```asm
# Newlines and tabs
SETS 1, "Line1\nLine2\tTabbed"

# JSON with escaped quotes
SETS 255, "posts|{\"title\":\"Hello\",\"content\":\"World\"}"
TEXEC 0x9002, 255, 0    # DB_INSERT - works correctly!

# Nested quotes in HTML
SETS 10, "<div class=\"container\">Content</div>"
```

---

## Toolchain

| Binary | Description |
|--------|-------------|
| `l0vm` | Virtual Machine - executes .l0 bytecode |
| `l0asm`| Assembler - compiles .asm to .l0 bytecode |
| `l0cc` | AOT Compiler - generates C code from .l0 |

| Format | Purpose | AI Generates? |
|--------|---------|---------------|
| `.asm` | **Source** (labels, comments) | **YES** |
| `.l0`  | Binary bytecode | No (compiled) |
| `.c`   | AOT output | No (generated) |

### Introspection

```bash
./target/release/l0vm --info
```

Returns: `version`, `architecture`, `isa.primitives`, `tools`.

---

## Build

```bash
./scripts/build_dist.sh [version]
```

This script builds all workspace crates and creates a distribution package in `dist/`. For development builds only:
```bash
./scripts/build_release.sh
```

---

## Directory Structure

```
.
├── Cargo.toml              # Workspace configuration
├── crates/                 # Core components
│   ├── l0_core/            # ISA Definition (56 instructions)
│   ├── l0_vm/              # Virtual Machine
│   ├── l0_asm/             # Assembler
│   └── l0_compiler/        # AOT Compiler
├── plugins/                # External plugins
│   ├── file_plugin/        # File I/O operations
│   ├── data_plugin/        # JSON operations
│   ├── http_plugin/        # HTTP server
│   └── db_plugin/          # SQLite database
├── tools.json              # Tool Registry
├── test/                   # Test suite
└── README.md               # This file (ISA + Stdlib Patterns)
```

---

*L-0 Preview | 56 ISA Primitives | Semantic Computing | The Language of Autonomous AI*
