# AutoPilot-rs Conditions Guide

This guide explains all available condition types and how to use them in your automation jobs.

For the surrounding job format, schedules, and runtime behavior, see [Job files and schedules](JOBS.md). For the HTTP API, see the [API reference](API.md).

## Overview

Conditions are checks that must pass before a job's tasks are executed. A job with multiple conditions requires **ALL** conditions to be true before running. Use the [Logical Condition](#13-logical-condition-composite) when you need OR or NOR logic.

Conditions are synchronous `check() -> bool` implementations. A condition that cannot complete its backend check generally returns `false`; it does not raise a structured error into the job runner.

Every condition object also accepts an optional `check_interval` (milliseconds, as a string) that overrides the job-level `check_interval` for that condition, so each condition in an unscheduled job can be retried on its own cadence. The override only applies to top-level job conditions; intervals on conditions nested inside a `logical` condition are ignored.

## Condition Types

### 1. WiFi Condition

**Purpose:** Check if you're connected to a specific WiFi network.

**Schema:**

```jsonc
{
  "type": "wifi",
  "condition": {
    "ssid": "network_name" // Required: WiFi network name (SSID)
  }
}
```

**Example:**

```jsonc
{
  "type": "wifi",
  "condition": {
    "ssid": "HomeNetwork"
  }
}
```

**Platform Support:**

- **Linux:** Uses `nmcli` (NetworkManager) or `iwgetid`
- **macOS:** Uses `networksetup` and falls back to the `airport` utility
- **Windows:** Uses `netsh wlan show interfaces`, with a profile-based fallback

**Use Cases:**

- Auto-connect VPN when at home
- Enable/disable services based on network
- Sync files only when on trusted network

---

### 2. Bluetooth Condition

**Purpose:** Check if a specific Bluetooth device is connected.

**Schema:**

```jsonc
{
  "type": "bluetooth",
  "condition": {
    "device": "device_name_or_mac", // Required: Device name or MAC address
    "match_by_mac": false // Optional: true to match by MAC, false for name (default: false)
  }
}
```

**Examples:**

_Match by device name:_

```jsonc
{
  "type": "bluetooth",
  "condition": {
    "device": "My Headphones",
    "match_by_mac": false
  }
}
```

_Match by MAC address:_

```jsonc
{
  "type": "bluetooth",
  "condition": {
    "device": "AA:BB:CC:DD:EE:FF",
    "match_by_mac": true
  }
}
```

**Platform Support:**

- **Linux:** Uses `bluetoothctl` or `hcitool`
- **macOS:** Uses `system_profiler`, with a `defaults` fallback
- **Windows:** Uses PowerShell `Get-CimInstance`

**Use Cases:**

- Auto-play music when headphones connect
- Adjust volume for speakers
- Mute notifications when phone is nearby

---

### 3. Command Condition

**Purpose:** Execute arbitrary shell commands and check the result.

**Schema:**

```jsonc
{
  "type": "command",
  "condition": {
    "command": "shell_command", // Required: Command to execute
    "check_exit_code": true, // Optional: Check exit code 0 (default: true)
    "target_output": "expected_text" // Optional: Check if output matches this value
  }
}
```

**Examples:**

_Check exit code (success/failure):_

```jsonc
{
  "type": "command",
  "condition": {
    "command": "test -f /tmp/my-file",
    "check_exit_code": true
  }
}
```

_Check command output:_

```jsonc
{
  "type": "command",
  "condition": {
    "command": "cat /sys/class/power_supply/BAT0/status",
    "check_exit_code": false,
    "target_output": "Discharging"
  }
}
```

**Use Cases:**

- Check if a file exists: `test -f /path/to/file`
- Check battery status: `cat /sys/class/power_supply/BAT0/status`
- Check if process is running: `pgrep process_name`
- Check user input: `whoami`
- Custom scripts: `/usr/local/bin/my-check.sh`

**Shell Platform Notes:**

- **Linux/macOS:** Uses `sh -c`
- **Windows:** Uses `cmd /C`

---

### 4. Variable Condition

**Purpose:** Check if an environment variable matches an expected value.

**Schema:**

```jsonc
{
  "type": "variable",
  "condition": {
    "variable": "VAR_NAME", // Required: Environment variable name
    "target": "expected_value" // Required: Expected value
  }
}
```

