# IUMenu Server/Client Mode

IUMenu now supports a server/client architecture to avoid reloading the application every time you need to show the launcher.

## How it works

- **Server mode**: The application stays running in the background with GTK initialized and ready
- **Client mode**: A lightweight client sends a toggle command to the server via Unix socket (< 10ms typical response time)
- The window hides/shows instantly without reloading desktop entries or reinitializing GTK
- Search text is automatically cleared when hiding the window

## Usage

### 1. Start the server (daemon mode)

Run the application with the `--server` flag to start it in daemon mode:

```bash
# With config file
./iumenu --config /path/to/config.toml --server

# Without config file (uses defaults)
./iumenu --server
```

The server will:

- Start in the background with the window hidden
- Listen for toggle commands on a Unix socket at `$XDG_RUNTIME_DIR/iumenu.sock`
- Stay running until you explicitly quit it

**Tip**: Add this to your window manager/desktop environment startup script to auto-start on login.

### 2. Toggle the window (client mode)

To show/hide the launcher, use the `--toggle` flag:

```bash
./iumenu --toggle
```

This will:

- Send a toggle command to the running server
- Show the window if it's hidden
- Hide the window if it's visible
- Exit immediately

**Tip**: Bind this command to your preferred keyboard shortcut in your window manager.

### 3. Regular mode (legacy)

You can still run IUMenu in the original mode without server/client:

```bash
./iumenu --config /path/to/config.toml
```

This will open the window normally and quit when you close it.

## Example Setup

### i3wm / Sway

Add to your config:

```
# Start IUMenu server on startup (config file optional)
exec --no-startup-id /path/to/iumenu --server

# Bind Super+D to toggle the launcher
bindsym $mod+d exec /path/to/iumenu --toggle
```

### Systemd User Service

Create `~/.config/systemd/user/iumenu.service`:

```ini
[Unit]
Description=IUMenu Application Launcher Server
After=graphical-session.target

[Service]
Type=simple
ExecStart=/path/to/iumenu --server
Restart=on-failure

[Install]
WantedBy=graphical-session.target
```

Enable and start:

```bash
systemctl --user enable iumenu.service
systemctl --user start iumenu.service
```

## Benefits

- ⚡ **Instant response** - Toggle response time < 10ms (with timeouts for reliability)
- 💾 **Lower resource usage** - Desktop entries are loaded once at startup
- 🎯 **Better UX** - Feels like a native component rather than a separate app
- 🧹 **Clean state** - Search text automatically clears when hiding
- 🛡️ **Robust** - Window close requests (Alt+F4, etc.) hide the window instead of closing the server

## Technical Details

- Uses Unix domain sockets for IPC (Inter-Process Communication)
- Socket location: `$XDG_RUNTIME_DIR/iumenu.sock` (typically `/run/user/1000/iumenu.sock`)
- Server runs on a separate Tokio runtime thread
- Toggle commands are forwarded to GTK main thread via glib async channels
- Client-side timeouts ensure fast failure detection (500ms connect, 200ms read/write)
- Window state is preserved between toggles
- Search text is cleared automatically when hiding for clean UX

## Optional Configuration

The `--config` flag is optional. If not provided, IUMenu uses default settings:

- Window size: 800x400
- Opacity: 1.0
- No custom CSS
