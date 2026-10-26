# Contributing

Thanks for helping improve AutoPilot-rs.

## Prerequisites

- A current stable Rust toolchain (the workspace uses Rust 2024 edition).
- Cargo.
- On Linux, GUI builds need XCB/XRandR development packages. On Debian/Ubuntu:

  ```bash
  sudo apt-get install libxcb1-dev libxau-dev libxrandr-dev
  ```

A Nix/devenv setup is also included (`devenv.nix`, `flake.nix`) if you prefer it.

## Build and run

```bash
cargo build
cargo run -p autopilot-rs -- --help
cargo run -p autopilot-rs -- serve
```

Use a temporary configuration root when testing:

```bash
cargo run -p autopilot-rs -- --config-path /tmp/autopilot-dev serve
```

## Tests

Run the full workspace test suite:

```bash
cargo test --workspace --locked
```

Focused checks for the API crate:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test -p autopilot-api --locked
cargo clippy -p autopilot-api --all-targets --locked
```

Tests live beside the code and are declared as:

```rust
#[cfg(test)]
#[path = "module.spec.rs"]
mod spec;
```

Shared test fixtures belong in a `test_support` module, not in production code. End-to-end CLI/API tests live in `crates/autopilot-cli/tests/e2e.rs`.

## Formatting and linting

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
```

The repository contains existing clippy warnings in some crates. Avoid adding new warnings in code you touch.

## JSON schema

The job schema is generated from Rust types:

```bash
cargo run -q -p autopilot-core --example schema > json_schema/schema.json
```

Regenerate it whenever job, schedule, task, or condition fields change. The core test suite checks that the committed schema matches the types.

## Documentation

Documentation lives in:

- `README.md` — landing page and quick start.
- `docs/README.md` — documentation index.
- `docs/JOBS.md` — job schema and scheduling.
- `docs/CONDITIONS.md` — condition reference.
- `docs/CLI.md` — CLI reference.
- `docs/API.md` — HTTP API and security.
- `docs/ARCHITECTURE.md` — codebase overview.
- `docs/CONTRIBUTING.md` — this file.
- `AGENTS.md` — instructions and guardrails for AI coding agents.
- `llms.txt` — compact documentation index for LLM tools.

When changing behavior, update the relevant document and examples. Keep JSONC snippets valid, use relative links, and call out current limitations rather than documenting behavior that does not exist yet.

Examples live in `examples/` and `templates/`. Test changed commands manually and ensure the example still parses as JSON/JSONC.

## Just recipes

If you have `just` installed:

```bash
just build
just test
just schema
just clean
```

For local runs, use `cargo run -p autopilot-rs -- ...` as shown above.

## Pull requests

1. Open an issue or discussion for larger changes before implementing them.
2. Use a focused branch and keep unrelated changes out of the PR.
3. Include a short summary, the behavior affected, and the commands you ran.
4. Link the issue the PR addresses, for example `Closes #54`.
5. Update tests and documentation with the code change.
6. Make sure CI passes before requesting review.

Prefer conventional commit-style messages, such as:

```text
fix(api): return not found for missing jobs
docs: clarify API reload behavior
refactor(api): split handlers into modules
```

## Security-sensitive changes

The API is currently unauthenticated. Do not add documentation or examples that encourage exposing it to untrusted networks without an authenticating proxy or tunnel. If you are changing network binding, authentication, command execution, or input validation, describe the security impact in the PR.

## License

By contributing, you agree that your contributions are licensed under the MIT license.
