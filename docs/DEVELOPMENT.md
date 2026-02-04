# Development Guide

## Workspace Structure

ZED42 uses a Cargo workspace with multiple crates:

- `zed42-core` - Foundational VOX (Versatile Orchestration eXchange) types & traits
- `zed42-blackboard` - MOM Reactive Substrate & AURA Sentinel
- `zed42-agents` - Agent implementations with AURA Pulse integration
- `zed42-mom` - Model Orchestration Matrix (Routing & Governance)
- `zed42-ledger` - Financial Governance (Budgeting & Costs)
- `zed42-memory` - Four-tier Memory Fabric
- `zed42-mcp` - MCP bridge
- `zed42-ui` - Desktop interface

## Development Workflow

### Building

```bash
# Build entire workspace
cargo build --workspace

# Build specific crate
cargo build -p zed42-cortex

# Build for release
cargo build --workspace --release
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p zed42-blackboard

# Run with logging
RUST_LOG=debug cargo test
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --all-targets --workspace

# Run linter with strict mode
cargo clippy --all-targets --workspace -- -D warnings
```

## Environment Setup

1. Copy `.env.example` to `.env`
2. Add your OpenRouter API key
3. Configure model preferences in `config/providers.toml`

## Database Setup

ZED42 will automatically create databases on first run:
- SurrealDB for blackboard and knowledge graph
- SQLite for session storage
- DuckDB for analytics

Data is stored in `./data/` (gitignored).

## Adding New Agents

1. Define agent type in `crates/agents/src/lib.rs`
2. Implement behavior in appropriate team module (`red.rs`, `blue.rs`, `green.rs`)
3. Assign toolboxes in `AgentType::default_toolbox()`
4. Add tests

## Adding New Tools

1. Define tool in appropriate toolbox module
2. Implement `Tool` trait
3. Register in `ToolboxRegistry`
4. Update toolbox assignments for relevant agents

## Debugging

Set `RUST_LOG` environment variable:
```bash
RUST_LOG=debug cargo run
RUST_LOG=zed42_cortex=trace cargo run
```

Use `tracing` macros in code:
```rust
use tracing::{debug, info, warn, error};

info!("Agent spawned: {:?}", agent_id);
debug!("Blackboard state: {:?}", state);
```

## Performance Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin zed42

# Memory profiling with valgrind
valgrind --tool=massif cargo run
```

## IDE Setup

### VS Code
- Install rust-analyzer extension
- Settings are preconfigured in `.vscode/settings.json`

### CLion / RustRover
- Import Cargo project
- Enable Clippy integration
- Set rustfmt as formatter

## Common Tasks

### Adding a dependency
```bash
# Add to workspace dependencies in root Cargo.toml
[workspace.dependencies]
new-crate = "1.0"

# Use in crate
new-crate.workspace = true
```

### Creating a new crate
```bash
mkdir crates/new-crate
cargo init --lib crates/new-crate
# Add to workspace members in root Cargo.toml
```

### Running examples
```bash
cargo run --example example_name
```

## Troubleshooting

### Build fails with linker errors
- Ensure you have the latest Rust toolchain: `rustup update`
- Check system dependencies are installed

### Database connection errors
- Ensure `./data/` directory exists and is writable
- Check SurrealDB is properly initialized

### Tests fail intermittently
- May be due to async timing issues
- Add `#[tokio::test]` attribute
- Use `tokio::time::sleep()` for timing-sensitive tests

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [SurrealDB Documentation](https://surrealdb.com/docs)
- [Tauri Guide](https://tauri.app/v2/guides/)
