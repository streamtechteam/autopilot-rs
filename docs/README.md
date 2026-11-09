# AutoPilot-rs documentation

Start with the [README](../README.md) if you are new to AutoPilot-rs. This directory contains the detailed references.

## User documentation

- [Job files and schedules](JOBS.md) — job schema, scheduling, runtime behavior, and status values.
- [Conditions guide](CONDITIONS.md) — every condition type, JSON schema, platform notes, and patterns.
- [CLI reference](CLI.md) — commands, global options, and interactive workflows.
- [API reference](API.md) — HTTP endpoints, environment variables, reload behavior, and security.

## Contributor documentation

- [Architecture](ARCHITECTURE.md) — workspace layout, runtime flow, storage, API, and tests.
- [Contributing](CONTRIBUTING.md) — build, test, documentation, schema, and pull-request workflow.

## AI and agent documentation

- [AGENTS.md](../AGENTS.md) — repository rules, commands, and guardrails for AI coding agents.
- [llms.txt](../llms.txt) — compact documentation index for LLM and retrieval tools.

## A note on current behavior

These documents describe the current implementation, including behavior that is tracked for improvement. Where a known limitation affects how you use AutoPilot-rs, it is called out explicitly. For example:

- the API requires a bearer token, is loopback-only by default, and needs an explicit opt-in for non-loopback binds;
- API job CRUD writes or removes files and sets `reload_required`; the live runtime is unchanged until `POST /start`, `POST /reload`, or `POST /restart`. Direct file edits require a reload or restart.
- `file` condition `min_size` checks a minimum size threshold; `size_changed` is a deprecated alias and does not detect a change from a previous size.

See the open issues in the [GitHub repository](https://github.com/streamtechteam/autopilot-rs/issues) for planned changes.
