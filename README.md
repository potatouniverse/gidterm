# GidTerm - Graph-Driven Semantic Terminal Controller

A semantic terminal controller that integrates gid's project/task graph with intelligent process management.

## 🎯 Core Concept

Not just a terminal multiplexer - a complete project orchestration system with:
- Multi-project management
- Task dependency graphs (DAG)
- Semantic understanding of tasks
- Unified dashboard
- Intelligent control

## 📁 Project Structure

```
gidterm/
├── docs/               # Documentation
│   └── design.md       # Complete design document
├── src/                # Source code (TBD)
├── examples/           # Example configurations
└── README.md           # This file
```

## 🚀 Status

**Current Phase:** Design & Architecture

## 📚 Documentation

See [docs/design.md](docs/design.md) for the complete design document.

## 🛠️ Technology Stack (Proposed)

- Language: Rust
- TUI: ratatui
- PTY: portable-pty
- Terminal: crossterm
- Graph: petgraph
- Config: serde + YAML/TOML

## 🔗 Related Projects

- gid (provides graph structure)
- mprocs (inspiration)
- procmux (inspiration)
- tmux (comparison)

---

*Created: 2026-01-30*
