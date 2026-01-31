//! ML Training output parser

use crate::semantic::{MetricValue, OutputParser, ParsedMetrics, TaskMetrics};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Parser for ML training output
pub struct MLTrainingParser {
    epoch_regex: Regex,
    loss_regex: Regex,
    accuracy_regex: Regex,
    lr_regex: Regex,
}

impl MLTrainingParser {
    /// Create a new ML training parser
    pub fn new() -> Self {
        Self {
            epoch_regex: Regex::new(r"(?i)epoch\s*(\d+)/(\d+)").unwrap(),
            loss_regex: Regex::new(r"(?i)loss:\s*([\d.]+)").unwrap(),
            accuracy_regex: Regex::new(r"(?i)(?:acc|accuracy):\s*([\d.]+)").unwrap(),
            lr_regex: Regex::new(r"(?i)(?:lr|learning.?rate):\s*([\d.e-]+)").unwrap(),
        }
    }
    
    /// Extract epoch progress
    fn extract_epoch(&self, output: &str) -> Option<(i64, i64)> {
        for line in output.lines().rev() {
            if let Some(captures) = self.epoch_regex.captures(line) {
                let current = captures.get(1)?.as_str().parse::<i64>().ok()?;
                let total = captures.get(2)?.as_str().parse::<i64>().ok()?;
                return Some((current, total));
            }
        }
        None
    }
    
    /// Extract loss value
    fn extract_loss(&self, output: &str) -> Option<f64> {
        for line in output.lines().rev() {
            if let Some(captures) = self.loss_regex.captures(line) {
                return captures.get(1)?.as_str().parse::<f64>().ok();
            }
        }
        None
    }
    
    /// Extract accuracy value
    fn extract_accuracy(&self, output: &str) -> Option<f64> {
        for line in output.lines().rev() {
            if let Some(captures) = self.accuracy_regex.captures(line) {
                return captures.get(1)?.as_str().parse::<f64>().ok();
            }
        }
        None
    }
    
    /// Extract learning rate
    fn extract_lr(&self, output: &str) -> Option<f64> {
        for line in output.lines().rev() {
            if let Some(captures) = self.lr_regex.captures(line) {
                return captures.get(1)?.as_str().parse::<f64>().ok();
            }
        }
        None
    }
}

impl Default for MLTrainingParser {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputParser for MLTrainingParser {
    fn name(&self) -> &str {
        "ml_training"
    }
    
    fn parse(&self, output: &str) -> Result<ParsedMetrics> {
        let mut metrics = HashMap::new();
        
        // Extract epoch progress
        let progress = if let Some((current, total)) = self.extract_epoch(output) {
            metrics.insert("epoch".to_string(), MetricValue::Int(current));
            metrics.insert("total_epochs".to_string(), MetricValue::Int(total));
            
            if total > 0 {
                current as f32 / total as f32
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        // Extract loss
        if let Some(loss) = self.extract_loss(output) {
            metrics.insert("loss".to_string(), MetricValue::Float(loss));
        }
        
        // Extract accuracy
        if let Some(acc) = self.extract_accuracy(output) {
            metrics.insert("accuracy".to_string(), MetricValue::Float(acc));
        }
        
        // Extract learning rate
        if let Some(lr) = self.extract_lr(output) {
            metrics.insert("learning_rate".to_string(), MetricValue::Float(lr));
        }
        
        // Detect phase
        let phase = if output.contains("Validating") || output.contains("Validation") {
            Some("Validation".to_string())
        } else if output.contains("Testing") || output.contains("Test") {
            Some("Testing".to_string())
        } else if output.contains("Training") || output.contains("Epoch") {
            Some("Training".to_string())
        } else {
            None
        };
        
        // Detect errors
        let mut errors = Vec::new();
        for line in output.lines() {
            if line.contains("NaN") || line.contains("nan") {
                errors.push("Loss is NaN - training diverged".to_string());
            }
            if line.contains("CUDA out of memory") {
                errors.push("Out of GPU memory".to_string());
            }
            if line.to_lowercase().contains("error:") {
                errors.push(line.to_string());
            }
        }
        
        Ok(TaskMetrics {
            progress,
            metrics,
            phase,
            errors,
        })
    }
    
    fn can_parse(&self, output: &str) -> bool {
        // Can parse if it looks like ML training output
        self.extract_epoch(output).is_some()
            || self.extract_loss(output).is_some()
            || output.to_lowercase().contains("epoch")
    }
    
    fn supported_types(&self) -> Vec<&str> {
        vec!["ml_training", "deep_learning", "training"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pytorch_output() {
        let parser = MLTrainingParser::new();
        
        let output = r#"
        Epoch 45/100
        Train Loss: 0.234 | Train Acc: 0.876
        Valid Loss: 0.245 | Valid Acc: 0.865
        Learning Rate: 0.001
        "#;
        
        let metrics = parser.parse(output).unwrap();
        
        assert_eq!(metrics.progress, 0.45);
        assert_eq!(metrics.metrics["epoch"].as_int(), Some(45));
        assert_eq!(metrics.metrics["total_epochs"].as_int(), Some(100));
        // Parser extracts the last loss/acc (Valid Loss in this case)
        assert_eq!(metrics.metrics["loss"].as_float(), Some(0.245));
        assert_eq!(metrics.metrics["accuracy"].as_float(), Some(0.865));
        assert_eq!(metrics.metrics["learning_rate"].as_float(), Some(0.001));
    }
    
    #[test]
    fn test_tensorflow_output() {
        let parser = MLTrainingParser::new();
        
        let output = "Epoch 10/50 - loss: 0.567 - acc: 0.789";
        
        let metrics = parser.parse(output).unwrap();
        
        assert_eq!(metrics.progress, 0.20);
        assert_eq!(metrics.metrics["loss"].as_float(), Some(0.567));
        assert_eq!(metrics.metrics["accuracy"].as_float(), Some(0.789));
    }
    
    #[test]
    fn test_error_detection() {
        let parser = MLTrainingParser::new();
        
        let output = "Epoch 5/10 | Loss: NaN";
        
        let metrics = parser.parse(output).unwrap();
        
        assert!(!metrics.errors.is_empty());
        assert!(metrics.errors[0].contains("NaN"));
    }
    
    #[test]
    fn test_phase_detection() {
        let parser = MLTrainingParser::new();
        
        let output1 = "Training Epoch 1/10";
        let metrics1 = parser.parse(output1).unwrap();
        assert_eq!(metrics1.phase, Some("Training".to_string()));
        
        let output2 = "Validating...";
        let metrics2 = parser.parse(output2).unwrap();
        assert_eq!(metrics2.phase, Some("Validation".to_string()));
    }
}
