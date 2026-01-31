//! Application state and main event loop

use crate::core::{Executor, Graph, Scheduler, TaskEvent};
use crate::session::{Session, TaskStatus};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Application state
pub struct App {
    pub scheduler: Scheduler,
    pub executor: Executor,
    pub event_rx: mpsc::UnboundedReceiver<TaskEvent>,
    pub task_outputs: std::collections::HashMap<String, Vec<String>>,
    pub should_quit: bool,
    pub selected_task: usize,
    pub last_update: Instant,
    pub session: Session,
    pub workspace_mode: bool,
    pub project_names: Vec<String>, // For workspace mode
}

impl App {
    /// Create a new app from graph (single project mode)
    pub fn new(graph: Graph) -> Self {
        let scheduler = Scheduler::new(graph.clone());
        let (executor, event_rx) = Executor::new();

        // Create session - use project name from metadata or "unknown"
        let project_name = graph
            .metadata
            .as_ref()
            .map(|m| m.project.clone())
            .unwrap_or_else(|| "unknown".to_string());
        
        let session = Session::new(project_name);

        Self {
            scheduler,
            executor,
            event_rx,
            task_outputs: std::collections::HashMap::new(),
            should_quit: false,
            selected_task: 0,
            last_update: Instant::now(),
            session,
            workspace_mode: false,
            project_names: Vec::new(),
        }
    }

    /// Create app from workspace (multi-project mode)
    pub fn from_workspace(workspace: &crate::workspace::Workspace) -> Self {
        // Create unified graph with namespaced tasks
        let unified_graph = workspace.to_unified_graph();
        let scheduler = Scheduler::new(unified_graph);
        let (executor, event_rx) = Executor::new();

        let session = Session::new("workspace".to_string());
        let project_names = workspace.project_names();

        Self {
            scheduler,
            executor,
            event_rx,
            task_outputs: std::collections::HashMap::new(),
            should_quit: false,
            selected_task: 0,
            last_update: Instant::now(),
            session,
            workspace_mode: true,
            project_names,
        }
    }

    /// Start all ready tasks
    pub async fn start_ready_tasks(&mut self) -> Result<()> {
        let ready = self.scheduler.schedule_next();

        for task_id in ready {
            let task = self.scheduler.graph().get_task(&task_id).unwrap();
            
            if let Some(command) = &task.command {
                log::info!("Starting task: {} ({})", task_id, command);
                
                // Track in session
                self.session.start_task(task_id.clone());
                
                self.executor.start_task(&task_id, command).await?;
                self.scheduler.mark_started(&task_id)?;
            } else {
                // No command, mark as done immediately
                self.scheduler.mark_done(&task_id)?;
            }
        }

        // Save session after starting tasks
        if let Err(e) = self.session.save() {
            log::warn!("Failed to save session: {}", e);
        }

        Ok(())
    }

    /// Process events from executor
    pub fn process_events(&mut self) {
        let mut session_updated = false;

        // Process all available events (non-blocking)
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                TaskEvent::Started { task_id } => {
                    log::info!("Task started: {}", task_id);
                }
                TaskEvent::Output { task_id, line } => {
                    if !line.is_empty() {
                        self.task_outputs
                            .entry(task_id.clone())
                            .or_insert_with(Vec::new)
                            .push(line.clone());
                        
                        // Track in session
                        self.session.add_output(&task_id, line);
                        session_updated = true;
                    }
                }
                TaskEvent::Completed { task_id, exit_code } => {
                    log::info!("Task completed: {} (exit: {})", task_id, exit_code);
                    let _ = self.scheduler.mark_done(&task_id);
                    
                    // Track in session
                    self.session.end_task(&task_id, TaskStatus::Done, Some(exit_code));
                    session_updated = true;
                }
                TaskEvent::Failed { task_id, error } => {
                    log::warn!("Task failed: {} - {}", task_id, error);
                    let _ = self.scheduler.mark_failed(&task_id);
                    
                    // Track in session
                    self.session.end_task(&task_id, TaskStatus::Failed, None);
                    session_updated = true;
                }
            }
        }

        // Save session if updated
        if session_updated {
            if let Err(e) = self.session.save() {
                log::warn!("Failed to save session: {}", e);
            }
        }

        self.last_update = Instant::now();
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('r') => {
                // Refresh / restart ready tasks
                log::info!("Manual refresh requested");
            }
            KeyCode::Up => {
                if self.selected_task > 0 {
                    self.selected_task -= 1;
                }
            }
            KeyCode::Down => {
                let task_count = self.scheduler.graph().all_tasks().len();
                if self.selected_task + 1 < task_count {
                    self.selected_task += 1;
                }
            }
            _ => {}
        }
    }

    /// Check if we should poll for input
    pub fn should_poll_input() -> Result<bool> {
        Ok(event::poll(Duration::from_millis(100))?)
    }

    /// Get keyboard event
    pub fn read_event() -> Result<Event> {
        Ok(event::read()?)
    }

    /// Get task output lines (last N)
    pub fn get_task_output(&self, task_id: &str, last_n: usize) -> Vec<String> {
        self.task_outputs
            .get(task_id)
            .map(|lines| {
                let start = lines.len().saturating_sub(last_n);
                lines[start..].to_vec()
            })
            .unwrap_or_default()
    }

    /// Get all task IDs sorted
    pub fn get_task_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.scheduler.graph().all_tasks().keys().cloned().collect();
        ids.sort();
        ids
    }

    /// Extract project name from namespaced task ID
    /// "agentverse:backend-dev" -> "agentverse"
    pub fn get_project_name(&self, task_id: &str) -> Option<String> {
        if self.workspace_mode {
            task_id.split(':').next().map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Get tasks grouped by project (for workspace mode)
    pub fn get_tasks_by_project(&self) -> HashMap<String, Vec<String>> {
        let mut grouped: HashMap<String, Vec<String>> = HashMap::new();

        if self.workspace_mode {
            for task_id in self.get_task_ids() {
                if let Some(project) = self.get_project_name(&task_id) {
                    grouped
                        .entry(project)
                        .or_insert_with(Vec::new)
                        .push(task_id);
                }
            }
        } else {
            // Single project mode - group everything under one key
            let project = self.session.project.clone();
            grouped.insert(project, self.get_task_ids());
        }

        grouped
    }
}
