# System Monitor v0.1.0 - Release Builds

Released: 2026-06-01

## Choose Your Build

### 🔴 Red Hat / CentOS / Rocky Linux
- **Location**: [redhat/](redhat/)
- **Works on**: RHEL 6/7/8/9, CentOS, Rocky, AlmaLinux (all versions)
- **Type**: Static binary (musl)
- **Size**: 787 KB
- **Compatibility**: ✅ Maximum (works on any Linux)

### 🔵 Ubuntu / Debian / Modern Distributions
- **Location**: [debian/](debian/)
- **Works on**: Ubuntu 22.04+, Debian 12+, RHEL 9+
- **Type**: Dynamic binary (requires GLIBC 2.34+)
- **Size**: 734 KB
- **Compatibility**: ⚠️ Modern systems only

## Not Sure Which to Choose?

```bash
# Check your GLIBC version
ldd --version | head -1
```

- **GLIBC 2.34 or newer** → Either build works, debian/ is slightly smaller
- **GLIBC 2.33 or older** → Use redhat/ (static build)
- **Don't know / Don't care** → Use redhat/ (always works)

## Installation

See subdirectory READMEs for detailed installation instructions:
- [Red Hat Installation Guide](redhat/README.md)
- [Debian/Ubuntu Installation Guide](debian/README.md)

## Build Comparison

| Feature | Red Hat Build | Debian Build |
|---------|--------------|--------------|
| **File** | `system-monitor-v0.1.0-linux-x86_64-static.tar.gz` | `system-monitor-v0.1.0-linux-x86_64.tar.gz` |
| **Size (compressed)** | 787 KB | 734 KB |
| **Size (binary)** | 1.8 MB | 1.8 MB |
| **Linking** | Static (musl) | Dynamic (glibc) |
| **Dependencies** | None | GLIBC 2.34+ |
| **RHEL 6/7** | ✅ Works | ❌ Too old |
| **RHEL 8** | ✅ Works | ❌ Too old |
| **RHEL 9** | ✅ Works | ✅ Works |
| **Ubuntu 20.04** | ✅ Works | ❌ Too old |
| **Ubuntu 22.04+** | ✅ Works | ✅ Works |
| **CentOS 7** | ✅ Works | ❌ Too old |

## Release Notes

See [RELEASE_NOTES.md](../../RELEASE_NOTES.md) for full changelog and features.
