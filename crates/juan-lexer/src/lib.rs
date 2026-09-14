pub mod tokens;

use juan_span::Span;

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
            return Token::new(TokenKind::Eof, Span::new(self.cursor, self.cursor));
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
        let start = self.cursor;
        self.cursor += 1;

        Token::new(kind, Span::new(start, self.cursor))
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
        let start = self.cursor;
        let mut end = self.cursor + 1;
        while end < self.text.len() {
            let byte = self.text[end];

            if byte == b'"' {
                break;
            }

            end += 1;
        }

        if end < self.text.len() {
            self.cursor = end + 1;
            Token::new(TokenKind::Str, Span::new(start, end + 1))
        } else {
            self.cursor = end;
            Token::new(TokenKind::UnterminatedStr, Span::new(start, end))
        }
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.cursor;
        let mut end = self.cursor;
        while end < self.text.len() {
            let byte = self.text[end];
            if !byte.is_ascii_alphanumeric() && byte != b'_' {
                break;
            }

            end += 1;
        }

        let ident = self.text.get(start..end);
        let Some(ident) = ident else {
            return Token::new(TokenKind::Eof, Span::new(start, end));
        };

        self.cursor = end;

        if let Some(kind) = KEYWORDS.get(ident) {
            Token::new(kind.clone(), Span::new(start, end))
        } else {
            Token::new(TokenKind::Identifier, Span::new(start, end))
        }
    }
}
