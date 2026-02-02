# L-0 发布后推进计划 v4

> 前提：v0.2.0-preview 已于 2/8 完成 GitHub 开源 + Zenodo DOI
>
> 本文档覆盖 2/9 起的工作，目标是将 L-0 从"能跑的代码"变成"能产出学术成果的研究平台"

---

## 战略目标回顾

上次讨论的结论：L-0 作为通用语言替代 Lua 很难成功，但有两个真正有价值的方向：

**方向 A：Governance 论文** — 在 ISA 层做语义漂移检测（LATCH/GUARD/TRAP/YIELD），这个想法有差异化，可以发 workshop 甚至主会论文。

**方向 B：LLM 代码生成 Benchmark** — L-0 不在任何 LLM 训练数据中，是天然的"零样本代码生成"测试平台，直接服务于你 CodeSymbiosis 的研究方向。

这两个方向不冲突，共享同一个基础设施（L-0 本身），但需要的工程支撑不同。下面按时间线规划。

---

## 总览：8 周路线图

```
        工程基础                    研究产出
        ────────                    ────────
Week 1  v0.2.0 发布 ✅              -
Week 2  Plugin 进程池              Benchmark 实验设计
Week 3  Assembler 错误消息          Benchmark 数据收集（Pilot）
Week 4  基础测试套件                Benchmark 初步结果
Week 5  宏系统 .macro              Governance 实验设计
Week 6  缓冲 / 修 bug              Governance 实验 + 数据
Week 7  v0.3.0 发布                论文初稿（选一篇）
Week 8  缓冲                       论文修改 + 投稿
```

每周预算：假设你 MSc 课业占 60% 时间，L-0 可用约 **15-20 小时/周**。

---

## Week 2（2/9-2/15）：Plugin 进程池 + Benchmark 设计

### 工程：Plugin 进程池（~10 小时）

**为什么先做这个**：Benchmark 实验需要大量运行 L-0 程序，每次 TEXEC 都 fork 的话跑 100 个测试程序要等很久。进程池是后续一切实验的性能前提。

**改动范围**：只改 vm.rs 的 `call_plugin` 方法，plugin 端不动。

```rust
// vm.rs 新增
struct PluginPool {
    processes: HashMap<String, PluginProcess>,
}

impl PluginPool {
    fn call(&mut self, binary: &str, env_dir: &str, method: &str, arg: &str) -> String {
        let proc = self.get_or_spawn(binary, env_dir);
        // 写请求 → 读响应，复用同一进程
        write_request(proc, method, arg);
        read_response(proc)
    }

    fn get_or_spawn(&mut self, binary: &str, env_dir: &str) -> &mut PluginProcess {
        if !self.processes.contains_key(binary) || !self.is_alive(binary) {
            self.spawn(binary, env_dir);
        }
        self.processes.get_mut(binary).unwrap()
    }
}
```

**验收标准**：REST API 示例单次请求延迟从 ~50ms 降到 <5ms。

### 研究：Benchmark 实验设计（~5 小时）

目标是设计一个实验方案，回答这个问题：

> **不同 LLM 在零训练数据语言上的代码生成能力差异有多大？**

#### 实验变量

| 变量 | 取值 |
|------|------|
| LLM | Claude Sonnet 4, GPT-4o, Gemini 2.0 Flash, DeepSeek-V3, Llama 3.3 70B |
| Prompt 策略 | Zero-shot / Few-shot (3 examples) / Full-spec (README as system prompt) |
| 任务难度 | L1 基础 / L2 中等 / L3 复杂 |

#### 任务集（15 题，每级 5 题）

**L1 基础**（单一控制流，无 plugin）：
1. 求 1 到 N 的和
2. 判断奇偶
3. 字符串反转（byte level）
4. 数组最大值
5. Fibonacci 第 N 项

**L2 中等**（需要 TEXEC + 字符串操作）：
1. 读文件并统计行数
2. JSON 解析并提取指定字段
3. 用户输入猜数字游戏
4. 字符串构建器拼接 HTML 片段
5. 数组冒泡排序并打印

