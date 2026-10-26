# CLI reference

The CLI binary is `autopilot-rs`.

```text
autopilot-rs [OPTIONS] [COMMAND]
```

Global options must appear **before** the subcommand:

```bash
autopilot-rs --verbose serve
autopilot-rs --config-path /tmp/autopilot-dev serve
```

## Global options

| Option | Description |
| --- | --- |
| `--config-path <PATH>` | Use a custom configuration root instead of the platform default. |
| `-v, --verbose` | Enable debug logging. |
| `-h, --help` | Show help. |
| `-V, --version` | Show the version. |

## Commands

### `serve`

```bash
autopilot-rs serve
autopilot-rs serve --api
```

Initializes the configuration, loads jobs, starts logging, and starts the HTTP API. The API listens on `127.0.0.1:7883` by default.

- Without `--api`, jobs start immediately.
- With `--api`, jobs remain stopped until `POST /start` is called.

The API always starts in either mode. `--api` only controls whether jobs auto-start.

`serve` runs until it receives `Ctrl+C`, `SIGTERM`, or (on Unix) `SIGHUP`. `SIGHUP` reloads the job configuration.\n\n> **Current signal behavior:** the Unix signal handler processes one `SIGHUP`, then continues waiting for termination. Use `POST /reload` for repeated reloads. `Ctrl+C` and `SIGTERM` exit the process directly; use `autopilot-rs stop` when you want the API shutdown path.

### `status`

```bash
autopilot-rs status
```

Calls the daemon's `/health` endpoint and prints whether AutoPilot is running. This is the command to use when you want to know whether the daemon is alive.

### `stop`

```bash
autopilot-rs stop
```

Sends `POST /shutdown` to the daemon. The daemon stops its jobs, responds, and exits shortly afterward. If no daemon is listening, the command has nothing to stop.

### `list`

```bash
autopilot-rs list
```

Opens an interactive job browser. It reads job files directly from disk; it does **not** check whether the daemon is running and does not require the daemon.

The browser supports viewing job details and deleting a job file. Deleting from this menu removes the file from disk.

### `create`

```bash
autopilot-rs create
```

Opens the interactive job creation form. The new job is written to the configured `jobs/` directory.

### `remove`

```bash
autopilot-rs remove
```

Opens an interactive job removal flow. The command is currently deprecated in favor of `list`, but still works.

### `gui`

```bash
autopilot-rs gui
```

Launches the native egui GUI. This requires a graphical session and the platform GUI dependencies used at build time.

## Examples

```bash
# Start with debug logging and the default configuration
autopilot-rs --verbose serve

# Start the API but leave jobs stopped until POST /start
autopilot-rs serve --api

# Check the daemon
autopilot-rs status

# Use a temporary configuration root
autopilot-rs --config-path /tmp/autopilot-demo create
autopilot-rs --config-path /tmp/autopilot-demo serve

# Stop the daemon
autopilot-rs stop
```

## Configuration and environment

The default configuration root is `~/.config/autopilot/` on Linux/macOS and `%APPDATA%\autopilot\` on Windows.

```text
<config-root>/
├── autopilot.jsonc
├── jobs/
└── logs/
```

The API address and the address used by `status`/`stop` come from:

| Variable | Default | Description |
| --- | --- | --- |
| `AUTOPILOT_HOST` | `127.0.0.1` | API host. |
| `AUTOPILOT_PORT` | `7883` | API port. |

```bash
AUTOPILOT_PORT=9000 autopilot-rs serve
AUTOPILOT_PORT=9000 autopilot-rs status
```

Do not expose the API to an untrusted network without an authentication layer. See [API security](API.md#security).

## Related documentation

- [Job files and schedules](JOBS.md)
- [API reference](API.md)
- [Troubleshooting](../README.md#troubleshooting)
