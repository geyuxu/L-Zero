# L-0 一周发布计划 v3

> **目标**：2026-02-08（周日）前完成 GitHub 公开 + Zenodo DOI
>
> **原则**：只做阻塞发布的事。改进类工作全部推到 v0.3.0。

---

## 发布标准

一个 preview 版本的开源项目需要满足：

1. ✅ 代码能编译运行（已满足）
2. ❌ 文档自洽——读者按文档操作不会撞墙
3. ❌ LICENSE 文件
4. ❌ 最低限度的示例可运行
5. ❌ Zenodo metadata

第 2 点是这一周的核心工作量。第 3-5 点是半天的事。

---

## 日历视图

```
周一(2/2)    周二(2/3)    周三(2/4)    周四(2/5)    周五(2/6)    周六(2/7)    周日(2/8)
─────────   ─────────   ─────────   ─────────   ─────────   ─────────   ─────────
README      README      JSON_GET    冒烟测试     开源准备     Zenodo       缓冲日
数字修复    FILE_EXISTS  代码修复    基础测试     LICENSE      DOI
ISA分类表   示例修正    tools.json   README校对   CHANGELOG    发布!
AOT小修     Routing表                            .gitignore
```

---

## Day 1（周一）：README 数字 + AOT 小修

**预计 2-3 小时**。全部是文本替换，不涉及逻辑。

### 1.1 统一指令数量为 56

| 文件 | 行号 | 当前 | 改为 |
|------|------|------|------|
| README.md | L74 | `ISA Core (54 primitives)` | `ISA Core (56 primitives)` |
| README.md | L118 | `The Holographic ISA (54 Instructions)` | `The Holographic ISA (56 Instructions)` |
| README.md | L2303 | `54 ISA Primitives` | `56 ISA Primitives` |

### 1.2 更新 ISA 分类表（L122-134）

```markdown
| Control | 6 | CMP, SCMP, JMP, BEQ, BGT, BLT |
| Memory | 9 | NEW, NEWR, FREE, READ, WRITE, READR, WRITER, STORE64, LOAD64 |
```

总数验证：6+4+5+4+**6**+**9**+4+5+7+4+2 = **56** ✅

### 1.3 Routing 总结表（L1764）

```markdown
| **Routing** | Compare path prefix and method | `SCMP`, `BEQ`, `SLICE`, `HLEN` |
```

### 1.4 AOT UPPER/LOWER null terminator

aot.rs L246-251，UPPER 循环后加一行：
```c
buf[i] = '\0';
```
L249-251 LOWER 同理。

**完成标志**：`grep -c "54" README.md` 涉及指令计数的结果为 0。

DONE
---

## Day 2（周二）：FILE_EXISTS + 示例一致性

**预计 3-4 小时**。

### 2.1 修正 FILE_EXISTS 文档

README L324，当前：
```
| `0x4005` | FILE_EXISTS | "path" | `regs[dest] = exists(path) ? "1" : "0"` |
```

改为：
```
| `0x4005` | FILE_EXISTS | "path" | `regs[dest] = exists(path) ? "true" : "false"` |
```

### 2.2 修正 FILE_EXISTS 使用示例

README L336-342，当前用 ATOI 方式，改为 SCMP 方式：
```asm
# Example: Check if file exists
SETS 1, "data.json"
TEXEC 0x4005, 1, 2          # R2 = "true" or "false"
SETS 3, "true"
SCMP 2, 3                   # String compare
BEQ FileExists
```

### 2.3 修正 File I/O 完整示例

README L1782-1789，当前用 HLEN 判断长度：
```asm
HLEN 2, 1                      # R2 = string length
SET 3, 4                       # "true" = 4 chars
CMP 2, 3
```

改为直接 SCMP：
```asm
SETS 3, "true"
SCMP 1, 3                     # String compare with "true"
BEQ FileExists
```

### 2.4 Routing 总结表 FILE_EXISTS 行

同步检查 README 中所有 FILE_EXISTS 的使用，确保一致。

**完成标志**：README 中不再出现 `exists(path) ? "1" : "0"`。

---

## Day 3（周三）：JSON_GET 修复

**预计 4-5 小时**。这是本周最重的一天，涉及代码改动。

### 3.1 问题分析

REST API 示例使用 `SCAT json, key` + `TEXEC 0x6002` 来提取 JSON 字段。但 data.rs 的 json_get（0x6002）是从内存 HashMap 按 key 查找，不能从 JSON 字符串提取。

