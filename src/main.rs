//! GidTerm CLI entry point

use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    println!("🚀 GidTerm v{}", env!("CARGO_PKG_VERSION"));
    println!("Graph-Driven Semantic Terminal Controller");
    println!();

    // TODO: Parse CLI args
    // TODO: Load graph
    // TODO: Start TUI

    Ok(())
}