**Example:**

```jsonc
{
  "type": "variable",
  "condition": {
    "variable": "USER",
    "target": "alice"
  }
}
```

**Use Cases:**

- Run tasks only for specific users
- Check custom environment variables
- Prevent execution in certain environments

---

### 5. Power Condition

**Purpose:** Check charging status or battery level.

**Schema:**

```jsonc
{
  "type": "power",
  "condition": {
    "check_charging": true, // Optional: Check if charging (default: false)
    "threshold": 20, // Optional: Battery percentage threshold (0-100)
    "operator": "less" // Optional: "greater" or "less" (default: "greater")
  }
}
```

**Examples:**

_Check if charging:_

```jsonc
{
  "type": "power",
  "condition": {
    "check_charging": true
  }
}
```

_Check battery level:_

```jsonc
{
  "type": "power",
  "condition": {
    "threshold": 20,
    "operator": "less"
  }
}
```

**Platform Support:**

- **Linux:** Reads from `/sys/class/power_supply/`
- **macOS:** Uses `pmset -g batt`
- **Windows:** Uses PowerShell `Win32_Battery`

**Use Cases:**

- Pause heavy tasks when battery is low
- Run backup only when charging
- Alert when battery drops below threshold

---

### 6. Resource Condition

**Purpose:** Check CPU or RAM usage.

**Schema:**

```jsonc
{
  "type": "resource",
  "condition": {
    "resource_type": "cpu", // Required: "cpu" or "memory"
    "threshold": 80, // Required: Percentage threshold (0-100)
    "operator": "less" // Optional: "greater" or "less" (default: "greater")
  }
}
```

**Examples:**

_Run when CPU is idle:_

```jsonc
{
  "type": "resource",
  "condition": {
    "resource_type": "cpu",
    "threshold": 20,
    "operator": "less"
  }
}
```

_Check if RAM is high:_

```jsonc
{
  "type": "resource",
  "condition": {
    "resource_type": "memory",
    "threshold": 80,
    "operator": "greater"
  }
}
```

**Use Cases:**

- Run heavy tasks only when system is idle
- Trigger cleanup when memory is high
- Pause background jobs during high CPU usage

---

### 7. Internet Condition

**Purpose:** Check internet reachability (ping-based).

**Schema:**

```jsonc
{
  "type": "internet",
  "condition": {
    "host": "8.8.8.8", // Optional: Host to ping (default: "8.8.8.8")
    "timeout": 2 // Optional: Timeout in seconds (default: 2)
  }
}
```

**Example:**

```jsonc
{
  "type": "internet",
  "condition": {
    "host": "google.com",
    "timeout": 5
  }
}
```

**Use Cases:**

- Sync files only when online
- Skip network-dependent tasks when offline
- Check if a specific server is reachable

---

### 8. Process Condition

**Purpose:** Check if a process is running (or not running).

**Schema:**

```jsonc
{
  "type": "process",
  "condition": {
    "process_name": "firefox", // Required: Process name to check
    "should_be_running": true // Optional: true = must be running, false = must NOT be running (default: true)
  }
}
```

**Examples:**

_Run only if Firefox is running:_

```jsonc
{
  "type": "process",
  "condition": {
    "process_name": "firefox",
    "should_be_running": true
  }
}
```

_Run only if no browser is open:_

```jsonc
{
  "type": "process",
  "condition": {
    "process_name": "chrome",
    "should_be_running": false
  }
}
```

**Use Cases:**

- Take action when an app starts
- Wait until a process exits
- Prevent conflicts with running applications

---

### 9. Disk Space Condition

**Purpose:** Check available disk space.

**Schema:**

```jsonc
{
  "type": "diskspace",
  "condition": {
    "path": "/", // Required: Path to check (mount point)
    "min_free_gb": 10, // Required: Minimum free space in GB
    "max_used_gb": 100 // Optional: Maximum used space in GB
  }
}
```

**Example:**

```jsonc
{
  "type": "diskspace",
  "condition": {
    "path": "/",
    "min_free_gb": 5
  }
}
```

