# IUMENU

A simple, easy-to-use, GTK-based app launcher and menu written in Rust!

---

## Features

- **GTK-based**: Designed with GTK4 for a modern and responsive interface.
-  **Rust-based**: Powered by the Rust programming language for safety and performance.
- **Lightweight**: Minimal resource usage, focusing on performance.
- **Cross-platform**: Supports Linux, macOS, and Windows with platform-specific app discovery.

---

## Building

> **Note**: This project is under development, and this section may change in future updates.

### Requirements

#### Linux

Refer to the [GTK4 installation guide for Linux](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation_linux.html) for more details.

- **Fedora and derivatives**:
  ```sh
  sudo dnf install gtk4-devel gcc
  ```

- **Debian and derivatives**:
  ```sh
  sudo apt install libgtk-4-dev build-essential
  ```

- **Arch and derivatives**:
  ```sh
  sudo pacman -S gtk4 base-devel
  ```

#### macOS

Install GTK4 via Homebrew:
```sh
brew install gtk4
```

#### Windows

Setting up GTK4 on Windows requires additional steps. Follow the [GTK4 installation guide for Windows](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation_windows.html) for detailed instructions.

---

### Build and Install Using Cargo

1. Clone the repository:
   ```sh
   git clone https://github.com/igorunderplayer/iumenu.git
   ```

2. Navigate to the project directory:
   ```sh
   cd iumenu
   ```

3. Build and install the application:
   ```sh
   cargo install --path .
   ```

---

## Platform Support

IUMENU provides platform-specific application discovery:

### Linux
- Searches `.desktop` files in:
  - `/usr/share/applications` (system-wide applications)
  - `/usr/local/share/applications` (locally installed applications)
  - `~/.local/share/applications` (user-specific applications)

### Windows
- Discovers applications from:
  - Windows Registry uninstall entries (HKEY_LOCAL_MACHINE and HKEY_CURRENT_USER)
  - Start Menu shortcuts in `%ProgramData%\Microsoft\Windows\Start Menu\Programs`
  - User Start Menu shortcuts in `%APPDATA%\Microsoft\Windows\Start Menu\Programs`

### macOS
- Currently uses the Linux/FreeDesktop implementation
- macOS-specific app discovery is planned for future updates

---

## Usage

> **Note**: This section is incomplete and may not cover all available functionality. Updates are forthcoming

Once installed, you can run **IUMENU** by executing:
```sh
iumenu
iumenu --help # Show help menu
```

---

## Contributing

Contributions are welcome! If you'd like to help improve **IUMENU**, please:

1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Submit a pull request with a clear description of your changes.

---


## Acknowledgments

Special thanks to the [GTK4-rs](https://gtk-rs.org/) community for providing excellent documentation and tools for building GTK applications in Rust.

