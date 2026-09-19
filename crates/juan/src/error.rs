use juan_compiler::error::CompilerError;
use juan_parser::ParserError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JuanError {
    #[error("Parser Error: {0}")]
    Parser(#[from] ParserError),

    #[error("Compiler Error: {0}")]
    Compiler(#[from] CompilerError),
}
