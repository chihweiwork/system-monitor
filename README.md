# System Monitor

[![Release](https://img.shields.io/github/v/release/chihweiwork/system-monitor)](https://github.com/chihweiwork/system-monitor/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux-lightgrey.svg)](https://www.kernel.org/)

A unified, modern system monitoring tool written in Rust, integrating the best features from **btop**, **atop**, **iotop**, **nvtop**, and **iftop**.

---

## ✨ Features

### 📊 Comprehensive Monitoring
- **CPU**: Per-core statistics with Total%, User%, System%, I/O Wait%, IRQ%
- **Memory**: System memory usage with process-level breakdown
- **Processes**: Detailed process list with filtering and sorting
- **Network**: Network activity tracking by process
- **Disk I/O**: Real-time I/O read/write statistics per process
- **Disk Usage**: Filesystem usage for all mount points
- **GPU**: Multi-vendor GPU monitoring (NVIDIA, AMD, Intel)

### 🎨 Modern TUI Interface
- **Unified Popup Windows**: Press `d` on any panel for detailed 80% screen view
- **Search & Filter**: Real-time search with `/` key in all popups
- **Multi-field Sorting**: Press `s` to cycle sort fields, `r` to reverse order
- **Color-coded Stats**: High/medium/low values with intuitive color schemes
- **Responsive Layout**: Adapts to terminal size automatically

### 🔍 Popup Window Features
Every panel supports detailed popup views with:
- **Scrollable Lists**: j/k, arrow keys, PageUp/PageDown, Home/End navigation
- **Instant Search**: `/` to filter by name, user, device, or other fields
- **Flexible Sorting**: Multiple sort fields with ascending/descending order
- **Consistent UX**: Same keyboard shortcuts across all panels

---

## 📦 Installation

### Download Pre-built Binary (Linux x86_64)

**Choose your version** based on your operating system:

| Your System | Version to Download | Works On |
|-------------|---------------------|----------|
| 🔴 **Red Hat / CentOS / Rocky** | [**Static Build**](release/v0.1.0/redhat/) | RHEL 6/7/8/9, CentOS, Rocky, AlmaLinux (all versions) |
| 🔵 **Ubuntu / Debian** | [**Dynamic Build**](release/v0.1.0/debian/) | Ubuntu 22.04+, Debian 12+, RHEL 9+ |
| 🟢 **Not Sure?** | [**Static Build**](release/v0.1.0/redhat/) | Works everywhere (recommended) |

#### Quick Install

```bash
# For Red Hat / CentOS / Rocky / any Linux (static build)
wget https://github.com/chihweiwork/system-monitor/releases/download/v0.1.0/system-monitor-v0.1.0-linux-x86_64-static.tar.gz
tar -xzf system-monitor-v0.1.0-linux-x86_64-static.tar.gz
sudo mv system-monitor /usr/local/bin/
system-monitor

# For Ubuntu 22.04+ / Debian 12+ (dynamic build, smaller)
wget https://github.com/chihweiwork/system-monitor/releases/download/v0.1.0/system-monitor-v0.1.0-linux-x86_64.tar.gz
tar -xzf system-monitor-v0.1.0-linux-x86_64.tar.gz
sudo mv system-monitor /usr/local/bin/
system-monitor
```

📖 **Detailed installation guides** (with checksums, troubleshooting, and multiple install methods):
- [Red Hat / CentOS Installation Guide](release/v0.1.0/redhat/README.md)
- [Ubuntu / Debian Installation Guide](release/v0.1.0/debian/README.md)
- [Version Comparison & Selection Guide](release/v0.1.0/README.md)

### Build from Source

**Requirements**: Rust 1.70+

```bash
# Clone the repository
git clone https://github.com/chihweiwork/system-monitor.git
cd system-monitor

# Build release binary
cargo build --release

# Run
./target/release/system-monitor

# Optional: Install to system
sudo cp target/release/system-monitor /usr/local/bin/
```

---

## 🎮 Usage

### Quick Start

```bash
# Launch the monitor
system-monitor
```

### Keyboard Shortcuts

#### Global Controls
| Key | Action |
|-----|--------|
| `1` | Switch to CPU panel |
| `2` | Switch to Memory panel |
| `3` | Switch to Processes panel |
| `4` | Switch to Network panel |
| `5` | Switch to Disk I/O panel |
| `6` | Switch to Disk Usage panel |
| `7` | Switch to GPU panel |
| `?` | Show help screen |
| `q` or `ESC` | Quit (when not in popup) |

#### Popup Window Controls
| Key | Action |
|-----|--------|
| `d` | Open detail popup for current panel |
| `j` or `↓` | Scroll down one line |
| `k` or `↑` | Scroll up one line |
| `PageDown` | Scroll down one page |
| `PageUp` | Scroll up one page |
| `Home` | Jump to top |
| `End` | Jump to bottom |
| `/` | Enter search mode |
| `s` | Cycle to next sort field |
| `r` | Reverse sort order |
| `ESC` or `q` | Close popup |

#### Search Mode (in popups)
| Key | Action |
|-----|--------|
| `Any char` | Add to search filter |
| `Backspace` | Remove last character |
| `ESC` | Exit search mode |

---

## 📋 Panel Details

### 1. CPU Monitor
- **Main View**: Overall CPU usage with visual bar
- **Popup View**: Per-core breakdown showing:
  - Core ID
  - Total%, User%, System%, I/O Wait%, IRQ%, Idle%
  - Color-coded by usage intensity
- **Sort Options**: Core ID, Total%, User%, System%, I/O%, IRQ%

### 2. Memory Monitor
- **Main View**: Total memory usage, used/available
- **Popup View**: Process memory ranking with:
  - PID, User, Process Name
  - CPU%, Memory%, Memory Size (MB)
- **Sort Options**: PID, Name, User, Memory%, Size, CPU%

### 3. Processes Panel
- **Main View**: Full process list with filtering
- **Modal View**: Detailed process information (press Enter)
- **Features**: Filter by name, sort by CPU/Memory/PID/Name

### 4. Network Monitor
- **Main View**: Network activity summary
- **Popup View**: Process network activity ranking
- **Sort Options**: PID, Name, User, CPU%

### 5. Disk I/O Monitor
- **Main View**: Total I/O read/write rates
- **Popup View**: Per-process I/O statistics:
  - Read MB/s, Write MB/s, Total I/O
- **Sort Options**: PID, Name, User, I/O Read, I/O Write, I/O Total

### 6. Disk Usage Monitor
- **Main View**: Disk usage bars for all mount points
- **Popup View**: Detailed filesystem information:
  - Mount Point, Device, Filesystem Type
  - Usage%, Used GB, Available GB
- **Sort Options**: Mount Point, Usage%, Used GB, Available GB, FS Type

### 7. GPU Monitor
- **Main View**: GPU utilization summary
- **Popup View**: Detailed GPU statistics:
  - GPU ID, Name, Vendor
  - Utilization%, VRAM Usage, Temperature (°C), Power (W)
- **Sort Options**: GPU ID, Util%, VRAM%, Temp, Power
- **Note**: Requires vendor-specific drivers (NVIDIA/AMD/Intel)

---

## 🏗️ Architecture

### Project Structure

```
system-monitor/
├── src/
│   ├── main.rs              # Application entry point and event loop
│   ├── lib.rs               # Library root
│   ├── collectors/          # Data collection modules
│   │   ├── cpu.rs           # CPU statistics from /proc/stat
│   │   ├── memory.rs        # Memory info from /proc/meminfo
│   │   ├── process.rs       # Process info from /proc/[pid]/
│   │   ├── disk.rs          # Disk usage via statvfs
│   │   ├── io.rs            # I/O stats from /proc/[pid]/io
│   │   └── network.rs       # Network stats from /proc/net/
│   ├── ui/                  # Terminal UI components
│   │   ├── theme.rs         # Color schemes and styling
│   │   ├── widgets.rs       # Panel widgets (CPU, Memory, etc.)
│   │   ├── layout.rs        # Multi-panel layout manager
│   │   ├── state.rs         # Application state management
│   │   └── detail_popup.rs  # Popup window renderer
│   ├── gpu/                 # GPU monitoring
│   │   ├── nvidia.rs        # NVIDIA GPU support (NVML)
│   │   ├── amd.rs           # AMD GPU support (ROCm)
│   │   └── intel.rs         # Intel GPU support
│   └── core/                # Core utilities
│       ├── error.rs         # Error handling
│       ├── config.rs        # Configuration
│       └── types.rs         # Common types
├── release/                 # Pre-built binaries
│   └── v0.1.0/
│       ├── redhat/          # Static builds (RHEL, CentOS, etc.)
│       └── debian/          # Dynamic builds (Ubuntu, Debian, etc.)
├── atop/                    # Reference: atop source code
├── btop/                    # Reference: btop source code
├── iotop/                   # Reference: iotop source code
├── nvtop/                   # Reference: nvtop source code
├── iftop/                   # Reference: iftop source code
├── Cargo.toml               # Rust dependencies
└── CLAUDE.md                # AI assistant guidelines
```

### Technology Stack

- **Language**: Rust 1.70+
- **TUI Framework**: [Ratatui](https://github.com/ratatui-org/ratatui)
- **Terminal Backend**: [Crossterm](https://github.com/crossterm-rs/crossterm)
- **Data Sources**: Linux `/proc` filesystem, `statvfs` syscall
- **GPU APIs**: NVML (NVIDIA), ROCm (AMD), Intel GPU tools

### Data Collection

All monitoring data is collected from Linux kernel interfaces:
- **CPU**: `/proc/stat`, `/proc/cpuinfo`
- **Memory**: `/proc/meminfo`, `/proc/[pid]/status`
- **Processes**: `/proc/[pid]/{stat,status,cmdline,io}`
- **Disk I/O**: `/proc/[pid]/io`, `/proc/diskstats`
- **Network**: `/proc/net/{dev,tcp,udp}`
- **Filesystem**: `statvfs()` system call

---

## 🛠️ Development

### Prerequisites

- Rust 1.70 or higher
- Linux kernel 3.2.0+
- Git

### Building for Development

```bash
# Clone the repository
git clone https://github.com/chihweiwork/system-monitor.git
cd system-monitor

# Run in debug mode (with logging)
RUST_LOG=debug cargo run

# Run tests
cargo test

# Check code style
cargo clippy

# Format code
cargo fmt
```

### Running Examples

```bash
# Test disk monitoring
cargo run --example test_disk

# Test I/O monitoring
cargo run --example test_io
```

### Code Intelligence with GitNexus

This project uses [GitNexus](https://github.com/gitnexus/gitnexus) for code intelligence:

```bash
# List indexed repositories
gitnexus list

# Search for functions in btop
gitnexus cypher "MATCH (n:Function) WHERE n.name CONTAINS 'cpu' RETURN n.name, n.filePath LIMIT 10" -r btop

# Get context for a symbol
gitnexus context "collect" -r btop

# Impact analysis before changes
gitnexus impact "function_name" -r atop
```

See `CLAUDE.md` for detailed GitNexus usage examples.

---

## 📚 Reference Implementations

This repository includes source code from five established monitoring tools for reference and study:

| Tool | Language | Purpose | Stars |
|------|----------|---------|-------|
| [btop](https://github.com/aristocratos/btop) | C++23 | Modern resource monitor | 20k+ |
| [atop](https://github.com/Atoptool/atop) | C | System & process monitor | 800+ |
| [iotop](https://github.com/Tomas-M/iotop) | C | I/O usage monitor | 600+ |
| [nvtop](https://github.com/Syllo/nvtop) | C | GPU monitor (multi-vendor) | 8k+ |
| [iftop](https://code.blinkace.com/pdw/iftop) | C | Network bandwidth monitor | - |

All reference codebases are indexed with GitNexus for code exploration and learning.

**Note**: These are included for development reference only. The Rust implementation is original work inspired by their designs.

---

## 🐛 Known Limitations

- **Linux Only**: Currently supports Linux only (cross-platform support planned)
- **GPU Support**: Requires vendor-specific drivers and libraries
  - NVIDIA: `libnvidia-ml.so` (NVML)
  - AMD: ROCm runtime
  - Intel: Intel GPU tools
- **Network Process Tracking**: Uses CPU usage as proxy metric (kernel limitations)
- **Root Required**: Some I/O statistics require root or `/proc` read permissions

---

## 🔮 Roadmap

- [ ] macOS support (via `sysctl`, `top`, `vm_stat`)
- [ ] FreeBSD/OpenBSD support
- [ ] Process-level GPU usage tracking
- [ ] Historical data logging (inspired by atop)
- [ ] Configuration file support
- [ ] Custom color themes
- [ ] Export statistics to JSON/CSV
- [ ] Docker container monitoring
- [ ] Remote monitoring (client/server mode)
- [ ] Plugin system for custom panels

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

### How to Contribute

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust naming conventions and idioms
- Run `cargo fmt` and `cargo clippy` before committing
- Add tests for new functionality
- Update documentation for significant changes
- Keep commits atomic and well-described

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### Reference Code Licenses

The reference implementations in subdirectories maintain their original licenses:
- **btop**: Apache License 2.0
- **atop**: GPL v2
- **iotop**: GPL v2
- **nvtop**: GPL v3
- **iftop**: GPL v2

---

## 🙏 Acknowledgments

- **Inspiration**: btop, atop, iotop, nvtop, iftop developers
- **TUI Framework**: [Ratatui](https://github.com/ratatui-org/ratatui) team
- **Terminal Library**: [Crossterm](https://github.com/crossterm-rs/crossterm) contributors
- **Code Intelligence**: [GitNexus](https://github.com/gitnexus/gitnexus)
- **AI Assistant**: Built with [Claude Code](https://claude.ai/code)

---

## 📞 Contact

- **GitHub**: [@chihweiwork](https://github.com/chihweiwork)
- **Email**: chihweiwork@gmail.com
- **Issues**: [GitHub Issues](https://github.com/chihweiwork/system-monitor/issues)

---

## ⭐ Star History

If you find this project useful, please consider giving it a star! ⭐

---

**Made with ❤️ and Rust**
