use juan_lexer::tokens::TokenKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Unexpected Primary Token: {0:?}")]
    UnexpectedPrimaryToken(TokenKind),

    #[error("Unexpected Token To Parse: {0:?}")]
    UnexpectedTokenParsed(TokenKind),

    #[error("Unexpected Token gotten: {0:?}, wanted: {0:?}")]
    UnexpectedTokenParsedAndWanted(TokenKind, TokenKind),
}
