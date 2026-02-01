# L-Zero (Preview): The Native Language of AI Agents

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
- **52 Primitives + Infinite Tools**: Core ISA compiled into VM; plugins extend via JSON-RPC

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

## The Physics (ISA Reference)

### Architecture

```
+---------------------------------------------------------------+
| Layer 1: ISA Core (52 primitives)                             |
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

### Instructions (52 Total)

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

#### System
| Op | Args | Description |
|----|------|-------------|
| NOP | - | No operation |
| HALT | - | Stop execution |
| PANIC | msg: String | Abort with message |
| DUMP | - | Dump VM state (regs/heap) |
| GAS | reg | R[reg] = current gas limit |
| ASSERT | reg | Abort if R[reg] == 0 |

#### Registers
| Op | Args | Description |
|----|------|-------------|
| SET | reg, val | R[reg] = val (integer) |
| SETS | reg, val | R[reg] = heap_alloc(val) (string) |
| MOV | dest, src | R[dest] = R[src] |
| SWAP | r1, r2 | Swap R[r1] and R[r2] |

#### Math (all: dest, s1, s2)
| Op | Description |
|----|-------------|
| ADD | R[dest] = R[s1] + R[s2] |
| SUB | R[dest] = R[s1] - R[s2] |
| MUL | R[dest] = R[s1] * R[s2] |
| DIV | R[dest] = R[s1] / R[s2] |
| MOD | R[dest] = R[s1] % R[s2] |

#### Logic
| Op | Description |
|----|-------------|
| AND | R[dest] = R[s1] & R[s2] |
| OR | R[dest] = R[s1] \| R[s2] |
| XOR | R[dest] = R[s1] ^ R[s2] |
| NOT | R[dest] = ~R[src] |

#### Control Flow
| Op | Args | Description |
|----|------|-------------|
| CMP | r1, r2 | Set flags: EQ (==), GT (>), LT (<) |
| JMP | target | Jump to label |
| BEQ | target | Jump if EQ flag set |
| BGT | target | Jump if GT flag set |
| BLT | target | Jump if LT flag set |

#### Memory
| Op | Args | Description |
|----|------|-------------|
| NEW | dest, size | R[dest] = heap_alloc(size bytes) **size is literal, not register** |
| FREE | ptr | Free heap[R[ptr]] |
| READ | dest, ptr, offset | R[dest] = heap[R[ptr]][offset] (u8) **offset is literal** |
| WRITE | ptr, offset, val | heap[R[ptr]][offset] = R[val] (u8) **offset is literal** |
| READR | dest, ptr, off | R[dest] = heap[R[ptr]][R[off]] (dynamic offset from register) |
| WRITER | ptr, off, val | heap[R[ptr]][R[off]] = R[val] (dynamic offset from register) |
| STORE64 | ptr, off, val | heap[R[ptr]][R[off]*8..] = R[val] (i64, **off is register**) |
| LOAD64 | dest, ptr, off | R[dest] = heap[R[ptr]][R[off]*8..] (i64, **off is register**) |

#### Batch Operations
| Op | Args | Description |
|----|------|-------------|
| MEMCPY | dst, doff, src, soff, len | Copy R[len] bytes between heap regions |
| HLEN | dest, ptr | R[dest] = length of heap[R[ptr]] |
| SLICE | dest, ptr, off, len | R[dest] = new heap with substring |
| MEMSET | ptr, off, len, val | Fill R[len] bytes with R[val] |

#### Extensions
| Op | Args | Description |
|----|------|-------------|
| TEXEC | tool, arg, dest | Call tool with heap[R[arg]], store result in R[dest] |
| ITOA | dest, src | R[dest] = str(R[src]) - integer to string |
| ATOI | dest, src | R[dest] = int(R[src]) - string to integer |
| SCAT | dest, s1, s2 | R[dest] = R[s1] + R[s2] - string concatenation |
| REGEX | dest, pat, text | R[dest] = 1 if R[text] matches R[pat], else 0 |

#### Vector Operations (Semantic Computing)
| Op | Args | Description |
|----|------|-------------|
| VNEW | dest, dims | R[dest] = allocate vector of R[dims] dimensions |
| VSET | vec, idx, val | vec[R[idx]] = f64::from_bits(R[val]) |
| VGET | dest, vec, idx | R[dest] = vec[R[idx]] as i64 bits |
| VDOT | dest, v1, v2 | R[dest] = dot(v1, v2) * 1,000,000 |
| VSIM | dest, v1, v2 | R[dest] = cosine_similarity(v1, v2) * 1,000,000 |
| VMAG | dest, vec | R[dest] = magnitude(vec) * 1,000,000 |
| VNORM | vec | Normalize vec in-place to unit length |

#### Governance (Autonomous AI Control)
| Op | Args | Description |
|----|------|-------------|
| LATCH | target, threshold | R240 = R[target] (vec ptr), R242 = R[threshold] |
| GUARD | state | Compute VSIM(R240, R[state]); TRAP if below R242 |
| TRAP | code | Suspend VM, emit JSON context to supervisor |
| YIELD | query, dest | Send R[query] to supervisor, receive into R[dest] |

---

## The Protocol (Tools)

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
| `0x9000` | DB_INIT | "path" | Initialize database |
| `0x9001` | DB_CREATE_TABLE | "table\|schema" | Create table |
| `0x9002` | DB_INSERT | "table\|{json}" | Insert row |
| `0x9003` | DB_SELECT | "table[\|condition]" | Query rows |
| `0x9004` | DB_UPDATE | "table\|condition\|{json}" | Update rows |
| `0x9005` | DB_DELETE | "table\|condition" | Delete rows |
| `0x9006` | DB_DROP_TABLE | "table_name" | Drop table |
| `0x9007` | DB_LIST_TABLES | - | List all tables |

**HTTP (0x8xxx)**
| ID | Name | Description |
|----|------|-------------|
| `0x8000` | HTTP_INIT | Initialize HTTP server with port |
| `0x8001` | HTTP_ROUTE | Register static route |
| `0x8002` | HTTP_SERVE | Start server (blocking) |
| `0x8003` | HTTP_SERVE_ONCE | Handle one request |
| `0x8004` | HTTP_REQUEST | Make HTTP request |
| `0x8005` | HTTP_LIST_ROUTES | List registered routes |
| `0x8006` | HTTP_STATIC | Set static file directory |
| `0x8007` | HTTP_DB | Set database path for /api/* CRUD |
| `0x8008` | HTTP_LISTEN | Wait for request, return {method, path, body, addr} |
| `0x8009` | HTTP_SEND | Send response for pending request |

**AI (0x3xxx)** - Optional
| ID | Name | Description |
|----|------|-------------|
| `0x3000` | AI_EMBED | Generate embeddings |
| `0x3004` | AI_GENERATE | Generate text |

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
    NEW 0, 4               # R0 = new heap allocation
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
    SETS 255, ""
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

### Complete CMS Application
```asm
# L-0 CMS - combines Database + HTTP + Auto-CRUD
# Open http://127.0.0.1:8080 after running

