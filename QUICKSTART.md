# Quick Start Guide

## Prerequisites

- **Windows PC**: Windows 10 or later
- **Stream Deck**: Any Elgato Stream Deck (can run on macOS, Windows, or Linux)
- **Network**: Both devices on the same network (or routable to each other)

## Step 1: Setup Windows Service

### Option A: Build from source (requires Rust)

1. Install Rust from https://rustup.rs/
2. Clone this repository
3. Build the service:
   ```cmd
   cd windows-service
   cargo build --release
   ```
4. The executable will be at `target/release/deck-remote-service.exe`

### Option B: Download pre-built executable (if available)

1. Download `deck-remote-service.exe` from releases
2. Place it in a folder of your choice (e.g., `C:\Program Files\DeckRemote\`)

### Running the Service

1. Double-click `deck-remote-service.exe` or run from command line
2. You should see: "DeckRemote service running on http://0.0.0.0:7394"
3. A system tray icon will appear (if supported)
4. **Optional**: Create a shortcut in the Startup folder to run automatically:
   - Press `Win+R` and type `shell:startup`
   - Create a shortcut to `deck-remote-service.exe`

### Configure Firewall

Open Command Prompt as Administrator and run:
```cmd
netsh advfirewall firewall add rule name="DeckRemote" dir=in action=allow protocol=TCP localport=7394
```

### Find Your PC's IP Address

In Command Prompt:
```cmd
ipconfig
```
Look for "IPv4 Address" (e.g., `192.168.1.100`)

## Step 2: Setup Stream Deck Plugin

### Build the Plugin

1. Install Node.js 20+ from https://nodejs.org/
2. Navigate to the plugin directory:
   ```bash
   cd streamdeck-plugin
   npm install
   npm run build
   ```

### Install to Stream Deck

1. Open the Stream Deck software
2. Drag the `com.deckremote.keyblock.sdPlugin` folder to the Stream Deck app window
   - Or use File → Install Plugin and select the folder

The plugin should now appear in the "DeckRemote" category.

## Step 3: Configure and Use

1. **Add the action** to your Stream Deck:
   - Find "Toggle Key Block" under the "DeckRemote" category
   - Drag it to a button on your Stream Deck

2. **Configure the action**:
   - Click the action in Stream Deck software
   - In the Property Inspector on the right:
     - Set "Windows PC Server URL" to `http://192.168.1.100:7394` (use your PC's IP)
     - Select "Left Windows Key" (or the key you want to block)

3. **Test it**:
   - Press the button on your Stream Deck
   - The icon should change to red (blocked)
   - Try pressing the Windows key on your PC - it should be blocked!
   - Press the Stream Deck button again to unblock

## Troubleshooting

### Button shows "❌" or doesn't respond

- Verify the Windows service is running
- Check the IP address in the plugin settings
- Test connectivity: `curl http://YOUR_PC_IP:7394/state`
- Verify firewall allows port 7394

### Windows key still works when blocked

- The service may need administrator privileges
- Right-click `deck-remote-service.exe` and "Run as administrator"
- Check the console for error messages

### Can't find the plugin in Stream Deck

- Make sure you built the plugin with `npm run build`
- Check that `com.deckremote.keyblock.sdPlugin/bin/plugin.js` exists
- Restart the Stream Deck software
- Check Stream Deck → Preferences → Plugins

## Tips

- **Gaming Setup**: Create a profile in Stream Deck for gaming with this button
- **Multiple Keys**: Add multiple buttons to block different keys (Win, Alt, etc.)
- **Profiles**: Use Stream Deck profiles to switch between different configurations
- **Auto-start**: Add the service to Windows Startup for automatic launching

## What's Next?

- Read [CONFIGURATION.md](CONFIGURATION.md) for advanced configuration
- See [README.md](README.md) for detailed documentation
- Check the API documentation for custom integrations
