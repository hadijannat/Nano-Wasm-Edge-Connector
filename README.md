# Nano-Wasm Edge Connector

A lightweight Rust-based Dataspace Connector using WebAssembly for dynamic policy enforcement on edge devices. Targets **<10MB RAM** operation with a single binary deployment.

## Features

- 🦀 **Pure Rust** - Memory-safe, no GC pauses
- 🔒 **WebAssembly Sandboxing** - Isolated policy execution
- ⚡ **Hot-Reload** - Update policies without restart
- 📦 **Tiny Footprint** - <10MB total RAM usage
- 🔄 **Fuel Metering** - DoS protection built-in

## Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add Wasm target
rustup target add wasm32-unknown-unknown

# Optional: Install wasm-opt for smaller binaries
cargo install wasm-opt
```

### Build & Run

```bash
# Build guest Wasm module
cargo build -p guest --target wasm32-unknown-unknown --release

# Copy to policies directory
mkdir -p policies
cp target/wasm32-unknown-unknown/release/guest.wasm policies/default.wasm

# Build and run host
cargo run -p host --release
```

### Test

```bash
# Health check
curl http://localhost:3000/health

# Admin access (allowed)
curl -X POST http://localhost:3000/evaluate \
  -H "Content-Type: application/json" \
  -d '{"role": "admin", "resource": "secret"}'

# Blocked request (denied)
curl -X POST http://localhost:3000/evaluate \
  -H "Content-Type: application/json" \
  -d '{"blocked": true}'

# Force reload
curl -X POST http://localhost:3000/reload

# Check memory usage
curl http://localhost:3000/metrics
```

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    Edge Device (<10MB RAM)               │
│  ┌────────────────────────────────────────────────────┐  │
│  │                  Axum HTTP Server                  │  │
│  │  /health  /evaluate  /reload  /metrics             │  │
│  └──────────────────────┬─────────────────────────────┘  │
│                         │                                │
│  ┌──────────────────────▼─────────────────────────────┐  │
│  │              Wasmtime Runtime (2MB)                │  │
│  │  • Per-request Store isolation                     │  │
│  │  • 10,000 fuel limit                               │  │
│  │  • 64KB stack                                      │  │
│  └──────────────────────┬─────────────────────────────┘  │
│                         │                                │
│  ┌──────────────────────▼─────────────────────────────┐  │
│  │           Policy Module (~2KB Wasm)                │  │
│  │  • no_std, no allocator                            │  │
│  │  • Bump allocator (64KB heap)                      │  │
│  │  • Pattern-based policy rules                      │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │              File Watcher (Hot-Reload)             │  │
│  │  policies/default.wasm → Atomic swap               │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

## Policy Module

The guest Wasm module (`guest/src/lib.rs`) implements simple pattern-based rules:

| Pattern | Result | Reason |
|---------|--------|--------|
| `"blocked": true` | DENY | Explicitly blocked |
| `"admin"` | ALLOW | Admin role |
| `"operator"` + sensitive resource | DENY | Operator restrictions |
| `"viewer"` + write action | DENY | Read-only access |
| Default | ALLOW | Permissive edge policy |

## License

MIT