实际能解析 JSON 字符串的是 json_parse（0x6004），它期望 `"json_str|field"` 格式。

### 3.2 选择修复方案

**推荐方案 A：改 tools.json + README 示例**

理由：改最少的代码，不动 plugin 逻辑。

步骤：
1. tools.json 中 `0x6002` 的 method 从 `json_get` 改为 `json_parse`
2. README 中所有 `JSON_GET` 使用改为 pipe 分隔格式：

当前（L1541-1544）：
```asm
SETS 255, "method"
SCAT 1, 0, 255               # R1 = json + "method" (不工作)
TEXEC 0x6002, 1, 2
```

改为：
```asm
SETS 255, "|method"           # pipe + field
SCAT 1, 0, 255               # R1 = json + "|method"
TEXEC 0x6002, 1, 2            # json_parse -> R2 = "GET"
```

3. 同步修改 REST API 中所有 JSON_GET 调用（约 6 处：L1543, L1548, L1553, L1639, L1643, L1645）
4. README JSON section（L345-363）的示例也要改

### 3.3 更新 JSON tool 文档表

README L350，当前：
```
| `0x6002` | JSON_GET | "json\|key" | `regs[dest] = json[key]` |
```

改为更准确的描述：
```
| `0x6002` | JSON_PARSE | "json_str\|field_path" | `regs[dest] = extract(json_str, field_path)` |
```

### 3.4 验证

手动构造测试：
```asm
SETS 1, "{\"name\":\"Alice\",\"age\":30}"
SETS 2, "|name"
SCAT 3, 1, 2
TEXEC 0x6002, 3, 4       # 应该得到 "Alice"
TEXEC 0x5000, 4, 0       # 打印 "Alice"
HALT
```

编译运行确认输出正确。

**完成标志**：上述测试通过。

DONE

---

## Day 4（周四）：冒烟测试 + README 校对

**预计 3-4 小时**。

### 4.1 创建最小冒烟测试

在 `examples/` 目录下创建 3 个基础测试文件：

**examples/hello.asm**（已有，确认能跑）：
```asm
SETS 1, "Hello, L-0!"
TEXEC 0x5000, 1, 0
HALT
```

**examples/test_scmp.asm**（新建）：
```asm
# Test SCMP instruction
SETS 1, "hello"
SETS 2, "hello"
SETS 3, "world"
SCMP 1, 2
BEQ equal_ok
PANIC "SCMP equal failed"
equal_ok:
SCMP 1, 3
BEQ should_not_equal
JMP not_equal_ok
should_not_equal:
PANIC "SCMP not-equal failed"
not_equal_ok:
SETS 255, "SCMP tests passed"
TEXEC 0x5000, 255, 0
HALT
```

**examples/test_newr.asm**（新建）：
```asm
# Test NEWR instruction
SET 1, 64
NEWR 0, 1              # Allocate 64 bytes from register
HLEN 2, 0              # Should be 64
SET 3, 64
CMP 2, 3
BEQ size_ok
PANIC "NEWR size mismatch"
size_ok:
SETS 255, "NEWR tests passed"
TEXEC 0x5000, 255, 0
HALT
```

### 4.2 运行验证

```bash
# 编译 + 运行每个示例
for f in examples/*.asm; do
    echo "=== Testing $f ==="
    ./target/release/l0asm "$f" > /tmp/test.l0
    ./target/release/l0vm /tmp/test.l0
done
```

### 4.3 README 最终校对

通读全文一遍，重点检查：
- [ ] 所有指令计数 = 56
- [ ] SCMP 和 NEWR 在 ISA 表、分类表、示例中均出现
- [ ] FILE_EXISTS 描述一致
- [ ] JSON_GET/JSON_PARSE 描述一致
- [ ] 无 CMP 用于字符串比较的残留
- [ ] Routing 总结表正确

**完成标志**：3 个示例文件全部编译运行通过，校对清单全部打勾。

DONE
---

## Day 5（周五）：开源准备

**预计 2-3 小时**。

### 5.1 添加 LICENSE

项目根目录创建 `LICENSE` 文件，使用 MIT：

```
MIT License

Copyright (c) 2025-2026 [你的名字]

Permission is hereby granted, free of charge, to any person obtaining a copy
...
```

### 5.2 创建 CHANGELOG.md

