use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Standard IO error: {0:?}")]
    StdIo(#[from] std::io::Error),
}
