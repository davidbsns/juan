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

    Identifier(String),
    Str(String),

    Eof,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token(pub TokenKind);

pub static KEYWORDS: phf::Map<&'static [u8], TokenKind> = phf_map! {
    b"fn" => TokenKind::Fn,
    b"module" => TokenKind::Module,
    b"import" => TokenKind::Import,
};