```markdown
# Changelog

## v0.0.2-preview (2026-02-08)

### New Instructions
- `SCMP` - String content comparison (sets flags like CMP but compares heap strings)
- `NEWR` - Dynamic-size heap allocation from register value

### Bug Fixes
- Fixed: String routing in REST API examples (CMP → SCMP)
- Fixed: AOT HLEN inconsistency (now matches VM behavior)
- Fixed: PRINT/PRINTN description in Known Limitations
- Fixed: GAS instruction AOT value (1000000 → 1000)
- Fixed: AOT missing SUBSTR/SPLIT implementations
- Fixed: FILE_EXISTS documentation (returns "true"/"false")
- Fixed: JSON field extraction pattern (SCAT+pipe format)
- Fixed: README instruction count unified to 56
- Fixed: stdlib patterns (sb_init, sb_append, sb_finish, array_new)

### Known Issues
- Plugin processes spawned per TEXEC call (performance)
- HTTP dynamic mode uses file-based IPC
- See roadmap for planned improvements

## v0.0.1 (initial)
- 54-instruction ISA
- VM interpreter + AOT compiler
- 4 plugins: file, data, http, db
- Governance layer (LATCH/GUARD/TRAP/YIELD)
```

### 5.3 清理 .gitignore

```
/target/
*.l0
.l0_db_config.json
.l0_http_config.json
.l0_http_pending_*.json
/tmp/
*.o
*.out
```

### 5.4 确认目录结构

```
.
├── LICENSE
├── CHANGELOG.md
├── README.md
├── Cargo.toml
├── tools.json                ← 确认 0x6002 已修正
├── crates/
│   ├── l0_core/src/lib.rs    ← isa.rs
│   ├── l0_vm/src/main.rs     ← vm.rs
│   ├── l0_asm/src/main.rs    ← asm.rs
│   └── l0_compiler/src/main.rs ← aot.rs
├── plugins/
│   ├── file_plugin/src/main.rs
│   ├── data_plugin/src/main.rs
│   ├── http_plugin/src/main.rs
│   └── db_plugin/src/main.rs
├── examples/
│   ├── hello.asm
│   ├── test_scmp.asm
│   └── test_newr.asm
└── scripts/
    └── build_dist.sh
```

### 5.5 GitHub 仓库设置

- Repository 设为 public
- 添加 description: "ISA-level virtual machine for AI agent code generation with semantic drift governance"
- Topics: `ai`, `virtual-machine`, `code-generation`, `agent`, `assembly`, `rust`, `governance`
- 创建 Release: tag `v0.2.0-preview`, 标题 "L-Zero v0.2.0 Preview"
- Release notes 用 CHANGELOG 内容

**完成标志**：`git push` 成功，GitHub 页面可公开访问。

---

## Day 6（周六）：Zenodo 发布

**预计 1-2 小时**。

### 6.1 连接 Zenodo

1. 登录 https://zenodo.org（用 GitHub 账号）
2. Settings → GitHub → 开启你的 L-0 仓库
3. 回到 GitHub 确认 Release `v0.2.0-preview` 已创建
4. Zenodo 自动从 GitHub Release 生成 DOI

### 6.2 编辑 Zenodo Metadata

| 字段 | 填写内容 |
|------|----------|
| Title | L-Zero: An ISA-Level Virtual Machine for AI Agent Code Generation with Semantic Drift Governance |
| Authors | [你的全名], ORCID: [你的 ORCID] |
| Description | L-Zero (L-0) is a register-based virtual machine designed for AI code generation. It features a 56-instruction ISA, AOT compilation to C, a plugin architecture for database/HTTP/file operations, and an ISA-level semantic drift governance mechanism (LATCH/GUARD/TRAP/YIELD) for autonomous AI agents. |
| Keywords | AI code generation, virtual machine, assembly language, semantic drift detection, agent governance, LLM |
| License | MIT |
| Resource type | Software |
| Version | v0.2.0-preview |
| Related identifiers | (如有论文或博客链接) |

### 6.3 获取 DOI

Zenodo 生成后记录 DOI（格式：`10.5281/zenodo.XXXXXXX`）。

### 6.4 回填 DOI

在 README.md 顶部加一行 badge：
```markdown
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.XXXXXXX.svg)](https://doi.org/10.5281/zenodo.XXXXXXX)
```

