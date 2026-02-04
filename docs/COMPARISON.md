# Tool Comparison: gidterm vs Similar Tools

## Mission Control vs gidterm

**Repository:** https://github.com/crshdn/mission-control

### Core Differences

**Positioning is fundamentally different:**

| | Mission Control | gidterm |
|---|---|---|
| **Interface** | Web Dashboard (Next.js) | Terminal TUI |
| **Core Purpose** | AI Agent Orchestration | Graph-driven Automation |
| **Task Source** | Human-created + drag-drop assignment | `.gid/graph.yml` definitions |
| **Execution Model** | Agent receives task → autonomously completes | DAG auto-schedules → PTY executes |
| **Use Case** | "Managing AI employees" | "Automating build pipelines" |

### Similarities

- ✅ Both manage multi-project workspaces
- ✅ Both have task dependency resolution
- ✅ Both provide real-time status monitoring
- ✅ Both support parallel execution

### Key Distinctions

**Mission Control** = **Human directs AI**
- Kanban board: INBOX → ASSIGNED → IN PROGRESS → REVIEW → DONE
- Agent chat: inter-agent dialogue and collaboration
- Human approval: Charlie reviews work quality
- Cross-machine: remote agents collaborate via HTTP API

**gidterm** = **Graph directs Terminal**
- Auto DAG: dependencies auto-resolve, auto-trigger
- PTY management: real terminal sessions
- Output capture: live stdout/metrics extraction
- Local-first: direct filesystem operations

### Complementarity

**The two can work together:**

```
Mission Control: Assigns task to agent
        ↓
Agent: Runs `gidterm --workspace`
        ↓
gidterm: Executes build → test → deploy DAG
        ↓
Agent: Reports "TASK_COMPLETE: Backend deployed to staging"
```

**Example workflow:**
```
Mission Control: "Charlie, deploy the backend"
    ↓
Charlie (agent): Run gidterm --workspace
    ↓
gidterm: Execute build → test → deploy DAG
    ↓
Charlie: "TASK_COMPLETE: Backend deployed to staging"
```

### When to Use Each

**gidterm is better for:**
- DevOps automation (CI/CD pipelines)
- Local development workflows
- Complex dependency graphs (dozens to hundreds of tasks)
- Terminal enthusiasts
- Deterministic, repeatable builds
- Projects with well-defined task graphs

**Mission Control is better for:**
- Non-technical users managing AI teams
- Cross-machine agent collaboration
- Task review workflows
- Web UI preference
- Open-ended creative tasks
- Dynamic task assignment

### Architecture Perspective

**Mission Control** = Upper-layer task management
- Human → AI delegation
- Quality control and oversight
- Cross-machine coordination

**gidterm** = Lower-layer execution engine
- Graph → Terminal automation
- Deterministic scheduling
- Local process orchestration

**Not competitors** — different layers of abstraction. Mission Control could use gidterm as its execution backend.

---

## Other Similar Tools

### Turborepo / Nx

**Similarities:**
- Multi-project workspace management
- Task dependency resolution
- Caching and parallelization

**Differences:**
- Focused on JavaScript/TypeScript monorepos
- Less flexible task graph model (package.json scripts)
- No real-time TUI monitoring
- No semantic layer (gidterm can extract metrics from output)

### Make / Just / Task

**Similarities:**
- DAG-based task execution
- Dependency resolution

**Differences:**
- Single-project focused
- No multi-project workspace discovery
- No real-time status dashboard
- No session persistence/history
- No integration with project graphs (`.gid/graph.yml`)

### Jenkins / GitHub Actions / GitLab CI

**Similarities:**
- CI/CD automation
- Parallel execution
- Task dependencies

**Differences:**
- Remote/cloud-first (gidterm is local-first)
- YAML-based config (gidterm uses gid graphs)
- No interactive TUI
- Heavier weight (requires server setup)

### Justfile / Makefile with TUI wrappers

**Similarities:**
- Task automation
- Terminal-based

**Differences:**
- No multi-project workspace mode
- No automatic dependency graph extraction
- No intelligent scheduling
- No semantic output parsing

---

## gidterm's Unique Value

1. **Graph-native:** First-class integration with gid project graphs
2. **Multi-project workspace:** Discover and manage multiple projects simultaneously
3. **Semantic layer:** Extract metrics and meaning from command output
4. **Live TUI:** Real-time dashboard with task status and output streaming
5. **Session persistence:** Full task history tracking
6. **Local-first:** Direct filesystem operations, no server required

---

*Last updated: 2026-02-04*
