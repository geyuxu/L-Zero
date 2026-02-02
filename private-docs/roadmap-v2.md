# L-0 改进工作计划 v2

> 基于 isa.rs / asm.rs / vm.rs / aot.rs / README.md / db.rs / http.rs / file.rs / data.rs 的完整审查
>
> 更新日期：2026-02-01

---

## 已完成修复清单 ✅

### P0 关键问题（已修复）

**1. SCMP 字符串比较指令** ✅
- isa.rs L127-128: `SCMP { s1: u8, s2: u8 }` 定义，含 doc 注释
- vm.rs L642-652: 完整 strcmp 逻辑（`String::from_utf8_lossy` → `str1.cmp(&str2)`）
- asm.rs L131: 解析支持
- aot.rs L728-730: C 代码生成（`strcmp(heap_str(...), heap_str(...))`）

**2. NEWR 动态大小分配** ✅
- isa.rs L138-139: `NEWR { dest: u8, size_reg: u8 }` 定义
- vm.rs L661-665: 从寄存器读取 size
- asm.rs L140: 解析支持
- aot.rs L740-741: C 代码生成

**3. README 示例修复** ✅
- `array_new`: `NEW 0, 4` → `NEWR 0, 4`（L1084）
- `sb_init`: 改为 `MOV 50, 0` + `NEWR 0, 50`（L735-736）
- `sb_append`: 添加 `SET 5, 0` 作为 MEMCPY 的 soff 参数（L750-753）
- `sb_finish`: 添加 `SET 50, 0` 作为 SLICE 的 off 参数（L768-769）
- REST API: 所有字符串路由改用 `SCMP`（L1566, L1571, L1583, L1587, L1591）
- REST API: `SLICE 13, 4, 0, 12` → `SET 15, 0` + `SLICE 13, 4, 15, 12`（L1564-1565）

### P1 中等问题（已修复）

**4. AOT HLEN 一致性** ✅
- aot.rs L144-164: `heap_alloc_str` 存储 len+1 字节（C 安全），记录 size 为 len（与 VM 一致）

**5. PRINT/PRINTN 描述修正** ✅
- README L2196-2206: 0x5000 = PRINT（带换行），0x5001 = PRINTN（无换行），示例正确

### P2 小问题（已修复）

**6. GAS 一致性** ✅ — AOT L694 改为 1000，与 VM L603 一致

**7. AOT SUBSTR/SPLIT** ✅ — aot.rs L274-331 补全了 0x500A 和 0x500B 实现

**8. ISA 反射** ✅ — `Instruction::introspection()` 宏自动生成，保持同步

---

## 仍待修复的问题

以下是通过审查全部 9 个源文件发现的尚未解决的问题，按优先级排列。

---

### 🔴 P0：阻塞性问题（影响正确性）

#### P0-1. README 指令数量仍然不一致

isa.rs 实际定义 **56 条**指令（手工核实：6+4+5+4+6+7+4+5+5+7+4+2 = 56，含新增的 SCMP 和 NEWR）。但 README 存在多处计数矛盾：

| 位置 | 当前值 | 应改为 |
|------|--------|--------|
| L29 | "56 Primitives" | ✅ 正确 |
| L63 | "56 instructions" | ✅ 正确 |
| L74 | "Layer 1: ISA Core (**54** primitives)" | ❌ → 56 |
| L118 | "The Holographic ISA (**54** Instructions)" | ❌ → 56 |
| L128 | Control 类计数 = **5**（CMP, JMP, BEQ, BGT, BLT）| ❌ → 6（漏了 SCMP）|
| L129 | Memory 类计数 = **8**（无 NEWR）| ❌ → 9（漏了 NEWR）|
| L2287 | "ISA Definition (**56** instructions)" | ✅ 正确 |
| L2303 | "**54** ISA Primitives" | ❌ → 56 |

**ISA 分类表也需要更新**（L122-134）：
```
| Control | 6 | CMP, SCMP, JMP, BEQ, BGT, BLT |        ← 加 SCMP
| Memory  | 9 | NEW, NEWR, FREE, READ, WRITE, READR, WRITER, STORE64, LOAD64 |  ← 加 NEWR
```

