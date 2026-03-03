# OpenLife 架构改进计划

## TL;DR

> **Quick Summary**: 重构OpenLife从"CLI包装器"升级为"真正集成ZeroClaw的生物信息学Agent"，修复架构缺陷，增强路由系统，完善功能实现。
> 
> **Deliverables**:
> - 真正集成ZeroClaw库级别的OpenLife核心
> - 基于LLM的智能路由系统
> - 完整的可复现性系统
> - 安全沙箱执行环境
> - 完善的测试和文档体系
> 
> **Estimated Effort**: XL (大型重构)
> **Parallel Execution**: YES - 5 waves
> **Critical Path**: Wave 1 → Wave 2 → Wave 3 → Wave 4 → Wave Final

---

## Context

### Original Request
研究OpenLife项目代码，了解zeroclaw和clawbio架构，找出问题并给出改进意见。

### Interview Summary
**研究范围**:
- OpenLife: 3个Rust源文件（~550行），22个skill定义
- ZeroClaw: 100+ Rust源文件，完整Agent框架
- ClawBio: 46个Python文件，成熟生物信息学技能库

**核心发现**:
1. **架构问题**: OpenLife仅通过`std::process::Command`调用外部zeroclaw二进制，非库级别集成
2. **功能缺失**: 未使用ZeroClaw的providers/tools/memory/channels/security等模块
3. **路由简单**: 仅关键词匹配，无法处理模糊/多意图查询
4. **可复现性弱**: ClawBio的`commands.sh + environment.yml + checksums`未实现
5. **安全缺失**: 无沙箱执行，Python脚本直接运行

### Research Findings

| 维度 | ClawBio | 当前OpenLife | ZeroClaw能力 |
|------|---------|-------------|-------------|
| LLM集成 | ✅ OpenClaw | ❌ 无 | ✅ 多provider |
| 工具系统 | Python脚本 | Python脚本 | ✅ 50+工具 |
| 记忆系统 | ❌ | ❌ | ✅ SQLite/向量 |
| 安全沙箱 | ⚠️ 基础 | ❌ | ✅ landlock/bubblewrap |
| 多渠道 | ❌ | ❌ | ✅ Telegram/Discord |
| 可复现 | ✅ 完整 | ⚠️ 部分 | N/A |
| Daemon | ❌ | ❌ | ✅ |

---

## Work Objectives

### Core Objective
将OpenLife从"CLI包装器"重构为"真正集成ZeroClaw的生物信息学Agent"，使其具备：
1. 高性能Rust原生执行
2. 智能LLM路由
3. 完整可复现性
4. 安全沙箱环境
5. 多渠道接入能力

### Concrete Deliverables
- `src/bio/core.rs` - 集成ZeroClaw的核心模块
- `src/bio/router.rs` - 基于LLM的智能路由
- `src/bio/repro.rs` - 可复现性系统
- `src/bio/sandbox.rs` - 安全沙箱
- `src/config/bio.rs` - Bio配置模块
- 更新的`Cargo.toml` - 真正依赖ZeroClaw

### Definition of Done
- [ ] `cargo test` 全部通过
- [ ] 所有bio skills可通过新架构执行
- [ ] 可复现性包自动生成
- [ ] Python脚本在沙箱中执行
- [ ] 文档更新完成

### Must Have
- 库级别集成ZeroClaw (非进程调用)
- LLM意图识别路由
- 强制可复现性输出
- 安全沙箱执行

### Must NOT Have (Guardrails)
- 不破坏现有skill兼容性
- 不删除现有Python脚本
- 不修改ZeroClaw源代码
- 不引入不必要的依赖

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: NO (OpenLife无测试)
- **Automated tests**: TDD (先写测试，再实现)
- **Framework**: Rust内置test + tokio-test
- **If TDD**: 每个任务遵循RED (failing test) → GREEN (minimal impl) → REFACTOR

### QA Policy
Every task MUST include agent-executed QA scenarios.

