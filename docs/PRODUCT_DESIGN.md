# OpenLife 产品设计文档

**项目**: OpenLife - 基于 ZeroClaw 的生物信息学 AI Agent  
**版本**: v0.1.0  
**日期**: 2026-03-02  

---

## 1. 项目概述

### 1.1 愿景

**OpenLife** 是一个高效的生物信息学 AI Agent，基于 ZeroClaw 的 Rust 运行时构建，继承 ClawBio 的生物信息学技能库设计理念。

**核心理念**: "简洁高效 + 专业生物信息学"

### 1.2 与 ClawBio 的对比

| 维度 | ClawBio (基于 OpenClaw) | OpenLife (基于 ZeroClaw) |
|------|------------------------|-------------------------|
| 底层运行时 | OpenClaw (Python) | ZeroClaw (Rust) |
| 执行效率 | 中等 | 高 |
| 二进制体积 | 大 | 小 (~10MB) |
| 启动速度 | 慢 (~2s) | 快 (~100ms) |
| Skill 定义 | SKILL.md + Python | SKILL.toml + 多种工具 |
| 扩展性 | 中 | 高 (Trait + Factory) |
| 内存占用 | 高 | 低 |

---

## 2. 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                        OpenLife                              │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐ │
│  │   CLI/API    │  │   Gateway    │  │    Daemon        │ │
│  │  (clap.rs)   │  │  (webhooks)  │  │   (background)   │ │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘ │
│         │                 │                    │           │
│  ┌──────▼─────────────────▼────────────────────▼─────────┐ │
│  │                    Agent Core                          │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌────────────────┐  │ │
│  │  │  Orchestrator│  │   Skills    │  │    Tools       │  │ │
│  │  │  (Router)   │  │  (Bio-Skills)│  │  (shell/file)  │  │ │
│  │  └─────────────┘  └─────────────┘  └────────────────┘  │ │
│  └──────────────────────────┬────────────────────────────┘ │
│                             │                               │
│  ┌──────────────────────────▼────────────────────────────┐ │
│  │              Bio-Skills Library                        │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐  │ │
│  │  │ PharmGx  │ │ Ancestry │ │ Equity   │ │ Nutrigx   │  │ │
│  │  │ Reporter │ │   PCA    │ │  Scorer  │ │  Advisor  │  │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └───────────┘  │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 继承 ZeroClaw 的核心模块

OpenLife 将复用以下 ZeroClaw 核心模块:

| 模块 | 用途 | 复用方式 |
|------|------|---------|
| `src/skills/` | Skill 加载与执行 | 直接复用，扩展 bio-skill 定义 |
| `src/tools/` | 工具执行层 | 复用 shell/file 工具 |
| `src/providers/` | LLM 调用 | 复用所有 provider |
| `src/memory/` | 记忆存储 | 复用 markdown/sqlite |
| `src/config/` | 配置管理 | 扩展 bio 配置项 |
| `src/security/` | 安全策略 | 直接复用 |
| `src/gateway/` | Webhook | 可选用 |

### 2.3 新增 Bio-Skill 模块

```
src/
├── bio/                          # NEW: 生物信息学模块
│   ├── mod.rs                    # Bio 模块入口
│   ├── orchestrator.rs           # Bio 请求路由
│   ├── skills/                   # Bio-Skills 实现
│   │   ├── mod.rs
│   │   ├── pharmgx_reporter/     # 药物基因组学
│   │   ├── ancestry_pca/         # 祖先成分分析
│   │   ├── equity_scorer/        # 健康公平性评分
│   │   ├── nutrigx_advisor/     # 营养基因组学
│   │   └── vcf_annotator/        # 变异注释
│   ├── parser/                   # 生物数据解析器
│   │   ├── mod.rs
│   │   ├── dtc_genetics.rs       # 23andMe/AncestryDNA 解析
│   │   ├── vcf_parser.rs        # VCF 文件解析
│   │   └── ped_parser.rs         # PED 文件解析
│   ├── database/                  # 生物信息学数据库
│   │   ├── mod.rs
│   │   ├── cpic.rs               # CPIC 指南查询
│   │   ├── clinvar.rs            # ClinVar 注释
│   │   └── gnomad.rs             # gnomAD 频率查询
│   └── report/                    # 报告生成
│       ├── mod.rs
│       └── markdown.rs            # Markdown 报告生成器
└── ...
```

---

## 3. Skill 系统设计

### 3.1 Skill 定义格式

采用 ZeroClaw 的 `SKILL.toml` 格式，同时扩展支持生物信息学元数据:

