use thiserror::Error;

/// Errors that can occur during transaction processing
/// These are system-level errors (I/O, parsing), not business logic violations
#[derive(Error, Debug)]
pub enum EngineError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV parsing error: {0}")]
    Csv(#[from] csv::Error),
}

pub type Result<T> = std::result::Result<T, EngineError>;