- **Core Logic**: Use Rust `cargo test` — Assert outputs, error handling, edge cases
- **CLI**: Use Bash — Run commands, validate exit codes, check output
- **Skills**: Use Bash (python) — Run skill with demo data, verify report generation
- **Integration**: Use Bash — End-to-end workflow testing

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation - MUST complete first):
├── Task 1: ZeroClaw库级别集成 [deep]
├── Task 2: 配置系统重构 [quick]
├── Task 3: 错误处理框架 [quick]
└── Task 4: 日志系统集成 [quick]

Wave 2 (Core Systems - depends on Wave 1):
├── Task 5: LLM意图识别路由 [ultrabrain]
├── Task 6: Skill Registry重构 [deep]
├── Task 7: 工具执行层集成 [unspecified-high]
└── Task 8: 记忆系统集成 [unspecified-high]

Wave 3 (Bio-Specific - depends on Wave 2):
├── Task 9: 可复现性系统 [deep]
├── Task 10: 安全沙箱实现 [unspecified-high]
├── Task 11: VCF解析模块 [deep]
└── Task 12: HEIM评分移植 [deep]

Wave 4 (Enhancements - depends on Wave 3):
├── Task 13: 多技能链式调用 [artistry]
├── Task 14: Channel集成 [unspecified-high]
├── Task 15: Daemon模式 [unspecified-high]
└── Task 16: Web Gateway [visual-engineering]

Wave FINAL (Verification - after ALL tasks):
├── Task F1: 架构合规审计 (oracle)
├── Task F2: 代码质量检查 (unspecified-high)
├── Task F3: 集成测试 (deep)
└── Task F4: 文档完整性 (unspecified-high)

