use thiserror::Error;

#[derive(Debug, Error)]
pub enum DraftError {
    #[error("claude CLI not found: {0}")]
    CliMissing(String),
    #[error("claude CLI timed out after {0}s")]
    Timeout(u64),
    #[error("claude CLI exited with status {status}: {stderr}")]
    CliFailed { status: i32, stderr: String },
    #[error("gate output unparseable: {0}")]
    GateUnparseable(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}