pub mod tokens;

use crate::tokens::KEYWORDS;
use crate::tokens::Token;
use crate::tokens::TokenKind;

pub struct Lexer<'a> {
    text: &'a [u8],
    cursor: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            text: input.as_bytes(),
            cursor: 0,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let current_byte = self.text.get(self.cursor);
        let Some(current_byte) = current_byte else {
            return Token(TokenKind::Eof);
        };

        // Single character here, read_identifier manages the rest
        match current_byte {
            b'.' => self.advance(TokenKind::Dot),
            b'(' => self.advance(TokenKind::LParen),
            b')' => self.advance(TokenKind::RParen),
            b'{' => self.advance(TokenKind::LBrace),
            b'}' => self.advance(TokenKind::RBrace),

            b'"' => self.read_string(),
            _ => self.read_identifier(),
        }
    }

    fn advance(&mut self, kind: TokenKind) -> Token {
        self.cursor += 1;
        Token(kind)
    }

    fn skip_whitespace(&mut self) {
        while let Some(byte) = self.text.get(self.cursor) {
            if byte.is_ascii_whitespace() {
                self.cursor += 1;
            } else {
                break;
            }
        }
    }

    fn read_string(&mut self) -> Token {
        let mut end = self.cursor + 1;
        while end < self.text.len() {
            let byte = self.text[end];

            if byte == b'"' {
                break;
            }

            end += 1;
        }

        let str = &self.text[self.cursor..end];
        self.cursor = end + 1;

        // TODO: store a span
        Token(TokenKind::Str(String::from_utf8_lossy(str).to_string()))
    }

    fn read_identifier(&mut self) -> Token {
        let mut end = self.cursor;
        while end < self.text.len() {
            let byte = self.text[end];
            if !byte.is_ascii_alphanumeric() && byte != b'_' {
                break;
            }

            end += 1;
        }

        let ident = self.text.get(self.cursor..end);
        let Some(ident) = ident else {
            return Token(TokenKind::Eof);
        };

        self.cursor = end;

        if let Some(kind) = KEYWORDS.get(ident) {
            Token(kind.clone())
        } else {
            Token(TokenKind::Identifier(
                String::from_utf8_lossy(ident).to_string(),
            ))
        }
    }
}
