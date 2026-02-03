//! Agent Integration - Detect and track coding agent processes
//!
//! Supports common coding agents:
//! - Claude Code (`claude`)
//! - Codex CLI (`codex`)
//! - OpenCode (`opencode`)
//! - Pi Coding Agent (`pi`)
//!
//! Features:
//! - Process detection via `ps`
//! - Status parsing from output
//! - Agent task definition in graph.yml
//! - Dashboard integration with emoji indicators

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Known coding agent types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    /// Claude Code CLI
    Claude,
    /// Codex CLI
    Codex,
    /// OpenCode
    OpenCode,
    /// Pi Coding Agent
    Pi,
    /// Generic/unknown agent
    Generic,
}

impl AgentType {
    /// Get process name patterns for detection
    pub fn process_patterns(&self) -> Vec<&'static str> {
        match self {
            Self::Claude => vec!["claude", "claude-code"],
            Self::Codex => vec!["codex"],
            Self::OpenCode => vec!["opencode"],
            Self::Pi => vec!["pi"],
            Self::Generic => vec![],
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
            Self::OpenCode => "OpenCode",
            Self::Pi => "Pi",
            Self::Generic => "Agent",
        }
    }

    /// Get emoji for agent type
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Claude => "🤖",
            Self::Codex => "🧠",
            Self::OpenCode => "💻",
            Self::Pi => "🥧",
            Self::Generic => "🔧",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "claude" | "claude-code" | "claudecode" => Self::Claude,
            "codex" => Self::Codex,
            "opencode" | "open-code" => Self::OpenCode,
            "pi" => Self::Pi,
            _ => Self::Generic,
        }
    }
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Agent runtime status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentRuntimeStatus {
    /// Agent is not running
    #[default]
    NotRunning,
    /// Agent process is running (actively executing)
    Running,
    /// Agent is thinking/processing (e.g., waiting for LLM response)
    Thinking,
    /// Agent is waiting for user input
    WaitingInput,
    /// Agent has completed its task
    Completed,
    /// Agent encountered an error
    Error,
}

impl AgentRuntimeStatus {
    /// Get emoji for status
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::NotRunning => "⚫",
            Self::Running => "🤖",
            Self::Thinking => "💭",
            Self::WaitingInput => "⏳",
            Self::Completed => "✅",
            Self::Error => "❌",
        }
    }

    /// Get color for TUI display
    pub fn color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            Self::NotRunning => Color::DarkGray,
            Self::Running => Color::Green,
            Self::Thinking => Color::Yellow,
            Self::WaitingInput => Color::Blue,
            Self::Completed => Color::Gray,
            Self::Error => Color::Red,
        }
    }

    /// Human-readable status text
    pub fn display_text(&self) -> &'static str {
        match self {
            Self::NotRunning => "not running",
            Self::Running => "running",
            Self::Thinking => "thinking",
            Self::WaitingInput => "waiting for input",
            Self::Completed => "completed",
            Self::Error => "error",
        }
    }
}

/// Detected agent process info
#[derive(Debug, Clone)]
pub struct AgentProcess {
    /// Process ID
    pub pid: u32,
    /// Agent type
    pub agent_type: AgentType,
    /// Command line
    pub command: String,
    /// Working directory (if detectable)
    pub cwd: Option<String>,
    /// Process start time (approximate)
    pub start_time: Option<u64>,
}

/// Agent task definition (from graph.yml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    /// Agent type to use
    pub agent: AgentType,
    /// Prompt to pass to agent
    pub prompt: String,
    /// Task status
    #[serde(default)]
    pub status: AgentTaskStatus,
    /// Working directory (optional, defaults to project root)
    pub cwd: Option<String>,
    /// Additional arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Auto-approve agent actions (for Claude Code: --auto-approve)
    #[serde(default)]
    pub auto_approve: bool,
}

/// Status of an agent task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentTaskStatus {
    /// Task not yet started
    #[default]
    Pending,
    /// Agent is running
    Running,
    /// Agent completed successfully
    Done,
    /// Agent failed
    Failed,
    /// Task skipped
    Skipped,
}

