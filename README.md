<!--![autopilot-rs](assets/logo.png)-->
<img src="assets/logo.png" alt="AutoPilot-rs logo" width="500">

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/streamtechteam/autopilot-rs)

# AutoPilot-rs

A cross-platform automation tool that runs tasks when conditions are met. Write a job once and run it on Linux, macOS, and Windows.

AutoPilot-rs is a small, local-first scheduler for things like "sync my files when I am on my home WiFi", "open my work apps on weekday mornings", or "run a backup when the machine is idle and charging".

## Important Note

The project license changed from MIT to GPL-3.0-or-later.

## Features

- **Condition-based jobs** — 14 condition types covering WiFi, Bluetooth, power, CPU/memory, processes, files, disks, screens, and more.
- **Flexible schedules** — run immediately, once, daily, weekly, monthly, yearly, or with a cron expression.
- **Parallel tasks** — a job's shell commands run concurrently; one failing task does not stop the others.
- **JSON/JSONC job files** — human-readable files stored under your configuration directory.
- **Three interfaces** — CLI, HTTP API, and a native GUI.
- **Cross-platform** — Linux, macOS, and Windows backends for the built-in conditions.

## How it works

```jsonc
{
  "id": "morning-setup",
  "name": "Set up my workday",
  "when": { "type": "daily", "trigger": { "time": "08:00" } },
  "conditions": [
    { "type": "wifi", "condition": { "ssid": "OfficeNetwork" } },
    { "type": "power", "condition": { "check_charging": true } },
  ],
  "tasks": [
    { "command": "open /Applications/Slack.app" },
    { "command": "open /Applications/VSCode.app" },
  ],
}
```

At 08:00, AutoPilot checks the WiFi and power conditions. If both are true, the tasks run.

## Installation

### Prebuilt binaries

