# Release v0.1.0 - Initial Release

First public release of the unified Rust system monitor integrating features from btop, atop, iotop, nvtop, and iftop.

## 🎯 Features

### Monitoring Panels
- **CPU Monitor**: Per-core statistics with Total%, User%, System%, I/O Wait%, IRQ%
- **Memory Monitor**: Process memory ranking with detailed statistics
- **Network Monitor**: Network activity tracking by process
- **Disk I/O Monitor**: Per-process I/O read/write statistics
- **Disk Usage Monitor**: Mount point details with usage%, filesystem type
- **GPU Monitor**: Multi-vendor GPU support (NVIDIA, AMD, Intel)

### Unified Popup Windows
All 6 panels feature consistent popup windows (press `d`):
- **80% screen centered display** for better readability
- **Scrollable lists** with j/k or arrow keys navigation
- **Search functionality** (press `/`) with real-time filtering
- **Multi-field sorting** (press `s` to cycle, `r` to reverse)
- **PageUp/PageDown, Home/End** navigation support

### User Interface
- Modern TUI built with Ratatui
- Theming support with color-coded statistics
- Process filtering and modal details
- Help screen (`?` key)
- Panel switching with number keys (1-7)

### Architecture
- **collectors/**: Real-time data collection from /proc filesystem
- **ui/**: Ratatui widgets, layouts, state management, detail popups
- **gpu/**: Multi-vendor GPU support framework
- **core/**: Error handling, configuration, utilities

## 📦 Installation

### Download Binary

**Two builds available:**

#### Static Binary (Recommended - Works Everywhere)

✅ **Works on all Linux distributions** including:
- Red Hat Enterprise Linux (RHEL) 6/7/8/9
- CentOS 6/7/8 Stream
- Rocky Linux / AlmaLinux
- Ubuntu (all versions)
- Debian (all versions)
- Any other Linux distribution

```bash
# Download static build
wget https://github.com/chihweiwork/system-monitor/releases/download/v0.1.0/system-monitor-v0.1.0-linux-x86_64-static.tar.gz

# Verify checksum
echo "f919614e0a468eb87a0acbe34e098aff5d60cda6eaa5a123a99d9a82a57c1391  system-monitor-v0.1.0-linux-x86_64-static.tar.gz" | sha256sum -c

# Extract and run
tar -xzf system-monitor-v0.1.0-linux-x86_64-static.tar.gz
./system-monitor
```

#### Dynamic Binary (Smaller, Modern Systems Only)

⚠️ **Requires GLIBC 2.34+** (Ubuntu 22.04+, RHEL 9+, Debian 12+)

```bash
# Download dynamic build
wget https://github.com/chihweiwork/system-monitor/releases/download/v0.1.0/system-monitor-v0.1.0-linux-x86_64.tar.gz

# Verify checksum
echo "cabdf7a3314a8970b2f35ce2f5f5b4f50f1ecb035e5a80e57bc5f99c8a6efb29  system-monitor-v0.1.0-linux-x86_64.tar.gz" | sha256sum -c

# Extract and run
tar -xzf system-monitor-v0.1.0-linux-x86_64.tar.gz
./system-monitor
```

### Build from Source

```bash
git clone https://github.com/chihweiwork/system-monitor.git
cd system-monitor
cargo build --release
./target/release/system-monitor
```

## 🎮 Usage

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `1-7` | Switch between panels |
| `d` | Open detail popup for current panel |
| `j/k` or `↑/↓` | Scroll up/down |
| `/` | Enter search mode (in popups) |
| `s` | Cycle sort field (in popups) |
| `r` | Reverse sort order (in popups) |
| `ESC` or `q` | Close popup / Exit program |
| `?` | Show help |

### Panel Overview

1. **CPU** - Core-level CPU statistics
2. **Memory** - System memory usage
3. **Processes** - Process list with filtering
4. **Network** - Network activity
5. **Disk I/O** - I/O operations
6. **Disk Usage** - Filesystem usage
7. **GPU** - GPU utilization

## 🔧 System Requirements

- **OS**: Linux (kernel 3.2.0+)
- **Architecture**: x86_64
- **Dependencies**: None (statically linked Rust binary)

## 📚 Reference Implementations

This project includes reference implementations for study:
- **btop** - Modern C++ system monitor
- **atop** - Advanced system & process monitor
- **iotop** - I/O usage monitor
- **nvtop** - Multi-vendor GPU monitor
- **iftop** - Network bandwidth monitor

All indexed with GitNexus for code intelligence.

## 🐛 Known Limitations

- GPU monitoring requires vendor-specific drivers
- Network process tracking uses CPU as proxy metric
- Currently Linux-only (cross-platform support planned)

## 📝 SHA256 Checksums

```
f919614e0a468eb87a0acbe34e098aff5d60cda6eaa5a123a99d9a82a57c1391  system-monitor-v0.1.0-linux-x86_64-static.tar.gz
cabdf7a3314a8970b2f35ce2f5f5b4f50f1ecb035e5a80e57bc5f99c8a6efb29  system-monitor-v0.1.0-linux-x86_64.tar.gz
```

## 📋 Binary Comparison

| Build Type | Size | GLIBC Required | Compatible With |
|------------|------|----------------|-----------------|
| **Static** | 787 KB | None (musl) | All Linux distributions (RHEL 6+, Ubuntu, Debian, etc.) |
| **Dynamic** | 734 KB | 2.34+ | Modern distributions only (Ubuntu 22.04+, RHEL 9+) |

**Recommendation**: Use the **static build** for maximum compatibility, especially on:
- Red Hat Enterprise Linux (any version)
- CentOS / Rocky Linux / AlmaLinux
- Production servers with older base systems

## 🙏 Credits

Built with:
- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal library

Inspired by btop, atop, iotop, nvtop, and iftop.

---

**Full Changelog**: https://github.com/chihweiwork/system-monitor/commits/v0.1.0
