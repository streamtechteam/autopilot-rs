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

```bash
AUTOPILOT_HOST=127.0.0.1 AUTOPILOT_PORT=9000 autopilot-rs serve
curl http://127.0.0.1:9000/health
```

## Security

> **The API has no authentication, no authorization, and no TLS.** It speaks plain HTTP.

The default bind address is loopback, which limits access to processes on the same machine. Do **not** set `AUTOPILOT_HOST` to a non-loopback address unless you understand the consequences.

Anyone who can reach the port can:

- list and read jobs;
- create, update, and delete job files;
- start and stop jobs;
- reload or restart the daemon;
- shut the daemon down.

There is no CORS policy, rate limiting, or audit log for API requests.

### Recommended remote-access pattern

Keep the API on loopback and use an SSH tunnel from the remote machine:

```bash
# On the machine running AutoPilot-rs
autopilot-rs serve

# On the client machine
ssh -L 7883:127.0.0.1:7883 user@autopilot-host
curl http://127.0.0.1:7883/health
```

If you must bind to a non-loopback interface, put the API behind a firewall and an authenticating TLS reverse proxy. Authentication is not built in. See issue #38 for the planned hardening work.

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
| `GET` | `/status` | Live running flag and job count. |
| `POST` | `/start` | Load jobs from disk, initialize status, and start jobs. |
| `POST` | `/stop` | Stop jobs. |
| `POST` | `/reload` | Reload jobs from disk and preserve the previous running state. |
| `POST` | `/restart` | Reload jobs from disk and start them. |
| `POST` | `/shutdown` | Stop jobs and shut down the daemon. |
| `GET` | `/jobs` | List jobs from disk with live status. |
| `POST` | `/jobs` | Create a job file. |
| `GET` | `/jobs/{id}` | Get one job by ID. |
| `PUT` | `/jobs/{id}` | Update a job file. |
| `DELETE` | `/jobs/{id}` | Delete a job file. |

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
curl http://127.0.0.1:7883/status
```

```json
{
  "running": true,
  "job_count": 3
}
```

`job_count` is the number of jobs currently loaded in the runtime, not the number of files on disk. `running` reflects whether the job runtime has been started.

## Lifecycle

### `POST /start`

Loads jobs from disk, initializes status entries, and starts job execution.

```bash
curl -X POST http://127.0.0.1:7883/start
```

```json
{
  "success": true,
  "message": "Jobs started"
}
```

Returns `409 Conflict` if jobs are already started. Returns `500` if loading jobs or initializing status fails.

### `POST /stop`

Stops the running jobs.

```bash
curl -X POST http://127.0.0.1:7883/stop
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
curl -X POST http://127.0.0.1:7883/reload
```

```json
{
  "success": true,
  "message": "Config reloaded."
}
```

**Current behavior:** a reload failure still returns HTTP `200` with `"success": false` and a message. Clients must inspect the `success` field rather than relying only on the status code.

### `POST /restart`

Stops jobs, reloads job files, initializes status, and starts jobs.

```bash
curl -X POST http://127.0.0.1:7883/restart
```

```json
{
  "success": true,
  "message": "Autopilot restarted."
}
```

**Current behavior:** a restart failure also returns HTTP `200` with `"success": false`. Inspect `success`.

### `POST /shutdown`

Stops jobs and terminates the daemon. The response is sent before the process exits, then the process exits after a short delay.

```bash
curl -X POST http://127.0.0.1:7883/shutdown
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

Job files are read from `<config-root>/jobs/`. The list is the set of valid files currently on disk. Invalid files are skipped by the loader; duplicate IDs prevent the job set from loading.

## Listing jobs

### `GET /jobs`

```bash
curl http://127.0.0.1:7883/jobs
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
curl http://127.0.0.1:7883/jobs/hello
```

Returns `404` if no job file with that ID exists.

## Creating a job

### `POST /jobs`

```bash
curl -X POST http://127.0.0.1:7883/jobs \
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
  "path": "/home/user/.config/autopilot/jobs/job_1234567890_0.jsonc"
}
```

Notes:

- The body uses the same schema as a job file. `conditions` and `tasks` are required arrays.
- The server generates the job ID. The `id` field is required by the schema but is ignored by this endpoint.
- The response `path` is the filesystem path where the job was written. It is absolute for the default configuration root, but may be relative if a relative `--config-path` was used.
- `400` is returned if the job cannot be written. Malformed JSON is rejected by the request extractor with `400`/`422` depending on the validation failure.

## Updating a job

### `PUT /jobs/{id}`

```bash
curl -X PUT http://127.0.0.1:7883/jobs/hello \
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
  "path": "/home/user/.config/autopilot/jobs/hello.jsonc"
}
```

The path ID selects the job; the body `id` is ignored. The body must include `conditions` and `tasks`. `name` and `description` are optional and keep their existing values when omitted. `when` and `check_interval` are replaced from the body, so omitting them clears a schedule/retry interval. `task_mode` is optional; when omitted on update, the existing task mode is preserved.

Returns `404` if the job does not exist. Returns `400` if the update is invalid.

## Deleting a job

### `DELETE /jobs/{id}`

```bash
curl -X DELETE http://127.0.0.1:7883/jobs/hello
```

```json
{
  "success": true,
  "message": "Job deleted"
}
```

Returns `404` if the ID does not match a job file. Returns `400` for an invalid ID.

## Lifecycle and reload behavior

The API's job CRUD operations write files on disk. They do **not** add, remove, or update jobs that are already loaded in the running runtime.

After creating, updating, or deleting jobs, apply the change with:

```bash
curl -X POST http://127.0.0.1:7883/reload
```

or:

```bash
curl -X POST http://127.0.0.1:7883/restart
```

- `reload` preserves whether jobs were running.
- `restart` always starts jobs after loading.
- `start` loads jobs from disk before starting them.
- `stop` stops job execution but leaves the API running.

Deleting or updating a job that is currently running does not stop the already-running task. Reload or restart to apply the change.

## Error responses

| Status | Meaning |
| --- | --- |
| `200` | Request handled. Check `success` for reload/restart operations. |
| `400` | Invalid request or job write failure. |
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
curl -X POST http://127.0.0.1:9000/jobs \
  -H 'Content-Type: application/json' \
  -d '{
    "id": "ignored",
    "name": "Hello",
    "conditions": [],
    "tasks": [{"command": "echo hello"}]
  }'

# Apply the new file to the running daemon
curl -X POST http://127.0.0.1:9000/reload

# Inspect status
curl http://127.0.0.1:9000/status
curl http://127.0.0.1:9000/jobs
```
