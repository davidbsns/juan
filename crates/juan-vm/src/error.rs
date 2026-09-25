use std::array::TryFromSliceError;

use juan_bytecode::Opcode;
use num_enum::TryFromPrimitiveError;
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

    #[error("Try from primitive opcode error: {0}")]
    TryFromPrimitiveOpcode(#[from] TryFromPrimitiveError<Opcode>),
}
