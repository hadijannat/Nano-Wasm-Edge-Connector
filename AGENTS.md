# Repository Guidelines

## Project Structure & Module Organization
- `host/` is the HTTP server and policy runtime (Axum + Wasmtime). Entry point: `host/src/main.rs` with supporting modules in `host/src/`.
- `guest/` is the no_std WebAssembly policy module compiled to `guest.wasm` (`guest/src/lib.rs`). `guest/policy.wat` is a low-level reference.
- `shared/` contains shared request/response types (`shared/src/lib.rs`).
- `policies/` holds runtime-loaded artifacts; `policies/default.wasm` is required at startup and hot-reloaded.
- `target/` is build output and should not be edited.

## Build, Test, and Development Commands
- `rustup target add wasm32-unknown-unknown` (one-time) enables guest builds.
- `cargo build -p guest --target wasm32-unknown-unknown --release` builds the policy module.
- `cp target/wasm32-unknown-unknown/release/guest.wasm policies/default.wasm` installs the active policy.
- `cargo run -p host --release` runs the server at `http://localhost:3000`.
- `cargo run -p host` runs a debug build.

## Coding Style & Naming Conventions
- Rust 2021 edition; use 4-space indentation and default `rustfmt` styling (`cargo fmt`).
- Naming: `snake_case` for functions/modules, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Keep the guest module `no_std` and avoid allocations unless essential for the policy.

## Testing Guidelines
- There is no automated test suite in this repo yet.
- Manual smoke checks (examples):
  - `curl http://localhost:3000/health`
  - `curl -X POST http://localhost:3000/evaluate -H "Content-Type: application/json" -d '{"role":"admin"}'`
  - `curl -X POST http://localhost:3000/reload`

## Commit & Pull Request Guidelines
- This checkout has no git history, so no commit convention is established. Prefer concise, imperative messages or Conventional Commits (e.g., `feat: add policy rule`).
- PRs should include: a short summary, testing steps (commands + results), and note any policy/ABI changes that affect `guest` or `policies/default.wasm`.

## Security & Configuration Notes
- `policies/default.wasm` is executed by the host; treat policy artifacts as trusted code and review before deploying.
