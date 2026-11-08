# API reference

AutoPilot-rs exposes a small HTTP API for health checks, lifecycle control, and job management. The API is started by `autopilot-rs serve`; there is no separate server binary.

## Base URL

The default address is:

```text
http://127.0.0.1:7883
```

The address is controlled by environment variables and is shared by the server, the CLI health check, and the CLI `stop` command:

| Variable | Default | Description |
| --- | --- | --- |
| `AUTOPILOT_HOST` | `127.0.0.1` | Host or IP address to bind. |
| `AUTOPILOT_PORT` | `7883` | TCP port. |
| `AUTOPILOT_API_TOKEN` | generated | Bearer token for API requests. Overrides `<config-root>/api-token`. |
| `AUTOPILOT_ALLOW_REMOTE` | unset | Must be set to `1` to bind to a non-loopback host. |

```bash
AUTOPILOT_HOST=127.0.0.1 AUTOPILOT_PORT=9000 autopilot-rs serve
curl http://127.0.0.1:9000/health
```

## Authentication

Every route except `GET /health` requires:

```http
Authorization: Bearer <token>
```

At startup the daemon generates a random 256-bit token and writes it to `<config-root>/api-token` with owner-only permissions on Unix. `AUTOPILOT_API_TOKEN` overrides the file. The CLI `stop` command reads the same token automatically.

```bash
TOKEN=$(cat ~/.config/autopilot/api-token)
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:7883/status
```

Missing or incorrect tokens return `401 Unauthorized`. Requests with an `Origin` header that does not match the server authority return `403 Forbidden`, and a mismatched `Host` header returns `400 Bad Request`. The exception is a wildcard bind such as `0.0.0.0`, where any `Host` is accepted because the reachable address is not known ahead of time; the bearer token is still required.

## Security

> **The API has bearer-token authentication but no TLS and no per-user authorization.** It speaks plain HTTP.

The default bind address is loopback. Binding to a non-loopback address requires both `AUTOPILOT_HOST` and `AUTOPILOT_ALLOW_REMOTE=1`. The opt-in exists to make remote exposure deliberate; it does not make it safe.

A caller with the token can:

- list and read jobs;
- create, update, and delete job files;
- start and stop jobs;
- reload or restart the daemon;
- shut the daemon down.

There is no per-user authorization, rate limiting, or request audit log. `/health` is intentionally unauthenticated and only reports liveness.

### Recommended remote-access pattern

Keep the API on loopback and use an SSH tunnel from the remote machine:

```bash
# On the machine running AutoPilot-rs
autopilot-rs serve

# On the client machine
ssh -L 7883:127.0.0.1:7883 user@autopilot-host
curl http://127.0.0.1:7883/health
```

If you must bind to a non-loopback interface, set `AUTOPILOT_ALLOW_REMOTE=1` and put the API behind a firewall and a TLS-terminating reverse proxy. Keep the token secret and do not use the wildcard bind unless the network is otherwise restricted.

```bash
# Only on a trusted network, behind a TLS reverse proxy:
AUTOPILOT_HOST=0.0.0.0 AUTOPILOT_ALLOW_REMOTE=1 autopilot-rs serve
```

Setting `AUTOPILOT_HOST=0.0.0.0` binds all interfaces, but the same value is also used by `status` and `stop` when they construct client addresses. Prefer an explicit loopback address for local use and a tunnel for remote use.

## Starting the API

`autopilot-rs serve` always starts the API. The `--api` flag controls job auto-start, not API startup:

| Command | API | Jobs |
| --- | --- | --- |
| `autopilot-rs serve` | starts | start immediately |
| `autopilot-rs serve --api` | starts | remain stopped until `POST /start` |

## Endpoints

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Health check used by the CLI. |
| `GET` | `/status` | Live running flag, runtime job count, and pending-reload flag. |
| `POST` | `/start` | Load jobs from disk, initialize status, and start jobs. |
| `POST` | `/stop` | Stop jobs. |
| `POST` | `/reload` | Reload jobs from disk and preserve the previous running state. |
| `POST` | `/restart` | Reload jobs from disk and start them. |
| `POST` | `/shutdown` | Stop jobs and shut down the daemon. |
| `GET` | `/jobs` | List jobs from disk with live status. |
| `POST` | `/jobs` | Create a job file. Reload required to apply it. |
| `GET` | `/jobs/{id}` | Get one job by ID. |
| `PUT` | `/jobs/{id}` | Update a job file. Reload required to apply it. |
| `DELETE` | `/jobs/{id}` | Delete a job file. Reload required to apply it. |

The API is not versioned and has no pagination; `GET /jobs` returns all configured jobs.

## Health and status

### `GET /health`

```bash
curl http://127.0.0.1:7883/health
```

