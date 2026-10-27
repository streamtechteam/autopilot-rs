# Architecture

AutoPilot-rs is a Cargo workspace. The binary is intentionally thin: most behavior lives in reusable crates, and the CLI, API, GUI, and runtime compose those crates.

## Workspace layout

| Crate | Responsibility |
| --- | --- |
| `autopilot-rs` (`crates/autopilot-cli`) | CLI entry point, argument parsing, interactive commands, daemon lifecycle. |
| `autopilot-api` | Axum HTTP server, routes, handlers, shared API state. |
| `autopilot-gui` | Native egui/eframe GUI. |
| `autopilot-core` | Runtime facade: `AutoPilot`, job/task/status orchestration, logging, scheduling. |
| `autopilot-core-job` | Job model, JSON/JSONC persistence, job lookup and mutation, schedule attachment. |
| `autopilot-core-conditions` | `Condition` trait and all condition schemes/backends. |
| `autopilot-core-task` | Shell task execution. |
| `autopilot-core-time` | Schedule types (`When`) and cron scheduler initialization. |
| `autopilot-core-status` | In-memory status manager and status values. |
| `autopilot-core-fs` | Configuration paths and JSON/JSONC file storage. |
| `autopilot-core-checks` | API address, health response, and daemon health check. |
| `autopilot-core-form` | UI-agnostic form schema/values used by the CLI and GUI. |
| `autopilot-core-cross-platform` | Platform helpers such as shell/editor discovery. |
| `autopilot-core-logging` | File/stdout logger setup. |
| `autopilot-core-error` | Shared error type. |
| `autopilot-core-language` | User-facing language strings. |
| `autopilot-core-utilities` | Small shared utilities. |

`autopilot-core` re-exports the lower-level crates so callers can use `autopilot_core::job`, `autopilot_core::conditions`, and so on without depending on every crate individually.

## Startup flow

1. `autopilot-rs` parses CLI arguments and initializes the configuration paths.
2. `AutoPilot::new()` creates the cron scheduler, empty job list, and status manager.
3. `AutoPilot::init()`:
   - initializes file/stdout logging;
   - checks for an existing daemon through `GET /health`;
   - loads jobs from disk;
   - initializes status entries for every loaded job.
4. `serve` starts the API, then starts jobs unless `--api` was passed.
5. The process waits for signals or API shutdown.

The API is started by `serve` in both modes. `--api` only suppresses the automatic `AutoPilot::start()` call.

## Job loading and runtime

A job file is deserialized into `JobScheme`. `Job::from_scheme()` converts each `ConditionScheme` into a boxed `Condition` and each task scheme into a `Task`.

The loader reads the jobs directory and skips files that fail to deserialize. Duplicate job IDs are rejected because status and file operations assume IDs are unique.

`AutoPilot::start()` spawns one Tokio task per job. Each job runs in one of two modes:

- **Immediate job** (`when` absent): check conditions when the runtime starts. If they fail, retry each condition on its own cadence — a `polling` wrapper's `check_interval` when set, otherwise the job-level `check_interval` — sleeping until the next condition is due. Conditions without an interval are checked once and cached. If nothing can change the outcome, mark the job `Unsatisfied`.
- **Scheduled job** (`when` present): register the schedule with `tokio-cron-scheduler`. When the schedule fires, check conditions once and either run tasks or mark the job `Unsatisfied`.

Immediate jobs evaluate conditions sequentially. Scheduled-job condition checks run concurrently on the blocking pool. Conditions are synchronous `check() -> bool` implementations, so blocking/background work is offloaded with `tokio::task::spawn_blocking`.

## Tasks and status

Tasks are shell commands:

- Unix: `sh -c`
- Windows: `cmd /C`

The job's `task_mode` controls execution. `concurrent` (the default) spawns all tasks and joins them. `sequential` runs tasks one at a time and continues after a failure. `sequential_stop_on_failure` runs tasks one at a time and skips the remaining tasks after the first failure. A task failure is logged, and the job status is set to `Failed` after execution finishes.

`StateManager` is an in-memory, cloneable status store. API handlers read it through the shared `AppState`. Status is not persisted across daemon restarts.

## API architecture

The API lives in `crates/autopilot-api`:

- `routes.rs` constructs the Axum router and binds a `TcpListener`.
- `handlers/` contains health, lifecycle, job, and response-type modules.
- `state.rs` defines `AppState`, which wraps `Arc<RwLock<AutoPilot>>` plus response types.

Handlers mutate the runtime under the write lock and read status under the read lock. Job CRUD handlers call the storage layer and return JSON status messages. They intentionally do not mutate the live runtime; a reload or restart applies file changes.

The API address comes from `autopilot_core_checks::address()` and defaults to `127.0.0.1:7883`.

## Storage

`autopilot-core-fs` owns configuration paths and JSON storage:

```text
<config-root>/
├── autopilot.jsonc
├── jobs/
│   └── *.jsonc
└── logs/
```

The config root is `~/.config/autopilot` on Unix and `%APPDATA%\autopilot` on Windows, unless `--config-path` overrides it.

Jobs are serialized as pretty JSON. The store also accepts JSONC when reading. `autopilot.jsonc` is created as a placeholder; job definitions live in `jobs/`.

## Testing

Tests are unit tests colocated with their modules using `#[path = "<name>.spec.rs"] mod spec;`. Shared fixtures live in `test_support.rs` modules.

The workspace also has integration tests under `crates/*/tests/`, including end-to-end CLI/API tests in `crates/autopilot-cli/tests/e2e.rs`.

Typical verification:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo clippy -p autopilot-api --all-targets --locked
```

## Documentation

- [Job files and schedules](JOBS.md)
- [Conditions guide](CONDITIONS.md)
- [CLI reference](CLI.md)
- [API reference](API.md)
- [Contributing](CONTRIBUTING.md)

## Known constraints

- The API is unauthenticated and plain HTTP; see [API security](API.md#security).
- Job CRUD writes files and requires reload/restart for live changes.
- Invalid job files are skipped by the loader; duplicate IDs are fatal for the load.
- `file` condition `size_changed` currently implements a minimum-size check.
- There is no plugin system for custom conditions yet.
- The GUI is built as part of the workspace and requires platform GUI dependencies.