此类不一致是高优先级问题——LLM 读到 "54 instructions" 后会认为任何不在这 54 条列表中的指令"不存在"。

#### P0-2. FILE_EXISTS 返回值矛盾：README vs 实际 plugin

**README L324 声称**：`exists(path) ? "1" : "0"`（字符串 "1" 或 "0"）

**file.rs L85-86 实际返回**：
```rust
format!(r#"{{"ok":true,"value":{}}}"#, exists)
// 生成: {"ok":true,"value":true} 或 {"ok":true,"value":false}
```
VM 的 `call_plugin` 处理 JSON 响应时（vm.rs L408-413），对非字符串 value 调用 `value.to_string()`，所以最终堆中放的是 `"true"` 或 `"false"`。

**README File I/O 示例（L1782-1789）又假设**：返回 `"true"` 是 4 字符，`"false"` 是 5 字符，用 `HLEN` 判断。

三种说法不一致。实际行为是返回 `"true"` / `"false"` 字符串。修复方案：
- L324: `regs[dest] = exists(path) ? "true" : "false"`
- L336-338 的示例改为基于字符串比较：
  ```asm
  SETS 1, "data.json"
  TEXEC 0x4005, 1, 2          # R2 = "true" or "false"
  SETS 3, "true"
  SCMP 2, 3                   # 用 SCMP 而不是 ATOI
  BEQ FileExists
  ```

#### P0-3. JSON_GET 使用模式与 data.rs 不匹配

README REST API 示例（L1541-1549）使用：
```asm
SETS 255, "method"
SCAT 1, 0, 255               # R1 = request_json + "method"
TEXEC 0x6002, 1, 2            # JSON_GET -> R2 = "GET"
```

但 data.rs 的 `json_parse` 方法（用于 0x6004）期望 `"json_str|field_path"` 格式，而 `json_get` 方法（用于 0x6002）从内存存储中按 key 查找，不是从 JSON 字符串中提取字段。

SCAT 将 JSON 字符串和 key 字符串二进制拼接，结果形如 `{"method":"GET","path":"/"}method`——这不是 data.rs 任何方法能解析的有效输入。

**实际需求**：tools.json 中 0x6002 应该对应 `json_parse` 方法而非 `json_get`，而且参数格式需要改为 pipe 分隔符 `json_str|field`。

**修复方案**：

方案 A（改 tools.json）：将 0x6002 映射到 `json_parse` 方法，并更新 README 示例使用 pipe 分隔：
```asm
SETS 255, "|method"           # pipe + field
SCAT 1, 0, 255               # R1 = request_json + "|method"
TEXEC 0x6002, 1, 2            # json_parse -> R2 = "GET"
```

方案 B（改 data.rs）：让 `json_get` 方法也接受 `json_str|field` 格式作为 inline 参数。

#### P0-4. HTTP 动态模式架构风险

http.rs 的 `listen` / `send` 使用文件系统 IPC（L270, L364）：
- daemon 写请求到 `.l0_http_pending_request.json`
- VM 轮询读取，处理后写响应到 `.l0_http_pending_response.json`
- daemon 轮询读取响应，发回客户端

**并发问题**：如果两个 HTTP 请求在 100ms 窗口内到达，第二个请求会覆盖第一个的 request 文件（在 daemon 的 for loop L228 中，accept 到下一个连接会覆盖 PENDING_REQUEST_PATH）。单连接串行化只有在 daemon 的 accept loop 阻塞等待 response 完成后才安全——但当前实现确实是在同一循环中 accept → write request → poll response → accept next，所以只要 VM 不超时就是串行的。

**真正风险**：60 秒轮询超时（L286, L336）。如果 VM 的请求处理逻辑有 bug 导致跳过 `HTTP_SEND`，daemon 和 VM 都会挂 60 秒。

**建议**：短期可接受，但中期需要替换为 Unix socket 或 pipe 通信。

---

### 🟡 P1：中等问题（影响可靠性与可维护性）

#### P1-1. Plugin 每次 spawn 的性能问题