Critical Path: Task 1 → Task 5 → Task 9 → Task 13 → F1-F4
Parallel Speedup: ~60% faster than sequential
Max Concurrent: 4 (Waves 1 & 2)
```

### Dependency Matrix

- **1**: — — 2,3,4,5,6,7,8
- **2**: 1 — 5,6
- **3**: 1 — 5,6,7
- **4**: 1 — 5,7,8
- **5**: 1,2,3,4 — 9,13
- **6**: 1,2 — 7,9,13
- **7**: 1,3,6 — 9,10,11
- **8**: 1,4 — 9,13
- **9**: 5,6,7,8 — 13,F3
- **10**: 7 — 13,F3
- **11**: 7 — 12,F3
- **12**: 7,11 — F3
- **13**: 5,6,8,9,10 — F3
- **14**: 1 — 15
- **15**: 14 — 16
- **16**: 15 — F3
- **F1-F4**: 1-16 — —

### Agent Dispatch Summary

- **Wave 1**: **4** — T1→deep, T2-T4→quick
- **Wave 2**: **4** — T5→ultrabrain, T6→deep, T7-T8→unspecified-high
- **Wave 3**: **4** — T9,T11,T12→deep, T10→unspecified-high
- **Wave 4**: **4** — T13→artistry, T14-T15→unspecified-high, T16→visual-engineering
- **FINAL**: **4** — F1→oracle, F2-F4→unspecified-high

---

## TODOs

> Implementation + Test = ONE Task. Never separate.
> EVERY task MUST have: Recommended Agent Profile + Parallelization info + QA Scenarios.
> **A task WITHOUT QA Scenarios is INCOMPLETE. No exceptions.**

---

### Wave 1: Foundation (必须先完成)

- [ ] 1. **ZeroClaw库级别集成**

  **What to do**:
  - 移除`run_zeroclaw()`进程调用函数
  - 添加ZeroClaw作为Cargo依赖
  - 创建`src/bio/core.rs`集成ZeroClaw的核心trait
  - 重构main.rs使用库级别调用

  **Must NOT do**:
  - 不要修改ZeroClaw源代码
  - 不要破坏现有CLI命令签名

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: 需要深入理解Rust trait系统和ZeroClaw架构
  - **Skills**: [`git-master`]
    - `git-master`: 需要仔细管理依赖变更

  **Parallelization**:
  - **Can Run In Parallel**: NO (其他任务依赖此任务)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 2-16
  - **Blocked By**: None (can start immediately)

  **References**:
  - `src/main.rs:62-68` - 当前run_zeroclaw实现
  - `../zeroclaw/src/lib.rs` - ZeroClaw公共API
  - `../zeroclaw/src/providers/traits.rs` - Provider trait
  - `../zeroclaw/src/tools/traits.rs` - Tool trait

  **Acceptance Criteria**:
  - [ ] Cargo.toml包含zeroclaw依赖
  - [ ] 无`std::process::Command::new("zeroclaw")`调用
  - [ ] `cargo build`成功

  **QA Scenarios**:
  ```
  Scenario: 库级别集成验证
    Tool: Bash (cargo)
    Steps:
      1. cargo build --release
      2. grep -r "std::process::Command::new.*zeroclaw" src/ && exit 1 || exit 0
    Expected Result: grep返回1（未找到），build成功
    Evidence: .sisyphus/evidence/task-1-lib-integration.txt
  ```

  **Commit**: NO (groups with Wave 1)

---

- [ ] 2. **配置系统重构**

  **What to do**:
  - 创建`src/config/mod.rs`和`src/config/bio.rs`
  - 定义BioConfig结构体
  - 实现配置文件加载(~/.openlife/config.toml)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 3, 4)
  - **Blocks**: Tasks 5, 6
  - **Blocked By**: Task 1

  **References**:
  - `../zeroclaw/src/config/schema.rs` - ZeroClaw配置模式
  - `docs/PRODUCT_DESIGN.md:324-356` - 设计文档中的配置设计

  **Acceptance Criteria**:
  - [ ] BioConfig结构体定义
  - [ ] 配置文件加载函数

  **QA Scenarios**:
  ```
  Scenario: 配置加载
    Tool: Bash (cargo test)
    Steps:
      1. cargo test config::tests
    Expected Result: 所有配置测试通过
    Evidence: .sisyphus/evidence/task-2-config-test.txt
  ```

  **Commit**: NO (groups with Wave 1)

---

- [ ] 3. **错误处理框架**

  **What to do**:
  - 定义BioError枚举类型
  - 实现From trait转换
  - 添加错误链追踪

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 4)
  - **Blocks**: Tasks 5, 6, 7
  - **Blocked By**: Task 1

  **QA Scenarios**:
  ```
  Scenario: 错误处理验证
    Tool: Bash (cargo test)
    Steps:
      1. cargo test error
    Expected Result: 错误处理测试通过
    Evidence: .sisyphus/evidence/task-3-error-test.txt
  ```

  **Commit**: NO (groups with Wave 1)

---

- [ ] 4. **日志系统集成**

  **What to do**:
  - 添加tracing依赖
  - 创建日志初始化函数
  - 添加关键路径的instrumentation

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Tasks 2, 3)
  - **Blocks**: Tasks 5, 7, 8
  - **Blocked By**: Task 1

  **QA Scenarios**:
  ```
  Scenario: 日志验证
    Tool: Bash (RUST_LOG)
    Steps:
      1. RUST_LOG=debug cargo run -- version 2>&1 | grep -i "openlife"
    Expected Result: 日志输出包含openlife相关信息
    Evidence: .sisyphus/evidence/task-4-logging.txt
  ```

  **Commit**: NO (groups with Wave 1)

---

### Wave 2: Core Systems (依赖Wave 1)

- [ ] 5. **LLM意图识别路由**

  **What to do**:
  - 扩展BioIntent枚举支持多意图
  - 实现LLM意图识别函数
  - 添加置信度评分
  - 实现意图澄清对话

  **Must NOT do**:
  - 不要使用硬编码关键词作为唯一路由方式

  **Recommended Agent Profile**:
  - **Category**: `ultrabrain`
    - Reason: 需要深度理解NLP和意图识别，设计复杂的fallback策略
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 6, 7, 8)
  - **Blocks**: Tasks 9, 13
  - **Blocked By**: Tasks 1, 2, 3, 4

  **References**:
  - `src/bio/orchestrator.rs` - 当前关键词路由
  - `../zeroclaw/src/providers/traits.rs` - Provider trait

  **QA Scenarios**:
  ```
  Scenario: 意图识别 - 药物基因组
    Tool: Bash (cargo test)
    Steps:
      1. cargo test intent_pharmacogenomics
    Expected Result: 识别"我的CYP2D6基因对药物有什么影响"为Pharmacogenomics
    Evidence: .sisyphus/evidence/task-5-intent-pgx.txt
  ```

  **Commit**: NO (groups with Wave 2)

---

- [ ] 6. **Skill Registry重构**

  **What to do**:
  - 创建SkillRegistry结构体
  - 实现skill加载和缓存
  - 添加skill版本管理

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 7, 8)
  - **Blocks**: Tasks 7, 9, 13
  - **Blocked By**: Tasks 1, 2

  **QA Scenarios**:
  ```
  Scenario: Skill加载
    Tool: Bash (cargo test)
    Steps:
      1. cargo test skill_registry
    Expected Result: 所有skill正确加载和缓存
    Evidence: .sisyphus/evidence/task-6-registry.txt
  ```

  **Commit**: NO (groups with Wave 2)

---

- [ ] 7. **工具执行层集成**

  **What to do**:
  - 集成ZeroClaw的Tool trait
  - 实现BioTool适配器
  - 添加Python执行工具
  - 实现执行超时和取消

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 8)
  - **Blocks**: Tasks 9, 10, 11
  - **Blocked By**: Tasks 1, 3, 6

  **QA Scenarios**:
  ```
  Scenario: Python执行
    Tool: Bash (cargo test)
    Steps:
      1. cargo test tool_python_execution
    Expected Result: Python脚本执行成功
    Evidence: .sisyphus/evidence/task-7-tool-exec.txt
  ```

  **Commit**: NO (groups with Wave 2)

---

- [ ] 8. **记忆系统集成**

  **What to do**:
  - 集成ZeroClaw的Memory trait
  - 实现分析历史存储
  - 添加查询缓存

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2 (with Tasks 5, 6, 7)
  - **Blocks**: Tasks 9, 13
  - **Blocked By**: Tasks 1, 4

  **QA Scenarios**:
  ```
  Scenario: 记忆存储
    Tool: Bash (cargo test)
    Steps:
      1. cargo test memory_integration
    Expected Result: 分析结果正确存储和检索
    Evidence: .sisyphus/evidence/task-8-memory.txt
  ```

  **Commit**: NO (groups with Wave 2)

---

### Wave 3: Bio-Specific (依赖Wave 2)

- [ ] 9. **可复现性系统**

  **What to do**:
  - 创建ReproducibilityBundle结构体
  - 实现commands.sh生成
  - 实现environment.yml导出
  - 实现checksums.sha256计算

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 10, 11, 12)
  - **Blocks**: Task 13, F3
  - **Blocked By**: Tasks 5, 6, 7, 8

  **References**:
  - `../ClawBio/skills/nutrigx_advisor/repro_bundle.py` - ClawBio实现参考

  **QA Scenarios**:
  ```
  Scenario: 可复现性包生成
    Tool: Bash
    Steps:
      1. openlife bio run pharmgx-reporter --input demo.txt --output /tmp/repro-test
      2. ls /tmp/repro-test/commands.sh /tmp/repro-test/environment.yml /tmp/repro-test/checksums.sha256
    Expected Result: 三个文件都存在
    Evidence: .sisyphus/evidence/task-9-repro.txt
  ```

  **Commit**: NO (groups with Wave 3)

---

- [ ] 10. **安全沙箱实现**

  **What to do**:
  - 集成ZeroClaw的安全模块
  - 实现Landlock/Bubblewrap沙箱
  - 添加文件系统隔离
  - 实现网络隔离

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 9, 11, 12)
  - **Blocks**: Task 13, F3
  - **Blocked By**: Task 7

  **References**:
  - `../zeroclaw/src/security/landlock.rs` - Landlock实现
  - `../zeroclaw/src/security/bubblewrap.rs` - Bubblewrap实现

  **QA Scenarios**:
  ```
  Scenario: 沙箱隔离
    Tool: Bash
    Steps:
      1. 在沙箱中运行skill
      2. 尝试访问隔离外的文件
    Expected Result: 访问被拒绝
    Evidence: .sisyphus/evidence/task-10-sandbox.txt
  ```

  **Commit**: NO (groups with Wave 3)

---

- [ ] 11. **VCF解析模块**

  **What to do**:
  - 从ClawBio移植完整的VCF解析
  - 实现基因型编码(0/1/2/-1)
  - 添加群体映射
  - 实现等位基因频率计算

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 9, 10, 12)
  - **Blocks**: Task 12, F3
  - **Blocked By**: Task 7

  **References**:
  - `../ClawBio/skills/equity-scorer/equity_scorer.py` - ClawBio VCF解析

  **QA Scenarios**:
  ```
  Scenario: VCF解析
    Tool: Bash (cargo test)
    Steps:
      1. cargo test vcf_parsing
    Expected Result: VCF正确解析为基因型矩阵
    Evidence: .sisyphus/evidence/task-11-vcf.txt
  ```

  **Commit**: NO (groups with Wave 3)

---

- [ ] 12. **HEIM评分移植**

  **What to do**:
  - 移植HEIM评分计算
  - 实现四个组件指标
  - 添加可视化函数

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 3 (with Tasks 9, 10, 11)
  - **Blocks**: F3
  - **Blocked By**: Tasks 7, 11

  **References**:
  - `../ClawBio/skills/equity-scorer/equity_scorer.py` - HEIM计算

  **QA Scenarios**:
  ```
  Scenario: HEIM评分
    Tool: Bash (cargo test)
    Steps:
      1. cargo test heim_score
    Expected Result: HEIM评分正确计算(0-100)
    Evidence: .sisyphus/evidence/task-12-heim.txt
  ```

  **Commit**: NO (groups with Wave 3)

---

### Wave 4: Enhancements (依赖Wave 3)

- [ ] 13. **多技能链式调用**

  **What to do**:
  - 实现AnalysisPlan结构体
  - 添加技能间数据传递
  - 实现依赖解析

  **Recommended Agent Profile**:
  - **Category**: `artistry`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 14, 15, 16)
  - **Blocks**: F3
  - **Blocked By**: Tasks 5, 6, 8, 9, 10

  **QA Scenarios**:
  ```
  Scenario: 链式调用
    Tool: Bash
    Steps:
      1. openlife "分析这个VCF的多样性并注释变异"
    Expected Result: 先运行equity-scorer，再运行vcf-annotator
    Evidence: .sisyphus/evidence/task-13-chain.txt
  ```

  **Commit**: NO (groups with Wave 4)

---

- [ ] 14. **Channel集成**

  **What to do**:
  - 集成ZeroClaw的Channel trait
  - 支持Telegram/Discord
  - 添加生物信息学命令处理

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 13, 15, 16)
  - **Blocks**: Task 15
  - **Blocked By**: Task 1

  **References**:
  - `../zeroclaw/src/channels/traits.rs` - Channel trait
  - `../zeroclaw/src/channels/telegram.rs` - Telegram实现

  **QA Scenarios**:
  ```
  Scenario: Channel初始化
    Tool: Bash (cargo test)
    Steps:
      1. cargo test channel_init
    Expected Result: Channel正确初始化
    Evidence: .sisyphus/evidence/task-14-channel.txt
  ```

  **Commit**: NO (groups with Wave 4)
---

- [ ] 15. **Daemon模式**

  **What to do**:
  - 实现后台服务
  - 添加定时任务
  - 实现健康检查

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 13, 14, 16)
  - **Blocks**: Task 16
  - **Blocked By**: Task 14

  **References**:
  - `../zeroclaw/src/daemon/` - Daemon实现
  - `../zeroclaw/src/cron/` - Cron任务

  **QA Scenarios**:
  ```
  Scenario: Daemon启动
    Tool: Bash
    Steps:
      1. openlife daemon --test
    Expected Result: Daemon正确启动并响应健康检查
    Evidence: .sisyphus/evidence/task-15-daemon.txt
  ```

  **Commit**: NO (groups with Wave 4)
---

- [ ] 16. **Web Gateway**

  **What to do**:
  - 集成ZeroClaw的Gateway模块
  - 添加REST API
  - 实现Web Dashboard

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4 (with Tasks 13, 14, 15)
  - **Blocks**: F3
  - **Blocked By**: Task 15
  **References**:
  - `../zeroclaw/src/gateway/` - Gateway实现
  - `../zeroclaw/src/gateway/api.rs` - REST API

  **QA Scenarios**:
  ```
  Scenario: Web Dashboard访问
    Tool: Bash (curl)
    Steps:
      1. curl http://localhost:3000/health
    Expected Result: 返回200 OK
    Evidence: .sisyphus/evidence/task-16-gateway.txt
  ```

  **Commit**: NO (groups with Wave 4)

---
## Final Verification Wave

> 4 review agents run in PARALLEL. ALL must APPROVE. Rejection → fix → re-run.

- [ ] F1. **架构合规审计** — `oracle`
  审核所有改进是否符合原始设计目标。验证ZeroClaw集成是否正确。检查是否遵循了所有Guardrails。
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **代码质量检查** — `unspecified-high`
  运行 `cargo clippy --all-targets -- -D warnings` + `cargo test`。检查unsafe代码使用、错误处理完整性、日志覆盖。
  Output: `Clippy [PASS/FAIL] | Tests [N pass/N fail] | Code [N clean/N issues] | VERDICT`

- [ ] F3. **集成测试** — `deep`
  执行端到端测试：运行pharmgx-reporter skill，验证报告生成，验证可复现性包，验证沙箱隔离。
  Output: `Skills [N/N pass] | Reproducibility [N/N] | Sandbox [PASS/FAIL] | VERDICT`

- [ ] F4. **文档完整性** — `unspecified-high`
  验证README、CLAUDE.md、PRODUCT_DESIGN.md更新。检查所有public API有文档注释。验证skill文档完整。
  Output: `Docs [N/N complete] | API Docs [N/N] | Skills [N/N documented] | VERDICT`

---

## Commit Strategy

按Wave分批提交，每个Wave完成后创建一个提交：

- **Wave 1**: `refactor(core): integrate ZeroClaw at library level`
- **Wave 2**: `feat(core): add LLM routing and skill registry`
- **Wave 3**: `feat(bio): add reproducibility and sandbox systems`
- **Wave 4**: `feat(integration): add channels and daemon support`
- **FINAL**: `test: comprehensive integration tests`

---

## Success Criteria

### Verification Commands
```bash
# 核心测试
cargo test --all

# Lint检查
cargo clippy --all-targets -- -D warnings

# 构建检查
cargo build --release

# Skill执行测试
openlife bio run pharmgx-reporter --input skills/pharmgx-reporter/demo_patient.txt --output /tmp/test-report

# 可复现性验证
ls /tmp/test-report/commands.sh /tmp/test-report/environment.yml /tmp/test-report/checksums.sha256
```

### Final Checklist
- [ ] 所有"Must Have"功能实现
- [ ] 所有"Must NOT Have"未实现
- [ ] 所有测试通过
- [ ] 文档完整更新
- [ ] 无unsafe代码（除安全模块必要部分）
- [ ] 可复现性包自动生成
- [ ] 沙箱执行正常工作