```toml
# SKILL.toml - OpenLife Bio-Skill 示例

[skill]
name = "openlife-pharmgx-reporter"
version = "0.1.0"
description = "Pharmacogenomic report from DTC genetic data"
author = "OpenLife Team"
tags = ["pharmacogenomics", "CPIC", "precision-medicine", "bioinformatics"]

[skill.metadata]
category = "bioinformatics"
openclaw = true
min_python = "3.9"

[skill.inputs]
[[skill.inputs.param]]
name = "input"
type = "file"
format = ["23andme", "ancestrydna", "tsv"]
description = "Raw genetic data file"

[skill.outputs]
[[skill.outputs.param]]
name = "report"
type = "file"
format = "markdown"

[[skill.tools]]
name = "run_pharmgx"
kind = "shell"
command = "python3 {script_dir}/pharmgx_reporter.py --input {input} --output {output}"
description = "Generate pharmacogenomic report"

[skill.tools.args]
script_dir = "Path to pharmgx_reporter.py"
input = "Input genetic data file"
output = "Output report directory"
```

### 3.2 预置 Bio-Skills

#### MVP 阶段 (从 ClawBio 移植)

| Skill | 描述 | 复杂度 |
|-------|------|--------|
| `pharmgx-reporter` | 药物基因组学报告 (31 SNPs, 12 genes, 51 drugs) | 高 |
| `ancestry-pca` | 祖先 PCA 分析 | 高 |
| `equity-scorer` | 健康公平性评分 (HEIM) | 中 |
| `nutrigx-advisor` | 营养基因组学建议 | 中 |

#### v0.2 阶段

| Skill | 描述 |
|-------|------|
| `vcf-annotator` | VCF 变异注释 (VEP, ClinVar, gnomAD) |
| `lit-synthesizer` | 文献综合 (PubMed/bioRxiv) |
| `scrna-orchestrator` | 单细胞 RNA-seq (Scanpy) |

#### v0.3 阶段

| Skill | 描述 |
|-------|------|
| `struct-predictor` | 蛋白结构预测 (AlphaFold) |
| `repro-enforcer` | 可复现性导出 (Conda/Nextflow) |

---

## 4. 核心功能实现

### 4.1 Bio 请求路由 (Orchestrator)

```rust
// src/bio/orchestrator.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BioIntent {
    Pharmacogenomics,
    Ancestry,
    Diversity,
    Nutrition,
    VariantAnnotation,
    Literature,
    SingleCell,
    ProteinStructure,
    Reproducibility,
    Unknown,
}

impl BioIntent {
    pub fn from_query(query: &str) -> Self {
        let query_lower = query.to_lowercase();
        
        // 药物基因组学关键词
        if query_lower.contains("drug") 
            || query_lower.contains("pharmacogen")
            || query_lower.contains("cyp2d6")
            || query_lower.contains("cpic") {
            return BioIntent::Pharmacogenomics;
        }
        
        // 祖先分析关键词
        if query_lower.contains("ancestry")
            || query_lower.contains("pca")
            || query_lower.contains("population") {
            return BioIntent::Ancestry;
        }
        
        // ... 其他路由规则
    }
}
```

### 4.2 数据解析器

#### DTC 基因数据解析

```rust
// src/bio/parser/dtc_genetics.rs

pub enum DTCFormat {
    TwentyThreeAndMe,
    AncestryDNA,
    GenericTSV,
}

pub struct DTCParser;

impl DTCParser {
    pub fn detect_format(content: &str) -> DTCFormat {
        // 检测文件格式
    }
    
    pub fn parse_snp(&self, line: &str) -> Option<SNP> {
        // 解析单行 SNP 数据
    }
    
    pub fn extract_pharmgx_snps(&self, records: &[SNP]) -> Vec<PharmGxSNP> {
        // 提取药物基因组学相关 SNPs
    }
}
```

### 4.3 可复现性报告生成

```rust
// src/bio/report/reproducibility.rs

pub struct ReproducibilityBundle {
    pub report_md: String,
    pub figures: Vec<PathBuf>,
    pub commands_sh: String,
    pub environment_yml: String,
    pub checksums_sha256: String,
}

impl ReproducibilityBundle {
    pub fn generate(
        skill_name: &str,
        input_files: &[PathBuf],
        output_dir: &Path,
    ) -> Result<Self> {
        // 生成完整的可复现性包
    }
}
```

---

## 5. 用户交互接口

### 5.1 CLI 命令

```bash
# 安装 bio-skill
openlife skill install openlife/pharmgx-reporter

# 运行 bio-skill (通过自然语言)
openlife "分析我的23andMe文件中的药物基因组学"

# 直接运行
openlife bio run pharmgx-reporter --input data.txt --output report/

# 列出所有 skills
openlife bio list

# 查看 skill 详情
openlife bio info pharmgx-reporter
```

### 5.2 自然语言理解

OpenLife 的 Bio-Orchestrator 将分析用户输入:

```
输入: "我的CYP2D6基因是什么代谢类型?"
  ↓
意图识别: Pharmacogenomics
  ↓
提取参数: gene=CYP2D6, 需调用 pharmgx-reporter
  ↓
执行 Skill 并返回结果
```

---

## 6. 配置文件设计

### 6.1 主配置文件

