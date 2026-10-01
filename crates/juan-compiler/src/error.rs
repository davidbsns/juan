use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Standard IO error: {0:?}")]
    StdIo(#[from] std::io::Error),

    #[error("Duplicate Parameter name: {0}")]
    DuplicateParameterName(String),

    #[error("Duplicate Function name: {0}")]
    DuplicateFunctionName(String),

    #[error("Function {0} doesn't exist!")]
    NonExistentFunction(String),

    #[error("Function {0}: expected argument count {1}, received {2}")]
    ArgumentCountMismatch(String, u32, u32),
}
