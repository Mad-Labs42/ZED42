# ZED42 Setup Instructions

## Prerequisites Installation

### 1. Install Rust

**Windows:**
```powershell
# Download and run rustup-init.exe from https://rustup.rs/
winget install Rustlang.Rustup
```

**macOS/Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify installation:
```bash
rustc --version
cargo --version
```

### 2. Install PNPM (Replaces npm)

Install PNPM globally for efficient, deduplicated dependency management:
```bash
npm install -g pnpm
```
Or using Corepack (built into newer Node.js):
```bash
corepack enable && corepack prepare pnpm@latest --activate
```

Verify installation:
```bash
pnpm -v
```

### 3. Install Sccache (Rust Global Cache)

```bash
cargo install sccache --locked
```

Configure it as the compiler wrapper:
```powershell
# PowerShell (set permanently for User)
[System.Environment]::SetEnvironmentVariable('RUSTC_WRAPPER', 'sccache', [System.EnvironmentVariableTarget]::User)
```

### 4. Install Git (if not already installed)

**Windows:**
```powershell
winget install Git.Git
```

## Project Setup

### 1. Clone and Configure Environment

```bash
git clone https://github.com/yourusername/zed42.git
cd zed42
cp .env.example .env
# Edit .env and add your OpenRouter API key
```

### 2. Set Build Flags

Some features require specific flags. Set these in your shell profile or before running `cargo`:
```powershell
$env:RUSTFLAGS='--cfg surrealdb_unstable'
```

### 3. Build the Project

```bash
# Check for compilation errors (low resource)
cargo check --workspace -j 1

# Full build
cargo build --workspace
```

### 4. Install UI (Frontend) Dependencies

Navigate to the UI crate and use PNPM:
```bash
cd crates/ui
pnpm install
```

### 5. Verify Installation

```bash
# Rust toolchain
rustc --version && cargo --version && sccache --version

# PNPM
pnpm -v

# Check project structure
cargo tree --workspace
```

## Development Tools (Recommended)

### rust-analyzer (LSP for IDE support)
```bash
rustup component add rust-analyzer
```

### VS Code Extensions
- rust-analyzer
- CodeLLDB (for debugging)
- Even Better TOML
- crates (dependency management)

## Troubleshooting

### Compilation errors about missing dependencies
```bash
cargo clean && cargo build --workspace -j 1
```

### SurrealDB type inference errors (E0282)
Ensure `RUSTFLAGS` includes the unstable config:
```powershell
$env:RUSTFLAGS='--cfg surrealdb_unstable'
```

### System freeze during compilation
Limit parallelism:
```bash
cargo build -j 1
```

### Frontend issues
Ensure you are using `pnpm`, not `npm`:
```bash
cd crates/ui && pnpm install
```

## Current Status

✅ Project structure created
✅ All crates initialized
✅ PNPM & Sccache global stores configured
✅ Documentation and ADRs created

🔄 Next: Begin Phase 2 implementation (Agent Lifecycle)

---
**Last Updated**: 2026-01-20