```json
{
  "status": "ok",
  "service": "autopilot-api"
}
```

This endpoint is used by `autopilot-rs status` and by the daemon's own instance check.

### `GET /status`

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:7883/status
```

```json
{
  "running": true,
  "job_count": 3,
  "reload_required": false
}
```

`job_count` is the number of jobs currently loaded in the runtime, not the number of files on disk. `running` reflects whether the job runtime has been started. `reload_required` is `true` after a successful API job create, update, or delete until `POST /start`, `POST /reload`, or `POST /restart` applies the files to the runtime.

While `reload_required` is `true`, `job_count` and job status can describe the old runtime rather than the current files. Direct file edits are not detected, so the flag does not change for them.

## Lifecycle

### `POST /start`

Loads jobs from disk, initializes status entries, and starts job execution.

```bash
curl -H "Authorization: Bearer $TOKEN" -X POST http://127.0.0.1:7883/start
```

```json
{
  "success": true,
  "message": "Jobs started"
}
```

Returns `409 Conflict` if jobs are already started. Returns `500` if loading jobs or initializing status fails. On success, this clears `reload_required`.

### `POST /stop`

Stops the running jobs.

```bash
curl -H "Authorization: Bearer $TOKEN" -X POST http://127.0.0.1:7883/stop
```

```json
{
  "success": true,
  "message": "Jobs stopped"
}
```

Returns `409 Conflict` if jobs are not running. Returns `500` if stopping fails.

### `POST /reload`

Stops the current jobs, reloads job files from disk, reinitializes status, and restarts jobs if they were running before the reload.

```bash
curl -H "Authorization: Bearer $TOKEN" -X POST http://127.0.0.1:7883/reload
```

```json
{
  "success": true,
  "message": "Config reloaded."
}
```

A reload failure returns HTTP `500` with `"success": false` and a message describing the error. A successful reload clears `reload_required`.

### `POST /restart`

Stops jobs, reloads job files, initializes status, and starts jobs.

```bash
curl -H "Authorization: Bearer $TOKEN" -X POST http://127.0.0.1:7883/restart
```

```json
{
  "success": true,
  "message": "Autopilot restarted."
}
```

A restart failure returns HTTP `500` with `"success": false` and a message describing the error. A successful restart clears `reload_required`.

### `POST /shutdown`

Stops jobs and terminates the daemon. The response is sent before the process exits, then the process exits after a short delay.

```bash
curl -H "Authorization: Bearer $TOKEN" -X POST http://127.0.0.1:7883/shutdown
```

```json
{
  "success": true,
  "message": "Autopilot shutting down"
}
```

Returns `500` if stopping jobs fails. This is the endpoint used by `autopilot-rs stop`.

## Job responses

Most job endpoints return a `JobResponse` or an array of them:

```json
{
  "id": "morning-setup",
  "name": "Set up my workday",
  "description": "Opens work apps",
  "loaded": true,
  "status": "Running"
}
```

| Field | Description |
| --- | --- |
| `id` | Job ID. |
| `name` | Display name. |
| `description` | Description from the job file. |
| `loaded` | Live-runtime hint. For `GET /jobs` it is true when the status manager has an entry for the job; for `GET /jobs/{id}` it reflects the runtime job's loaded flag. |
| `status` | Debug representation of the live status, or `"None"` when no live status exists. Examples: `Unknown`, `Running`, `Completed`, `Failed`, `Unsatisfied`, `Scheduled`. |

Job files are read from `<config-root>/jobs/`. Invalid files are reported with their path and prevent the job set from loading; duplicate IDs also prevent loading.

## Listing jobs

### `GET /jobs`

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:7883/jobs
```

```json
[
  {
    "id": "hello",
    "name": "Say hello",
    "description": " ",
    "loaded": false,
    "status": "None"
  }
]
```

### `GET /jobs/{id}`

```bash
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:7883/jobs/hello
```

Returns `404` if no job file with that ID exists.

## Creating a job

### `POST /jobs`

```bash
curl -H "Authorization: Bearer $TOKEN" -X POST http://127.0.0.1:7883/jobs \
  -H 'Content-Type: application/json' \
  -d '{
    "id": "ignored-by-create",
    "name": "Say hello",
    "description": "Example job",
    "when": null,
    "check_interval": null,
    "conditions": [],
    "tasks": [{"command": "echo hello"}]
  }'
```

```json
{
  "success": true,
  "message": "Job created",
  "path": "/home/user/.config/autopilot/jobs/job_1234567890_0.jsonc",
  "reload_required": true
}
```

Notes:

