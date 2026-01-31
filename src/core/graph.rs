//! Graph parser - parses .gid/graph.yml and builds task DAG

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Task graph representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub metadata: Option<Metadata>,
    pub nodes: HashMap<String, Node>,
    pub tasks: HashMap<String, Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub project: String,
    pub version: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    #[serde(rename = "type")]
    pub node_type: String,
    pub description: String,
    pub layer: Option<String>,
    pub status: String,
    pub priority: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    #[serde(rename = "type")]
    pub task_type: String,
    pub description: String,
    pub command: Option<String>,
    pub status: String,
    pub priority: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub component: Option<String>,
    pub estimated_hours: Option<u32>,
    pub tags: Option<Vec<String>>,
}

impl Graph {
    /// Load graph from YAML file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let graph: Graph = serde_yaml::from_str(&content)?;
        Ok(graph)
    }

    /// Get all tasks ready to run (dependencies met)
    pub fn get_ready_tasks(&self) -> Vec<String> {
        // TODO: Implement DAG traversal
        Vec::new()
    }

    /// Check if a task can start
    pub fn can_start(&self, _task_id: &str) -> bool {
        // TODO: Check dependencies
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_graph() {
        // TODO: Add test
    }
}
