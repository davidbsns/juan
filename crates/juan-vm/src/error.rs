use std::array::TryFromSliceError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum VMError {
    #[error("Chunk already exists: {0}")]
    ChunkAlreadyExists(String),

    #[error("Chunk doesn't exist: {0}")]
    ChunkDoesntExist(String),

    #[error("Invalid Opcode: {0}")]
    InvalidOpcode(u8),

    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Integer overflow: {0} {2} {1}")]
    IntegerOverflow(i32, i32, char),

    #[error("Unexpected End of Bytecode: {0}")]
    UnexpectedEndOfBytecode(usize),

    #[error("Try from slice error: {0}")]
    TryFromSlice(#[from] TryFromSliceError),
}
