use juan_compiler::error::CompilerError;
use juan_parser::error::ParserError;
use juan_vm::error::VMError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JuanError {
    #[error("Parser Error: {0}")]
    Parser(#[from] ParserError),

    #[error("Compiler Error: {0}")]
    Compiler(#[from] CompilerError),

    #[error("VM Error: {0}")]
    VM(#[from] VMError),
}
