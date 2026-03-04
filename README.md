# 🧬 OpenLife

<p align="center">
  <img src="img/openlife-logo.png" alt="OpenLife" width="400">
</p>

<p align="center">
  <strong>ZeroClaw-based Bioinformatics AI Agent</strong><br>
  Local-first. Privacy-focused. High-performance.
</p>

<p align="center">
  <a href="https://github.com/openlife-ai/openlife/actions/workflows/ci.yml">
    <img src="https://github.com/openlife-ai/openlife/actions/workflows/ci.yml/badge.svg" alt="CI">
  </a>
  <a href="https://github.com/openlife-ai/openlife/releases">
    <img src="https://img.shields.io/github/v/release/openlife-ai/openlife?include_prereleases" alt="Release">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-green" alt="MIT License">
  </a>
  <a href="https://crates.io/crates/openlife">
    <img src="https://img.shields.io/badge/rust-1.75+-blue?logo=rust" alt="Rust 1.75+">
  </a>
</p>

---

## What is OpenLife?

**OpenLife** is a high-performance bioinformatics AI agent built on [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw) (Rust). It combines the efficiency and low resource usage of ZeroClaw with a specialized bioinformatics skill system.

```
┌─────────────────────────────────────────────────────────────┐
│                        OpenLife                              │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐ │
│  │   CLI/API    │  │   Gateway    │  │    Daemon        │ │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘ │
│         │                 │                    │           │
│  ┌──────▼─────────────────▼────────────────────▼─────────┐ │
│  │                    Agent Core                          │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌────────────────┐  │ │
│  │  │ Orchestrator│  │ Bio-Skills │  │ ZeroClaw Tools │  │ │
│  │  └─────────────┘  └─────────────┘  └────────────────┘  │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Why OpenLife?

| Feature | OpenLife | ClawBio |
|---------|----------|---------|
| Runtime | ZeroClaw (Rust) | OpenClaw (Python) |
| Performance | High | Medium |
| Binary Size | ~10MB | N/A |
| Startup | <100ms | ~2s |
| Memory Usage | Low | High |

## Features

### 🧬 Bioinformatics Skills

OpenLife routes your queries to specialized bioinformatics skills:

| Category | Skill | Description |
|----------|-------|-------------|
| **Pharmacogenomics** | `pharmgx-reporter` | 31 SNPs, 12 genes, 51 drugs, CPIC guidelines |
| **Nutrigenomics** | `nutrigx-advisor` | Personalized nutrition from genetic data |
| **Diversity & Equity** | `equity-scorer` | HEIM diversity scoring, FST, PCA |
| **Variant Annotation** | `vcf-annotator` | VEP, ClinVar, gnomAD |
| **Literature** | `lit-synthesizer` | PubMed/bioRxiv synthesis |
| **Single-Cell** | `scrna-orchestrator` | Scanpy automation |
| **Sequence Analysis** | `seq-wrangler` | FASTQ, alignment, BAM, trimming |
| **Ancestry** | `ancestry-pca` | Population structure, PCA |
| **Metagenomics** | `metagenomics` | 16S, shotgun profiling |
| **Semantic Analysis** | `semantic-sim` | Research gaps, NTDs |
| **Protein Structure** | `struct-predictor` | AlphaFold, PDB |
| **Reproducibility** | `repro-enforcer` | Nextflow, Conda |

### 🗄️ Database Access Skills

Specialized skills for querying major bioinformatics databases:

| Database | Skill | Description |
|----------|-------|-------------|
| GWAS Catalog | `gwas-database` | SNP-trait associations, p-values |
| ClinVar | `clinvar-database` | Variant pathogenicity |
| PubMed | `pubmed-database` | Literature search |
| ChEMBL | `chembl-database` | Drug/compound data |
| COSMIC | `cosmic-database` | Cancer mutations |
| UniProt | `uniprot-database` | Protein sequences |
| Ensembl | `ensembl-database` | Genome data |
| STRING | `string-database` | Protein interactions |
| KEGG | `kegg-database` | Pathway analysis |


### 🔧 ZeroClaw Features

When you use OpenLife, you also get all ZeroClaw features:

- **Interactive AI Agent** - Chat with AI using natural language
- **Daemon Mode** - Run as a long-running service
- **Channel Integrations** - Telegram, Discord, Slack, WhatsApp, etc.
- **Cron Scheduling** - Automated tasks
- **Memory** - SQLite, markdown, or vector-based memory
- **Security** - Local-first, privacy-focused

## Installation

### Prerequisites

OpenLife requires [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw) to be installed first. ZeroClaw provides the core agent functionality.

```bash
# Install ZeroClaw (required)
cargo install zeroclaw
```

### From Binary (Recommended)

Download the latest release for your platform:

```bash
# Linux/macOS
curl -L https://github.com/openlife-ai/openlife/releases/latest/download/openlife -o openlife
chmod +x openlife
sudo mv openlife /usr/local/bin/
```

Note: Binaries are not yet published. Please build from source for now.

### From Source

```bash
# Clone the repository
git clone https://github.com/openlife-ai/openlife.git
cd openlife