vm.rs L390：每次 TEXEC 对 plugin 类型工具都 fork+exec 新进程。

```rust
let mut child = Command::new(&bin_path)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn();
```

db.rs、http.rs、file.rs、data.rs 均支持 stdin readline loop（长连接友好），但 VM 没有利用这一点。DB_SELECT 每次调用的进程启动开销约 5-20ms，而 SQLite 查询本身可能只需 <1ms。

REST API 示例中一次请求至少 3-5 次 TEXEC plugin 调用，总开销 15-100ms，对比 Lua 的 C function call 级别开销完全不在一个量级。

**建议**：实现 Plugin 进程池（详见下方 Phase 2）。

#### P1-2. AOT SUBSTR/SPLIT 与 VM 行为差异

AOT 的 SUBSTR（aot.rs L274-296）使用 `strncpy` + 固定 4096 字节 buffer，遇到参数中包含逗号的字符串会错误分割。

VM 的 SUBSTR（vm.rs L521-533）使用 `splitn(3, ',')`，最多分 3 段，更安全。

**差异场景**：`SUBSTR "hello,world,test,extra" 0 5`
- VM：`splitn(3, ',')` → ["hello", "world", "test,extra"]，正确提取第一段的前 5 字符
- AOT：`strchr` 找第一个逗号，截断原始字符串 → `"hello"` 的 start=0 len=... 正确

实际上这个特定场景两边结果相同，但 SPLIT 存在更严重的歧义：如果被分割的字符串本身包含逗号，`splitn(3, ',')` 会错误地将其当成分隔符。

**根本修复**：改用 pipe 分隔符 `str|delim|index` 而非逗号，与 L-0 的整体约定一致。

#### P1-3. db.rs SQL 注入风险

cmd_update（db.rs L372-397）在 `col=val` 格式下直接拼接 SQL：
```rust
Some(format!("{} = '{}'", col, val))
```
虽然对单引号做了 escape（`replace("'", "''")`），但仍然不是参数化查询。

cmd_select（db.rs L332）直接将条件插入 WHERE 子句：
```rust
format!("SELECT {} FROM {} WHERE {}", columns, table, condition)
```

对于 AI 生成的代码，这不是传统意义上的安全漏洞（LLM 控制输入，不是用户控制），但如果 L-0 程序处理外部 HTTP 请求体并将其传入 DB 查询，就会暴露注入面。REST API 示例中 `HandlePost` 的 body parsing → db insert 路径就是这种场景。

**建议**：cmd_select 的条件部分至少增加基础关键词过滤，或在文档中明确标注安全边界。

#### P1-4. data.rs 使用 `unsafe` 的全局 HashMap

```rust
static mut JSON_STORE: Option<HashMap<String, String>> = None;

fn get_store() -> &'static mut HashMap<String, String> {
    unsafe { ... }
}
```

由于 data.rs 是单线程 stdin loop，当前不会有真正的 data race，但这是 Rust 社区强烈不推荐的模式。如果未来改为多线程处理，会直接 UB。

**建议**：改用 `std::sync::Mutex<HashMap>` 或 `lazy_static` + `Mutex`。

#### P1-5. README "Routing" 总结表仍写 CMP

L1764:
```
| **Routing** | Compare path prefix and method | `CMP`, `BEQ`, `SLICE`, `HLEN` |
```

示例代码已改用 SCMP，但总结表还写 CMP。应改为 `SCMP`。

---

### 🟢 P2：小问题（影响体验和一致性）

#### P2-1. asm.rs 错误消息无上下文

```rust
fn parse_u8(args: &[String], idx: usize) -> u8 {
    args.get(idx).expect("Missing Argument").parse().expect("Invalid u8")
}
```

当 LLM 生成的代码出错时，错误消息只有 "Missing Argument" 或 "Invalid u8"，没有行号、指令名、期望格式等信息。对于 LLM-in-the-loop 的自动修复流程，这严重影响修复成功率。

#### P2-2. aot.rs UPPER/LOWER 缺少 null terminator