- The body uses the same schema as a job file. `conditions` and `tasks` are required arrays.
- The server generates the job ID. The `id` field is required by the schema but is ignored by this endpoint.
- The response `path` is the filesystem path where the job was written. It is absolute for the default configuration root, but may be relative if a relative `--config-path` was used.
- The endpoint only writes the file. It does not change the live runtime. Call `POST /reload` or `POST /restart` to apply the new job; `reload_required` becomes `true` until then.
- `400` is returned for an invalid job definition. Malformed JSON is rejected by the request extractor with `400`/`422` depending on the validation failure. Invalid `check_interval` values and cron expressions are rejected with `400`.
- `500` is returned for filesystem failures.

## Updating a job

### `PUT /jobs/{id}`

```bash
curl -H "Authorization: Bearer $TOKEN" -X PUT http://127.0.0.1:7883/jobs/hello \
  -H 'Content-Type: application/json' \
  -d '{
    "id": "ignored-by-update",
    "name": "Say hello again",
    "description": "Updated",
    "conditions": [],
    "tasks": [{"command": "echo updated"}]
  }'
```

```json
{
  "success": true,
  "message": "Job updated",
  "path": "/home/user/.config/autopilot/jobs/hello.jsonc",
  "reload_required": true
}
```

The path ID selects the job; the body `id` is ignored. The body must include `conditions` and `tasks`. `name` and `description` are optional and keep their existing values when omitted. `when` and `check_interval` are replaced from the body, so omitting them clears a schedule/retry interval. `task_mode` is optional; when omitted on update, the existing task mode is preserved.

Updating a job only writes the file. It does not change the live runtime. Call `POST /reload` or `POST /restart` to apply the change; `reload_required` becomes `true` until then.

Returns `404` if the job does not exist. Returns `400` if the update is invalid. Returns `500` for filesystem failures.

## Deleting a job

### `DELETE /jobs/{id}`

```bash
curl -H "Authorization: Bearer $TOKEN" -X DELETE http://127.0.0.1:7883/jobs/hello
```

```json
{
  "success": true,
  "message": "Job deleted",
  "reload_required": true
}
```

Deleting a job only removes the file. It does not change the live runtime. Call `POST /reload` or `POST /restart` to apply the deletion; `reload_required` becomes `true` until then.

Returns `404` if the ID does not match a job file. Returns `400` for an invalid ID. Returns `500` for filesystem failures.

## Lifecycle and reload behavior

The API's job CRUD operations write job files on disk. They do not modify the live runtime:

- Creating, updating, or deleting a job sets `reload_required` to `true`.
- `POST /reload`, `POST /restart`, and `POST /start` load the files from disk and clear `reload_required` on success.
- `POST /stop` only stops execution; it does not apply pending file changes.
- A failed reload or restart leaves `reload_required` set.

This keeps a CRUD request from interrupting unrelated running tasks. It also means the daemon can serve stale runtime state until the reload is applied. Apply API or direct file changes explicitly:

```bash
curl -H "Authorization: Bearer $TOKEN" -X POST http://127.0.0.1:7883/reload
```

```bash
curl -H "Authorization: Bearer $TOKEN" -X POST http://127.0.0.1:7883/restart
```

- `reload` preserves whether jobs were running.
- `restart` always starts jobs after loading.
- `start` loads jobs from disk before starting them.
- `stop` stops job execution but leaves the API running.

A reload restarts the scheduler, so it interrupts tasks that are still running. Plan updates accordingly. Direct file edits are not tracked by `reload_required`; call `POST /reload` or `POST /restart` after editing files outside the API.

## Error responses

| Status | Meaning |
| --- | --- |
| `200` | Request handled successfully. |
| `400` | Invalid request, invalid Host header, or invalid job definition. |
| `401` | Missing or invalid bearer token. |
| `403` | Cross-origin request rejected. |
| `404` | Job ID not found. |
| `409` | Lifecycle conflict, such as starting an already-started runtime or stopping a stopped one. |
| `422` | Request body failed schema validation (Axum JSON extractor). |
| `500` | Internal failure, including job loading, runtime, or shutdown errors. |

Error bodies are currently empty status responses unless the handler returns a JSON body. Do not assume a structured error payload.

## Complete example

```bash
# Start the API with jobs auto-started
AUTOPILOT_PORT=9000 autopilot-rs serve

# Health
curl http://127.0.0.1:9000/health

# Create a job
curl -H "Authorization: Bearer $TOKEN" -X POST http://127.0.0.1:9000/jobs \
  -H 'Content-Type: application/json' \
  -d '{
    "id": "ignored",
    "name": "Hello",
    "conditions": [],
    "tasks": [{"command": "echo hello"}]
  }'

# Apply the new file to the running daemon
curl -H "Authorization: Bearer $TOKEN" -X POST http://127.0.0.1:9000/reload

# Inspect status
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:9000/status
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:9000/jobs
```