**L3 复杂**（多 plugin 协作 + 复杂控制流）：
1. SQLite CRUD 完整操作
2. HTTP 静态服务器（3 个路由）
3. 文件读取 → 转大写 → 写新文件 → 验证
4. 向量余弦相似度计算
5. Governance: LATCH 目标向量 → GUARD 检测漂移

#### 评估指标

| 指标 | 定义 |
|------|------|
| **编译率** | 生成代码能通过 l0asm 编译的比例 |
| **执行率** | 编译后能运行不 panic 的比例 |
| **正确率** | 输出与预期完全匹配的比例 |
| **Token 效率** | 生成代码的 token 数 / 任务复杂度 |
| **修复率** | 首次失败后，给出错误信息再生成一次的修复成功率 |

**本周交付物**：实验设计文档（experiment_design.md），包含完整任务集 + 预期输出 + 评分标准。

---

## Week 3（2/16-2/22）：Assembler 错误消息 + Pilot 实验

### 工程：Assembler 结构化错误（~8 小时）

**为什么现在做**：Benchmark 的"修复率"指标依赖于 assembler 给出有意义的错误信息。当前的 "Missing Argument" 对 LLM 没用。

改 asm.rs 的所有 `parse_*` 函数，添加行号和指令上下文：

```rust
fn parse_u8(args: &[String], idx: usize, line: usize, op: &str) -> u8 {
    let s = args.get(idx).unwrap_or_else(|| {
        eprintln!("L{}: {} expects arg #{} (register 0-255), got {} args",
                  line, op, idx + 1, args.len());
        std::process::exit(1);
    });
    s.parse().unwrap_or_else(|_| {
        eprintln!("L{}: {} arg #{} '{}' is not a valid register (0-255)",
                  line, op, idx + 1, s);
        std::process::exit(1);
    })
}
```

可选增加 `--error-format=json`：
```json
{"line":42,"op":"SCMP","error":"expects arg #2 (register), got 'hello'"}
```

**验收标准**：故意写错的 .asm 文件，错误消息包含行号和指令名。

### 研究：Pilot 实验（~8 小时）

用 **1 个 LLM（Claude Sonnet 4）+ 3 种 prompt 策略 + 5 个 L1 任务** 做 pilot：

```
实验矩阵（pilot）：1 × 3 × 5 = 15 次生成
```

每次生成记录：
- 原始 prompt
- LLM 输出
- 编译结果（pass/fail + 错误信息）
- 执行结果（pass/fail + 输出）
- 如果失败：将错误反馈给 LLM，记录第二次尝试

**目的**：验证实验流程可行，调整任务难度（如果 L1 全通过就太简单了，如果全挂就太难了，目标是 40-70% 首次正确率）。

**本周交付物**：pilot_results.md，含 15 次实验的原始数据和初步分析。

---

## Week 4（2/23-3/1）：测试套件 + 完整 Benchmark 数据

### 工程：基础测试套件（~6 小时）

根据 pilot 经验，完善测试基础设施：

```
tests/
  runner.sh                    # 自动化测试脚本
  benchmark/
    tasks.json                 # 任务定义：题目、预期输出、难度级别
    prompts/
      zero_shot.txt            # prompt 模板
      few_shot.txt
      full_spec.txt
    results/                   # 实验结果存放
  unit/                        # 回归测试
    test_all_instructions.asm  # 56 条指令各测一次
```

`runner.sh` 核心逻辑：
```bash
for task in $(jq -r '.tasks[].id' tasks.json); do
    expected=$(jq -r ".tasks[] | select(.id==\"$task\") | .expected" tasks.json)
    echo "$generated_code" | l0asm /dev/stdin > /tmp/test.l0 2>compile_err.txt
    if [ $? -eq 0 ]; then
        actual=$(timeout 5 l0vm /tmp/test.l0 2>/dev/null)
        [ "$actual" = "$expected" ] && echo "PASS" || echo "WRONG: got '$actual'"
    else
        echo "COMPILE_FAIL: $(cat compile_err.txt)"
    fi
done
```

