# DeckRemote

Cross-network keyboard control system that allows a Stream Deck (running on macOS or any OS) to toggle key blocking on a Windows gaming PC.

## System Architecture

- **Windows Service (Rust)**: Single executable that runs in the system tray and provides an HTTP API on port 7394 for controlling keyboard blocking
- **Stream Deck Plugin (Node.js)**: Plugin for Elgato Stream Deck that communicates with the Windows service to toggle key blocking

## Features

- 🎮 Block Windows keys during gaming to prevent accidental interruptions
- 🔄 Toggle key blocking with a single Stream Deck button press
- 📊 Visual feedback showing current blocking state
- 🌐 Works across networks (Stream Deck on macOS can control Windows PC)
- 🎯 System tray integration on Windows
- ⚡ Low-level keyboard hook for reliable key blocking

## Windows Service

### Building

The Windows service is a Rust application that must be built on or for Windows:

```bash
cd windows-service
cargo build --release
```

The compiled executable will be at `target/release/deck-remote-service.exe`

### Running

Simply run the executable:

```bash
deck-remote-service.exe
```

The service will:
- Start an HTTP server on `http://0.0.0.0:7394`
- Install a low-level keyboard hook to block specified keys
- Add an icon to the system tray
- Print status messages to the console

### API Endpoints

#### GET /state
Returns the current state of blocked keys.

**Response:**
```json
{
  "blockedKeys": ["LWIN"]
}
```

#### POST /keys/toggle
Toggles the blocking state of a specific key.

**Request:**
```json
{
  "key": "LWIN"
}
```

**Response:**
```json
{
  "success": true,
  "blocked": true
}
```

### Supported Keys

- `LWIN` - Left Windows key (VK code 0x5B)
- `RWIN` - Right Windows key (VK code 0x5C)
- `APPS` - Applications/Menu key (VK code 0x5D)

## Stream Deck Plugin

### Building

The Stream Deck plugin is a Node.js application:

```bash
cd streamdeck-plugin
npm install
npm run build
```

This will create the plugin in `com.deckremote.keyblock.sdPlugin/`

### Installation

1. Build the plugin as described above
2. Double-click the `com.deckremote.keyblock.sdPlugin` folder or use Stream Deck's plugin installation process
3. The plugin will appear in the Stream Deck app under the "DeckRemote" category

### Configuration

When you add the "Toggle Key Block" action to your Stream Deck:

1. Click on the action to open the Property Inspector
2. Enter the **Windows PC Server URL** (e.g., `http://192.168.1.100:7394`)
3. Select the **Key to Block** from the dropdown:
   - Left Windows Key
   - Right Windows Key
   - Applications Key

### Usage

- **Press the button** to toggle the key blocking state
- **Visual feedback**: The button icon changes between:
  - Grey with X (unblocked)
  - Red with indicator (blocked)
- The plugin polls the server every 2 seconds to keep the state synchronized

## Network Setup

1. **Find your Windows PC's IP address**:
   ```cmd
   ipconfig
   ```
   Look for the IPv4 Address

2. **Ensure firewall allows port 7394**:
   ```cmd
   netsh advfirewall firewall add rule name="DeckRemote" dir=in action=allow protocol=TCP localport=7394
   ```

3. **Configure the Stream Deck plugin** with your Windows PC's IP address

## Development

### Windows Service Development

```bash
cd windows-service
cargo build
cargo run
```

### Stream Deck Plugin Development

```bash
cd streamdeck-plugin
npm install
npm run watch  # Watches for changes and rebuilds automatically
```

## Security Notes

- The service listens on all interfaces (`0.0.0.0`) by default. Consider restricting this to specific networks
- No authentication is implemented. Use on trusted networks only
- The keyboard hook only blocks non-injected events to avoid blocking legitimate programmatic input

## Troubleshooting

### Windows Service won't start
- Make sure you're running on Windows
- Check if port 7394 is already in use: `netstat -an | findstr 7394`
- Run as Administrator if the keyboard hook fails to install

### Stream Deck can't connect
- Verify the Windows service is running
- Check the IP address and port in the plugin settings
- Ensure firewall allows connections on port 7394
- Test connectivity: `curl http://<windows-pc-ip>:7394/state`

### Keys aren't being blocked
- The keyboard hook requires the service to have proper permissions
- Try running the service as Administrator
- Check that the correct key name is configured (LWIN, RWIN, or APPS)

## License

MIT