//! Parser Registry - Register and manage output parsers

use super::TaskMetrics;
use anyhow::Result;
use std::collections::HashMap;

/// Parsed metrics from output
pub type ParsedMetrics = TaskMetrics;

/// Trait for output parsers
pub trait OutputParser: Send + Sync {
    /// Parser name/identifier
    fn name(&self) -> &str;
    
    /// Parse output and extract metrics
    fn parse(&self, output: &str) -> Result<ParsedMetrics>;
    
    /// Check if this parser can handle the output
    fn can_parse(&self, output: &str) -> bool;
    
    /// Get supported task types
    fn supported_types(&self) -> Vec<&str>;
}

/// Parser registry for managing multiple parsers
pub struct ParserRegistry {
    parsers: HashMap<String, Box<dyn OutputParser>>,
    type_mappings: HashMap<String, String>, // task_type -> parser_name
}

impl ParserRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
            type_mappings: HashMap::new(),
        }
    }
    
    /// Register a parser
    pub fn register(&mut self, parser: Box<dyn OutputParser>) {
        let name = parser.name().to_string();
        
        // Map supported types to this parser
        for task_type in parser.supported_types() {
            self.type_mappings.insert(task_type.to_string(), name.clone());
        }
        
        self.parsers.insert(name, parser);
    }
    
    /// Get parser by name
    pub fn get(&self, name: &str) -> Option<&dyn OutputParser> {
        self.parsers.get(name).map(|p| p.as_ref())
    }
    
    /// Get parser for task type
    pub fn get_for_type(&self, task_type: &str) -> Option<&dyn OutputParser> {
        let parser_name = self.type_mappings.get(task_type)?;
        self.get(parser_name)
    }
    
    /// Find a parser that can handle the output
    pub fn find_parser(&self, output: &str) -> Option<&dyn OutputParser> {
        for parser in self.parsers.values() {
            if parser.can_parse(output) {
                return Some(parser.as_ref());
            }
        }
        None
    }
    
    /// Parse output with appropriate parser
    pub fn parse(&self, task_type: Option<&str>, output: &str) -> Result<ParsedMetrics> {
        // Try task type mapping first
        if let Some(task_type) = task_type {
            if let Some(parser) = self.get_for_type(task_type) {
                return parser.parse(output);
            }
        }
        
        // Fall back to auto-detection
        if let Some(parser) = self.find_parser(output) {
            return parser.parse(output);
        }
        
        // No parser found, return default
        Ok(ParsedMetrics {
            progress: 0.0,
            metrics: HashMap::new(),
            phase: None,
            errors: vec!["No suitable parser found".to_string()],
        })
    }
    
    /// List all registered parsers
    pub fn list_parsers(&self) -> Vec<&str> {
        self.parsers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestParser;
    
    impl OutputParser for TestParser {
        fn name(&self) -> &str {
            "test"
        }
        
        fn parse(&self, _output: &str) -> Result<ParsedMetrics> {
            Ok(ParsedMetrics {
                progress: 0.5,
                metrics: HashMap::new(),
                phase: None,
                errors: vec![],
            })
        }
        
        fn can_parse(&self, output: &str) -> bool {
            output.contains("test")
        }
        
        fn supported_types(&self) -> Vec<&str> {
            vec!["test_task"]
        }
    }
    
    #[test]
    fn test_registry() {
        let mut registry = ParserRegistry::new();
        registry.register(Box::new(TestParser));
        
        assert_eq!(registry.list_parsers(), vec!["test"]);
        assert!(registry.get("test").is_some());
        assert!(registry.get_for_type("test_task").is_some());
    }
    
    #[test]
    fn test_parse() {
        let mut registry = ParserRegistry::new();
        registry.register(Box::new(TestParser));
        
        let result = registry.parse(Some("test_task"), "test output").unwrap();
        assert_eq!(result.progress, 0.5);
    }
}
