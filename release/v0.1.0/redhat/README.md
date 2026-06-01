# Red Hat / CentOS / Rocky Linux Build

## Compatibility

✅ Red Hat Enterprise Linux 6/7/8/9  
✅ CentOS 6/7/8 Stream  
✅ Rocky Linux / AlmaLinux (all versions)  
✅ Fedora (all versions)  
✅ **Any Linux distribution** (static binary)

This build is **statically linked** with musl libc, meaning it has **zero dependencies** and will run on any Linux system.

---

## Installation

### Quick Install (System-wide)

```bash
# Download
wget https://github.com/chihweiwork/system-monitor/releases/download/v0.1.0/system-monitor-v0.1.0-linux-x86_64-static.tar.gz

# Verify checksum (recommended)
wget https://github.com/chihweiwork/system-monitor/releases/download/v0.1.0/system-monitor-v0.1.0-linux-x86_64-static.tar.gz.sha256
sha256sum -c system-monitor-v0.1.0-linux-x86_64-static.tar.gz.sha256

# Extract
tar -xzf system-monitor-v0.1.0-linux-x86_64-static.tar.gz

# Install to system
chmod +x system-monitor
sudo mv system-monitor /usr/local/bin/

# Run
system-monitor
```

### Local Installation (No sudo required)

```bash
# Download and extract
wget https://github.com/chihweiwork/system-monitor/releases/download/v0.1.0/system-monitor-v0.1.0-linux-x86_64-static.tar.gz
tar -xzf system-monitor-v0.1.0-linux-x86_64-static.tar.gz

# Run directly
./system-monitor
```

### Install to Custom Location

```bash
# Extract
tar -xzf system-monitor-v0.1.0-linux-x86_64-static.tar.gz

# Move to custom directory
mkdir -p ~/bin
mv system-monitor ~/bin/

# Add to PATH (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/bin:$PATH"

# Run
system-monitor
```

---

## Build Information

| Property | Value |
|----------|-------|
| **Type** | Static binary (musl libc) |
| **Dependencies** | None (fully self-contained) |
| **Size (compressed)** | 787 KB |
| **Size (uncompressed)** | 1.8 MB |
| **Linking** | static-pie |
| **Architecture** | x86_64 |
| **Minimum Kernel** | Linux 3.2.0+ |

---

## SHA256 Checksum

```
f919614e0a468eb87a0acbe34e098aff5d60cda6eaa5a123a99d9a82a57c1391  system-monitor-v0.1.0-linux-x86_64-static.tar.gz
```

### Verify Locally

If you already have the file:

```bash
echo "f919614e0a468eb87a0acbe34e098aff5d60cda6eaa5a123a99d9a82a57c1391  system-monitor-v0.1.0-linux-x86_64-static.tar.gz" | sha256sum -c
```

---

## Usage

```bash
# Launch the monitor
system-monitor

# Show help
system-monitor --help
```

### Keyboard Shortcuts

- `1-7` - Switch between panels (CPU, Memory, Processes, Network, DiskIO, DiskUsage, GPU)
- `d` - Open detail popup for current panel
- `j/k` or `↑/↓` - Scroll
- `/` - Search (in popups)
- `s` - Cycle sort field (in popups)
- `r` - Reverse sort order (in popups)
- `?` - Show help
- `q` or `ESC` - Quit

---

## Troubleshooting

### Permission Denied

```bash
chmod +x system-monitor
```

### Command Not Found (after installing to /usr/local/bin)

```bash
# Verify installation
ls -l /usr/local/bin/system-monitor

# Check PATH
echo $PATH | grep -o /usr/local/bin

# If not in PATH, add to ~/.bashrc
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### I/O Statistics Not Showing

Some I/O statistics require read permissions on `/proc/[pid]/io`. Run with sudo if needed:

```bash
sudo system-monitor
```

---

## Uninstall

```bash
# If installed to /usr/local/bin
sudo rm /usr/local/bin/system-monitor

# If installed to ~/bin
rm ~/bin/system-monitor
```

---

## More Information

- [Main README](../../../README.md)
- [Release Notes](../../../RELEASE_NOTES.md)
- [GitHub Repository](https://github.com/chihweiwork/system-monitor)
- [Report Issues](https://github.com/chihweiwork/system-monitor/issues)

---

**Why Static Build?**

This build uses musl libc instead of glibc, making it completely self-contained. It's perfect for:
- Older systems (RHEL 6/7/8, CentOS 7)
- Minimal/container environments
- Systems where you don't have sudo access to install dependencies
- Guaranteed compatibility across all Linux distributions