### 研究：完整 Benchmark 数据收集（~10 小时）

扩展到全部 5 个 LLM × 3 种 prompt × 15 个任务：

```
实验矩阵（完整）：5 × 3 × 15 = 225 次生成
每次生成含最多 2 轮（首次 + 修复），总计最多 450 次 API 调用
```

数据收集自动化脚本（可以用 Python 调各家 API）。

**本周交付物**：
- `benchmark_results.csv`：225 行，每行含 LLM/prompt/task/compile/execute/correct/tokens/repair 字段
- 初步统计分析：编译率、正确率、修复率的分布

---

## Week 5（3/2-3/8）：宏系统 + Governance 实验设计

### 工程：宏系统 .macro/.endm（~10 小时）

**为什么做**：Governance 实验需要写更复杂的 L-0 程序，没有宏的话 SCAT 链会让实验代码极其冗长，而且 Benchmark 论文的 "Full-spec prompt" 也可以从宏的角度讨论 token 效率。

asm.rs 新增预处理阶段（Pass 0）：

```
源码 → Pass 0（宏展开）→ Pass 1（label 收集）→ Pass 2（指令编码）
```

宏语法：
```asm
.macro print_str reg
    TEXEC 0x5000, $reg, 0
.endm

.macro json_get json_reg, field_name, dest_reg
    SETS 255, "|$field_name"
    SCAT 254, $json_reg, 255
    TEXEC 0x6002, 254, $dest_reg
.endm
```

**验收标准**：REST API 示例用宏重写后，代码量减少 50%+。

### 研究：Governance 实验设计（~5 小时）

目标是设计实验回答：

> **ISA 层语义漂移检测能否有效约束 AI agent 的行为偏离？**

#### 实验场景

一个 L-0 agent 被赋予任务："搜索并总结关于 climate change 的文章"。任务通过 LATCH 向量编码。

```asm
# 任务编码：climate_change 向量
VNEW 240, 10          # 10 维目标向量
VSET 240, 0, ...      # 设置各维度值（预先编码）
SET 242, 800000       # 阈值 = 0.8（× 1M）
LATCH 240, 242        # 锁定目标

# Agent 主循环
AgentLoop:
    # ... agent 行为 ...
    # 更新当前状态向量
    VNEW 241, 10
    VSET 241, 0, ...  # 基于当前行为计算状态向量
    GUARD 241          # 检测漂移 → 如果 <0.8 自动 TRAP
    JMP AgentLoop
```

#### 对比组

| 组 | 设置 | 预期 |
|----|------|------|
| Control | 无 Governance，agent 自由执行 | 容易漂移到无关话题 |
| Low threshold | LATCH 阈值 = 0.5 | 宽松约束，部分漂移被捕获 |
| High threshold | LATCH 阈值 = 0.8 | 严格约束，大部分漂移被捕获 |
| Adaptive | 阈值随任务进度动态调整 | 最佳平衡 |

#### 评估指标

| 指标 | 定义 |
|------|------|
| **漂移检测率** | GUARD 触发 TRAP 时，确实发生语义漂移的比例（precision） |
| **漂移遗漏率** | 实际发生漂移但 GUARD 未触发的比例（1 - recall） |
| **任务完成度** | agent 最终输出与任务目标的相关性（人工评分 1-5） |
| **TRAP 频率** | 每 N 步触发一次 TRAP（太频繁说明阈值太紧） |

**本周交付物**：governance_experiment.md，含完整实验方案。

---

## Week 6（3/9-3/15）：Governance 实验 + 缓冲

### 研究：Governance 数据收集（~12 小时）

运行实验：
- 编写 4 组 L-0 agent 程序（control + 3 实验组）
- 每组运行 20 次（不同随机种子 / LLM 输入变化）
- 记录 TRAP 触发次数、时机、agent 输出质量

**关键挑战**：状态向量的编码方式。需要将 agent 的"当前行为语义"映射到向量空间。

