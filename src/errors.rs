use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShellError {
    #[error("Timeout")]
    Timeout,

    #[error("Failed prepare: {0}")]
    FailedPrepare(#[source] std::io::Error),

    #[error("Failed join: {0}")]
    FailedJoin(#[source] tokio::task::JoinError),

    #[error("Failed spawn: {0}")]
    FailedSpawn(#[source] std::io::Error),

    #[error("Failed output: {0}")]
    FailedOutput(#[source] std::io::Error),

    #[error("{0}")]
    Failure(String),
}

pub type Result<T> = std::result::Result<T, ShellError>;
