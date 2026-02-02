# AutoPilot-rs

A cross-platform automation tool that runs tasks when conditions are met. Write once, run on Linux, macOS, and Windows.

## What It Does

Define a job, set conditions, list tasks. AutoPilot does the rest.

```jsonc
{
  "id": "morning-setup",
  "name": "Setup my workday",
  "conditions": [
    { "type": "time", "condition": { "date": "2026/02/02", "time": "09:00:00" } },
    { "type": "wifi", "condition": { "ssid": "OfficeNetwork" } }
  ],
  "tasks": [
    { "command": "open /Applications/Slack.app" },
    { "command": "open /Applications/VSCode.app" },
    { "command": "notify-send 'Good morning!'" }
  ]
}
```

AutoPilot checks: Is it 9 AM? Are you on office WiFi? If both yes, run the tasks. Simple.

## Installation

### macOS

```bash
brew install autopilot-rs
# or build from source
cargo build --release
./target/release/autopilot-rs --help
```

### Linux

```bash
cargo build --release
./target/release/autopilot-rs --help
```

### Windows

```bash
cargo build --release
.\target\release\autopilot-rs.exe --help
```

## Quick Start

### 1. Start the Daemon

```bash
autopilot-rs serve
```

AutoPilot runs in the background, checking conditions and running tasks.

### 2. Create a Job

```bash
autopilot-rs new
```

Or manually create a JSON file in `~/.autopilot-rs/jobs/my-job.jsonc`

### 3. List Jobs

```bash
autopilot-rs list
```

See all your jobs and their status.

### 4. Stop AutoPilot

```bash
autopilot-rs stop
```

## Condition Types

### Time

Run at a specific date and time:

```jsonc
{
  "type": "time",
  "condition": {
    "date": "2026/02/02",
    "time": "14:30:00",
    "tolerance_seconds": 30
  }
}
```

### WiFi

Run when connected to a specific network:

```jsonc
{
  "type": "wifi",
  "condition": { "ssid": "HomeNetwork" }
}
```

### Bluetooth

Run when a device is connected:

```jsonc
{
  "type": "bluetooth",
  "condition": {
    "device": "My Headphones",
    "match_by_mac": false
  }
}
```

### Custom

Run a shell command and check the result:

```jsonc
{
  "type": "custom",
  "condition": {
    "command": "test -f /tmp/trigger-file",
    "check_exit_code": true
  }
}
```

Or check command output:

```jsonc
{
  "type": "custom",
  "condition": {
    "command": "date +%A",
    "target_output": "Monday",
    "check_exit_code": false
  }
}
```

### Output

Check if command output matches:

```jsonc
{
  "type": "output",
  "condition": {
    "command": "whoami",
    "target": "alice"
  }
}
```

### Variable

Check environment variables:

```jsonc
{
  "type": "variable",
  "condition": {
    "variable": "USER",
    "target": "alice"
  }
}
```

### Multiple Conditions

All conditions must be true (AND logic):

```jsonc
{
  "conditions": [
    { "type": "wifi", "condition": { "ssid": "Office" } },
    { "type": "time", "condition": { "time": "09:00:00" } },
    { "type": "custom", "condition": { "command": "test -f /etc/hosts" } }
  ]
}
```

If any condition fails, the task doesn't run.

## Job Structure

```jsonc
{
  "id": "unique-job-id",
  "name": "Human readable name",
  "description": "What this job does",
  "conditions": [
    // Zero or more conditions
  ],
  "tasks": [
    // One or more tasks
    { "command": "echo 'Hello'" },
    { "command": "touch /tmp/file" }
  ]
}
```

- **id:** Unique identifier (required)
- **name:** Display name for humans (optional)
- **description:** What this job does (optional)
- **conditions:** List of conditions to check (optional, defaults to always run)
- **tasks:** List of commands to execute (required)

## Tasks

Each task is a shell command:

```jsonc
{
  "command": "ls -la /home"
}
```

Or multi-step with pipes:

```jsonc
{
  "command": "cat /var/log/system.log | grep ERROR | wc -l"
}
```

Tasks run sequentially. If one fails, remaining tasks still run (for now).

## CLI Commands

```bash
autopilot-rs serve              # Start daemon
autopilot-rs stop               # Stop daemon
autopilot-rs list               # List all jobs
autopilot-rs new                # Create new job
autopilot-rs remove <job-id>    # Remove a job
autopilot-rs --verbose          # Verbose logging
autopilot-rs --help             # Show help
```

## Configuration

AutoPilot reads from `~/.autopilot-rs/`:

```
~/.autopilot-rs/
├── jobs/
│   ├── morning.jsonc
│   ├── evening.jsonc
│   └── cleanup.jsonc
├── state/
│   └── (internal state files)
└── logs/
    └── autopilot.log
```

## Examples

### Run backup at midnight

