# Troubleshooting Conditions

## Common Issues & Solutions

### When Property Not Triggering

**Issue:** Scheduled time passes but job doesn't run

**Debug steps:**
1. Verify time format: `HH:MM:SS` in 24-hour format (e.g., `09:00:00`)
2. Verify date format: `YYYY/MM/DD` (e.g., `2026/02/03`) if specified
3. Check system time: `date`
4. Increase tolerance if system time drifts: `"tolerance_seconds": 60`
5. Remember that conditions AND time must both be satisfied

**Example fix:**
```jsonc
{
  "when": {
    "time": "14:30:00",
    "tolerance_seconds": 60  // Increase from 30 to 60
  }
}
```

### WiFi Condition Not Detecting Network

**Linux Issues:**

1. **No nmcli installed**
   ```bash
   # Install NetworkManager
   sudo apt install network-manager  # Debian/Ubuntu
   sudo dnf install NetworkManager   # Fedora
   ```

2. **nmcli not finding network**
   ```bash
   # Debug: Check connected network
   nmcli -t -f active,ssid dev wifi
   
   # Fallback: Check with iwgetid
   iwgetid -r
   ```

3. **iwgetid not installed**
   ```bash
   sudo apt install wireless-tools  # Debian/Ubuntu
   ```

**macOS Issues:**

1. **airport command not working**
   ```bash
   # Ensure it's in PATH
   ls /System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport
   ```

**Windows Issues:**

1. **PowerShell not working**
   ```bash
   # Try running as Administrator
   # Or check PowerShell version: $PSVersionTable
   ```

---

### Bluetooth Condition Not Detecting Device

**Linux Issues:**

1. **bluetoothctl not installed**
   ```bash
   sudo apt install bluez  # Debian/Ubuntu
   sudo dnf install bluez  # Fedora
   ```

2. **Bluetooth daemon not running**
   ```bash
   sudo systemctl start bluetooth
   sudo systemctl enable bluetooth
   ```

3. **Device not showing as connected**
   ```bash
   # Debug: List connected devices
   bluetoothctl devices Connected
   
   # Try pairing first
   bluetoothctl pair <MAC_ADDRESS>
   bluetoothctl connect <MAC_ADDRESS>
   ```

**Finding Device MAC:**
```bash
# Get all paired devices with MAC
bluetoothctl devices

# Example output:
# Device AA:BB:CC:DD:EE:FF My Headphones
```

**macOS Issues:**

1. **Framework detection not working**
   - Device must be previously paired
   - Try checking System Preferences > Bluetooth

**Windows Issues:**

1. **PowerShell Get-CimInstance failing**
   - May need administrator privileges
   - Try in PowerShell (Admin)

---

### Custom Condition Not Executing

**Issue:** Command works manually but fails in condition

**Debug:**

1. **Test command in shell first**
   ```bash
   # Try the exact command from your condition
   test -f /tmp/myfile && echo "Success" || echo "Failed"
   ```

2. **Check exit code**
   ```bash
   command_here
   echo $?  # Should be 0 for success, 1+ for failure
   ```

3. **Shell differences**
   - Linux: Uses `sh` (POSIX)
   - macOS: Uses `zsh`
   - Windows: Uses PowerShell

4. **File paths**
   - Use absolute paths: `/tmp/file` (not `~/file`)
   - Escape special characters
   - Check file permissions

**Example with debugging:**
```jsonc
{
  "type": "custom",
  "condition": {
    "command": "test -f /tmp/myfile && echo 'File exists' || echo 'File not found'",
    "check_exit_code": false,
    "target_output": "File exists"  // Verify exact output
  }
}
```

---

### Output Condition Output Mismatch

**Issue:** Command output doesn't match target

**Debug:**

1. **Check exact output (including whitespace)**
   ```bash
   # Get output with hexdump to see hidden characters
   command_here | od -c
   ```

2. **Trim whitespace**
   - The condition automatically trims leading/trailing spaces
   - But not whitespace in the middle

3. **Newlines**
   - Commands often add trailing newlines
   - Condition automatically strips them

**Example with careful matching:**
```jsonc
{
  "type": "output",
  "condition": {
    "command": "date +%A",
    "target": "Monday"  // No whitespace, just the day name
  }
}
```

---

### "Condition Check Failed" Error

**Common causes:**

1. **Parsing error** - Invalid JSON/JSONC
   - Check brackets, quotes, commas
   - Validate with: `cat file.jsonc | jq .`

2. **Type mismatch** - Wrong `type` value
   - Valid: `"type": "wifi"` (lowercase)
   - Invalid: `"type": "WiFi"` (uppercase)

3. **Missing required fields**
   - WiFi: needs `ssid`
   - Bluetooth: needs `device`
   - Custom: needs `command`
   - Output: needs `command` and `target`