# Build
cargo build --release

# Install
cargo install --path .
```

## Quick Start

### 1. Initialize

```bash
openlife onboard
```

### 2. Run Bioinformatics Analysis

```bash
# List available skills
openlife bio list

# Run a skill directly
openlife bio run pharmgx-reporter --input data.txt --output report/

# Natural language query (auto-routes to skill)
openlife bio query "What drugs should I avoid based on my CYP2D6 gene?"

# Install a skill from path
openlife bio install /path/to/skill
```

### 3. Use the AI Agent

```bash
# Interactive mode
openlife agent

# Single query
openlife agent -m "Analyze my genetic ancestry from this VCF file"
```

## CLI Reference

### Bioinformatics Commands

```bash
openlife bio list              # List all available skills
openlife bio info <skill>     # Show skill information
openlife bio run <skill> --input <file> --output <dir>  # Run a skill
openlife bio install <path>    # Install a skill
openlife bio query "<query>" --input <file> --output <dir>  # Natural language query
```

### ZeroClaw Commands

```bash
openlife onboard              # Initialize workspace
openlife agent               # Start AI agent
openlife daemon              # Start daemon
openlife status              # Show system status
openlife doctor              # Run diagnostics
openlife channel              # Manage channels
openlife cron                # Manage scheduled tasks
openlife skill               # Manage skills
openlife update              # Self-update
openlife estop               # Emergency stop
```

## Creating a Skill

A skill is a directory with `SKILL.toml` and a Python script:

```toml
# SKILL.toml
[skill]
name = "my-skill"
version = "0.1.0"
description = "My bioinformatics skill"
author = "Your Name"
tags = ["bioinformatics"]

[skill.inputs]
[[skill.inputs.param]]
name = "input"
type = "file"
description = "Input file"

[skill.outputs]
[[skill.outputs.param]]
name = "output"
type = "directory"
description = "Output directory"
```

```python
# my_skill.py
#!/usr/bin/env python3
import argparse

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input")
    parser.add_argument("--output")
    args = parser.parse_args()
    
    # Your analysis here
    print("Skill executed!")

if __name__ == "__main__":
    main()
```

## Architecture

OpenLife is built on ZeroClaw's trait-driven architecture:

- **Bio-Orchestrator**: Routes queries to the right skill
- **Skill Loader**: Loads and manages bioinformatics skills
- **Report Generator**: Creates reproducible markdown reports
- **ZeroClaw Core**: Provides agent, tools, memory, and security

## Safety & Privacy

1. **Genetic data never leaves your machine** - All processing is local
2. **Always include the disclaimer** in reports:
   > "OpenLife is a research and educational tool. It is not a medical device and does not provide clinical diagnoses. Consult a healthcare professional before making any medical decisions."
3. **Use SKILL.md methodology only** - Never hallucinate bioinformatics parameters

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Adding New Skills

OpenLife skills follow the [Agent Skills](https://agentskills.io/) standard. You can:

1. **Create from scratch** - Add a skill directory with `SKILL.md` and optional Python script
2. **Port from claude-scientific-skills** - Copy from [K-Dense-AI/claude-scientific-skills](https://github.com/K-Dense-AI/claude-scientific-skills) (140+ skills available)
3. **Install via CLI** - `openlife bio install /path/to/skill`

3. **Install via CLI** - `openlife bio install /path/to/skill`

## License

MIT License - see [LICENSE](LICENSE) for details.

## Links

- [GitHub](https://github.com/openlife-ai/openlife)
- [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw)
- [ClawBio](https://github.com/ClawBio/ClawBio)
- [K-Dense-AI Scientific Skills](https://github.com/K-Dense-AI/claude-scientific-skills)
- [CPIC Guidelines](https://cpicpgx.org)

---

<p align="center">
  <sub>Built with 🦀 by the OpenLife Team</sub>
</p>

## License

MIT License - see [LICENSE](LICENSE) for details.

## Links

- [GitHub](https://github.com/openlife-ai/openlife)
- [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw)
- [ClawBio](https://github.com/ClawBio/ClawBio)
- [K-Dense-AI Scientific Skills](https://github.com/K-Dense-AI/claude-scientific-skills)
- [CPIC Guidelines](https://cpicpgx.org)

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Links

- [GitHub](https://github.com/openlife-ai/openlife)
- [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw)
- [ClawBio](https://github.com/ClawBio/ClawBio)
- [CPIC Guidelines](https://cpicpgx.org)

---

<p align="center">
  <sub>Built with 🦀 by the OpenLife Team</sub>
</p>
