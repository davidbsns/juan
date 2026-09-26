use juan_lexer::tokens::TokenKind;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Unexpected Token parsed: {0:?}")]
    UnexpectedToken(TokenKind),

    #[error("Unexpected Token: {0:?}, expected: {1:?}")]
    TokenMismatch(TokenKind, TokenKind),

    #[error("Unexpected Token: {0:?}, expected one of: {1:?}")]
    ExpectedOneOf(TokenKind, Vec<TokenKind>),
}
