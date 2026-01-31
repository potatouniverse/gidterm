//! Output parsers for different task types

pub mod regex;
pub mod ml_training;

pub use regex::RegexParser;
pub use ml_training::MLTrainingParser;
