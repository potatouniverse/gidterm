# GidTerm - Open Questions & Discussion Topics

*Last updated: 2026-01-30*

---

## 🔴 **Critical Decisions Needed**

### 1. Configuration File Format

**Question:** YAML vs TOML vs Custom DSL?

**Options:**
```yaml
# Option A: YAML (more readable, nested structures)
project: ML-Training
tasks:
  preprocess:
    type: data_processing
    command: python preprocess.py
    depends_on: [download_data]
```

```toml
# Option B: TOML (more explicit, less indentation)
[project]
name = "ML-Training"

[[tasks]]
name = "preprocess"
type = "data_processing"
command = "python preprocess.py"
depends_on = ["download_data"]
```

```
# Option C: Custom DSL (gid-specific)
project ML-Training {
    task preprocess(data_processing) {
        run: python preprocess.py
        after: download_data
    }
}
```

**Considerations:**
- Compatibility with gid format?
- User-friendliness
- Extensibility
- Editor support (syntax highlighting)

**Decision:** ?

---

### 2. Task Type System

**Question:** How to define and extend task types?

**Options:**

**A. Built-in types only:**
```rust
enum TaskType {
    MLTraining,
    BuildTask,
    Service,
    DataProcessing,
    Generic,
}
```
- Simple
- Limited

**B. Plugin-based:**
```rust
struct TaskType {
    name: String,
    parser: Box<dyn OutputParser>,
    actions: Vec<SemanticAction>,
}
```
- Flexible
- Complex

**C. Hybrid (built-in + custom):**
```yaml
task: train_model
  type: ml_training  # built-in
  # OR
  type: 
    custom: my_custom_type
    parser: ./parsers/custom.wasm
```

**Decision:** ?

---

### 3. Parser Implementation

**Question:** How to implement output parsers?

**Options:**

**A. Regex-based (simple):**
```rust
struct RegexParser {
    patterns: HashMap<String, Regex>,
}
```
- Fast to implement
- Limited flexibility
- Fragile

**B. Tree-sitter (structured):**
```rust
struct TreeSitterParser {
    grammar: Grammar,
}
```
- More robust
- Complex setup
- Better for structured output

**C. LLM-based (flexible):**
```rust
struct LLMParser {
    llm_client: Client,
    examples: Vec<Example>,
}
```
- Most flexible
- API cost
- Latency

**D. Hybrid:**
- Regex for simple patterns
- LLM for complex/ambiguous output

**Decision:** ?

---

### 4. Semantic Command Storage

**Question:** Where to store semantic commands?

**Options:**

**A. In task config:**
```yaml
task: train_model
  semantic_commands:
    save: "model.save('checkpoint.pth')"
```
- Centralized
- Can get verbose

**B. Separate command library:**
```yaml
# commands/ml_training.yaml
ml_training:
  save: "model.save('checkpoint.pth')"
  adjust_lr: "optimizer.param_groups[0]['lr'] = {value}"
```
- Reusable
- More files to manage

**C. Inline scripts:**
```yaml
task: train_model
  control_script: ./scripts/model_control.py
```
- Most flexible
- Requires scripting

**Decision:** ?

---

### 5. Graph Representation

**Question:** How to represent and query the task graph?

**Options:**

**A. petgraph (standard Rust graph library):**
```rust
use petgraph::Graph;
let graph = Graph::<Task, Dependency>::new();
```
- Well-tested
- Good APIs
- May be overkill

**B. Custom implementation:**
```rust
struct TaskGraph {
    tasks: HashMap<TaskId, Task>,
    dependencies: HashMap<TaskId, Vec<TaskId>>,
}
```
- Simpler
- Custom to our needs
- More work

**C. Use gid's internal representation:**
- Maximum compatibility
- Depends on gid's API

**Decision:** ?

---

## 🟡 **Important but Not Blocking**

### 6. Progress Tracking Granularity

**Question:** How detailed should progress tracking be?

**Levels:**
```
Level 1: Binary (running/done)
Level 2: Percentage (45%)
Level 3: Stage-based (Init → Load → Train → Validate)
Level 4: Real-time metrics (epoch, loss, etc.)
```

**Trade-offs:**
- More granular = better UX, more complex parsing
- Less granular = simpler, less informative

**Decision:** ?

---

### 7. Multi-Project UI Layout

**Question:** How to organize multi-project view?

**Options:**

**A. Tabs:**
```
[Project A] [Project B] [Project C]
─────────────────────────────────
Current project content here
```

**B. Sidebar:**
```
┌──────────┬─────────────┐
│ Projects │ Content     │
│ > Proj A │             │
│   Proj B │             │
│   Proj C │             │
└──────────┴─────────────┘
```

**C. Nested tree:**
```
┌─────────────────────────┐
│ > Project A             │
│   > Task 1              │
│   > Task 2              │
│ □ Project B             │
└─────────────────────────┘
```

**Decision:** ?

---

### 8. State Persistence

**Question:** How to save/restore session state?

**What to persist:**
- Task status (running/paused/done)
- Terminal scrollback
- Metrics history
- User preferences

**Format:**
- SQLite database?
- JSON/YAML files?
- Binary format?

**When to save:**
- On every state change?
- Periodically (every N seconds)?
- On explicit save command?

**Decision:** ?

---

### 9. Remote Control API

**Question:** Should we support remote control? How?

**Use cases:**
- Trigger tasks from CI/CD
- Monitor from mobile
- Control from another terminal

**Options:**

**A. HTTP REST API:**
```
POST /api/tasks/train_model/start
GET /api/tasks/status
```

**B. Unix socket:**
```
gidterm-ctl start train_model
```

**C. Both:**

**Decision:** ?

---

### 10. Error Recovery

**Question:** How to handle task failures?

**Strategies:**

**A. Auto-retry:**
```yaml
task: flaky_job
  retry: 3
  retry_delay: 5s
```

**B. Fallback:**
```yaml
task: primary
  on_failure:
    run: fallback_task
```

**C. Manual:**
- User decides

**Decision:** ?

---

## 🟢 **Nice to Have**

### 11. Notification System

- Desktop notifications?
- Sound alerts?
- Email/Slack on completion?

### 12. Metrics Export

- Prometheus format?
- JSON export?
- CSV logs?

### 13. Themes

- Customizable colors?
- Preset themes?

### 14. Keybindings

- Vim-style?
- Emacs-style?
- Custom?

---

## 🎯 **Immediate Next Steps**

What should we tackle first?

**Suggested priority:**
1. ✅ Configuration format (blocks everything else)
2. ✅ Task type system (determines parser design)
3. ✅ Parser implementation approach
4. Graph representation (affects performance)
5. UI layout (affects UX)

**Your thoughts?**

---

## 💭 **Additional Discussion Topics**

### Integration with gid

- How tightly coupled should gidterm be with gid?
- Should it work standalone?
- Can it read gid's native format directly?

### Terminal Compatibility

- Which terminals to test on?
- Handle 256 colors vs truecolor?
- Fallback for limited terminals?

### Performance

- How many tasks should it handle?
- Memory constraints?
- CPU usage targets?

---

*Add your notes and decisions as we discuss*