aot.rs L246-251:
```c
for (int i = 0; arg[i] && i < sizeof(buf)-1; i++)
    buf[i] = (arg[i] >= 'a' && arg[i] <= 'z') ? arg[i] - 32 : arg[i];
```
循环结束后没有 `buf[len] = '\0'`。如果 `buf` 之前被用过且残留更长的字符串，UPPER/LOWER 的结果会包含上次调用的尾部。

**修复**：在循环后添加 `buf[i] = '\0';` 或在函数入口处 `memset(buf, 0, sizeof(buf))`。

#### P2-3. http.rs 大量 debug 日志残留

http.rs 的 `listen` / `send` 方法中包含约 15 处 `fs::OpenOptions::new().append(true).open("/tmp/l0_http_daemon.log")` 调用。生产代码不应有此类文件 I/O 级别的 debug 输出。

**建议**：引入 `--debug` 命令行参数或 `L0_DEBUG` 环境变量控制。

#### P2-4. db.rs config 文件硬编码路径

```rust
const DB_CONFIG_PATH: &str = ".l0_db_config.json";
```

如果 VM 的工作目录在多次 TEXEC 之间改变（理论上可能），config 文件会丢失。且多个 L-0 程序同时运行会互相覆盖。

**建议**：config 路径加入 PID 或实例 ID，或直接改为 plugin 启动参数传递数据库路径。

---

## 改进路线图

### Phase 1：文档修复（1-2 天）

| 任务 | 文件 | 具体修改 |
|------|------|----------|
| 1.1 统一指令数量 | README.md | L74, L118, L2303 → 56 |
| 1.2 更新 ISA 分类表 | README.md | L128 Control→6 含 SCMP, L129 Memory→9 含 NEWR |
| 1.3 修复 Routing 总结表 | README.md | L1764 CMP → SCMP |
| 1.4 修正 FILE_EXISTS 文档 | README.md | L324 改为 `"true"` / `"false"`，示例用 SCMP |
| 1.5 修复 JSON_GET 使用模式 | README.md + tools.json | 确保 0x6002 → `json_parse`，示例改用 pipe 分隔 |

### Phase 2：Plugin 架构升级（5-7 天）

**目标**：从每次 fork+exec 改为长连接进程池，性能提升 100-1000x。

#### 2.1 VM 端 Plugin 进程池

```rust
struct PluginProcess {
    child: std::process::Child,
    stdin: std::io::BufWriter<std::process::ChildStdin>,
    stdout: std::io::BufReader<std::process::ChildStdout>,
}

struct PluginPool {
    processes: HashMap<String, PluginProcess>,
}

impl PluginPool {
    fn call(&mut self, binary: &str, env_dir: &str, method: &str, arg: &str) -> String {
        let proc = self.get_or_spawn(binary, env_dir);
        let req = format!("{{\"method\":\"{}\",\"args\":[\"{}\"]}}\n",
                          method, escape_json(arg));
        proc.stdin.write_all(req.as_bytes()).ok();
        proc.stdin.flush().ok();

        let mut line = String::new();
        proc.stdout.read_line(&mut line).ok();
        parse_response(&line)
    }
}
```

所有 4 个 plugin（db.rs, http.rs, file.rs, data.rs）已经使用 stdin readline loop，无需修改 plugin 端。只改 vm.rs 的 `call_plugin` 方法。

#### 2.2 Plugin 健康检查 + 自动重启

```rust
fn is_alive(proc: &PluginProcess) -> bool {
    // 检查子进程是否还在运行
    proc.child.try_wait().ok().flatten().is_none()
}
```

如果 plugin 崩溃，自动 respawn 并重试。

### Phase 3：Assembler 强化（3-5 天）

#### 3.1 结构化错误消息

```rust
fn parse_u8(args: &[String], idx: usize, line: usize, op: &str) -> u8 {
    let s = args.get(idx).unwrap_or_else(|| {
        eprintln!("Error at line {}: '{}' expects argument #{}, got {} args total",
                  line, op, idx + 1, args.len());
        std::process::exit(1);
    });
    s.parse().unwrap_or_else(|_| {
        eprintln!("Error at line {}: '{}' arg #{} ('{}') is not a valid register (0-255)",
                  line, op, idx + 1, s);
        std::process::exit(1);
    })
}
```

