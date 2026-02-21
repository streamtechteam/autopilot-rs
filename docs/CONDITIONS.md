# AutoPilot-rs Conditions Guide

This guide explains all available condition types and how to use them in your automation jobs.

## Overview

Conditions are checks that must pass before a job's tasks are executed. A job with multiple conditions requires **ALL** conditions to be true before running.

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
- **macOS:** Uses `airport -I` command
- **Windows:** Uses `netsh wlan show interfaces`

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
- **macOS:** Uses defaults/Bluetooth framework
- **Windows:** Uses PowerShell Get-CimInstance

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

- **Linux:** Uses `sh` (POSIX shell)
- **macOS:** Uses `sh`
- **Windows:** Uses windows default shell

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
    "path": "/home",
    "min_free_gb": 5
  }
}
```

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
    "size_threshold": 1024 // Optional: Bytes for "size_changed"
  }
}
```

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

**Use Cases:**

- Wait for a file to appear
- React to file changes
- Trigger on log activity

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

- All fields are **optional**, but **at least one** must be specified
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
    "operator": "and", // Required: "and", "or", or "nor"
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
    "operator": "or",
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
    "operator": "and",
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
    "operator": "nor",
    "conditions": [
      { "type": "process", "condition": { "process_name": "chrome" } },
      { "type": "process", "condition": { "process_name": "firefox" } }
    ]
  }
}
```

**Available Operators:**

- `and`: All conditions must be true
- `or`: At least one condition must be true  
- `nor`: None of the conditions must be true (NOT OR)

**Platform Support:** All platforms

**Use Cases:**

- Run tasks in multiple network environments (OR)
- Complex multi-factor conditions (AND)
- Negated condition combinations (NOR)
- Fallback scenarios (OR)
- Nested condition logic
- Reusable condition groups



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
- **`"type": "or"` condition:** OR logic (at least one must pass)
- **`"type": "and"` condition:** AND logic (all must pass, useful for nesting)

**Example combining both:**

```jsonc
{
  "conditions": [
    {
      "type": "or",
      "condition": {
        "conditions": [
          { "type": "wifi", "condition": { "ssid": "HomeNetwork" } },
          { "type": "wifi", "condition": { "ssid": "OfficeNetwork" } }
        ]
      }
    },
    {
      "type": "power",
      "condition": { "check_charging": true }
    }
  ]
}
```

This runs if:
- (Home WiFi **OR** Office WiFi) **AND** Charging

---

## Tips & Best Practices

### Performance

- **Avoid expensive commands** in conditions (they run before each task)
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

- **Enable verbose logging** with `--verbose` flag
- **Check logs** in your configured log directory
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
  "type": "or",
  "condition": {
    "conditions": [
      { "type": "wifi", "condition": { "ssid": "HomeNetwork" } },
      { "type": "wifi", "condition": { "ssid": "OfficeNetwork" } }
    ]
  }
}
```

### Only When at Home

```jsonc
[
  {
    "type": "wifi",
    "condition": { "ssid": "HomeNetwork" }
  },
  {
    "type": "bluetooth",
    "condition": { "device": "HomeHub" }
  }
]
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

If a condition fails to parse or execute:

- The job **will not run** (safe default)
- An error is logged with details
- Execution continues to the next job

Example logs:

```
Error parsing condition: Invalid command syntax
Error executing condition command 'bad syntax': (error details)
```

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
| or             | `conditions` (array of conditions)               | All      |
| and            | `conditions` (array of conditions)               | All      |

---

## See Also

- [Example Jobs](../templates/)
