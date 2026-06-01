# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## ⚡ Quick Reference: GitNexus Usage

**🚫 DO NOT USE**: `gitnexus query` - semantic search is broken (no embeddings)

**✅ USE INSTEAD**: `gitnexus cypher` - direct graph queries (works perfectly)

```bash
# Find functions by name pattern
gitnexus cypher "MATCH (n:Function) WHERE n.name CONTAINS 'cpu' RETURN n.name, n.filePath, n.startLine LIMIT 10" -r btop

# Get function context (callers/callees)
gitnexus context "function_name" -r btop

# Find all symbols in a file
gitnexus cypher "MATCH (n) WHERE n.filePath = 'src/file.cpp' RETURN n.name, labels(n)[0], n.startLine" -r btop
```

See detailed examples in the "Using GitNexus" section below.

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
| btop    | 7,739   | 14,413        | 300             |
| atop    | 7,837   | 13,502        | 140             |
| iotop   | 789     | 1,329         | 30              |
| nvtop   | 3,164   | 5,286         | 122             |
| iftop   | 936     | 1,540         | 78              |

### ⚠️ Important: Use Cypher Queries, NOT `gitnexus query`

**The `gitnexus query` semantic search does NOT work** because embeddings are not available (embeddings=0 for all repos).

**✅ Use `gitnexus cypher` instead** - it's more powerful and works perfectly:

```bash
# Find all functions with "cpu" in the name
gitnexus cypher "MATCH (n:Function) WHERE n.name CONTAINS 'cpu' OR n.name CONTAINS 'Cpu' RETURN n.name, n.filePath, n.startLine LIMIT 20" -r btop

# Find all functions in a specific file
gitnexus cypher "MATCH (n:Function) WHERE n.filePath = 'src/linux/btop_collect.cpp' RETURN n.name, n.startLine ORDER BY n.startLine" -r btop

# Find specific function (e.g., collect)
gitnexus cypher "MATCH (n:Function) WHERE n.name = 'collect' RETURN n.name, n.filePath, n.startLine" -r btop

# Find all symbols in a file (functions, structs, etc.)
gitnexus cypher "MATCH (n) WHERE n.filePath CONTAINS 'cpu' RETURN n.name, labels(n)[0] as type, n.filePath, n.startLine LIMIT 30" -r btop
```

**Available node types**: Function, Struct, Class, Method, Variable, Namespace
**Key properties**: `name`, `filePath`, `startLine`, `endLine`

### Understanding Reference Implementations

When studying how a feature works in any of the reference codebases:

1. **Search with Cypher**: Use the examples above to find relevant functions/symbols
   - Example: Find CPU collection logic in btop

2. **Get symbol context**: `gitnexus context <symbol> -r <project-name>`
   - Shows callers, callees, and execution flows for a function/class
   - Example: `gitnexus context "Function:src/linux/btop_collect.cpp:collect" -r btop`

3. **Read source code directly**: Use the Read tool to examine files
   - GitNexus tells you where (file + line), then read the actual implementation

4. **Impact analysis**: Before modifying code
   - `gitnexus impact <symbol> -r <project-name>`

### GitNexus Safety Protocol

**Before examining code in any reference project:**

- Run `gitnexus status` in the project directory to verify index freshness
- If stale, run `gitnexus analyze` to refresh

**When exploring a feature:**

- Start with Cypher queries to find relevant symbols
- Use `context` to understand function relationships
- Read the actual source code with the Read tool
- Use `impact` before making changes

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

# Search with Cypher (NOT 'query' - that doesn't work!)
gitnexus cypher "MATCH (n:Function) WHERE n.name CONTAINS 'memory' RETURN n.name, n.filePath LIMIT 10" -r btop
gitnexus cypher "MATCH (n:Function) WHERE n.name CONTAINS 'gpu' RETURN n.name, n.filePath LIMIT 10" -r nvtop

# Get context for a specific symbol
gitnexus context "collect" -r btop

# Impact analysis before making changes
gitnexus impact "function_name" -r btop

# Check repository status
cd <project> && gitnexus status

# Detect what your changes affected
cd <project> && gitnexus detect-changes
```

## Notes

- This is currently a reference collection phase - no Rust code exists yet
- All reference projects are indexed and ready for analysis
- Use GitNexus extensively to understand implementation patterns before writing Rust code
- Each reference project has its own CLAUDE.md with project-specific GitNexus guidance

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **system-monitor** (20331 symbols, 35401 relationships, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus cypher` with pattern matching instead of grepping. See examples in the "Using GitNexus" section above.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/system-monitor/context` | Codebase overview, check index freshness |
| `gitnexus://repo/system-monitor/clusters` | All functional areas |
| `gitnexus://repo/system-monitor/processes` | All execution flows |
| `gitnexus://repo/system-monitor/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