The current implementation compares `path` against the beginning of each detected disk mount point. Use a mount point such as `/` or `C:\`; an arbitrary subdirectory such as `/home/user` will not match unless the filesystem is actually mounted at that path.

**Use Cases:**

- Trigger cleanup when disk is almost full
- Skip downloads if space is low
- Alert on disk space issues

---

### 10. File Condition

**Purpose:** Check file existence or properties.

**Schema:**

```jsonc
{
  "type": "file",
  "condition": {
    "path": "/path/to/file", // Required: File or directory path
    "check_type": "exists", // Required: "exists", "modified_recently", or "size_changed"
    "time_threshold": 300, // Optional: Seconds for "modified_recently" (default: 300)
    "size_threshold": 1024 // Optional: Minimum size in bytes for "size_changed" (default: 0)
  }
}
```

> **Current behavior:** `size_changed` is a minimum-size check, not change detection. It passes when the file exists and its current size is greater than or equal to `size_threshold`. It does not compare against a previous size. With no threshold, any existing file passes. See issue #42 for the planned change-detection fix.

**Examples:**

_Check if file exists:_

```jsonc
{
  "type": "file",
  "condition": {
    "path": "/tmp/ready.flag",
    "check_type": "exists"
  }
}
```

_Check if file was recently modified:_

```jsonc
{
  "type": "file",
  "condition": {
    "path": "/var/log/app.log",
    "check_type": "modified_recently",
    "time_threshold": 60
  }
}
```

_Check that a file has reached at least 1 MiB (current `size_changed` behavior):_

```jsonc
{
  "type": "file",
  "condition": {
    "path": "/tmp/export.zip",
    "check_type": "size_changed",
    "size_threshold": 1048576
  }
}
```

**Use Cases:**

- Wait for a file to appear
- Trigger on recent log activity with `modified_recently`
- Wait for a download or export to reach a minimum size with `size_changed`

---

### 11. External Device Condition

**Purpose:** Check for connected USB/external drives.

**Schema:**

```jsonc
{
  "type": "externaldevice",
  "condition": {
    "device_identifier": "USB_Drive", // Required: Device name or mount point
    "check_by_name": true // Optional: true = match by name, false = match by mount point (default: false)
  }
}
```

**Examples:**

_Check by device name:_

```jsonc
{
  "type": "externaldevice",
  "condition": {
    "device_identifier": "My USB Drive",
    "check_by_name": true
  }
}
```

_Check by mount point:_

```jsonc
{
  "type": "externaldevice",
  "condition": {
    "device_identifier": "/media/backup",
    "check_by_name": false
  }
}
```

**Use Cases:**

- Auto-backup when USB drive is connected
- Eject drive after operation completes
- Sync files when specific drive is mounted

---

### 12. Screen Condition

**Purpose:** Check screen/monitor configuration (number of screens, active screen, screen names).

**Schema:**

```jsonc
{
  "type": "screen",
  "condition": {
    "screen_count": 2, // Optional: Expected number of connected screens
    "active_screen_name": "HDMI-1", // Optional: Name of the active/primary screen
    "screen_names": ["eDP-1", "HDMI-1"] // Optional: List of expected screen names
  }
}
```

**Examples:**

_Check for dual monitor setup:_

```jsonc
{
  "type": "screen",
  "condition": {
    "screen_count": 2
  }
}
```

_Check for specific active screen:_

```jsonc
{
  "type": "screen",
  "condition": {
    "active_screen_name": "HDMI-1"
  }
}
```

_Check for specific screen names:_

```jsonc
{
  "type": "screen",
  "condition": {
    "screen_names": ["eDP-1", "HDMI-1", "DP-1"]
  }
}
```

_Combine multiple screen checks:_

```jsonc
{
  "type": "screen",
  "condition": {
    "screen_count": 2,
    "active_screen_name": "HDMI-1",
    "screen_names": ["eDP-1", "HDMI-1"]
  }
}
```

**Platform Support:**

- **Linux:** Uses `display_info` crate with X11/Wayland support
- **macOS:** Uses native display APIs
- **Windows:** Uses Win32 display APIs

**Important Notes:**

- All fields are **optional**, but at least one check should be specified. The interactive form enforces this; an empty raw JSON condition has no constraints and currently passes.
- The condition passes only if **all specified checks** are met (AND logic)
- Screen names are platform-specific (e.g., "HDMI-1", "eDP-1" on Linux)
- If `screen_names` is specified, all listed screens must be present

**Use Cases:**

- Run tasks only when docked (multiple monitors)
- Adjust display settings when external monitor is connected
- Trigger presentations mode when projector is detected
- Run specific layouts based on monitor configuration

---

### 13. Logical Condition (Composite)

**Purpose:** Combine multiple conditions with various logical operators (AND, OR, NOR).

**Schema:**

```jsonc
{
  "type": "logical",
  "condition": {
    "operator": "And", // Required: "And", "Or", or "Nor"
    "conditions": [
      // Array of conditions to combine with the specified operator
      { "type": "wifi", "condition": { "ssid": "HomeNetwork" } },
      { "type": "bluetooth", "condition": { "device": "MyPhone" } }
    ]
  }
}
```

**Examples:**

_OR logic (at least one must pass):_

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

_AND logic (all must pass):_

```jsonc
{
  "type": "logical",
  "condition": {
    "operator": "And",
    "conditions": [
      { "type": "wifi", "condition": { "ssid": "HomeNetwork" } },
      { "type": "power", "condition": { "check_charging": true } }
    ]
  }
}
```

_NOR logic (none must pass):_

```jsonc
{
  "type": "logical",
  "condition": {
    "operator": "Nor",
    "conditions": [
      { "type": "process", "condition": { "process_name": "chrome" } },
      { "type": "process", "condition": { "process_name": "firefox" } }
    ]
  }
}
```

**Available Operators:**

- `And`: All conditions must be true
- `Or`: At least one condition must be true
- `Nor`: None of the conditions must be true (NOT OR)

**Platform Support:** All platforms

**Use Cases:**

- Run tasks in multiple network environments (OR)
- Complex multi-factor conditions (AND)
- Negated condition combinations (NOR)
- Fallback scenarios (OR)
- Nested condition logic
- Reusable condition groups

---

### 14. Desktop Environment Condition

**Purpose:** Check which desktop environment is currently running.

**Schema:**

```jsonc
{
  "type": "desktopenv",
  "condition": {
    "target": "GNOME" // Required: desktop environment name
  }
}
```

**Example:**

```jsonc
{
  "type": "desktopenv",
  "condition": {
    "target": "KDE"
  }
}
```

The comparison is case-insensitive. AutoPilot checks `XDG_CURRENT_DESKTOP`,
`DESKTOP_SESSION`, and common desktop-specific environment variables, then
falls back to `WAYLAND_DISPLAY`.

**Platform Support:**

- **Linux:** Detected from the active desktop session environment
- **macOS:** Reports `MACOS`
- **Windows:** Reports `WINDOWS`

**Use Cases:**

- Apply desktop-specific window or notification commands
- Run GNOME-only extensions or settings
- Skip tasks that only make sense under a particular desktop environment

## Multiple Conditions (AND Logic)

When a job has multiple conditions in the `conditions` array at the job level, **all conditions must be true** for the job to execute:

```jsonc
{
  "id": "complex-automation",
  "name": "Home evening routine",
  "conditions": [
    {
      "type": "wifi",
      "condition": { "ssid": "HomeNetwork" }
    },
    {
      "type": "bluetooth",
      "condition": { "device": "MyPhone" }
    }
  ],
  "tasks": [{ "command": "notify-send 'Welcome home!'" }]
}
```

In this example, the job runs **only if**:

- You're connected to "HomeNetwork" WiFi **AND**
- Your phone is connected via Bluetooth

## OR vs AND Logic

- **Job-level conditions array:** Always AND logic (all must pass)
- **`"type": "logical"` condition with `"operator": "Or"`:** OR logic (at least one must pass)
- **`"type": "logical"` condition with `"operator": "And"`:** AND logic (all must pass, useful for nesting)

**Example combining both:**

```jsonc
{
  "type": "logical",
  "condition": {
    "operator": "And",
    "conditions": [
      {
        "type": "logical",
        "condition": {
          "operator": "Or",
          "conditions": [
            { "type": "wifi", "condition": { "ssid": "HomeNetwork" } },
            { "type": "wifi", "condition": { "ssid": "OfficeNetwork" } }
          ]
        }
      },
      { "type": "power", "condition": { "check_charging": true } }
    ]
  }
}
```

This runs if:
- (Home WiFi **OR** Office WiFi) **AND** Charging

---

## Tips & Best Practices

### Performance

- **Avoid expensive commands** in conditions (they run before a job executes)
- **Use platform-native conditions** (WiFi, Bluetooth, Process) instead of shell commands when possible
- **Use appropriate intervals** - don't check conditions every millisecond

### Reliability

- **Test commands manually** before using in conditions:
  ```bash
  command && echo "Success" || echo "Failed"
  ```
- **Handle errors gracefully** - failed conditions prevent task execution
- **Use `check_exit_code: true`** for simple success/failure checks

### Security

- **Avoid shell injection** - don't use untrusted input in commands
- **Be careful with `target_output`** - output comparison can be slow for large outputs
- **Use absolute paths** in commands for predictability

### Debugging

- **Enable verbose logging** with the global `--verbose` option, before the subcommand:
  ```bash
  autopilot-rs --verbose serve
  ```
- **Check logs** in your configured `logs/` directory
- **Test conditions in isolation** using the shell manually

---

## Common Patterns

### Only When Docked (Multiple Monitors)

```jsonc
{
  "type": "screen",
  "condition": {
    "screen_count": 2
  }
}
```

### Only on Home OR Office WiFi

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

### Only When at Home

```jsonc
{
  "type": "logical",
  "condition": {
    "operator": "And",
    "conditions": [
      { "type": "wifi", "condition": { "ssid": "HomeNetwork" } },
      { "type": "bluetooth", "condition": { "device": "HomeHub" } }
    ]
  }
}
```

### Only on Weekdays

```jsonc
{
  "type": "command",
  "condition": {
    "command": "test $(date +%u) -le 5",
    "check_exit_code": true
  }
}
```

### Only When Idle

```jsonc
{
  "type": "resource",
  "condition": {
    "resource_type": "cpu",
    "threshold": 20,
    "operator": "less"
  }
}
```

### Only When Charging with Sufficient Battery

```jsonc
[
  {
    "type": "power",
    "condition": { "check_charging": true }
  },
  {
    "type": "power",
    "condition": { "threshold": 50, "operator": "greater" }
  }
]
```

---

## Error Handling

Conditions are boolean checks, and most built-in conditions return `false` when their backend command or platform API cannot provide an answer. A `false` condition prevents the job from running; that is the safe default.

Other failure modes to be aware of:

- **Invalid job file:** the loader skips files that fail to deserialize. There is no per-file error surfaced in the CLI today; validate the file manually and check the loaded job list.
- **Duplicate job IDs:** job loading fails and jobs do not start. Resolve duplicate IDs across files.
- **Condition panic:** a panic in a condition check is not a supported failure path. Unscheduled jobs may terminate their job task; scheduled jobs treat a panicked check as `false`.
- **Task failure:** a failed task is logged, and the job status is set to `Failed`. The remaining tasks still run in `concurrent` and `sequential` modes; `sequential_stop_on_failure` skips them.

Use verbose logging and the [API status](API.md#health-and-status) endpoints when diagnosing a job that does not run.

---

## Condition Reference Table

| Type           | Key Fields                                       | Platform |
| -------------- | ------------------------------------------------ | -------- |
| wifi           | `ssid`                                           | All      |
| bluetooth      | `device`, `match_by_mac`                         | All      |
| command        | `command`, `check_exit_code`, `target_output`    | All      |
| variable       | `variable`, `target`                             | All      |
| power          | `check_charging`, `threshold`, `operator`        | All      |
| resource       | `resource_type`, `threshold`, `operator`         | All      |
| internet       | `host`, `timeout`                                | All      |
| process        | `process_name`, `should_be_running`              | All      |
| diskspace      | `path`, `min_free_gb`, `max_used_gb`             | All      |
| file           | `path`, `check_type`, `time_threshold`, `size_threshold` | All      |
| externaldevice | `device_identifier`, `check_by_name`             | All      |
| screen         | `screen_count`, `active_screen_name`, `screen_names` | All      |
| logical        | `operator` (`And`, `Or`, or `Nor`) and `conditions` | All      |
| desktopenv     | `target`                                         | All      |

---

## See Also

- [Job files and schedules](JOBS.md)
- [CLI reference](CLI.md)
- [API reference](API.md)
- [Example jobs](../examples/)
- [Job templates](../templates/)
