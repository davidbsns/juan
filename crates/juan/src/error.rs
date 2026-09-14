use juan_parser::ParserError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JuanError {
    #[error("Parser Error: {0}")]
    ParserError(#[from] ParserError),
}
