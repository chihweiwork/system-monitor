# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust rewrite project that aims to integrate functionality from five established Linux monitoring tools:

- **btop** (C++23) - System resource monitor (CPU, memory, disk, network)
- **atop** (C) - Process and system activity monitor with logging capabilities
- **iotop** (C) - I/O usage monitor
- **nvtop** (C) - GPU monitor supporting NVIDIA, AMD, Intel, and other vendors
- **iftop** (C) - Network bandwidth monitor

**Goal**: Create a unified, efficient monitoring tool in Rust that combines the best features of all five tools.

## Project Structure

```
system-monitor/
├── atop/        # Reference implementation: process/system activity monitoring
├── btop/        # Reference implementation: system resource monitoring UI
├── iotop/       # Reference implementation: I/O monitoring
├── nvtop/       # Reference implementation: GPU monitoring
├── iftop/       # Reference implementation: network monitoring
└── (future Rust implementation to be added here)
```

## Using GitNexus for Code Intelligence

All five reference implementations are indexed by GitNexus with complete knowledge graphs:

| Project | Symbols | Relationships | Execution Flows |
|---------|---------|---------------|-----------------|
| btop    | 7,720   | 14,403        | 300             |
| atop    | 7,837   | 13,502        | 140             |
| iotop   | 778     | 1,319         | 30              |
| nvtop   | 3,164   | 5,286         | 122             |
| iftop   | 936     | 1,540         | 78              |

### Understanding Reference Implementations

When studying how a feature works in any of the reference codebases:

1. **Query for concepts**: `gitnexus query "<concept>" --repo <project-name>`
   - Example: `gitnexus query "cpu usage collection" --repo btop`

2. **Get symbol context**: `gitnexus context <symbol> --repo <project-name>`
   - Shows callers, callees, and execution flows for a function/class

3. **Trace execution flows**: Check `gitnexus://repo/<project>/processes`
   - Understand end-to-end data flow through the application

4. **View architecture**: `gitnexus://repo/<project>/clusters`
   - See functional groupings and module boundaries

### GitNexus Safety Protocol

**Before examining code in any reference project:**

- Run `gitnexus status` in the project directory to verify index freshness
- If stale, run `gitnexus analyze <project-directory>` to refresh

**When exploring a feature:**

- Start with `query` to find relevant execution flows
- Use `context` to understand individual symbols
- Check cluster groupings to understand architectural boundaries

## Development Workflow (Once Rust Implementation Begins)

### Initial Setup

```bash
# Initialize Rust project (when ready)
cargo init --name system-monitor

# Build
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Architecture Guidelines

When implementing features from the reference codebases:

1. **Study first, then implement**: Use GitNexus to understand the algorithm and data structures in the original implementation before translating to Rust

2. **Modular design**: Separate concerns into distinct modules:
   - `core/` - Shared monitoring infrastructure
   - `collectors/` - Data collection (CPU, memory, disk, network, GPU, I/O)
   - `ui/` - Terminal UI rendering
   - `storage/` - Data persistence and logging (inspired by atop)
   - `gpu/` - GPU-specific monitoring (multi-vendor support from nvtop)

3. **Cross-platform considerations**: 
   - btop supports Linux, macOS, *BSD - study its platform abstraction layer
   - Start with Linux support, design with portability in mind

4. **Performance**: These are monitoring tools that must have minimal overhead
   - Study btop's update intervals and optimization techniques
   - Use async Rust for concurrent data collection

### Key Features to Port

From the reference implementations, prioritize:

- **btop**: Modern TUI, efficient resource monitoring, theming
- **atop**: Historical logging, comprehensive process tracking
- **iotop**: Per-process I/O statistics
- **nvtop**: Multi-vendor GPU support architecture
- **iftop**: Network flow tracking

## GitNexus MCP Tools Reference

If any GitNexus MCP tool is needed, available tools are documented in:
- `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` - Complete tools reference
- `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` - Architecture exploration
- `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` - Dependency analysis

## Commands

### Reference Projects

Each reference project has its own build system. See individual README files:
- `atop/README.md`
- `btop/README.md`
- `iotop/README.md`
- `nvtop/README.markdown`
- `iftop/README`

### GitNexus Operations

```bash
# List all indexed repositories
gitnexus list

# Query across all projects
gitnexus query "memory allocation" --repo btop
gitnexus query "gpu utilization" --repo nvtop

# Analyze changes in reference code
cd <project> && gitnexus detect-changes

# Generate wiki for a reference project
gitnexus wiki <project-directory>
```

## Notes

- This is currently a reference collection phase - no Rust code exists yet
- All reference projects are indexed and ready for analysis
- Use GitNexus extensively to understand implementation patterns before writing Rust code
- Each reference project has its own CLAUDE.md with project-specific GitNexus guidance
