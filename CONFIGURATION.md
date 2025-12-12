# DeckRemote Configuration Example

## Windows Service

The Windows service doesn't require a configuration file. However, you can customize the behavior by editing the source code:

### Changing the Port

Edit `windows-service/src/main.rs` and change:
```rust
let listener = tokio::net::TcpListener::bind("0.0.0.0:7394")
```

### Adding More Keys to Block

Edit the `keyboard_hook_proc` function in `windows-service/src/main.rs` and add more key mappings:

```rust
let key_name = match kb_struct.vkCode {
    0x5B => "LWIN",  // Left Windows key
    0x5C => "RWIN",  // Right Windows key
    0x5D => "APPS",  // Applications key
    0x70 => "F1",    // F1 key
    0x71 => "F2",    // F2 key
    // Add more as needed
    _ => "",
};
```

Common Virtual Key Codes:
- 0x70-0x87: F1-F24
- 0x5B: Left Windows
- 0x5C: Right Windows
- 0x5D: Applications/Menu
- 0x2C: Print Screen
- 0x91: Scroll Lock
- 0x13: Pause

## Stream Deck Plugin

The Stream Deck plugin is configured through the Property Inspector UI. Default settings:

- **Server URL**: `http://192.168.1.100:7394`
- **Key**: `LWIN` (Left Windows Key)

### Finding Your Windows PC IP Address

On Windows, open Command Prompt and run:
```cmd
ipconfig
```

Look for "IPv4 Address" under your active network adapter.

### Testing the Connection

You can test if the service is accessible using curl:

```bash
# Test from macOS or Linux
curl http://192.168.1.100:7394/state

# Test from Windows PowerShell
Invoke-WebRequest -Uri "http://localhost:7394/state"
```

Expected response:
```json
{"blockedKeys":[]}
```

### Firewall Configuration

If you can't connect, add a firewall rule on Windows:

```cmd
netsh advfirewall firewall add rule name="DeckRemote" dir=in action=allow protocol=TCP localport=7394
```

Or through Windows Defender Firewall with Advanced Security GUI:
1. Open "Windows Defender Firewall with Advanced Security"
2. Click "Inbound Rules" → "New Rule"
3. Select "Port" → Next
4. Select "TCP" and enter "7394" → Next
5. Select "Allow the connection" → Next
6. Select all profiles → Next
7. Name it "DeckRemote" → Finish
