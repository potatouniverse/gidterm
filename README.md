# GidTerm - Graph-Driven Semantic Terminal Controller

A semantic terminal controller that integrates gid's project/task graphs with intelligent process management and real-time monitoring.

## 🎯 What It Does

Manage multiple projects with complex task dependencies - all in one unified TUI dashboard.

```
╔═══════════════════════════════════════════════════╗
║ 🌐 Workspace (3 projects) - GidTerm               ║
╠═══════════════════════════════════════════════════╣
║ 📁 backend                                        ║
║   ⚙ install [in-progress] 🟢 (12L)              ║
║   □ build [pending] ⏳                           ║
║                                                   ║
║ 📁 frontend                                       ║
║   ✓ webpack [done] ✅                            ║
║   ⚙ dev [running] 🟢 (45L)                      ║
╚═══════════════════════════════════════════════════╝
```

## ✨ Features

- **🌐 Multi-project workspace** - Manage multiple projects simultaneously
- **📊 DAG scheduling** - Automatic dependency resolution
- **⚡ Parallel execution** - Run independent tasks concurrently
- **🔄 gid integration** - Auto-loads from `.gid/graph.yml`
- **💾 Session persistence** - Full task history tracking
- **📺 Live TUI** - Real-time dashboard with task status & output

## 🚀 Quick Start

### Single Project
```bash
cd my-project
gidterm                    # Auto-detects .gid/graph.yml
```

### Workspace Mode (Multiple Projects)
```bash
cd my-monorepo
gidterm --workspace        # Discovers all projects
```

## 📁 Project Structure

```
my-monorepo/
├── backend/
│   └── .gid/
│       └── graph.yml      # Backend tasks
├── frontend/
│   └── .gid/
│       └── graph.yml      # Frontend tasks
└── database/
    └── .gid/
        └── graph.yml      # Database tasks
```

## 🎯 Status

**Current Phase:** ✅ Production Ready!

**What Works:**
- ✅ Multi-project workspace management
- ✅ Task dependency resolution (DAG)
- ✅ Parallel task execution
- ✅ Real-time TUI dashboard
- ✅ Session persistence & history
- ✅ gid project integration

## 📖 Usage

### CLI Commands

```bash
# Single project mode
gidterm                     # Auto-detect .gid/graph.yml
gidterm my-tasks.yml        # Explicit file

# Workspace mode
gidterm --workspace         # Discover all projects
gidterm -w                  # Short form

# Help
gidterm --help
```

### Keyboard Controls

- `↑`/`↓` - Select task
- `r` - Refresh / restart ready tasks
- `q` - Quit

### Task Graph Example

```yaml
# .gid/graph.yml
metadata:
  project: "my-app"

tasks:
  install:
    command: "npm install"
    status: "pending"
  
  build:
    command: "npm run build"
    depends_on: ["install"]
    status: "pending"
  
  dev:
    command: "npm run dev"
    depends_on: ["build"]
    status: "pending"
```

## 📚 Documentation

- [MULTI-PROJECT.md](MULTI-PROJECT.md) - Multi-project workspace guide
- [GID-INTEGRATION.md](GID-INTEGRATION.md) - gid integration details
- [IMPLEMENTATION-SUMMARY.md](IMPLEMENTATION-SUMMARY.md) - Implementation notes
- [STATUS.md](STATUS.md) - Current development status
- [docs/design.md](docs/design.md) - Original design document
- [docs/COMPARISON.md](docs/COMPARISON.md) - How gidterm compares to similar tools

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
