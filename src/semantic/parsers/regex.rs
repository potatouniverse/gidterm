//! Regex-based output parser

use crate::semantic::{MetricValue, OutputParser, ParsedMetrics, TaskMetrics};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Generic regex-based parser
pub struct RegexParser {
    name: String,
    patterns: ParserPatterns,
}

/// Parser patterns for different metrics
#[derive(Clone)]
pub struct ParserPatterns {
    /// Progress patterns (e.g., "45/100", "45%", "[====>  ] 45%")
    pub progress: Vec<ProgressPattern>,
    
    /// Named metric patterns (e.g., "Loss: 0.234")
    pub metrics: Vec<MetricPattern>,
    
    /// Phase/stage patterns (e.g., "Phase: Training")
    pub phase: Option<Regex>,
    
    /// Error patterns (e.g., "Error:", "Failed:")
    pub errors: Vec<Regex>,
}

#[derive(Clone)]
pub struct ProgressPattern {
    pub regex: Regex,
    pub current_group: usize,
    pub total_group: Option<usize>,
}

#[derive(Clone)]
pub struct MetricPattern {
    pub name: String,
    pub regex: Regex,
    pub value_group: usize,
    pub value_type: MetricType,
}

#[derive(Clone)]
pub enum MetricType {
    Float,
    Int,
    String,
}

impl RegexParser {
    /// Create a new regex parser
    pub fn new(name: impl Into<String>, patterns: ParserPatterns) -> Self {
        Self {
            name: name.into(),
            patterns,
        }
    }
    
    /// Create a default parser with common patterns
    pub fn default_parser() -> Self {
        Self::new("regex", ParserPatterns::default())
    }
    
    /// Extract progress from output
    fn extract_progress(&self, output: &str) -> Option<f32> {
        for pattern in &self.patterns.progress {
            if let Some(captures) = pattern.regex.captures(output) {
                let current = captures
                    .get(pattern.current_group)
                    .and_then(|m| m.as_str().parse::<f32>().ok())?;
                
                if let Some(total_group) = pattern.total_group {
                    let total = captures
                        .get(total_group)
                        .and_then(|m| m.as_str().parse::<f32>().ok())?;
                    
                    if total > 0.0 {
                        return Some(current / total);
                    }
                } else {
                    // Assume current is already a percentage (0-100)
                    return Some(current / 100.0);
                }
            }
        }
        
        None
    }
    
    /// Extract metrics from output
    fn extract_metrics(&self, output: &str) -> HashMap<String, MetricValue> {
        let mut metrics = HashMap::new();
        
        for pattern in &self.patterns.metrics {
            if let Some(captures) = pattern.regex.captures(output) {
                if let Some(value_match) = captures.get(pattern.value_group) {
                    let value_str = value_match.as_str();
                    
                    let value = match pattern.value_type {
                        MetricType::Float => {
                            value_str.parse::<f64>().ok().map(MetricValue::Float)
                        }
                        MetricType::Int => {
                            value_str.parse::<i64>().ok().map(MetricValue::Int)
                        }
                        MetricType::String => Some(MetricValue::String(value_str.to_string())),
                    };
                    
                    if let Some(value) = value {
                        metrics.insert(pattern.name.clone(), value);
                    }
                }
            }
        }
        
        metrics
    }
    
    /// Extract phase/stage from output
    fn extract_phase(&self, output: &str) -> Option<String> {
        if let Some(ref phase_regex) = self.patterns.phase {
            if let Some(captures) = phase_regex.captures(output) {
                return captures.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }
    
    /// Extract errors from output
    fn extract_errors(&self, output: &str) -> Vec<String> {
        let mut errors = Vec::new();
        
        for error_regex in &self.patterns.errors {
            for line in output.lines() {
                if error_regex.is_match(line) {
                    errors.push(line.to_string());
                }
            }
        }
        
        errors
    }
}

impl OutputParser for RegexParser {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn parse(&self, output: &str) -> Result<ParsedMetrics> {
        let progress = self.extract_progress(output).unwrap_or(0.0);
        let metrics = self.extract_metrics(output);
        let phase = self.extract_phase(output);
        let errors = self.extract_errors(output);
        
        Ok(TaskMetrics {
            progress,
            metrics,
            phase,
            errors,
        })
    }
    
    fn can_parse(&self, output: &str) -> bool {
        // Can parse if any pattern matches
        self.extract_progress(output).is_some()
            || !self.extract_metrics(output).is_empty()
            || self.extract_phase(output).is_some()
    }
    
    fn supported_types(&self) -> Vec<&str> {
        vec!["generic", "build", "test", "data_processing"]
    }
}

impl Default for ParserPatterns {
    fn default() -> Self {
        Self {
            progress: vec![
                // "45/100"
                ProgressPattern {
                    regex: Regex::new(r"(\d+)/(\d+)").unwrap(),
                    current_group: 1,
                    total_group: Some(2),
                },
                // "45%"
                ProgressPattern {
                    regex: Regex::new(r"(\d+)%").unwrap(),
                    current_group: 1,
                    total_group: None,
                },
                // "[====>   ] 45%"
                ProgressPattern {
                    regex: Regex::new(r"\[=+>\s+\]\s*(\d+)%").unwrap(),
                    current_group: 1,
                    total_group: None,
                },
            ],
            
            metrics: vec![],
            phase: Some(Regex::new(r"(?:Phase|Stage):\s*(\w+)").unwrap()),
            errors: vec![
                Regex::new(r"(?i)error:").unwrap(),
                Regex::new(r"(?i)failed").unwrap(),
                Regex::new(r"(?i)exception").unwrap(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_progress_extraction() {
        let parser = RegexParser::default_parser();
        
        // Test "45/100" format
        let output1 = "Processing 45/100 files...";
        let metrics1 = parser.parse(output1).unwrap();
        assert_eq!(metrics1.progress, 0.45);
        
        // Test "45%" format
        let output2 = "Progress: 45%";
        let metrics2 = parser.parse(output2).unwrap();
        assert_eq!(metrics2.progress, 0.45);
        
        // Test progress bar format
        let output3 = "[=====>   ] 45%";
        let metrics3 = parser.parse(output3).unwrap();
        assert_eq!(metrics3.progress, 0.45);
    }
    
    #[test]
    fn test_error_extraction() {
        let parser = RegexParser::default_parser();
        
        let output = "Processing...\nError: File not found\nContinuing...";
        let metrics = parser.parse(output).unwrap();
        
        assert_eq!(metrics.errors.len(), 1);
        assert!(metrics.errors[0].contains("Error"));
    }
    
    #[test]
    fn test_custom_metrics() {
        let mut patterns = ParserPatterns::default();
        
        // Add custom metric pattern for "Loss: 0.234"
        patterns.metrics.push(MetricPattern {
            name: "loss".to_string(),
            regex: Regex::new(r"Loss:\s*([\d.]+)").unwrap(),
            value_group: 1,
            value_type: MetricType::Float,
        });
        
        let parser = RegexParser::new("ml", patterns);
        
        let output = "Epoch 10/100 | Loss: 0.234";
        let metrics = parser.parse(output).unwrap();
        
        assert_eq!(metrics.progress, 0.10);
        assert!(metrics.metrics.contains_key("loss"));
        assert_eq!(metrics.metrics["loss"].as_float(), Some(0.234));
    }
}