```jsonc
{
  "id": "nightly-backup",
  "name": "Run backup at midnight",
  "conditions": [
    {
      "type": "time",
      "condition": {
        "date": "2026/02/03",
        "time": "00:00:00",
        "tolerance_seconds": 60
      }
    }
  ],
  "tasks": [
    { "command": "/usr/local/bin/backup.sh" }
  ]
}
```

### Sync files only on home WiFi

```jsonc
{
  "id": "sync-home-wifi",
  "name": "Sync files when on home network",
  "conditions": [
    { "type": "wifi", "condition": { "ssid": "HomeNetwork" } }
  ],
  "tasks": [
    { "command": "rsync -av ~/Documents /mnt/nas" }
  ]
}
```

### Morning routine

```jsonc
{
  "id": "morning-routine",
  "name": "Morning setup",
  "conditions": [
    { "type": "time", "condition": { "time": "08:00:00", "tolerance_seconds": 300 } },
    { "type": "wifi", "condition": { "ssid": "HomeNetwork" } }
  ],
  "tasks": [
    { "command": "brew update" },
    { "command": "notify-send 'Good morning!'" },
    { "command": "open /Applications/Mail.app" }
  ]
}
```

### Run when device connects

```jsonc
{
  "id": "headphones-connected",
  "name": "Play sound when headphones connect",
  "conditions": [
    { "type": "bluetooth", "condition": { "device": "Sony Headphones" } }
  ],
  "tasks": [
    { "command": "pactl set-default-sink 'Sony Headphones'" },
    { "command": "speaker-test -t sine -f 1000 -l 1" }
  ]
}
```

### Conditional based on file

```jsonc
{
  "id": "process-if-ready",
  "name": "Process when trigger file exists",
  "conditions": [
    {
      "type": "custom",
      "condition": {
        "command": "test -f /tmp/ready-to-process",
        "check_exit_code": true
      }
    }
  ],
  "tasks": [
    { "command": "process_data.sh" },
    { "command": "rm /tmp/ready-to-process" }
  ]
}
```

## Troubleshooting

### Jobs not running

**Check if daemon is running:**
```bash
autopilot-rs list
```

If it hangs, daemon isn't running:
```bash
autopilot-rs serve
```

**Check condition logic:**

Run conditions manually to debug:
```bash
# Test time condition
date +"%Y/%m/%d %H:%M:%S"

# Test WiFi
nmcli dev show | grep CONNECTION

# Test custom command
test -f /tmp/file && echo "Exists" || echo "Missing"
```

**Check logs:**
```bash
tail -f ~/.autopilot-rs/logs/autopilot.log
```

**Enable verbose mode:**
```bash
autopilot-rs serve --verbose
```

### Condition not working

1. Test the command manually in your shell
2. Check exact paths (use absolute paths, not `~`)
3. Check file permissions
4. Check environment variables are set
5. See docs/TROUBLESHOOTING.md for detailed guides

### Task execution issues

- Commands must be shell-compatible (sh, zsh, PowerShell depending on OS)
- Use absolute paths for commands
- Redirect errors to see what went wrong: `command 2>&1`
- Test commands manually first

## Platform Support

| Feature | Linux | macOS | Windows |
|---------|-------|-------|---------|
| Time conditions | ✅ | ✅ | ✅ |
| WiFi detection | ✅ | ✅ | ✅ |
| Bluetooth detection | ✅ | ✅ | ✅ |
| Custom commands | ✅ | ✅ | ✅ |
| Output matching | ✅ | ✅ | ✅ |
| Variables | ✅ | ✅ | ✅ |

All conditions work on all platforms.

## Documentation

- **[CONDITIONS.md](docs/CONDITIONS.md)** - Detailed condition reference
- **[TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** - Debug guides
- **[PLUGIN-SYSTEM.md](PLUGIN-SYSTEM.md)** - Extend AutoPilot with custom plugins (coming soon)

## Contributing

Found a bug? Want a new condition type?

1. Check existing issues
2. Open an issue or PR on GitHub
3. Follow the code style (cargo fmt, cargo clippy)

## License

MIT

## Roadmap

- [ ] Plugin system for custom conditions
- [ ] REST API for job management
- [ ] Web dashboard
- [ ] Distributed execution (multiple machines)
- [ ] Performance optimizations
- [ ] Metrics and monitoring

## FAQ

**Q: Can I run multiple jobs?**  
A: Yes. Define as many jobs as you need. All conditions are checked independently.

**Q: What happens if a task fails?**  
A: Currently, remaining tasks still run. Error is logged.

**Q: Can I schedule jobs at specific intervals?**  
A: Use the Time condition with tolerance, or use a Custom condition to check system time/load.

**Q: Can I use environment variables in commands?**  
A: Yes. Shell expands them: `{ "command": "$HOME/backup.sh" }`

**Q: Does it work on minimal systems?**  
A: Mostly. Dependencies are minimal. Check docs for platform-specific requirements.

**Q: Can I use this with cron?**  
A: Yes, but AutoPilot is designed to replace constant cron tasks with condition-based execution.

---

Built with Rust. Runs everywhere. Automates anything.
