/*
* Reference Text:
* module main
*
* import std.io
*
* fn main() {
*   io.println("Hello, World")
* }*/

// Maybe I should make this it's own crate? esp with building

use juan_span::Span;
use phf::phf_map;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    Fn,
    Module,
    Import,

    Dot,
    LParen,
    RParen,
    LBrace,
    RBrace,

    Identifier,
    Str,
    UnterminatedStr,
    Int,
    Float,

    Plus,
    Minus,
    Star,
    Slash,

    Eof,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

pub static KEYWORDS: phf::Map<&'static [u8], TokenKind> = phf_map! {
    b"fn" => TokenKind::Fn,
    b"module" => TokenKind::Module,
    b"import" => TokenKind::Import,
};