简单方案：用 TEXEC 调外部 embedding API（如果有网络的话），或用预定义的 keyword→dimension 映射表。

务实方案（推荐）：不追求 embedding 的完美性，而是用手工设计的 10 维特征向量（topic relevance, output length, keyword overlap, ...），重点验证 GUARD 机制本身的有效性。

### 工程：修 bug / 缓冲（~5 小时）

前几周大概率会暴露新问题：
- Benchmark 过程中发现的 VM/AOT 不一致
- 宏系统的 edge case
- Plugin 进程池在长时间运行中的稳定性

**预留时间处理。**

---

## Week 7（3/16-3/22）：v0.3.0 发布 + 论文初稿

### 工程：v0.3.0 Release（~3 小时）

打包本轮改进：
- Plugin 进程池
- Assembler 结构化错误
- 宏系统
- 测试套件
- bug fixes

```
v0.3.0 CHANGELOG:
- Feature: Plugin process pool (100x faster TEXEC for plugins)
- Feature: Assembler structured error messages with line numbers
- Feature: .macro/.endm support
- Feature: Basic test suite
- Fix: [accumulated bug fixes]
```

GitHub Release + Zenodo 更新 DOI。

### 研究：论文初稿（~12 小时）

**选择一篇先写**。根据数据质量决定：

#### 选项 A：Benchmark 论文

适合投稿：EMNLP Workshop, NeurIPS Workshop, 或 arXiv preprint

标题方向：*"Beyond Memorization: Benchmarking LLM Code Generation on Zero-Training-Data Languages"*

结构：
1. Introduction：LLM 代码生成能力可能更多来自记忆而非理解
2. L-0 Language：56 指令 ISA，设计意图
3. Benchmark Design：15 任务 × 3 难度 × 3 prompt 策略
4. Results：5 个 LLM 的编译率/正确率/修复率对比
5. Analysis：哪些因素影响零样本代码生成？Prompt 策略的影响有多大？
6. Discussion：对 AI-assisted programming 工具设计的启示
7. Conclusion

#### 选项 B：Governance 论文

适合投稿：AAMAS, AAAI Workshop on Safe AI, 或 arXiv

标题方向：*"ISA-Level Semantic Drift Governance for Autonomous Code Agents"*

结构：
1. Introduction：AI agent 在长任务中的语义漂移问题
2. L-0 Governance ISA：LATCH/GUARD/TRAP/YIELD 的设计
3. Experimental Setup：agent 任务、漂移模拟、阈值设置
4. Results：漂移检测率、任务完成度、TRAP 频率
5. Analysis：阈值选择的 trade-off
6. Related Work：guardrails, constitutional AI, RLHF 的对比
7. Conclusion

**我的建议**：先写 Benchmark 论文。理由：
- 数据更客观（编译率是 0/1，不需要人工标注）
- 实验规模更大（225 个数据点 vs Governance 的 80 个）
- 与你 CodeSymbiosis 研究直接相关
- 更容易被 reviewer 理解（LLM benchmark 是热门话题）

Governance 论文作为第二篇，可以在 PhD 阶段深化。

---

## Week 8（3/23-3/29）：论文修改 + 投稿

### 论文修改（~10 小时）

- 找 1-2 个人 review（同学、导师、或 PhD supervisor 如果已确定）
- 补充 related work（搜索最新的 LLM code generation 论文）
- 润色图表（结果可视化：热力图显示 LLM × task × prompt 的正确率矩阵）
- 检查引用格式

### 投稿或发布（~3 小时）

**方案 1（推荐）**：先发 arXiv preprint，再投会议
- 优点：立即获得 citable 引用，不等审稿周期
- 操作：上传 arXiv，同步更新 Zenodo，简历加上 preprint 链接

**方案 2**：直接投 workshop
- 看 deadline 对齐：EMNLP 2026 workshop 通常 6-8 月截稿

### 传播（~2 小时）

- 博客文章：yuxu.ge 上写一篇 L-0 Benchmark 的科普版
- LinkedIn / Twitter 发布
- 发给 Southampton 的 Dr. Jackson（follow-up 邮件的完美借口）

