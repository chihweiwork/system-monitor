# Ubuntu / Debian / Modern Distribution Build

## Compatibility

✅ Ubuntu 22.04 LTS (Jammy) or newer  
✅ Debian 12 (Bookworm) or newer  
✅ Red Hat Enterprise Linux 9 or newer  
✅ Fedora 36 or newer  
⚠️ **Requires GLIBC 2.34+**

---

## Check Compatibility First

```bash
# Check your GLIBC version
ldd --version | head -1

# Should show:
#   Ubuntu: (Ubuntu GLIBC 2.35-...) 2.35
#   Debian: (Debian GLIBC 2.36-...) 2.36
#   RHEL 9: (GNU libc) 2.34
```

**If your version is 2.33 or lower**, use the [Red Hat build](../redhat/) instead (works everywhere).

---

## Installation

### Quick Install (System-wide)

```bash
# Download
wget https://github.com/chihweiwork/system-monitor/releases/download/v0.1.0/system-monitor-v0.1.0-linux-x86_64.tar.gz

# Verify checksum (recommended)
wget https://github.com/chihweiwork/system-monitor/releases/download/v0.1.0/system-monitor-v0.1.0-linux-x86_64.tar.gz.sha256
sha256sum -c system-monitor-v0.1.0-linux-x86_64.tar.gz.sha256

# Extract
tar -xzf system-monitor-v0.1.0-linux-x86_64.tar.gz

# Install to system
chmod +x system-monitor
sudo mv system-monitor /usr/local/bin/

# Run
system-monitor
```

### Local Installation (No sudo required)

```bash
# Download and extract
wget https://github.com/chihweiwork/system-monitor/releases/download/v0.1.0/system-monitor-v0.1.0-linux-x86_64.tar.gz
tar -xzf system-monitor-v0.1.0-linux-x86_64.tar.gz

# Run directly
./system-monitor
```

### Install to Custom Location

```bash
# Extract
tar -xzf system-monitor-v0.1.0-linux-x86_64.tar.gz

# Move to custom directory
mkdir -p ~/.local/bin
mv system-monitor ~/.local/bin/

# Add to PATH (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.local/bin:$PATH"

# Run
system-monitor
```

---

## Build Information

| Property | Value |
|----------|-------|
| **Type** | Dynamic binary (glibc) |
| **Dependencies** | GLIBC 2.34+, libgcc_s, libm, libc |
| **Size (compressed)** | 734 KB |
| **Size (uncompressed)** | 1.8 MB |
| **Linking** | dynamic |
| **Architecture** | x86_64 |
| **Minimum Kernel** | Linux 3.2.0+ |

### Why Dynamic Build?

This build is slightly smaller (734 KB vs 787 KB) and uses your system's standard C library. It's optimized for modern systems.

---

## SHA256 Checksum

```
cabdf7a3314a8970b2f35ce2f5f5b4f50f1ecb035e5a80e57bc5f99c8a6efb29  system-monitor-v0.1.0-linux-x86_64.tar.gz
```

### Verify Locally

If you already have the file:

```bash
echo "cabdf7a3314a8970b2f35ce2f5f5b4f50f1ecb035e5a80e57bc5f99c8a6efb29  system-monitor-v0.1.0-linux-x86_64.tar.gz" | sha256sum -c
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

### Error: version 'GLIBC_2.34' not found

This means your system has an older GLIBC version. Solutions:

1. **Recommended**: Use the [Red Hat build](../redhat/) (static, works everywhere)
2. Upgrade your system to a newer version

```bash
# Check your current GLIBC version
ldd --version | head -1

# If 2.33 or lower, download the static build instead
wget https://github.com/chihweiwork/system-monitor/releases/download/v0.1.0/system-monitor-v0.1.0-linux-x86_64-static.tar.gz
```

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

# If installed to ~/.local/bin
rm ~/.local/bin/system-monitor
```

---

## Supported Distributions

| Distribution | Version | GLIBC | Status |
|--------------|---------|-------|--------|
| Ubuntu | 22.04 LTS (Jammy) | 2.35 | ✅ Works |
| Ubuntu | 24.04 LTS (Noble) | 2.39 | ✅ Works |
| Ubuntu | 20.04 LTS (Focal) | 2.31 | ❌ Use [static build](../redhat/) |
| Debian | 12 (Bookworm) | 2.36 | ✅ Works |
| Debian | 11 (Bullseye) | 2.31 | ❌ Use [static build](../redhat/) |
| RHEL | 9 | 2.34 | ✅ Works |
| RHEL | 8 | 2.28 | ❌ Use [static build](../redhat/) |
| Fedora | 36+ | 2.35+ | ✅ Works |

---

## More Information

- [Main README](../../../README.md)
- [Release Notes](../../../RELEASE_NOTES.md)
- [GitHub Repository](https://github.com/chihweiwork/system-monitor)
- [Report Issues](https://github.com/chihweiwork/system-monitor/issues)

---

**Older System?**

If you're on Ubuntu 20.04, Debian 11, or RHEL 8, use the [Red Hat build](../redhat/) instead — it works on all systems.