可选：`--error-format=json` 输出 JSON 格式错误，供 LLM 自动修复流程解析。

#### 3.2 语义验证（Pass 3）

在编译完成后增加验证阶段：
- 跳转目标越界检查
- TEXEC tool ID 范围检查
- 寄存器 R240-R243 governance 保留区的意外使用告警
- HALT 后不可达代码告警
- SCAT 链未被 MARK/RESET 包围的泄漏告警

#### 3.3 宏支持（`.macro` / `.endm`）

```asm
.macro sb_append buf, len, str
    HLEN 4, $str
    SET 5, 0
    MEMCPY $buf, $len, $str, 5, 4
    ADD $len, $len, 4
.endm

# 使用
sb_append 0, 1, 2        # assembler 展开，自动 label 重命名
```

token 节省约 60-70%：LLM prompt 只需宏签名列表，不需完整 pattern 代码。

#### 3.4 `.include` 指令

```asm
.include "stdlib.l0inc"
```

标准库 pattern 打包为独立文件，减少 LLM system prompt 大小。

### Phase 4：AOT 一致性 + 安全（2-3 天）

| 任务 | 文件 | 具体修改 |
|------|------|----------|
| 4.1 UPPER/LOWER null terminator | aot.rs L246-251 | 循环后 `buf[i] = '\0'` |
| 4.2 SUBSTR/SPLIT 分隔符统一 | vm.rs + aot.rs | 改用 pipe `\|` 分隔，与 L-0 约定一致 |
| 4.3 data.rs unsafe 消除 | data.rs | `Mutex<HashMap>` 替代 `static mut` |
| 4.4 http.rs debug 日志控制 | http.rs | 环境变量 `L0_DEBUG=1` 控制 |
| 4.5 db.rs config 隔离 | db.rs | 路径加 PID / instance ID |

### Phase 5：LLM 生成闭环（5-7 天）

#### 5.1 `l0lint` 静态分析工具

独立 crate，检查项包括：
- Dead code（HALT 或无条件 JMP 之后的代码）
- SCAT 链泄漏（无 MARK/RESET）
- 相似 label 名冲突风险
- TEXEC 前 arg register 未赋值
- 无限循环检测（JMP target == 当前行且无条件改变）

#### 5.2 端到端测试套件

```
test/
  unit/                    # 每条指令独立测试
    test_scmp.asm          # 验证 SCMP
    test_newr.asm          # 验证 NEWR
    test_math.asm          # ADD/SUB/MUL/DIV/MOD
    ...
  integration/             # 完整场景测试
    test_fibonacci.asm
    test_string_builder.asm
    test_array_sort.asm
    test_db_crud.asm
  conformance/             # VM vs AOT 一致性
    test_vm_aot_parity/    # 同一 .asm 两种运行时，对比输出
```

每个测试文件头部包含预期行为注释：
```asm
# @test: output_equals
# @expect: "55"
# @timeout: 5
```

#### 5.3 README 自动一致性检测

从 README 提取所有 asm 代码块，自动编译运行并验证不报错：
```bash
#!/bin/bash
extract_asm_blocks README.md | while read block; do
    l0asm "$block" > /tmp/test.l0 2>&1 || echo "FAIL: $block"
done
```

### Phase 6：能力扩展（持续）

#### 6.1 Heredoc 多行字符串

Assembler 层语法糖，不改 ISA：
```asm
SETS 80, <<END
<!DOCTYPE html>
<html><body>Hello</body></html>
END
```

消除 REST API 示例中 18 行 SCAT 链。

#### 6.2 字符串插值

```asm
SETS 80, "Hello, ${R10}! You have ${R11} messages."
```

Assembler 展开为 SETS + ITOA + SCAT 链。

#### 6.3 HTTP 动态模式改进

将文件系统 IPC 替换为 Unix domain socket 或 named pipe，解决并发和超时问题。

#### 6.4 REPL 模式