```toml
# ~/.openlife/config.toml

[general]
version = "0.1.0"

[agent]
model = "claude-sonnet-4-20250514"
max_tokens = 4096

[bio]
# Bio-Skills 配置
[bio.skills]
enabled = true
workspace = "~/.openlife/bio-skills"
auto_update = true

# 默认使用本地处理 (隐私优先)
local_only = true

# 预设的生物信息学数据库路径
[bio.databases]
cpic_cache = "~/.openlife/cache/cpic.db"
clinvar_cache = "~/.openlife/cache/clinvar.db"

# 报告配置
[bio.report]
default_format = "markdown"
include_reproducibility = true
include_figures = true
```

---

## 7. 安全与隐私

### 7.1 继承 ZeroClaw 安全特性

- **本地优先**: 基因数据不离开用户机器
- **Secure by Default**: 默认拒绝网络访问
- **工具沙箱**: shell 命令在受限环境执行
- **无日志**: 敏感数据不记录

### 7.2 Bio 特定安全规则

```rust
// src/bio/security.rs

pub struct BioSecurityPolicy;

impl BioSecurityPolicy {
    // 基因数据本地处理
    pub fn requires_local_processing() -> bool {
        true
    }
    
    // 禁止上传基因数据到外部服务
    pub fn allow_external_upload() -> bool {
        false
    }
    
    // 报告免责声明
    pub fn disclaimer() -> &'static str {
        "OpenLife is a research and educational tool. \
         It is not a medical device and does not provide clinical diagnoses. \
         Consult a healthcare professional before making any medical decisions."
    }
}
```

---

## 8. 开发路线图

### Phase 1: 基础架构 (v0.1.0)

- [x] 基于 ZeroClaw 创建项目骨架
- [ ] 集成 bio 模块
- [ ] 实现 Bio-Orchestrator
- [ ] 移植 pharmgx-reporter skill
- [ ] 实现基础报告生成

### Phase 2: 核心 Skills (v0.2.0)

- [ ] 移植 ancestry-pca skill
- [ ] 移植 equity-scorer skill
- [ ] 移植 nutrigx-advisor skill
- [ ] 实现可复现性报告

### Phase 3: 扩展 Skills (v0.3.0)

- [ ] 实现 vcf-annotator
- [ ] 实现 lit-synthesizer
- [ ] 实现 scrna-orchestrator
- [ ] 实现 struct-predictor

### Phase 4: 生态完善 (v0.4.0)

- [ ] 社区 skill 模板
- [ ] 文档完善
- [ ] CI/CD 自动化
- [ ] i18n 支持

---

## 9. 技术选型

| 组件 | 选型 | 理由 |
|------|------|------|
| 运行时 | ZeroClaw (Rust) | 高性能、低资源占用 |
| Skill 定义 | SKILL.toml | ZeroClaw 原生支持 |
| Python 桥接 | std::process::Command | 复用现有 ClawBio 脚本 |
| 数据解析 | Rust (nom/serde) | 高性能 |
| 报告生成 | Rust + markdown | ZeroClaw 已集成 |
| 配置 | toml | ZeroClaw 原生 |

---

## 10. 命名规范

### 10.1 项目命名

```
OpenLife           # 主项目 (基于 ZeroClaw 的生物信息学 Agent)
openlife-core      # 核心库
openlife-bio       # 生物信息学模块
openlife-skills    # Bio-Skills 集合
```

### 10.2 Skill 命名

```
openlife-pharmgx-reporter    # 药物基因组学
openlife-ancestry-pca        # 祖先分析
openlife-equity-scorer       # 公平性评分
openlife-nutrigx-advisor     # 营养基因组学
```

---

## 附录 A: 关键文件清单

```
openlife/
├── CLAUDE.md                    # Agent 指令
├── README.md                    # 项目说明
├── Cargo.toml                   # Rust 依赖
├── build.rs                     # 构建脚本
├── src/
│   ├── main.rs                  # CLI 入口
│   ├── lib.rs                   # 库导出
│   ├── bio/                     # [NEW] 生物信息学模块
│   │   ├── mod.rs
│   │   ├── orchestrator.rs
│   │   ├── skills/
│   │   ├── parser/
│   │   ├── database/
│   │   └── report/
│   ├── skills/                  # 复用 ZeroClaw
│   ├── providers/               # 复用 ZeroClaw
│   ├── tools/                   # 复用 ZeroClaw
│   ├── config/                  # 复用 + 扩展
│   └── ...
├── skills/                      # Bio-Skills 目录
│   ├── pharmgx-reporter/
│   │   ├── SKILL.toml
│   │   ├── pharmgx_reporter.py  # 复用 ClawBio
│   │   ├── SKILL.md             # 方法论文档
│   │   └── demo/
│   └── ...
└── docs/
    └── bio/
        └── README.md
```

---

## 附录 B: 参考资料

1. **ClawBio**: https://github.com/ClawBio/ClawBio
2. **ZeroClaw**: https://github.com/zeroclaw-labs/zeroclaw
3. **CPIC Guidelines**: https://cpicpgx.org
4. **HEIM Index**: https://heim-index.org
