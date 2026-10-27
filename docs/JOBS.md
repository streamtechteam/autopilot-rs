# Job files and schedules

A job is a JSON or JSONC document stored in the configured `jobs/` directory. The daemon loads jobs at startup and on reload.

## Storage location

The default configuration root is:

```text
~/.config/autopilot/          # Linux/macOS
%APPDATA%\autopilot\          # Windows
```

Jobs live in `<config-root>/jobs/`. Use `--config-path` to choose a different root:

```bash
autopilot-rs --config-path /tmp/autopilot-dev serve
```

Files may use `.json` or `.jsonc`; comments and trailing commas are accepted by the JSONC parser.

The loader currently **skips job files that fail to parse** and **rejects duplicate job IDs**. A duplicate ID prevents the job set from loading, so keep IDs unique.

## Minimal job

```jsonc
{
  "id": "hello",
  "name": "Say hello",
  "conditions": [],
  "tasks": [{ "command": "echo hello" }]
}
```

`id`, `conditions`, and `tasks` are required. `name`, `description`, `when`, `check_interval`, and `task_mode` are optional.

## Fields

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `id` | string | yes | Unique job identifier. It is also used for status lookups and file naming. Avoid path separators and control characters. |
| `name` | string | no | Display name. Defaults to `job_<id>`. |
| `description` | string | no | Free-form description. Defaults to a single space. |
| `when` | schedule object | no | When to run. Omit to check conditions when jobs start. |
| `check_interval` | string | no | Default milliseconds between retries for an unscheduled job whose conditions are not yet satisfied. A `polling` wrapper may override this for one condition. Invalid values fall back to 1000 ms. |
| `conditions` | array | yes | Conditions that must all pass. May be empty. |
| `tasks` | array | yes | Tasks to run. May be empty. |
| `task_mode` | string | no | How tasks run: `concurrent` (default), `sequential`, or `sequential_stop_on_failure`. |

The job schema rejects unknown fields.

## Schedules

A schedule uses a `type` and a `trigger`:

```jsonc
{
  "id": "morning",
  "when": { "type": "daily", "trigger": { "time": "08:00" } },
  "conditions": [],
  "tasks": [{ "command": "echo good morning" }]
}
```

| Type | Trigger shape | Meaning |
| --- | --- | --- |
| `once` | `{ "date": "YYYY/MM/DD", "time": "HH:MM" }` | Run once at a local date and time. |
| `daily` | `{ "time": "HH:MM" }` | Run every day at the given local time. |
| `weekly` | `{ "time": "HH:MM", "weekday": 1 }` | Run on a weekday at the given local time. `weekday` is optional and defaults to `1` (Monday); `0` or `7` is Sunday, `1`–`6` are Monday–Saturday. |
| `monthly` | `{ "time": "HH:MM", "day": 1 }` | Run on a day of every month. `day` is optional and defaults to `1`. |
| `yearly` | `{ "time": "HH:MM", "month": 1, "day": 1 }` | Run on a month/day every year. `month` and `day` are optional and default to January 1. |
| `cron` | `"0 0 8 * * *"` | Run on a cron schedule. |

The `weekday`, `day`, and `month` fields are validated when the scheduler is built. Out-of-range values are rejected. A monthly or yearly day that does not exist in a given month is skipped by the cron scheduler.

Cron expressions are passed to `tokio-cron-scheduler`; use a seconds-first expression with six fields unless your expression intentionally includes a year field.

If `when` is omitted, the job is an **immediate job**: the runtime checks its conditions once when jobs start — at daemon startup, or on `POST /start` after `serve --api`. If they fail, each condition is retried on its own cadence: a `polling` wrapper's `check_interval` when set, otherwise the job-level `check_interval`. Conditions with no interval anywhere are checked once and their result is reused. If no condition has an interval to retry on, a failed immediate job becomes `Unsatisfied` and stops.

Scheduled jobs evaluate their conditions once per trigger. `check_interval` is not used for retrying scheduled jobs.

## Conditions

All conditions in the job-level `conditions` array use AND logic:

```jsonc
{
  "id": "home-evening",
  "conditions": [
    { "type": "wifi", "condition": { "ssid": "HomeNetwork" } },
    { "type": "bluetooth", "condition": { "device": "MyPhone" } }
  ],
  "tasks": [{ "command": "notify-send 'Welcome home!'" }]
}
```

Use a `logical` condition for OR/NOR combinations:

```jsonc
{
  "type": "logical",
  "condition": {
    "operator": "Or",
    "conditions": [
      { "type": "wifi", "condition": { "ssid": "HomeNetwork" } },
      { "type": "wifi", "condition": { "ssid": "OfficeNetwork" } }
    ]
  }
}
```

Wrap a condition in a `polling` condition to give it its own `check_interval` (milliseconds) that overrides the job-level `check_interval` for that condition. See the [conditions guide](CONDITIONS.md) for every condition type and field.

## Tasks

Each task is a shell command:

```jsonc
{
  "tasks": [
    { "command": "echo first" },
    { "command": "echo second" }
  ]
}
```

- Unix: commands run with `sh -c`.
- Windows: commands run with `cmd /C`.
- `task_mode` controls execution:
  - `concurrent` (default): all tasks start together.
  - `sequential`: tasks run one at a time; a failure is logged and the next task still runs.
  - `sequential_stop_on_failure`: tasks run one at a time and the remaining tasks are skipped after the first failure.
- A failing task is logged, and the job status becomes `Failed` after tasks finish if any task failed.
- The task error is still logged.

Use absolute paths and quote arguments carefully. Commands are passed to a shell, so do not interpolate untrusted input.

## Status values

The API and status manager use these values:

| Status | Meaning |
| --- | --- |
| `Unknown` | Registered but not evaluated yet. |
| `Running` | Job is active or its tasks are running. |
| `Completed` | Tasks were launched after conditions passed. |
| `Unsatisfied` | Conditions did not pass. |
| `Scheduled` | Job is waiting for its schedule. |
| `Failed` | A task failed during job execution. |
| `Pending`, `Waiting`, `Cancelled` | Defined in the status enum but not currently produced by the job runtime. |

`GET /jobs` reports `status` as the debug representation of this value, or `"None"` when the live status manager has no entry.

## Editing and applying changes

You can edit job files directly, or use the API/CLI. Changes on disk are not applied to the running daemon automatically:

```bash
curl -X POST http://127.0.0.1:7883/reload
```

`reload` preserves whether jobs were running; `restart` stops, reloads, and starts again. See [API lifecycle](API.md#lifecycle-and-reload-behavior).

## JSON schema

The machine-readable schema is generated from the Rust types:

```bash
cargo run -p autopilot-core --example schema > json_schema/schema.json
```

The committed schema is checked by the core test suite.
