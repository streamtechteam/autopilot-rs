# AGENTS.md

Guidance for AI coding agents and automated contributors working in this repository.

## Purpose

AutoPilot-rs is a Rust workspace for a cross-platform, condition-based task scheduler. The CLI, HTTP API, GUI, and runtime are separate crates. Most behavior lives below the CLI in reusable library crates.

When code and documentation disagree, treat the implementation as the source of truth. Update the documentation or flag the mismatch; do not silently document intended behavior as if it already exists.

## Read first

- [README](README.md) — project overview and quick start.
- [Documentation index](docs/README.md) — all user and contributor docs.
- [Architecture](docs/ARCHITECTURE.md) — workspace layout and runtime flow.
- [Contributing](docs/CONTRIBUTING.md) — build, test, schema, and PR workflow.
- [API reference](docs/API.md) — endpoints, reload behavior, and security.
- [Conditions guide](docs/CONDITIONS.md) — condition schemas and current semantics.
- [Job files and schedules](docs/JOBS.md) — job schema and runtime behavior.
- [LLM index](llms.txt) — compact map for retrieval-oriented tools.

## Repository at a glance

- Language: Rust, workspace edition 2024.
- CLI binary: `autopilot-rs` (`crates/autopilot-cli`).
- The workspace also builds `autopilot-gui`, so always identify the package or binary when using `cargo run`.
- Default config root: `~/.config/autopilot/` on Unix, `%APPDATA%\autopilot\` on Windows.
- Jobs live in `<config-root>/jobs/`; logs live in `<config-root>/logs/`.
- API default: `http://127.0.0.1:7883`, controlled by `AUTOPILOT_HOST` and `AUTOPILOT_PORT`.
- HTTP API and CLI are separate entry points into the same `AutoPilot` runtime.

## Commands

Use these from the repository root:

```bash
# Check the workspace
cargo check --workspace --all-targets --locked

# Run all tests
cargo test --workspace --locked

# Format
cargo fmt --all
cargo fmt --all -- --check

# Focused API lint (the workspace has existing warnings elsewhere)
cargo clippy -p autopilot-api --all-targets --locked

# Run the CLI (the workspace has multiple binaries)
cargo run -p autopilot-rs -- --help
cargo run -p autopilot-rs -- --config-path /tmp/autopilot-dev serve

# Regenerate the job schema after changing job/schedule/task/condition types
cargo run -q -p autopilot-core --example schema > json_schema/schema.json
```

Do not use plain `cargo run -- ...`; it cannot choose between the CLI and GUI binaries.

## Code map

| Path | What it owns |
| --- | --- |
| `crates/autopilot-cli` | Argument parsing, interactive commands, daemon lifecycle, signals. |
| `crates/autopilot-api` | Axum routes, handlers, shared API state. |
| `crates/autopilot-core` | `AutoPilot` runtime facade, API address, and health checks. |
| `crates/autopilot-core-job` | Job model, persistence, scheduling attachment, runtime execution. |
| `crates/autopilot-core-conditions` | Condition trait, schemas, and platform backends. |
| `crates/autopilot-core-task` | Shell task execution. |
| `crates/autopilot-core-time` | Schedule types and cron scheduler setup. |
| `crates/autopilot-core-fs` | Config paths and JSON/JSONC storage. |
| `crates/autopilot-core-status` | In-memory job status. |
| `crates/autopilot-core-form` | UI-agnostic form schema used by CLI and GUI. |
| `crates/autopilot-gui` | Native egui GUI. |

## Working rules

1. **Inspect before changing.** Read the relevant crate and tests. Treat unfamiliar edits as user work and preserve them.
2. **Keep changes focused.** One behavior change or documentation update per branch/PR unless the user asks otherwise.
3. **Add or update tests.** Unit tests use `#[cfg(test)] #[path = "module.spec.rs"] mod spec;` beside the source. Integration tests live under `crates/*/tests/`, including `crates/autopilot-cli/tests/e2e.rs`.
4. **Update the schema for type changes.** If job, schedule, task, or condition fields change, regenerate `json_schema/schema.json`; the core test suite checks it.
5. **Update the docs and examples.** Behavior changes require matching updates to `README.md`, `docs/`, and `examples/` when applicable.
6. **Do not change `Cargo.lock` unless dependencies actually changed.**
7. **Do not commit build output** (`target/`, `build/`) or local configuration.
8. **Do not commit, push, or open a PR unless explicitly asked.**
9. **Use explicit paths when staging.** Avoid `git add -A` in a dirty worktree.
10. **Prefer existing patterns** over new abstractions. This workspace deliberately splits responsibilities into small crates.

## Documentation rules

- Describe **current behavior**, including known limitations, unless the user explicitly asks for a proposal.
- Keep relative links valid and code fences balanced.
- Use JSONC for job examples; comments and trailing commas are accepted by the loader.
- Do not add examples that bind the API to a non-loopback interface or encourage exposing it.
- When documenting a condition, include its JSON shape and note platform-specific behavior only when the backend actually implements it.
- Keep `README.md` as the landing page; put detailed references in `docs/`.
- The docs currently call out these constraints; keep them accurate if you touch related behavior:
  - API is unauthenticated, plain HTTP, and loopback by default.
  - API job CRUD writes files but requires reload/restart to affect the live runtime.
  - Reload/restart failures currently return HTTP `200` with `"success": false`.
  - `file` condition `size_changed` is a minimum-size check, not change detection.
  - A failed task is logged, and the job status is set to `Failed`.
  - `task_mode` defaults to `concurrent`; `sequential` and `sequential_stop_on_failure` are also supported.
  - The Unix signal handler processes one `SIGHUP`; `SIGINT`/`SIGTERM` exit directly.

## API and security rules

- The API has no authentication, authorization, or TLS.
- Keep it bound to loopback in tests and examples.
- Do not add examples that use `AUTOPILOT_HOST=0.0.0.0` or otherwise expose the daemon.
- `AUTOPILOT_HOST` and `AUTOPILOT_PORT` affect the server, CLI health checks, and the CLI shutdown client.
- `autopilot-rs serve` always starts the API; `--api` only suppresses automatic job start.
- Do not put secrets, tokens, or machine-specific paths in committed examples.

## Validation checklist

Before declaring a change complete:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
```

For documentation changes, also verify:

- relative links and anchors resolve;
- Markdown code fences are balanced;
- changed JSON/JSONC examples parse;
- CLI commands shown in docs match `cargo run -p autopilot-rs -- --help` and subcommand help.

For API changes, run the relevant tests and, when practical, smoke-test:

```bash
# Terminal 1
AUTOPILOT_PORT=7899 cargo run -p autopilot-rs -- --config-path /tmp/autopilot-agent-test serve --api

# Terminal 2, after the server is listening
curl http://127.0.0.1:7899/health
curl -X POST http://127.0.0.1:7899/shutdown
```

## Do not

- Do not infer tool names, file paths, or APIs without checking the repository.
- Do not "fix" documentation to describe planned behavior that the code does not implement.
- Do not replace current-behavior notes with aspirational behavior.
- Do not weaken security warnings or add network-exposure examples.
- Do not rewrite unrelated files or reformat the workspace.
- Do not claim tests passed unless you ran them.
- Do not commit or push without explicit user approval.
