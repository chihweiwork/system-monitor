# System Monitor v0.2.0 - GPU Monitoring Enhancements

**Release Date**: 2026-06-10

## 🎉 What's New

### GPU Monitoring Features
- **Full NVIDIA GPU support** with NVML integration
- **AMD & Intel GPU monitoring** via sysfs
- **GPU Process tracking** - see which processes use GPU
- **Multi-vendor support** - NVIDIA, AMD, and Intel in one tool

### Process View Enhancements
- **New GPU columns** in process detail view (GPU MB, GPU%)
- **Sort by GPU usage** - memory or utilization
- **Color-coded indicators** for quick identification
- **Default GPU sorting** - GPU-heavy processes shown first

## 📦 Available Builds

### Debian/Ubuntu (Dynamic)
**File**: `debian/system-monitor-v0.2.0-linux-x86_64`

**Compatible with**:
- Ubuntu 22.04+ (Jammy, Noble)
- Debian 12+ (Bookworm)
- RHEL 9+ / Rocky Linux 9+ / AlmaLinux 9+

**Requirements**:
- glibc 2.35+
- NVIDIA drivers (if using NVIDIA GPU)

```bash
chmod +x debian/system-monitor-v0.2.0-linux-x86_64
./debian/system-monitor-v0.2.0-linux-x86_64
```

### Red Hat/CentOS (Static)
**File**: `redhat/system-monitor-v0.2.0-linux-x86_64-static`

**Compatible with**:
- Red Hat Enterprise Linux (all versions)
- CentOS 7/8/Stream
- Rocky Linux (all versions)
- AlmaLinux (all versions)
- Oracle Linux (all versions)

**Features**:
- Fully static build (no glibc dependency)
- Works on older systems
- Slightly larger binary

```bash
chmod +x redhat/system-monitor-v0.2.0-linux-x86_64-static
./redhat/system-monitor-v0.2.0-linux-x86_64-static
```

## 🚀 Quick Start

### Running the Monitor
```bash
./system-monitor-v0.2.0-linux-x86_64
```

### GPU-Specific Features

**View GPU Overview**:
- Press `g` to see all GPUs

**View GPU Processes**:
- Press `p` for process detail view
- GPU columns show memory and utilization
- Press `s` to cycle sort fields (includes GPU MB and GPU%)

**Keyboard Shortcuts**:
- `p` - Process detail (with GPU columns!)
- `g` - GPU overview
- `s` - Cycle sort field
- `r` - Reverse sort order
- `q` / `ESC` - Quit / Close popup

## 🔧 Technical Details

### Build Information
- **Rust Version**: 1.83+
- **Target**: x86_64-unknown-linux-gnu
- **NVIDIA Support**: via nvml-wrapper 0.10
- **Build Date**: 2026-06-10

### Dependencies (Debian build)
- glibc 2.35+
- libnvidia-ml.so.1 (for NVIDIA GPU support)

### GPU Support Matrix

| Vendor | Method | Features |
|--------|--------|----------|
| NVIDIA | NVML | Full metrics, process tracking |
| AMD | sysfs | GPU usage, memory, processes |
| Intel | sysfs | GPU usage, memory, processes |

## 📝 Changelog

### New Features
- GPU columns in process detail view
- GPU memory and utilization sorting
- NVIDIA NVML integration for accurate metrics
- Multi-vendor GPU support (NVIDIA/AMD/Intel)
- GPU process type identification (Graphics/Compute/Both)

### Bug Fixes
- Fixed NVML `UsedGpuMemory` enum handling
- Enabled NVIDIA feature by default

### Improvements
- Color-coded GPU usage indicators
- Better process-GPU correlation
- Unified multi-vendor GPU architecture

## 🐛 Known Issues

- AMD/Intel GPU per-process utilization not available (driver limitation)
- Requires GPU driver installed for full functionality

## 📚 Documentation

- [Main README](../../README.md)
- [Full Changelog](../../CHANGELOG.md)
- [GitHub Repository](https://github.com/chihweiwork/system-monitor)

## 💬 Support

Found a bug? Have a feature request?
- [Open an issue](https://github.com/chihweiwork/system-monitor/issues)
- [View releases](https://github.com/chihweiwork/system-monitor/releases)