4. **Schema validation failed**
   - Check format carefully against examples
   - All required fields present

---

### Multiple Conditions Not Working (AND Logic)

**Issue:** Some conditions pass but job doesn't run

**Remember:** ALL conditions must be true, AND if a "when" property is specified, the time requirement must also be satisfied

**Example that WON'T work (only 1 condition true):**
```jsonc
{
  "when": {
    "time": "18:00:00"
  },
  "conditions": [
    { "type": "wifi", "condition": { "ssid": "HomeNetwork" } },    // ✅ True
    { "type": "bluetooth", "condition": { "device": "MyPhone" } }  // ❌ False
  ]
}
// Job WON'T run because bluetooth condition is false (even if time is correct)
```

**Solution:** Only include conditions that are actually relevant, or use Custom conditions for OR logic:
```jsonc
{
  "conditions": [
    {
      "type": "custom",
      "condition": {
        "command": "test $(date +%H) -ge 09 && test $(date +%H) -lt 17",  // Between 9 AM and 5 PM
        "check_exit_code": true
      }
    }
  ]
}
```

---

## Logs & Debugging

### Check Logs

```bash
# Find log directory
cat ~/.autopilot/logs  # Default location (varies)

# Watch logs in real-time
tail -f ~/.autopilot/logs/autopilot.log

# Grep for condition errors
grep -i "condition" ~/.autopilot/logs/autopilot.log
```

### Enable Verbose Logging

```bash
autopilot-rs serve --verbose
```

This will print:
- Condition check results
- Command output for custom conditions
- Parsing errors
- Execution flow

---

## Testing Conditions Manually

### Test a WiFi Condition

**Linux:**
```bash
# Check connected SSID
nmcli -t -f active,ssid dev wifi
# or
iwgetid -r
```

**macOS:**
```bash
/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport -I | grep SSID
```

**Windows:**
```powershell
netsh wlan show interfaces
```

### Test a Bluetooth Condition

**Linux:**
```bash
# List connected devices
bluetoothctl devices Connected

# Check specific device
bluetoothctl info AA:BB:CC:DD:EE:FF
```

### Test a Custom Condition

```bash
# Just run the command
your_command_here

# Check exit code
echo $?  # 0 = success, 1+ = failure

# Check output
your_command_here | cat -A  # Shows hidden characters
```

---

## Performance Issues

### Slow Condition Checks

**Problem:** Conditions take too long to execute

**Solutions:**

1. **Cache results** - Don't check the same thing every few seconds
   ```jsonc
   // Instead of checking every minute, check every 5 minutes
   // in cron/scheduler, not in conditions
   ```

2. **Simplify commands**
   ```bash
   # Slow: Parsing complex output
   ps aux | grep firefox | wc -l
   
   # Fast: Direct process check
   pgrep firefox
   ```

3. **Use specific tools**
   ```bash
   # Slow: Shell variable parsing
   echo $MYVAR
   
   # Fast: Environment variable in condition
   "type": "variable", "condition": {"variable": "MYVAR", "target": "value"}
   ```

---

## Platform-Specific Help

### Linux-Specific

```bash
# List available tools
which nmcli iwgetid bluetoothctl

# NetworkManager status
systemctl status network-manager

# Bluetooth daemon status
systemctl status bluetooth
```

### macOS-Specific

```bash
# WiFi info
/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport -I

# Check Bluetooth
system_profiler SPBluetoothDataType
```

### Windows-Specific

```powershell
# WiFi networks
netsh wlan show networks

# Connected WiFi
netsh wlan show interfaces

# Bluetooth devices
Get-CimInstance -ClassName Win32_PnPDevice | Where-Object {$_.Name -like '*Bluetooth*'}
```

---

## Getting Help

If you're stuck:

1. **Check the logs** - Most issues are logged
2. **Test manually** - Run the command outside autopilot-rs
3. **Verify format** - Validate JSON/JSONC syntax
4. **Check examples** - See `templates/example-*.jsonc`
5. **Read docs** - See `docs/CONDITIONS.md`

---

## Common Success Examples

### This WILL Work

```jsonc
{
  "type": "wifi",
  "condition": {
    "ssid": "MyNetwork"
  }
}
```

```jsonc
{
  "type": "bluetooth",
  "condition": {
    "device": "MyHeadphones"
  }
}
```

```jsonc
{
  "type": "custom",
  "condition": {
    "command": "pgrep firefox",
    "check_exit_code": true
  }
}
```

### This WON'T Work

```jsonc
// ❌ Wrong: Missing required fields
{ "type": "wifi" }

// ❌ Wrong: Uppercase type
{ "type": "WiFi", "condition": {...} }

// ❌ Wrong: No command in custom
{ "type": "custom", "condition": { "check_exit_code": true } }
```

---

**Still stuck?** Check `CONDITIONS.md` for the full guide, or review your job configuration against the examples.