```bash
$ l0vm --repl
L-0 REPL v0.2 (56 instructions)
> SETS 1, "Hello"
> TEXEC 0x5000, 1, 0
Hello
> DUMP
```

---

## 优先级总览

```
紧急度
  ▲
  │ Phase 1 ★★★★★  文档修复（指令数量、FILE_EXISTS、JSON_GET、Routing 表）
  │ Phase 2 ★★★★☆  Plugin 进程池（性能瓶颈）
  │ Phase 3 ★★★★☆  Assembler 强化（错误消息、验证、宏）
  │ Phase 4 ★★★☆☆  AOT 一致性 + 安全（null terminator、unsafe、日志）
  │ Phase 5 ★★★☆☆  LLM 闭环（lint、测试、README 自动验证）
  │ Phase 6 ★★☆☆☆  能力扩展（heredoc、插值、REPL）
  └──────────────────────────────────────────────────────────► 时间
       Day 1-2     Week 1      Week 2      Week 3     持续
```

---

## 修改文件速查表

| 任务 | isa.rs | asm.rs | vm.rs | aot.rs | README | db.rs | http.rs | file.rs | data.rs | 新文件 |
|------|:------:|:------:|:-----:|:------:|:------:|:-----:|:-------:|:-------:|:-------:|:------:|
| 1.1-1.5 文档 | | | | | ✅ | | | | | tools.json |
| 2.1-2.2 Plugin池 | | | ✅ | | | | | | | |
| 3.1 错误消息 | | ✅ | | | | | | | | |
| 3.2 验证 | | ✅ | | | | | | | | |
| 3.3-3.4 宏/include | | ✅ | | | ✅ | | | | | stdlib.l0inc |
| 4.1 null term | | | | ✅ | | | | | | |
| 4.2 分隔符统一 | | | ✅ | ✅ | ✅ | | | | | |
| 4.3 unsafe | | | | | | | | | ✅ | |
| 4.4 debug日志 | | | | | | | ✅ | | | |
| 4.5 config隔离 | | | | | | ✅ | | | | |
| 5.1 lint | | | | | | | | | | l0lint crate |
| 5.2 测试 | | | | | | | | | | test/*.asm |
| 6.1 heredoc | | ✅ | | | ✅ | | | | | |
| 6.2 插值 | | ✅ | | | ✅ | | | | | |
| 6.3 HTTP IPC | | | | | | | ✅ | | | |
| 6.4 REPL | | | ✅ | | | | | | | |

---

## 附录 A：Plugin 能力一览

审查了全部 4 个 plugin，当前支持的方法如下：

| Plugin | Tool IDs | 方法 | 备注 |
|--------|----------|------|------|
| file.rs | 0x4000-0x4005 | read, write, list, exists, delete | exists 返回 bool，非 "1"/"0" |
| data.rs | 0x6000-0x6004 | json_load, json_save, json_get, json_set, json_parse | get 从内存查，parse 从字符串提取 |
| http.rs | 0x8000-0x8009 | init, static, route, serve, serve_once, request, list_routes, listen, send, stop_dynamic | listen/send 用文件 IPC |
| db.rs | 0x9000-0x9007 | init, connect, create_table, insert, select, update, delete, drop_table, list_tables, exec, query | 条件直接拼 SQL |

## 附录 B：已确认正确的实现

以下部分经审查确认实现与文档一致，无需改动：

- ✅ SCMP 的 VM 和 AOT 实现语义一致（均使用字典序比较）
- ✅ NEWR 的 VM 和 AOT 实现语义一致
- ✅ MARK/RESET watermark 机制在 VM 和 AOT 中均正确实现
- ✅ 所有 4 个 plugin 均使用 stdin readline loop，天然支持长连接
- ✅ Vector 操作（VNEW/VSET/VGET/VDOT/VSIM/VMAG/VNORM）实现完整
- ✅ Governance 指令（LATCH/GUARD/TRAP/YIELD）实现完整
- ✅ SCAT 正确处理二进制拼接
- ✅ STORE64/LOAD64 使用 8 字节对齐
- ✅ asm.rs 的 parse_args 正确处理引号内逗号