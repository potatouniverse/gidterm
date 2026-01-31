//! Task Scheduler - DAG-based task dependency scheduling

use super::Graph;
use anyhow::Result;

/// Task scheduler with dependency resolution
pub struct Scheduler {
    graph: Graph,
}

impl Scheduler {
    /// Create a new scheduler from graph
    pub fn new(graph: Graph) -> Self {
        Self { graph }
    }

    /// Schedule next tasks to run
    pub fn schedule_next(&mut self) -> Vec<String> {
        self.graph.get_ready_tasks()
    }

    /// Mark task as completed
    pub fn mark_done(&mut self, _task_id: &str) -> Result<()> {
        // TODO: Update graph state
        Ok(())
    }
}