/// Tracked agent state for a project
#[derive(Debug, Clone)]
pub struct AgentState {
    /// Agent type
    pub agent_type: AgentType,
    /// Associated project name
    pub project: String,
    /// Associated task ID (if any)
    pub task_id: Option<String>,
    /// Runtime status
    pub status: AgentRuntimeStatus,
    /// Process info (if running)
    pub process: Option<AgentProcess>,
    /// Last status update time
    pub last_update: Instant,
    /// Recent output lines (for status detection)
    pub recent_output: Vec<String>,
}

impl AgentState {
    /// Create new agent state
    pub fn new(agent_type: AgentType, project: String) -> Self {
        Self {
            agent_type,
            project,
            task_id: None,
            status: AgentRuntimeStatus::NotRunning,
            process: None,
            last_update: Instant::now(),
            recent_output: Vec::new(),
        }
    }

    /// Update with new output line
    pub fn add_output(&mut self, line: &str) {
        self.recent_output.push(line.to_string());
        // Keep last 50 lines for status detection
        if self.recent_output.len() > 50 {
            self.recent_output.remove(0);
        }
        self.last_update = Instant::now();
    }
}

/// Agent status parser - detects status from output
pub struct AgentStatusParser {
    /// Patterns indicating thinking/processing
    thinking_patterns: Vec<&'static str>,
    /// Patterns indicating waiting for input
    waiting_patterns: Vec<&'static str>,
    /// Patterns indicating completion
    completed_patterns: Vec<&'static str>,
    /// Patterns indicating errors
    error_patterns: Vec<&'static str>,
}

impl Default for AgentStatusParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentStatusParser {
    /// Create a new status parser with default patterns
    pub fn new() -> Self {
        Self {
            thinking_patterns: vec![
                "thinking",
                "processing",
                "analyzing",
                "generating",
                "working on",
                "computing",
                "waiting for response",
                "loading",
                "searching",
                "reading",
                "reviewing",
            ],
            waiting_patterns: vec![
                "waiting for input",
                "waiting for",
                "press enter",
                "press any key",
                "[y/n]",
                "(y/n)",
                "confirm",
                "continue?",
                "proceed?",
                "approve",
                "permission",
                "enter your",
                "type your",
                "would you like",
                "do you want",
                "please provide",
                "please enter",
            ],
            completed_patterns: vec![
                "done",
                "completed",
                "finished",
                "success",
                "all tasks complete",
                "goodbye",
                "bye",
                "exiting",
                "session ended",
                "task complete",
            ],
            error_patterns: vec![
                "error:",
                "error!",
                "failed",
                "failure",
                "exception",
                "panic",
                "crash",
                "aborted",
                "fatal",
                "cannot",
                "couldn't",
                "unable to",
                "permission denied",
            ],
        }
    }

    /// Parse status from output lines
    pub fn parse_status(&self, lines: &[String], process_running: bool) -> AgentRuntimeStatus {
        if !process_running {
            return AgentRuntimeStatus::NotRunning;
        }

        // Check recent lines (last 10) for status indicators
        let recent: Vec<&str> = lines.iter().rev().take(10).map(|s| s.as_str()).collect();

        for line in &recent {
            let lower = line.to_lowercase();

            // Check for errors first (highest priority)
            for pattern in &self.error_patterns {
                if lower.contains(pattern) {
                    return AgentRuntimeStatus::Error;
                }
            }

            // Check for waiting input
            for pattern in &self.waiting_patterns {
                if lower.contains(pattern) {
                    return AgentRuntimeStatus::WaitingInput;
                }
            }

            // Check for completion
            for pattern in &self.completed_patterns {
                if lower.contains(pattern) {
                    return AgentRuntimeStatus::Completed;
                }
            }

            // Check for thinking
            for pattern in &self.thinking_patterns {
                if lower.contains(pattern) {
                    return AgentRuntimeStatus::Thinking;
                }
            }
        }

        // Default to running if process is active
        AgentRuntimeStatus::Running
    }
}