Download the latest release for your platform from [GitHub Releases](https://github.com/streamtechteam/autopilot-rs/releases).

### Build from source

Install a current Rust toolchain, then:

```bash
cargo build --release
./target/release/autopilot-rs --help
```

On Linux, building the GUI requires the XCB/XRandR development packages. On Debian/Ubuntu:

```bash
sudo apt-get install libxcb1-dev libxau-dev libxrandr-dev
```

On Windows, use `.\target\release\autopilot-rs.exe`.

## Quick start

### 1. Create a job

Run the interactive form:

```bash
autopilot-rs create
```

Or create a JSONC file manually in your jobs directory:

```text
~/.config/autopilot/jobs/my-job.jsonc
%APPDATA%\autopilot\jobs\my-job.jsonc   # Windows
```

See [Job files and schedules](docs/JOBS.md) for the full format.

### 2. Start the daemon

```bash
autopilot-rs serve
```

By default `serve` starts the API and immediately starts the configured jobs. The API listens on `127.0.0.1:7883`.

To start the API without starting jobs automatically, use:

```bash
autopilot-rs serve --api
```

The API still starts; jobs remain stopped until you call `POST /start`.

### 3. Check whether it is running

```bash
autopilot-rs status
```

`status` calls the daemon's `/health` endpoint. `list`, by contrast, only reads job files and works whether or not the daemon is running.

### 4. Browse jobs

```bash
autopilot-rs list
```

This opens an interactive job browser. It reads files from the jobs directory and does not require the daemon.

### 5. Stop AutoPilot

```bash
autopilot-rs stop
```

This sends a shutdown request to the running daemon and exits the process.

## HTTP API quick start

> **Security warning:** the API has no authentication or TLS. It binds to loopback by default. Do not expose it to an untrusted network. See [docs/API.md](docs/API.md) for the full security guidance.

```bash
# Start the API without auto-starting jobs
autopilot-rs serve --api

# Check the daemon
curl http://127.0.0.1:7883/health

# Start jobs through the API
curl -X POST http://127.0.0.1:7883/start

# List configured jobs
curl http://127.0.0.1:7883/jobs
```

The API supports health/status checks, job listing and CRUD, lifecycle controls, and shutdown. Job CRUD writes files; use `POST /reload` or `POST /restart` to apply changes to the running daemon.

See the [API reference](docs/API.md).

## Job format

A job requires an `id`, a `conditions` array, and a `tasks` array. A minimal job with no conditions runs when the job runtime starts:

```jsonc
{
  "id": "hello",
  "name": "Say hello",
  "conditions": [],
  "tasks": [{ "command": "echo hello" }],
}
```

| Field            | Required | Description                                                            |
| ---------------- | -------- | ---------------------------------------------------------------------- |
| `id`             | yes      | Unique job identifier.                                                 |
| `name`           | no       | Display name. Defaults to `job_<id>`.                                  |
| `description`    | no       | Human-readable description.                                            |
| `when`           | no       | Schedule. Omit to check conditions when jobs start.                    |
| `check_interval` | no       | String of milliseconds between condition retries for unscheduled jobs. |
| `conditions`     | yes      | Array of conditions. May be empty. All must pass.                      |
| `tasks`          | yes      | Array of `{ "command": "..." }` objects. May be empty.                 |

For schedule types, condition composition, and runtime details, see [Job files and schedules](docs/JOBS.md).

## Conditions

All conditions in a job use AND logic. Use a `logical` condition for OR/NOR combinations.

| Type             | What it checks                                         |
| ---------------- | ------------------------------------------------------ |
| `wifi`           | Connected WiFi SSID                                    |
| `bluetooth`      | Connected Bluetooth device name or MAC                 |
| `command`        | Shell command exit code or output                      |
| `variable`       | Environment variable value                             |
| `power`          | Charging state or battery percentage                   |
| `resource`       | CPU or memory usage                                    |
| `internet`       | Reachability of a host with `ping`                     |
| `process`        | Whether a process is running                           |
| `diskspace`      | Free/used space on a path                              |
| `file`           | File existence, recent modification, or size threshold |
| `externaldevice` | Mounted external device                                |
| `screen`         | Number of screens, active screen, or screen names      |
| `logical`        | AND/OR/NOR composition of other conditions             |
| `desktopenv`     | Current desktop environment                            |

See the [conditions guide](docs/CONDITIONS.md) for schemas, examples, platform notes, and common patterns.

## CLI

```text
autopilot-rs [OPTIONS] [COMMAND]
```

| Command         | Purpose                                                                                              |
| --------------- | ---------------------------------------------------------------------------------------------------- |
| `serve [--api]` | Start the API daemon. Without `--api`, jobs start immediately. With `--api`, wait for `POST /start`. |
| `status`        | Check the daemon through `/health`.                                                                  |
| `stop`          | Send `POST /shutdown` to the daemon.                                                                 |
| `list`          | Interactive job browser; reads job files only.                                                       |
| `create`        | Interactive job creation.                                                                            |
| `remove`        | Interactive job removal.                                                                             |
| `gui`           | Launch the native GUI.                                                                               |

Global options:

- `--config-path <PATH>` — use a custom configuration root. Place global options before the subcommand.
- `-v, --verbose` — enable debug logging.
- `-h, --help` — show help.
- `-V, --version` — show the version.

See the [CLI reference](docs/CLI.md) for details.

## Configuration

The configuration root defaults to:

```text
~/.config/autopilot/
├── autopilot.jsonc
├── jobs/
│   └── *.jsonc
└── logs/
```

On Windows the default is `%APPDATA%\autopilot\`.

Use `--config-path` to override the root:

```bash
autopilot-rs --config-path /tmp/autopilot-dev serve
```

The API listens on loopback by default. These environment variables control the address used by the server, health checks, and CLI client:

| Variable         | Default     | Description                  |
| ---------------- | ----------- | ---------------------------- |
| `AUTOPILOT_HOST` | `127.0.0.1` | Host or IP the API binds to. |
| `AUTOPILOT_PORT` | `7883`      | Port the API listens on.     |

```bash
AUTOPILOT_PORT=9000 autopilot-rs serve
```

Only change `AUTOPILOT_HOST` when you intentionally want remote access. See [API security](docs/API.md#security).

## Examples

More examples live in [`examples/`](examples/) and job templates in [`templates/`](templates/).

### Sync files only on home WiFi

```jsonc
{
  "id": "sync-home-wifi",
  "name": "Sync files when on home network",
  "conditions": [{ "type": "wifi", "condition": { "ssid": "HomeNetwork" } }],
  "tasks": [{ "command": "rsync -av ~/Documents /mnt/nas" }],
}
```

### Morning routine

```jsonc
{
  "id": "morning-routine",
  "name": "Morning setup",
  "when": { "type": "daily", "trigger": { "time": "08:00" } },
  "conditions": [
    { "type": "bluetooth", "condition": { "device": "Headphones" } },
    { "type": "wifi", "condition": { "ssid": "HomeNetwork" } },
  ],
  "tasks": [
    { "command": "notify-send 'Good morning!'" },
    { "command": "open /Applications/Mail.app" },
  ],
}
```

### Backup when the machine is idle and charging

```jsonc
{
  "id": "idle-backup",
  "name": "Backup when CPU is idle",
  "conditions": [
    {
      "type": "resource",
      "condition": {
        "resource_type": "cpu",
        "threshold": 20,
        "operator": "less",
      },
    },
    { "type": "power", "condition": { "check_charging": true } },
  ],
  "tasks": [{ "command": "backup.sh" }],
}
```

## Troubleshooting

### Jobs are not running

1. Check whether the daemon is running:

   ```bash
   autopilot-rs status
   ```

2. Start it if needed:

   ```bash
   autopilot-rs serve
   ```

3. Check the logs in your configuration's `logs/` directory.
4. Enable debug logging. Global options go before the subcommand:

   ```bash
   autopilot-rs --verbose serve
   ```

### A job does not appear in `list`

`list` reads job files from disk. Check that the file is in `<config-root>/jobs/`, that it is valid JSON/JSONC, and that its job ID is unique. Invalid files are skipped by the loader; duplicate IDs prevent jobs from loading.

### A condition is not behaving as expected

- Test the underlying command manually in a shell.
- Use absolute paths; `~` is not expanded by every backend.
- Check the exact field names and values in the [conditions guide](docs/CONDITIONS.md).
- Remember that job-level conditions are ANDed. Use a `logical` condition for OR/NOR.

### API changes do not affect running jobs

Job CRUD writes files. Apply them with:

```bash
curl -X POST http://127.0.0.1:7883/reload
```

or restart the daemon. See [API lifecycle](docs/API.md#lifecycle-and-reload-behavior).

### Tasks fail

Tasks run with `sh -c` on Unix and `cmd /C` on Windows. Use syntax supported by those shells, test commands manually, and redirect errors while debugging (`command 2>&1`). Failed tasks are logged; the other tasks in the job still run.

## Documentation

- [Documentation index](docs/README.md)
- [Job files and schedules](docs/JOBS.md)
- [Conditions guide](docs/CONDITIONS.md)
- [CLI reference](docs/CLI.md)
- [API reference](docs/API.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Contributing](docs/CONTRIBUTING.md)
- [AI agent guide](AGENTS.md)
- [LLM index](llms.txt)
- [AI-powered documentation](https://deepwiki.com/streamtechteam/autopilot-rs)

## Development

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo clippy -p autopilot-api --all-targets --locked
```

See [Contributing](docs/CONTRIBUTING.md) for the full development workflow, test layout, and schema regeneration.

## Roadmap

- [x] REST API for job management
- [x] 14 user-facing condition types
- [ ] Plugin system for custom conditions
- [ ] Web dashboard
- [ ] Built-in API authentication
- [ ] Metrics and monitoring

## License

AutoPilot-rs is licensed under the GNU General Public License, version 3 or later (`GPL-3.0-or-later`). See [LICENSE](LICENSE) for the full text.
