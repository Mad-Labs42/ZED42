# ZED42 Project Structure

## Directory Tree

```
ZED42/
├── .agent/                        # Agent session data (gitignored)
├── .claude/                       # Claude Code session data
├── config/                        # Configuration files
│   ├── providers.toml            # LLM provider configuration
│   └── constraints.toml          # Architectural constraints
├── crates/                        # Rust workspace crates
│   ├── core/                     # ⭐ Foundational types & traits (NEW)
│   │   └── src/
│   │       ├── lib.rs           # Re-exports
│   │       ├── types.rs         # AgentId, SessionId, etc.
│   │       ├── messages.rs      # Message, MessageType, etc.
│   │       ├── traits.rs        # AgentBehavior trait
│   │       └── result.rs        # Error types
│   ├── cortex/                   # Executive Orchestrator
│   ├── blackboard/               # Living State Substrate (SurrealDB)
│   │   └── src/
│   │       ├── database.rs      # SurrealDB operations
│   │       ├── types.rs         # Blackboard-specific types
│   │       └── ...
│   ├── agents/                   # Red/Blue/Green Teams
│   ├── toolboxes/                # Capability Distribution
│   ├── memory/                   # Four-Tier Memory
│   │   └── src/
│   │       ├── working.rs       # Tier 1: Hot cache
│   │       ├── session/         # Tier 2: SQLite (Refactored)
│   │       ├── knowledge_graph/ # Tier 3: SurrealDB (Refactored)
│   │       └── archive/         # Tier 4: DuckDB
│   ├── llm/                      # LLM Integration (OpenRouter)
│   ├── mcp/                      # MCP Bridge
│   └── ui/                       # SolidJS + Vite (Managed by PNPM)
│       ├── package.json
│       ├── pnpm-lock.yaml       # ⭐ PNPM lockfile (NEW)
│       ├── .npmrc               # ⭐ PNPM config (NEW)
│       └── node_modules/        # Hardlinks to global store
   ├── ADR/                     # ⭐ Architectural Decision Records (NEW)
   │   ├── 0001-migration-to-pnpm-and-global-store.md
   │   ├── 0002-wide-and-flat-workspace.md
   │   ├── 0003-internal-language.md
   │   ├── 0004-message-oriented-middleware.md
   │   ├── 0005-aura-vitality-substrate.md
   │   ├── 0006-adversarial-multi-agent-teams.md
   │   ├── 0007-multi-tier-memory-substrate.md
   │   ├── 0008-surrealdb-integration-strategy.md
   │   ├── 0009-intelligence-ledger.md
   │   ├── 0010-model-orchestration-matrix.md
   │   ├── 0011-protocol-titan-substrate.md
   │   └── 0012-resilience-and-panic-recovery.md
│   ├── DEVELOPMENT.md
│   └── PROJECT_STRUCTURE.md      # This file
├── .env.example
├── .gitignore
├── ARCHITECTURE.txt
├── Cargo.toml                     # Workspace configuration
├── LICENSE
├── PROJECT_STATUS.md
└── README.md
```

## Crate Dependencies

```
zed42-core (Leaf Node - No internal dependencies)
├── types.rs
├── messages.rs
└── traits.rs

zed42-blackboard --> zed42-core
zed42-agents --> zed42-core
zed42-memory --> zed42-core
zed42-cortex --> zed42-core, zed42-blackboard, zed42-agents, zed42-memory
```

## Key Components by Crate

### zed42-core (NEW)
- **Purpose**: Foundational types and traits shared across all crates.
- **Key Types**: `AgentId`, `SessionId`, `Message`, `MessageType`, `AgentBehavior`.
- **Key Principle**: Zero internal dependencies.

### zed42-memory
- **Structure**: Modular subdirectories for each tier (`session/`, `knowledge_graph/`, `archive/`).
- **Compliance**: All files under 400 LOC limit.

## Developer Tooling

### Package Management
- **Rust**: Cargo + Sccache (global cache)
- **JavaScript**: PNPM (global content-addressable store at `~/.pnpm-store`)

### Build Flags
- `RUSTC_WRAPPER=sccache`
- `RUSTFLAGS='--cfg surrealdb_unstable'`

---

**Last Updated**: 2026-01-20
**Version**: 0.1.0-alpha