/// Agent detector - finds running agent processes
pub struct AgentDetector {
    /// Cache of detected processes
    cache: HashMap<u32, AgentProcess>,
    /// Last scan time
    last_scan: Option<Instant>,
    /// Minimum interval between scans (seconds)
    scan_interval: u64,
}

impl Default for AgentDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentDetector {
    /// Create a new detector
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            last_scan: None,
            scan_interval: 5,
        }
    }

    /// Set scan interval
    pub fn with_interval(mut self, seconds: u64) -> Self {
        self.scan_interval = seconds;
        self
    }

    /// Scan for agent processes
    pub fn scan(&mut self) -> Result<Vec<AgentProcess>> {
        // Check if we should scan
        if let Some(last) = self.last_scan {
            if last.elapsed().as_secs() < self.scan_interval {
                // Return cached results
                return Ok(self.cache.values().cloned().collect());
            }
        }

        self.cache.clear();
        let processes = self.detect_processes()?;

        for proc in &processes {
            self.cache.insert(proc.pid, proc.clone());
        }
        self.last_scan = Some(Instant::now());

        Ok(processes)
    }

    /// Detect agent processes using ps command
    fn detect_processes(&self) -> Result<Vec<AgentProcess>> {
        let mut agents = Vec::new();

        // Use ps to get process list
        // Format: pid, command
        let output = Command::new("ps")
            .args(["-eo", "pid,command"])
            .output()?;

        if !output.status.success() {
            return Ok(agents);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines().skip(1) {
            // Skip header
            let parts: Vec<&str> = line.trim().splitn(2, ' ').collect();
            if parts.len() < 2 {
                continue;
            }

            let pid: u32 = match parts[0].trim().parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let command = parts[1].trim();

            // Check against known agent patterns
            for agent_type in [
                AgentType::Claude,
                AgentType::Codex,
                AgentType::OpenCode,
                AgentType::Pi,
            ] {
                for pattern in agent_type.process_patterns() {
                    // Match if command starts with pattern or contains it as executable
                    let cmd_lower = command.to_lowercase();
                    if cmd_lower.starts_with(pattern)
                        || cmd_lower.contains(&format!("/{}", pattern))
                        || cmd_lower.contains(&format!(" {}", pattern))
                    {
                        agents.push(AgentProcess {
                            pid,
                            agent_type,
                            command: command.to_string(),
                            cwd: self.get_process_cwd(pid),
                            start_time: self.get_process_start_time(pid),
                        });
                        break;
                    }
                }
            }
        }

        Ok(agents)
    }

    /// Get process working directory (macOS/Linux)
    #[cfg(target_os = "macos")]
    fn get_process_cwd(&self, pid: u32) -> Option<String> {
        // On macOS, use lsof
        let output = Command::new("lsof")
            .args(["-a", "-p", &pid.to_string(), "-d", "cwd", "-Fn"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with('n') {
                return Some(line[1..].to_string());
            }
        }
        None
    }

    #[cfg(target_os = "linux")]
    fn get_process_cwd(&self, pid: u32) -> Option<String> {
        std::fs::read_link(format!("/proc/{}/cwd", pid))
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    fn get_process_cwd(&self, _pid: u32) -> Option<String> {
        None
    }

    /// Get process start time
    fn get_process_start_time(&self, _pid: u32) -> Option<u64> {
        // For now, just return current time as approximation
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs())
    }

    /// Check if a specific process is still running
    pub fn is_process_running(&self, pid: u32) -> bool {
        #[cfg(unix)]
        {
            Command::new("kill")
                .args(["-0", &pid.to_string()])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        #[cfg(not(unix))]
        {
            true // Conservative assumption on non-Unix
        }
    }

    /// Find agent process associated with a directory
    pub fn find_by_directory(&mut self, dir: &str) -> Option<AgentProcess> {
        self.scan().ok()?;

        for proc in self.cache.values() {
            if let Some(cwd) = &proc.cwd {
                if cwd.starts_with(dir) || dir.starts_with(cwd) {
                    return Some(proc.clone());
                }
            }
        }
        None
    }
}

/// Agent manager - tracks agents across projects
pub struct AgentManager {
    /// Agent states by project
    states: HashMap<String, AgentState>,
    /// Process detector
    detector: AgentDetector,
    /// Status parser
    parser: AgentStatusParser,
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentManager {
    /// Create new agent manager
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            detector: AgentDetector::new(),
            parser: AgentStatusParser::new(),
        }
    }

    /// Register a project for agent tracking
    pub fn register_project(&mut self, project: &str, agent_type: AgentType) {
        self.states
            .entry(project.to_string())
            .or_insert_with(|| AgentState::new(agent_type, project.to_string()));
    }

    /// Update agent state with new output
    pub fn update_output(&mut self, project: &str, line: &str) {
        if let Some(state) = self.states.get_mut(project) {
            state.add_output(line);

            // Re-evaluate status
            let process_running = state
                .process
                .as_ref()
                .map(|p| self.detector.is_process_running(p.pid))
                .unwrap_or(false);

            state.status = self.parser.parse_status(&state.recent_output, process_running);
        }
    }

    /// Scan for agent processes and update states
    pub fn scan_processes(&mut self) -> Result<()> {
        let processes = self.detector.scan()?;

        // Update states with detected processes
        for proc in processes {
            // Try to match to a project by cwd
            if let Some(cwd) = &proc.cwd {
                for state in self.states.values_mut() {
                    // Simple heuristic: if project name is in cwd
                    if cwd.contains(&state.project) {
                        state.process = Some(proc.clone());
                        if state.status == AgentRuntimeStatus::NotRunning {
                            state.status = AgentRuntimeStatus::Running;
                        }
                        break;
                    }
                }
            }
        }

        // Check for processes that stopped
        for state in self.states.values_mut() {
            if let Some(proc) = &state.process {
                if !self.detector.is_process_running(proc.pid) {
                    state.process = None;
                    if state.status == AgentRuntimeStatus::Running
                        || state.status == AgentRuntimeStatus::Thinking
                    {
                        // Assume completed if no error detected
                        state.status = AgentRuntimeStatus::Completed;
                    }
                }
            }
        }

        Ok(())
    }

    /// Get agent state for a project
    pub fn get_state(&self, project: &str) -> Option<&AgentState> {
        self.states.get(project)
    }

    /// Get agent status for a project
    pub fn get_status(&self, project: &str) -> AgentRuntimeStatus {
        self.states
            .get(project)
            .map(|s| s.status)
            .unwrap_or(AgentRuntimeStatus::NotRunning)
    }

    /// Get all states
    pub fn all_states(&self) -> impl Iterator<Item = &AgentState> {
        self.states.values()
    }

    /// Build command to launch an agent with a task
    pub fn build_agent_command(task: &AgentTask) -> Vec<String> {
        match task.agent {
            AgentType::Claude => {
                let mut cmd = vec!["claude".to_string()];
                if task.auto_approve {
                    cmd.push("--auto-approve".to_string());
                }
                cmd.extend(task.args.clone());
                // Claude Code accepts prompt as the last argument
                cmd.push(task.prompt.clone());
                cmd
            }
            AgentType::Codex => {
                let mut cmd = vec!["codex".to_string()];
                cmd.extend(task.args.clone());
                cmd.push(task.prompt.clone());
                cmd
            }
            AgentType::OpenCode => {
                let mut cmd = vec!["opencode".to_string()];
                cmd.extend(task.args.clone());
                cmd.push(task.prompt.clone());
                cmd
            }
            AgentType::Pi => {
                let mut cmd = vec!["pi".to_string()];
                cmd.extend(task.args.clone());
                cmd.push(task.prompt.clone());
                cmd
            }
            AgentType::Generic => {
                // Generic: first arg should be the agent command
                let mut cmd: Vec<String> = task.args.clone();
                cmd.push(task.prompt.clone());
                cmd
            }
        }
    }

    /// Build command string for shell execution
    pub fn build_agent_command_string(task: &AgentTask) -> String {
        let parts = Self::build_agent_command(task);
        // Quote arguments that contain spaces
        parts
            .iter()
            .map(|p| {
                if p.contains(' ') {
                    format!("\"{}\"", p.replace('"', "\\\""))
                } else {
                    p.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_from_str() {
        assert_eq!(AgentType::from_str("claude"), AgentType::Claude);
        assert_eq!(AgentType::from_str("Claude"), AgentType::Claude);
        assert_eq!(AgentType::from_str("claude-code"), AgentType::Claude);
        assert_eq!(AgentType::from_str("codex"), AgentType::Codex);
        assert_eq!(AgentType::from_str("opencode"), AgentType::OpenCode);
        assert_eq!(AgentType::from_str("pi"), AgentType::Pi);
        assert_eq!(AgentType::from_str("unknown"), AgentType::Generic);
    }

    #[test]
    fn test_agent_type_emoji() {
        assert_eq!(AgentType::Claude.emoji(), "🤖");
        assert_eq!(AgentType::Codex.emoji(), "🧠");
    }

    #[test]
    fn test_status_parser() {
        let parser = AgentStatusParser::new();

        // Test thinking detection
        let lines = vec!["processing request...".to_string()];
        assert_eq!(
            parser.parse_status(&lines, true),
            AgentRuntimeStatus::Thinking
        );

        // Test waiting detection
        let lines = vec!["Do you want to continue? [y/n]".to_string()];
        assert_eq!(
            parser.parse_status(&lines, true),
            AgentRuntimeStatus::WaitingInput
        );

        // Test error detection
        let lines = vec!["Error: failed to compile".to_string()];
        assert_eq!(parser.parse_status(&lines, true), AgentRuntimeStatus::Error);

        // Test completed detection
        let lines = vec!["Task completed successfully!".to_string()];
        assert_eq!(
            parser.parse_status(&lines, true),
            AgentRuntimeStatus::Completed
        );

        // Test running (no specific indicator)
        let lines = vec!["Writing file src/main.rs".to_string()];
        assert_eq!(
            parser.parse_status(&lines, true),
            AgentRuntimeStatus::Running
        );

        // Test not running
        let lines = vec!["Writing file src/main.rs".to_string()];
        assert_eq!(
            parser.parse_status(&lines, false),
            AgentRuntimeStatus::NotRunning
        );
    }

    #[test]
    fn test_build_agent_command() {
        let task = AgentTask {
            agent: AgentType::Claude,
            prompt: "Implement feature X".to_string(),
            status: AgentTaskStatus::Pending,
            cwd: None,
            args: vec![],
            auto_approve: true,
        };

        let cmd = AgentManager::build_agent_command(&task);
        assert_eq!(cmd[0], "claude");
        assert!(cmd.contains(&"--auto-approve".to_string()));
        assert!(cmd.last().unwrap().contains("Implement feature X"));
    }

    #[test]
    fn test_agent_state_output_tracking() {
        let mut state = AgentState::new(AgentType::Claude, "my-project".to_string());

        for i in 0..60 {
            state.add_output(&format!("line {}", i));
        }

        // Should keep last 50 lines
        assert_eq!(state.recent_output.len(), 50);
        assert_eq!(state.recent_output[0], "line 10");
        assert_eq!(state.recent_output[49], "line 59");
    }

    #[test]
    fn test_agent_runtime_status_emoji() {
        assert_eq!(AgentRuntimeStatus::Running.emoji(), "🤖");
        assert_eq!(AgentRuntimeStatus::Thinking.emoji(), "💭");
        assert_eq!(AgentRuntimeStatus::WaitingInput.emoji(), "⏳");
        assert_eq!(AgentRuntimeStatus::Completed.emoji(), "✅");
        assert_eq!(AgentRuntimeStatus::Error.emoji(), "❌");
    }
}