---

## 工程 vs 研究时间分配

```
         工程        研究
Week 2   ████████░░  ░░░░██████   10h : 5h
Week 3   ████████░░  ░░████████   8h : 8h
Week 4   ██████░░░░  ██████████   6h : 10h
Week 5   ██████████  ░░░░██████   10h : 5h
Week 6   █████░░░░░  ████████████ 5h : 12h
Week 7   ███░░░░░░░  ████████████ 3h : 12h
Week 8   ░░░░░░░░░░  █████████████ 0h : 13h
         ──────────  ──────────────
Total    ~42h        ~65h
```

前半段偏工程（为实验建基础设施），后半段偏研究（收数据、写论文）。总计约 107 小时，8 周平均每周 ~13 小时，在 MSc 课业之外可行。

---

## v2 遗留问题处理方式

| v2 原编号 | 问题 | 处理 |
|-----------|------|------|
| P0-4 HTTP 文件 IPC | 并发风险 | **推迟**。Benchmark 不测 HTTP 动态模式 |
| P1-2 AOT SUBSTR/SPLIT 差异 | 分隔符歧义 | **Week 6 缓冲期修**，改用 pipe |
| P1-3 db.rs SQL 注入 | 安全隐患 | **推迟**。文档加安全提示即可 |
| P1-4 data.rs unsafe | 代码卫生 | **Week 6 缓冲期修**，改 Mutex |
| P2-3 http.rs debug 日志 | 残留日志 | **Week 7 v0.3.0 发布前清理** |
| P2-4 db.rs config 硬编码 | 多实例冲突 | **推迟**。单实例下无影响 |

---

## 关键里程碑

| 日期 | 里程碑 | 可交付物 |
|------|--------|----------|
| 2/8 | v0.2.0-preview 发布 | GitHub public + Zenodo DOI |
| 2/15 | 实验设计完成 | experiment_design.md |
| 2/22 | Pilot 数据 | pilot_results.md（15 个数据点） |
| 3/1 | 完整 Benchmark 数据 | benchmark_results.csv（225 个数据点） |
| 3/8 | Governance 实验设计 | governance_experiment.md |
| 3/15 | Governance 数据 | governance_results.csv |
| 3/22 | v0.3.0 发布 + 论文初稿 | 代码 release + paper draft |
| 3/29 | arXiv preprint | citable paper + 博客 + 传播 |

---

## 风险与应对

| 风险 | 概率 | 影响 | 应对 |
|------|------|------|------|
| Benchmark 正确率全为 0% 或 100% | 中 | 论文无意义 | Pilot 阶段调整难度。如果太难就降低 L3，如果太简单就加 L4 |
| Governance 向量编码不靠谱 | 高 | 实验结果噪声大 | 用手工特征向量，不追求 embedding 质量。重点是验证 GUARD 机制本身 |
| 宏系统实现超预期复杂 | 中 | Week 5 延期 | 先做最简版本（无嵌套宏、无递归），够用就行 |
| MSc 课业突然加重 | 中 | 整体延期 | Week 6 和 8 是缓冲。最坏情况砍掉 Governance，只出 Benchmark 论文 |
| API 费用超预算 | 低 | 无法测所有 LLM | 砍掉 Llama（本地推理太慢），保留 4 个云端 LLM |

---

## 最低可行成果（如果只有 4 周时间）

如果时间极度紧张，砍到骨头：

- **Week 2**：Plugin 进程池 + 实验设计
- **Week 3**：Pilot + 完整数据收集（跳过 assembler 改进，手动处理错误消息）
- **Week 4**：论文初稿
- **Week 5**：arXiv

不做 Governance 实验、不做宏系统、不做 v0.3.0。用 v0.2.0 的代码 + Benchmark 数据直接出一篇 short paper。

这仍然是一个有价值的成果：一篇关于 LLM 零样本代码生成的 benchmark 论文 + 一个 citable 的开源项目。