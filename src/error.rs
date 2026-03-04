#![allow(dead_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BioError {
    #[error("Skill not found: {0}")]
    SkillNotFound(String),

    #[error("Skill execution failed: {0}")]
    SkillExecutionFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Intent recognition failed: {0}")]
    IntentRecognitionFailed(String),

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("Sandbox error: {0}")]
    SandboxError(String),

    #[error("Reproducibility error: {0}")]
    ReproducibilityError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type BioResult<T> = Result<T, BioError>;
