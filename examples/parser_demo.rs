//! Parser demo - showing how the semantic layer works

use gidterm::semantic::{parsers::*, ParserRegistry};

fn main() {
    println!("🎯 GidTerm Parser Demo\n");
    
    // Create parser registry
    let mut registry = ParserRegistry::new();
    
    // Register parsers
    registry.register(Box::new(RegexParser::default_parser()));
    registry.register(Box::new(MLTrainingParser::new()));
    
    println!("📋 Registered parsers: {:?}\n", registry.list_parsers());
    
    // Demo 1: Generic progress parsing
    demo_generic_progress(&registry);
    
    // Demo 2: ML training parsing
    demo_ml_training(&registry);
    
    // Demo 3: Error detection
    demo_error_detection(&registry);
}

fn demo_generic_progress(registry: &ParserRegistry) {
    println!("═══ Demo 1: Generic Progress Parsing ═══\n");
    
    let outputs = vec![
        "Processing files 45/100...",
        "Download progress: 67%",
        "[=========>  ] 85%",
    ];
    
    for output in outputs {
        let result = registry.parse(None, output).unwrap();
        println!("Input:  {}", output);
        println!("Progress: {:.0}%", result.progress * 100.0);
        println!();
    }
}

fn demo_ml_training(registry: &ParserRegistry) {
    println!("═══ Demo 2: ML Training Parsing ═══\n");
    
    let output = r#"
Epoch 45/100
Train Loss: 0.234 | Train Acc: 0.876
Valid Loss: 0.245 | Valid Acc: 0.865
Learning Rate: 0.001
    "#;
    
    let result = registry.parse(Some("ml_training"), output).unwrap();
    
    println!("Input:\n{}", output);
    println!("Parsed Metrics:");
    println!("  Progress: {:.0}%", result.progress * 100.0);
    println!("  Epoch: {}/{}", 
             result.metrics.get("epoch").and_then(|v| v.as_int()).unwrap_or(0),
             result.metrics.get("total_epochs").and_then(|v| v.as_int()).unwrap_or(0));
    println!("  Loss: {:.3}", 
             result.metrics.get("loss").and_then(|v| v.as_float()).unwrap_or(0.0));
    println!("  Accuracy: {:.1}%", 
             result.metrics.get("accuracy").and_then(|v| v.as_float()).unwrap_or(0.0) * 100.0);
    println!("  Learning Rate: {:.4}", 
             result.metrics.get("learning_rate").and_then(|v| v.as_float()).unwrap_or(0.0));
    println!("  Phase: {:?}", result.phase);
    println!();
}

fn demo_error_detection(registry: &ParserRegistry) {
    println!("═══ Demo 3: Error Detection ═══\n");
    
    let output = r#"
Epoch 10/50
Train Loss: NaN | Train Acc: 0.789
Error: CUDA out of memory
    "#;
    
    let result = registry.parse(Some("ml_training"), output).unwrap();
    
    println!("Input:\n{}", output);
    println!("Detected Errors:");
    for error in &result.errors {
        println!("  ⚠️  {}", error);
    }
    println!();
}