# === Database Setup ===
SETS 255, "cms.db"
TEXEC 0x9000, 255, 0               # DB_INIT

SETS 255, "articles|id INTEGER PRIMARY KEY AUTOINCREMENT,title TEXT,content TEXT,author TEXT"
TEXEC 0x9001, 255, 0               # DB_CREATE_TABLE

SETS 255, "articles|{\"title\":\"Welcome\",\"content\":\"Hello from L-0 CMS\",\"author\":\"System\"}"
TEXEC 0x9002, 255, 0               # DB_INSERT (sample data)

# === HTTP Server ===
SETS 255, "8080"
TEXEC 0x8000, 255, 0               # HTTP_INIT

SETS 255, "cms.db"
TEXEC 0x8007, 255, 0               # HTTP_DB (enables /api/articles auto-CRUD)

# === Web UI (inline HTML) ===
SETS 10, "<!DOCTYPE html><html><head><title>CMS</title></head><body>"
SETS 11, "<h1>L-0 CMS</h1><div id=\"posts\"></div>"
SETS 12, "<script>fetch('/api/articles').then(r=>r.json()).then(d=>{document.getElementById('posts').innerHTML=d.map(p=>'<article><h2>'+p.title+'</h2><p>'+p.content+'</p></article>').join('')})</script>"
SETS 13, "</body></html>"
SCAT 20, 10, 11
SCAT 21, 20, 12
SCAT 22, 21, 13                    # R22 = complete HTML

SETS 23, "GET /|"
SCAT 24, 23, 22
TEXEC 0x8001, 24, 0                # HTTP_ROUTE: GET / -> HTML

# === Start Server ===
SETS 255, "CMS running at http://127.0.0.1:8080"
TEXEC 0x5000, 255, 0

SETS 255, "100"
TEXEC 0x8002, 255, 0               # HTTP_SERVE (100 requests)
HALT
```

---

## Best Practices

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
| "PRINTN not found" | Use `TEXEC 0x5001` for PRINTLN, `TEXEC 0x5000` for PRINT |

**Print Examples:**
```asm
# PRINT (no newline)
SETS 255, "Hello"
TEXEC 0x5000, 255, 0

# PRINTLN (with newline)
SETS 255, "World"
TEXEC 0x5001, 255, 0

# Print integer
SET 10, 42
ITOA 11, 10          # R11 = "42"
TEXEC 0x5001, 11, 0  # Print "42\n"
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
| Escape sequences | `\n`, `\t`, `\"`, `\\` supported in SETS |
| Comma in strings | Works correctly: `SETS 1, "Hello, World"` |
| Trailing spaces | **Known issue**: Trailing spaces may be stripped from strings |

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
│   ├── l0_core/            # ISA Definition (52 instructions)
│   ├── l0_vm/              # Virtual Machine
│   ├── l0_asm/             # Assembler
│   └── l0_compiler/        # AOT Compiler
├── plugins/                # External plugins
│   ├── file_plugin/        # File I/O operations
│   ├── data_plugin/        # JSON operations
│   ├── http_plugin/        # HTTP server
│   └── db_plugin/          # SQLite database
├── tools.json              # Tool Registry
├── stdlib/                 # Standard library source
└── README.md               # This file (System Kernel)
```

---

*L-0 Preview | 52 ISA Primitives | Semantic Computing | The Language of Autonomous AI*
