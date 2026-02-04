# ZED42: Neuro-Symbolic SDLC Orchestrator

<p align="center">
  <strong>A desktop-native synthetic development teammate with FAANG+ code quality standards</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.0--alpha-blue" alt="Version">
  <img src="https://img.shields.io/badge/status-Titan%20Hardened-green" alt="Status">
  <img src="https://img.shields.io/badge/rust-1.75%2B-orange" alt="Rust">
  <img src="https://img.shields.io/badge/license-Proprietary-red" alt="License">
</p>

> **⚠️ PROPRIETARY SOFTWARE - ALL RIGHTS RESERVED**
>
> This source code is made available for **viewing and educational purposes only**.
> You may NOT use, copy, modify, distribute, or commercialize this software.
> See [LICENSE](LICENSE) for complete terms.

---

## Table of Contents

- [Overview](#overview)
- [Core Architecture](#core-architecture)
- [Major Systems](#major-systems)
  - [The Cortex](#1-the-cortex-executive-orchestrator)
  - [The Blackboard](#2-the-blackboard-living-state-substrate)
  - [Red/Blue/Green Teams](#3-redbluegreen-team-architecture)
  - [Memory Substrate](#4-four-tier-memory-substrate)
  - [Toolbox System](#5-toolbox-architecture)
  - [Intelligence Layer](#6-intelligence-architecture)
  - [Model Orchestration Matrix](#7-model-orchestration-matrix-mom)
  - [Titan Substrate](#8-titan-substrate)
- [Project Structure](#project-structure)
- [Technology Stack](#technology-stack)
- [Getting Started](#getting-started)
- [Production Roadmap](#production-roadmap)
- [Architectural Decision Records](#architectural-decision-records)
- [License](#license)

---

## Overview

**ZED42** (Zed) is a **desktop-native SDLC orchestrator** that functions as a synthetic development teammate, guiding users from intent through production-ready implementation. The system employs a **hierarchical adversarial multi-agent architecture** coordinated through a **living blackboard substrate**, combining deterministic constraint systems with frontier-level language models.

### Core Architectural Principles

| Principle | Description |
|-----------|-------------|
| **Local-First Sovereignty** | Zero external dependencies, complete user control over data and execution |
| **Memory as Foundation** | The knowledge graph is primary; agents are ephemeral workers that come and go |
| **Transparent Reasoning** | Every decision is traceable through the blackboard graph |
| **Adversarial Quality Assurance** | Red/Blue team opposition ensures robust outputs |
| **Progressive Disclosure** | Complexity hidden until needed, power available when required |

### Current Status

The **Titan Hardened Base** is complete. All core substrates have been forensically audited and hardened for production.

| Crate | Status | Description |
|-------|--------|-------------|
| `zed42-core` | ✅ Complete | Foundational types, traits, TitanSubstrate registry, SpaceSentry |
| `zed42-memory` | ✅ Complete | Four-tier memory hierarchy (Working, Session, KG, Archive) |
| `zed42-blackboard` | ✅ Complete | SurrealDB message bus, VOX protocol, AURA vitality |
| `zed42-ledger` | ✅ Complete | Intelligence Ledger with budget management & leasing |
| `zed42-llm` | ✅ Complete | OpenRouter wrapper, constrained generation, embeddings |
| `zed42-mom` | ✅ Complete | Model Orchestration Matrix with circuit breakers |
| `zed42-toolboxes` | ✅ Complete | 17 registered toolboxes, atomic FileStateGuard |
| `zed42-agents` | ✅ Partial | Blue Team Reflexion ✅, Red/Green stubs |
| `zed42-cortex` | ✅ Complete | Titan-wired OODA loop shell |
| `zed42-mcp` | 🚧 Scaffold | MCP Bridge shell (pending implementation) |
| `zed42-ui` | 🚧 Scaffold | SolidJS + Vite skeleton |

---

## Core Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         THE CORTEX                              │
│              (Executive Orchestrator & Team Spawner)            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Intent Parser│  │ Task Planner │  │ Team Manager │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
└─────────┼──────────────────┼──────────────────┼─────────────────┘
          │                  │                  │
          └──────────────────┼──────────────────┘
                             │
          ┌──────────────────▼──────────────────┐
          │     BLACKBOARD (SurrealDB FQL)      │
          │  • State Management  • Message Bus  │
          │  • Decision Graph    • Audit Trail  │
          └──────────┬─────────────┬────────────┘
                     │             │
    ┌────────────────┼─────────────┼────────────────┐
    │                │             │                │
┌───▼────┐     ┌────▼────┐   ┌────▼────┐    ┌──────▼──────┐
│RED TEAM│     │BLUE TEAM│   │GREEN TEAM│   │MEMORY FABRIC│
│(Offense)│    │(Defense)│   │ (Judge)  │   │ (4 Tiers)   │
└────┬────┘    └────┬────┘   └────┬────┘    └──────┬──────┘
     │              │             │                │
     └──────────────┼─────────────┘                │
                    │                              │
          ┌─────────▼──────────────────────────────▼─────────┐
          │              MCP BRIDGE (Tool Layer)             │
          │  • IDE Connectors  • File System  • Git          │
          │  • Build Systems   • Execution Sandboxes         │
          └──────────────────────────────────────────────────┘
```

---

## Major Systems

### 1. The Cortex (Executive Orchestrator)

The **Cortex** is the executive brain of ZED42, responsible for:

- **Intent Parsing**: Transforming user requests into structured `Task` objects
- **Task Planning**: Decomposing complex tasks into agent-assignable subtasks
- **Team Management**: Spawning and dissolving specialized agents on demand
- **OODA Loop Execution**: Observe → Orient → Decide → Act cycle for agent coordination

The Cortex is wired to the **Titan Substrate**, providing unified access to all subsystems through thread-safe `Arc<RwLock<T>>` handles.

**Location**: [`crates/cortex/`](crates/cortex/)

---

### 2. The Blackboard (Living State Substrate)

The **Blackboard** is not merely a message queue but the **shared cognitive substrate** of the entire system. Built on **SurrealDB 2.x**, it serves four critical functions:

| Function | Description |
|----------|-------------|
| **MOM Reactive Substrate** | Sub-millisecond real-time coordination via `LIVE SELECT` |
| **State Store** | Current system goals, constraints, and active tasks |
| **Decision Graph** | Immutable record of why decisions were made |
| **Vitality Substrate (AURA)** | 4-tiered health monitoring for agents |

#### VOX Protocol (Versatile Orchestration eXchange)

All agent communication uses the **VOX** typed message protocol:

```rust
pub enum VoxMessageType {
    Command,    // Direct task assignments
    Query,      // Information requests
    Proposal,   // Suggestions requiring approval
    Verdict,    // Judgments and approvals
    Notification, // Status updates
}
```

#### AURA Vitality Substrate

The **AURA** system monitors agent health through four tiers:

1. **Pulse**: Heartbeat monitoring (30-second intervals)
2. **Sentinel**: Lease expiration detection
3. **Reactive**: LIVE SELECT state change alerts
4. **Reaper**: Autonomous zombie agent dissolution

**Location**: [`crates/blackboard/`](crates/blackboard/)

---

### 3. Red/Blue/Green Team Architecture

ZED42 employs an **adversarial multi-agent system** where teams have opposing mandates:

#### Red Team (Offensive Security & Optimization)

| Agent | Role | Toolboxes |
|-------|------|-----------|
| `PenetrationTester` | Security vulnerability detection | StaticAnalysis, DependencyScanning, Fuzzing |
| `ChaosEngineer` | Failure mode testing | Sandboxing, ProcessManagement |
| `PerformanceAnalyst` | Bottleneck identification | PerformanceProfiling, MetricsAnalysis |
| `EdgeCaseMiner` | Boundary condition discovery | Fuzzing, Testing |
| `TechnicalDebtor` | Strategic shortcut identification | MetricsAnalysis, GraphQuery |

**Mandate**: Find flaws BEFORE code reaches production. Cannot modify code directly.

#### Blue Team (Defensive Implementation)

| Agent | Role | Toolboxes |
|-------|------|-----------|
| `FeatureImplementer` | New functionality with tests | CodeGeneration, FileManipulation, Testing |
| `Refactorer` | Code cleanup maintaining behavior | Refactoring, StaticAnalysis |
| `TestEngineer` | Comprehensive test suites | Testing, Fuzzing, CodeGeneration |
| `DocumentationWriter` | Docs, READMEs, ADRs | FileManipulation, DiagramGeneration |
| `MigrationSpecialist` | Safe schema/data migrations | CodeGeneration, Sandboxing |

**Mandate**: Implement safely with comprehensive testing. Requires Green approval for merges.

#### Green Team (Architectural Governance)

| Agent | Role | Toolboxes |
|-------|------|-----------|
| `Architect` | Structural pattern enforcement | GraphQuery, MetricsAnalysis, DiagramGeneration |
| `StandardsEnforcer` | Code style and best practices | StaticAnalysis, MetricsAnalysis |
| `SecurityReviewer` | Cryptographic correctness | StaticAnalysis, DependencyScanning |

**Mandate**: Enforce long-term architectural integrity. Cannot implement code directly.

**Location**: [`crates/agents/`](crates/agents/)

---

### 4. Four-Tier Memory Substrate

ZED42 uses a **biologically-inspired memory hierarchy** optimized for different access patterns:

#### Tier 1: Working Memory (Hot Cache)

| Property | Value |
|----------|-------|
| **Technology** | In-memory DashMap (Rust) |
| **Capacity** | ~500MB RAM |
| **Contents** | Active conversation (last 20 turns), open files, immediate task state |
| **Eviction** | LRU with semantic importance weighting |
| **Latency** | <1ms |

#### Tier 2: Session Memory (Warm Storage)

| Property | Value |
|----------|-------|
| **Technology** | SQLite with FTS5 full-text search + WAL mode |
| **Capacity** | ~50-200MB per session |
| **Contents** | Session history, undo/redo stack, recent decisions |
| **Persistence** | Write-ahead logging, 5-minute auto-checkpoint |
| **Latency** | 1-10ms |

#### Tier 3: Project Memory (Knowledge Graph)

| Property | Value |
|----------|-------|
| **Technology** | SurrealDB with MTREE vector indexing |
| **Capacity** | Scales to millions of nodes (tested to 1M LOC) |
| **Contents** | Complete codebase AST, architectural decisions, dependency graph |
| **Query Modes** | Semantic search, structural search, hybrid search, temporal search |
| **Latency** | 10-100ms |

#### Tier 4: Archive Memory (Cold Storage)

| Property | Value |
|----------|-------|
| **Technology** | DuckDB + Parquet columnar format |
| **Capacity** | Unlimited disk (10:1 compression) |
| **Contents** | Deprecated code, historical versions, past experiments |
| **Archival Policy** | Auto-archive content untouched for 90 days |
| **Latency** | 100ms-1s |

**Location**: [`crates/memory/`](crates/memory/)

---

### 5. Toolbox Architecture

Tools are **capabilities**, not free-for-all functions. Each agent receives a **curated toolbox** aligned with their role.

#### Registered Toolboxes (17 Total)

| Category | Toolboxes |
|----------|-----------|
| **Code Manipulation** | CodeGeneration, FileManipulation, Refactoring |
| **Analysis** | StaticAnalysis, DependencyScanning, MetricsAnalysis, GraphQuery |
| **Testing & Validation** | Testing, Fuzzing, PerformanceProfiling |
| **Version Control** | GitOperations, GitHistory |
| **External Integration** | BuildSystem, Sandboxing, ProcessManagement, Shell |
| **Visualization** | DiagramGeneration, VisualizationGeneration |

#### FileStateGuard (Atomic Operations)

All file system operations use the **RAII FileStateGuard** pattern for transactional integrity:

```rust
let guard = FileStateGuard::create(&path)?;
guard.write_content(data)?;
guard.commit()?; // Atomic rename from temp file
// If commit() not called, temp file is automatically cleaned up
```

**Location**: [`crates/toolboxes/`](crates/toolboxes/)

---

### 6. Intelligence Architecture

ZED42 uses a **three-layer hybrid reasoning model** to eliminate hallucinations:

#### Layer 1: Constrained Generation (Instant)

- **JSON Schema Enforcement**: Grammar-based constrained decoding
- **Deterministic Validation**: Tree-sitter parsing, type checking, linting
- **Benefit**: Eliminates ~90% of malformed outputs

#### Layer 2: Reflexion Self-Critique (1-5 seconds)

Agents review their own output against **constitutional principles**:

```rust
pub struct CritiqueResponse {
    pub issues: Vec<String>,
    pub pass: bool,
    pub suggestions: Vec<String>,
}
```

- **Max Iterations**: 3 (configurable)
- **Principles**: No God Objects, Error Handling, Test Coverage, etc.
- **Benefit**: Catches ~70% of violations before Red Team

#### Layer 3: Adversarial Verification (5-30 seconds)

- Red Team challenges Blue Team proposals
- Green Team validates against architectural standards
- Consensus required for high-impact changes

#### Confidence Scoring

All outputs include **Bayesian confidence scores**:

| Range | Action |
|-------|--------|
| 0.9-1.0 | High confidence → Proceed automatically |
| 0.7-0.89 | Medium confidence → Show reasoning, request confirmation |
| <0.7 | Low confidence → Spawn additional agents for second opinion |

**Location**: [`crates/llm/`](crates/llm/)

---

### 7. Model Orchestration Matrix (MOM)

The **MOM** manages LLM provider routing with production-grade resilience:

#### Features

- **Multi-Provider Routing**: OpenRouter, local llama.cpp, Ollama (future)
- **Circuit Breaker**: Prevents cascading failures with Closed/Open/HalfOpen states
- **Backpressure Signaling**: System-wide slowdown when providers overloaded
- **Tiered Model Selection**: Primary (complex) → Fast (simple) → Fallback
- **Budget Management**: Token tracking, cost limits, rate limiting

#### Circuit Breaker States

```
CLOSED → (failures exceed threshold) → OPEN
   ↑                                      │
   └── (canary succeeds) ── HALF-OPEN ←───┘
                              (after timeout)
```

**Location**: [`crates/mom/`](crates/mom/)

---

### 8. Titan Substrate

The **Titan Substrate** is the unified registry providing thread-safe access to all subsystems:

```rust
pub struct TitanSubstrate {
    registry: Arc<RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
}

impl TitanSubstrate {
    pub fn get_blackboard_handle(&self) -> Result<Arc<RwLock<BlackboardDb>>>;
    pub fn get_memory_handle(&self) -> Result<Arc<RwLock<MemoryFabric>>>;
    pub fn get_llm_handle(&self) -> Result<Arc<RwLock<LlmClient>>>;
}
```

#### SpaceSentry (Hardware Monitoring)

Integrated hardware monitoring with backpressure:

- **Disk Space**: Halts operations if <500MB free
- **Memory**: Triggers eviction if approaching limits
- **Health Alerts**: Logged and surfaced to agents

**Location**: [`crates/core/src/titan.rs`](crates/core/src/titan.rs)

---

## Project Structure

```
ZED42/
├── crates/
│   ├── core/             # Foundational types, TitanSubstrate, SpaceSentry
│   ├── cortex/           # Executive orchestrator & team spawner
│   ├── blackboard/       # Living state substrate (SurrealDB + VOX + AURA)
│   ├── agents/           # Red/Blue/Green team implementations
│   ├── toolboxes/        # 17 structured capability toolboxes
│   ├── memory/           # Four-tier memory hierarchy
│   ├── ledger/           # Intelligence Ledger (budget management)
│   ├── mom/              # Model Orchestration Matrix (routing + resilience)
│   ├── mcp/              # Model Context Protocol bridge
│   ├── llm/              # LLM integration (constrained generation)
│   └── ui/               # SolidJS + Vite frontend
├── config/
│   ├── providers.toml    # LLM provider configuration
│   └── constraints.toml  # Architectural constraints
├── docs/
│   └── ADR/              # 12 Architectural Decision Records
├── ARCHITECTURE.txt      # Detailed architecture specification (1200 lines)
└── README.md             # This file
```

---

## Technology Stack

### Core Runtime

| Component | Technology |
|-----------|------------|
| **Language** | Rust 2021 edition (stable) |
| **Async Runtime** | Tokio (multi-threaded scheduler) |
| **Desktop Framework** | Tauri 2.0 (planned) |
| **Frontend** | SolidJS + Vite |
| **Package Manager** | PNPM (global content-addressable store) |

### Data Layer

| Component | Technology |
|-----------|------------|
| **Primary Database** | SurrealDB 2.1+ (graph + vector + document) |
| **Session Storage** | SQLite with FTS5 + WAL mode |
| **Analytics Storage** | DuckDB + Parquet |
| **Concurrency** | parking_lot RwLock (lock-free reads) |

### AI/ML (Development Phase)

| Component | Technology |
|-----------|------------|
| **Provider** | OpenRouter (free tier) |
| **Primary Model** | qwen/qwen-2.5-coder-32b-instruct |
| **Fast Model** | deepseek/deepseek-coder-6.7b-instruct |
| **Embedding** | text-embedding-3-small (1536d) |
| **Constrained Gen** | JSON mode + schema validation |

### Future: Local Models

| Component | Technology |
|-----------|------------|
| **Inference** | llama.cpp (llama-cpp-rs bindings) |
| **Primary Model** | Qwen2.5-Coder 32B Q4_K_M (~20GB VRAM) |
| **Fast Model** | DeepSeek-Coder 6.7B Q5_K_M (~5GB VRAM) |
| **Embedding** | BGE-M3 (1024d, code-optimized) |

---

## Getting Started

### Prerequisites

- **Rust**: 1.75+ with stable toolchain
- **PNPM**: `npm install -g pnpm`
- **Sccache**: `cargo install sccache` (build acceleration)
- **Git**: For version control

### Installation

```bash
# Clone the repository
git clone https://github.com/Mad-Labs42/ZED42.git
cd ZED42

# Copy environment template
cp .env.example .env
# Edit .env with your OpenRouter API key

# Set environment variables (PowerShell)
$env:RUSTC_WRAPPER='sccache'
$env:RUSTFLAGS='--cfg surrealdb_unstable'

# Build the workspace
cargo build --workspace -j 2

# Install UI dependencies
cd crates/ui && pnpm install
```

### Development Commands

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p zed42-mom

# Check formatting
cargo fmt --check

# Run linter (zero warnings policy)
cargo clippy --workspace -- -D warnings
```

---

## Production Roadmap

### ✅ Phase 0: Foundation & Hardening (Complete)

- [x] Multi-crate workspace (10 crates)
- [x] Memory substrate (4-tier hierarchy)
- [x] Blackboard with SurrealDB + VOX + AURA
- [x] LLM integration with constrained generation
- [x] Intelligence Ledger (budget, leasing, fiscal immunity)
- [x] MOM (routing, circuit breakers, backpressure)
- [x] Blue Team agents with Reflexion loop
- [x] Titan Substrate (unified registry)
- [x] SpaceSentry hardware monitoring
- [x] Atomic FileStateGuard
- [x] 12 ADRs documenting architecture

### 🚧 Phase 1: Adversarial Multi-Agent System

- [ ] Red Team agents (PenetrationTester, ChaosEngineer, etc.)
- [ ] Green Team agents (Architect, StandardsEnforcer, SecurityReviewer)
- [ ] Adversarial workflow integration in Cortex

### 🚧 Phase 2: MCP Bridge Completion

- [ ] IDE connectors, filesystem, git, build, sandbox modules
- [ ] Toolbox-MCP wiring with permission checks

### 🚧 Phase 3: End-to-End Orchestration

- [ ] Intent parsing & task decomposition
- [ ] Dynamic agent spawning/dissolving
- [ ] Full OODA loop wiring

### 🚧 Phase 4: Observability & Security

- [ ] Structured tracing (OpenTelemetry)
- [ ] Metrics collection
- [ ] Final security audit

### 🚧 Phase 5: User Interface

- [ ] Tauri 2.0 + SolidJS desktop app
- [ ] Conversation interface with streaming
- [ ] Agent status & decision trace viewer

### 🚧 Phase 6: Production Launch

- [ ] E2E testing, profiling, packaging
- [ ] User documentation

**Estimated Timeline**: ~4-5 weeks to production

---

## Architectural Decision Records

All major decisions are documented in [`docs/ADR/`](docs/ADR/):

| ADR | Title | Summary |
|-----|-------|---------|
| [0001](docs/ADR/0001-migration-to-pnpm-and-global-store.md) | PNPM & Global Store | Content-addressable package management |
| [0002](docs/ADR/0002-wide-and-flat-workspace.md) | Wide & Flat Workspace | 10-crate monorepo structure |
| [0003](docs/ADR/0003-vox-protocol.md) | VOX Protocol | Typed agent communication |
| [0004](docs/ADR/0004-message-oriented-middleware.md) | Message-Oriented Middleware | SurrealDB LIVE SELECT coordination |
| [0005](docs/ADR/0005-aura-vitality-substrate.md) | Aura Vitality Substrate | 4-tier agent health monitoring |
| [0006](docs/ADR/0006-adversarial-multi-agent-teams.md) | Adversarial Multi-Agent Teams | Red/Blue/Green team architecture |
| [0007](docs/ADR/0007-multi-tier-memory-substrate.md) | Multi-Tier Memory Substrate | 4-tier memory hierarchy |
| [0008](docs/ADR/0008-surrealdb-integration-strategy.md) | SurrealDB Integration | Graph + vector + document store |
| [0009](docs/ADR/0009-intelligence-ledger.md) | Intelligence Ledger | Budget management & leasing |
| [0010](docs/ADR/0010-model-orchestration-matrix.md) | Model Orchestration Matrix | Multi-provider routing |
| [0011](docs/ADR/0011-protocol-titan-substrate.md) | Protocol Titan Substrate | Unified registry pattern |
| [0012](docs/ADR/0012-resilience-and-panic-recovery.md) | Resilience & Panic Recovery | Zero-panic policy |

---

## License

**All Rights Reserved - Source Available License**

This software is proprietary. You may view and study this code for educational purposes only. You may NOT use, copy, modify, distribute, or commercialize this software without explicit written permission.

See the [LICENSE](LICENSE) file for complete terms and restrictions.

---

<p align="center">
  <strong>Status</strong>: Titan Hardened Base (Backend Production-Ready)<br>
  <strong>Last Updated</strong>: 2026-02-04
</p>
