# ZED42: Neuro-Symbolic SDLC Orchestrator

> **⚠️ PROPRIETARY SOFTWARE - ALL RIGHTS RESERVED**
>
> This source code is made available for **viewing and educational purposes only**.
> You may NOT use, copy, modify, distribute, or commercialize this software.
> See [LICENSE](LICENSE) for complete terms.

**Version 0.1.0 - Development Phase**

Zed42 (Zed) is a **desktop-native SDLC orchestrator** that functions as a synthetic development teammate, guiding users from intent through production-ready implementation while maintaining FAANG+ code quality standards.

## Core Architecture

ZED42 employs a **hierarchical adversarial multi-agent architecture** coordinated through a **living blackboard substrate**, combining deterministic constraint systems with frontier-level local language models.

- **VOX (Versatile Orchestration eXchange)**: The formalized, type-safe communication protocol defined in `zed42-core` that governs all agent interactions and intent.
- **MOM (Message-Oriented Middleware)**: The reactive backbone of ZED42, using SurrealDB 2.x native `LIVE SELECT` for sub-millisecond coordination.
- **AURA (Aura Vitality Substrate)**: A 4-tiered health and immunity system that monitors agent heartbeats, Flags laggards, and reaps zombies.
- **Memory Substrate**: A 4-tier biological-inspired hierarchy: Working (RAM), Session (SQLite), Knowledge Graph (SurrealDB), and Archive (DuckDB).
- **Red/Blue/Green Teams**: Adversarial multi-agent system (Offensive, Defensive, Governance) ensuring FAANG+ code quality.

## Project Structure

```
ZED42/
├── crates/
│   ├── core/             # Foundational types & traits (zed42-core)
│   ├── cortex/           # Executive orchestrator & team spawner
│   ├── blackboard/       # Living state substrate (SurrealDB)
│   ├── agents/           # Red/Blue/Green team agent implementations
│   ├── toolboxes/        # Structured capability distribution
│   ├── memory/           # Four-tier memory hierarchy
│   ├── mcp/              # Model Context Protocol bridge
│   ├── llm/              # LLM integration (OpenRouter)
│   └── ui/               # SolidJS + Vite (managed by PNPM)
├── config/
│   ├── providers.toml    # LLM provider configuration
│   └── constraints.toml  # Architectural constraints
├── docs/
│   ├── ADR/              # Architectural Decision Records
│   └── ...
└── ARCHITECTURE.txt      # Detailed architecture specification
```

## Technology Stack

### Core Runtime
- **Language**: Rust (2021 edition, stable)
- **Async Runtime**: Tokio
- **Desktop Framework**: Tauri 2.0 (Future)
- **Frontend**: SolidJS + Vite (managed by **PNPM**)

### Data Layer
- **Primary Database**: SurrealDB 2.1+ (Native Graph + MOM Reactive Substrate)
- **Session Storage**: SQLite with FTS5
- **Knowledge Graph**: SurrealDB with MTREE Vector Indexing (Tier 4 Memory)
- **Analytics Storage**: DuckDB + Parquet

### AI/ML (Development Phase)
- **Provider**: OpenRouter Free Tier
- **Primary Model**: qwen/qwen-2.5-coder-32b-instruct
- **Fast Model**: deepseek/deepseek-coder-6.7b-instruct

### Developer Tooling (Hardware Optimized)
- **JS Package Manager**: **PNPM** (Global content-addressable store at `~/.pnpm-store`)
- **Rust Caching**: **Sccache** (Global compiler cache at `~/.cache/sccache`)

## Getting Started

### Prerequisites

- Rust 1.75 or later
- PNPM (`npm install -g pnpm`)
- Sccache (`cargo install sccache`)
- Git

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/zed42.git
   cd zed42
   ```

2. Copy environment template and configure:
   ```bash
   cp .env.example .env
   # Edit .env with your API keys
   ```

3. Set environment variables for optimization:
   ```powershell
   # PowerShell
   $env:RUSTC_WRAPPER='sccache'
   $env:RUSTFLAGS='--cfg surrealdb_unstable'
   ```

4. Build the project:
   ```bash
   cargo build --workspace -j 2
   ```

5. Install UI dependencies:
   ```bash
   cd crates/ui && pnpm install
   ```

### Development

- **Build all crates**: `cargo build --workspace`
- **Run specific crate tests**: `cargo test -p zed42-cortex`
- **Check formatting**: `cargo fmt --check`
- **Run linter**: `cargo clippy --all-targets`

## Roadmap

### Phase 1: Foundation (Current)
- [x] Project structure and workspace setup
- [x] Memory substrate implementation
- [x] Blackboard with SurrealDB integration
- [x] PNPM & Sccache global store infrastructure
- [ ] Basic LLM integration with OpenRouter

### Phase 2: Single-Agent Intelligence
- [ ] One sophisticated agent with full toolbox
- [ ] Constrained generation with JSON schemas
- [ ] Reflexion self-critique loop

All major decisions are documented in `docs/ADR/`:
- [ADR 0001](docs/ADR/0001-migration-to-pnpm-and-global-store.md): PNPM & Global Store
- [ADR 0002](docs/ADR/0002-wide-and-flat-workspace.md): Wide & Flat Workspace
- [ADR 0003](docs/ADR/0003-vox-protocol.md): VOX Protocol (Versatile Orchestration eXchange)
- [ADR 0004](docs/ADR/0004-message-oriented-middleware.md): Message-Oriented Middleware (MOM)
- [ADR 0005](docs/ADR/0005-aura-vitality-substrate.md): Aura Vitality Substrate (AURA)
- [ADR 0006](docs/ADR/0006-adversarial-multi-agent-teams.md): Adversarial Multi-Agent Teams

## License

**All Rights Reserved - Source Available License**

See the [LICENSE](LICENSE) file for complete terms and restrictions.

---

**Status**: Development Phase
**Last Updated**: 2026-01-20