在个人网站 yuxu.ge 和简历中添加引用。

**完成标志**：DOI 可解析，指向你的 Zenodo 页面。

---

## Day 7（周日）：缓冲日

如果前 6 天一切顺利，这天休息。

如果有延误，用这天做收尾。最可能的延误点：
- Day 3 的 JSON_GET 修复如果 tools.json 结构比预期复杂
- Day 4 的冒烟测试发现新 bug

---

## 明确不做的事情（推到 v0.3.0+）

| 任务 | 原 roadmap 阶段 | 为什么不做 |
|------|-----------------|-----------|
| Plugin 进程池 | Phase 2 | 工程量大（5-7天），preview 版不需要性能优化 |
| 宏支持 .macro/.endm | Phase 3 | 需要大改 asm.rs，与发布无关 |
| .include 指令 | Phase 3 | 同上 |
| l0lint 静态分析 | Phase 5 | 全新 crate，至少一周 |
| Heredoc / 字符串插值 | Phase 6 | 语法糖，不影响功能 |
| REPL 模式 | Phase 6 | 体验改进，不影响发布 |
| HTTP Unix socket IPC | Phase 6 | 架构重构，v0.3 |
| data.rs unsafe 消除 | Phase 4 | 实际无 bug，代码卫生问题 |
| db.rs SQL 注入防护 | Phase 4 | 需要设计参数化接口，不紧急 |
| 端到端测试套件 | Phase 5 | 3 个冒烟测试足够 preview |
| Assembler 结构化错误 | Phase 3 | 改进类，不阻塞 |

---

## 每日检查清单

### Day 1 ✅ checklist
- [ ] README L74: 54 → 56
- [ ] README L118: 54 → 56
- [ ] README L2303: 54 → 56
- [ ] README L128: Control 5 → 6, 加 SCMP
- [ ] README L129: Memory 8 → 9, 加 NEWR
- [ ] README L1764: CMP → SCMP
- [ ] aot.rs UPPER: 加 `buf[i] = '\0'`
- [ ] aot.rs LOWER: 加 `buf[i] = '\0'`
- [ ] `cargo build --release` 通过

### Day 2 ✅ checklist
- [ ] README L324: FILE_EXISTS 改为 "true"/"false"
- [ ] README L336-342: 示例改用 SCMP
- [ ] README L1782-1789: File I/O 示例改用 SCMP
- [ ] 全文搜索 `"1" : "0"` 无残留

### Day 3 ✅ checklist
- [ ] tools.json: 0x6002 method → json_parse
- [ ] README L350: JSON_GET → JSON_PARSE 描述
- [ ] README L354-363: JSON 示例改用 pipe 格式
- [ ] README REST API: 6 处 JSON_GET 调用全部改为 pipe 格式
- [ ] 手动测试 json_parse 能提取字段
- [ ] `cargo build --release` 通过

### Day 4 ✅ checklist
- [ ] examples/hello.asm 运行通过
- [ ] examples/test_scmp.asm 运行通过
- [ ] examples/test_newr.asm 运行通过
- [ ] README 通读校对完成
- [ ] 无 `54` 指令计数残留
- [ ] 无 CMP 用于字符串比较残留

### Day 5 ✅ checklist
- [ ] LICENSE 文件存在
- [ ] CHANGELOG.md 文件存在
- [ ] .gitignore 文件存在
- [ ] tools.json 已修正
- [ ] GitHub repo public
- [ ] GitHub Release v0.2.0-preview 已创建

### Day 6 ✅ checklist
- [ ] Zenodo 连接 GitHub
- [ ] Zenodo metadata 已编辑
- [ ] DOI 已生成
- [ ] README 添加 DOI badge
- [ ] 个人网站/简历已更新

### Day 7
- [ ] 所有上述检查项最终确认
- [ ] 或：休息

---

## Zenodo 引用格式（预填）

```bibtex
@software{l0vm2026,
  author       = {[你的名字]},
  title        = {L-Zero: An ISA-Level Virtual Machine for AI Agent
                  Code Generation with Semantic Drift Governance},
  month        = feb,
  year         = 2026,
  publisher    = {Zenodo},
  version      = {v0.2.0-preview},
  doi          = {10.5281/zenodo.XXXXXXX},
  url          = {https://doi.org/10.5281/zenodo.XXXXXXX}
}
```

发布后替换 XXXXXXX 为实际 DOI 编号。