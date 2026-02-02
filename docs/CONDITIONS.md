# AutoPilot-rs Conditions Guide

This guide explains all available condition types and how to use them in your automation jobs.

## Overview

Conditions are checks that must pass before a job's tasks are executed. A job with multiple conditions requires **ALL** conditions to be true before running.

## Condition Types

### 1. Time Condition

**Purpose:** Execute a task at a specific date and time.

**Schema:**
```jsonc
{
  "type": "time",
  "condition": {
    "date": "YYYY/MM/DD",      // Required: Target date
    "time": "HH:MM:SS",         // Required: Target time (24-hour format)
    "tolerance_seconds": 30     // Optional: Fuzzy matching window (in seconds)
  }
}
```

**Examples:**

*Exact time:*
```jsonc
{
  "type": "time",
  "condition": {
    "date": "2026/02/02",
    "time": "14:30:00"
  }
}
```

*With tolerance (runs within ±30 seconds of target time):*
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

**Notes:**
- Timezone uses your system's local timezone
- Useful for scheduled tasks (e.g., reminders, maintenance windows)
- `tolerance_seconds` is useful for reducing precision requirements

---

### 2. WiFi Condition

**Purpose:** Check if you're connected to a specific WiFi network.

**Schema:**
```jsonc
{
  "type": "wifi",
  "condition": {
    "ssid": "network_name"      // Required: WiFi network name (SSID)
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

### 3. Bluetooth Condition

**Purpose:** Check if a specific Bluetooth device is connected.

**Schema:**
```jsonc
{
  "type": "bluetooth",
  "condition": {
    "device": "device_name_or_mac",  // Required: Device name or MAC address
    "match_by_mac": false            // Optional: true to match by MAC, false for name (default: false)
  }
}
```

**Examples:**

*Match by device name:*
```jsonc
{
  "type": "bluetooth",
  "condition": {
    "device": "My Headphones",
    "match_by_mac": false
  }
}
```

*Match by MAC address:*
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

### 4. Custom Condition

**Purpose:** Execute arbitrary shell commands and check the result.

**Schema:**
```jsonc
{
  "type": "custom",
  "condition": {
    "command": "shell_command",       // Required: Command to execute
    "check_exit_code": true,          // Optional: Check exit code 0 (default: true)
    "target_output": "expected_text"  // Optional: Check if output matches this value
  }
}
```

**Examples:**

*Check exit code (success/failure):*
```jsonc
{
  "type": "custom",
  "condition": {
    "command": "test -f /tmp/my-file",
    "check_exit_code": true
  }
}
```

*Check command output:*
```jsonc
{
  "type": "custom",
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
- **macOS:** Uses `zsh`
- **Windows:** Uses PowerShell

---

### 5. Output Condition

**Purpose:** Execute a command and check if its output matches a target value.

**Schema:**
```jsonc
{
  "type": "output",
  "condition": {
    "command": "shell_command",  // Required: Command to execute
    "target": "expected_output"  // Required: Expected output
  }
}
```

**Example:**
```jsonc
{
  "type": "output",
  "condition": {
    "command": "date +%A",
    "target": "Monday"
  }
}
```

**Use Cases:**
- Run tasks on specific days of the week
- Check hostname or environment variables
- Parse log files or config values

---

### 6. Variable Condition

**Purpose:** Check if an environment variable matches an expected value.

**Schema:**
```jsonc
{
  "type": "variable",
  "condition": {
    "variable": "VAR_NAME",      // Required: Environment variable name
    "target": "expected_value"   // Required: Expected value
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

## Multiple Conditions (AND Logic)

When a job has multiple conditions, **all conditions must be true** for the job to execute:

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
    },
    {
      "type": "time",
      "condition": {
        "date": "2026/02/02",
        "time": "18:00:00",
        "tolerance_seconds": 300
      }
    }
  ],
  "tasks": [
    { "command": "notify-send 'Welcome home!'" }
  ]
}
```

In this example, the job runs **only if**:
- You're connected to "HomeNetwork" WiFi **AND**
- Your phone is connected via Bluetooth **AND**
- The current time is 6:00 PM (±5 minutes)

---

## Tips & Best Practices

### Performance
- **Avoid expensive commands** in custom conditions (they run before each task)
- **Use platform-native tools** (nmcli, bluetoothctl) instead of parsing complex output
- **Cache conditions** - if checking the same thing multiple times, consider cron jobs instead

### Reliability
- **Test commands manually** before using in conditions:
  ```bash
  command && echo "Success" || echo "Failed"
  ```
- **Handle errors gracefully** - failed conditions prevent task execution
- **Use `check_exit_code: true`** for simple success/failure checks

### Security
- **Avoid shell injection** - don't use user input in commands
- **Be careful with `target_output`** - can be slow for large outputs
- **Limit command scope** - use `test` command for file/directory checks instead of `ls`

### Debugging
- **Enable verbose logging** with `--verbose` flag
- **Check logs** in your configured log directory
- **Test conditions in isolation** using the shell manually

---

## Common Patterns

### Daily Task at Specific Time
```jsonc
{
  "type": "time",
  "condition": {
    "date": "2026/02/02",
    "time": "09:00:00",
    "tolerance_seconds": 60
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

### Only During Business Hours
```jsonc
{
  "type": "custom",
  "condition": {
    "command": "test $(date +%H) -ge 09 && test $(date +%H) -lt 17",
    "check_exit_code": true
  }
}
```

### Only on Weekdays
```jsonc
{
  "type": "custom",
  "condition": {
    "command": "test $(date +%u) -le 5",  // 1-5 = Mon-Fri
    "check_exit_code": true
  }
}
```

### Never on Holidays
```jsonc
{
  "type": "custom",
  "condition": {
    "command": "! grep -q \"$(date +%Y-%m-%d)\" ~/.holidays",
    "check_exit_code": true
  }
}
```

---

## Error Handling

If a condition fails to parse or execute:
- The job **will not run** (safe default)
- An error is logged with details
- Execution continues to the next job

Example logs:
```
Error parsing time condition: Invalid date format. Expected YYYY/MM/DD
Error executing custom condition command 'bad syntax': (error details)
```

---

## See Also

- [Job Configuration Guide](./JOBS.md)
- [Tasks Documentation](./TASKS.md)
- [Example Jobs](../templates/)
