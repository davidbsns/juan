use juan_lexer::{
    Lexer,
    tokens::{Token, TokenKind},
};
use juan_span::Span;
use la_arena::{Arena, Idx};
use thiserror::Error;

use crate::node::{Expr, Literal, Node, Op};

mod node;

// TODO: movee into separate file once big enough
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Unexpected Primary Token: {0:?}")]
    UnexpectedPrimaryToken(TokenKind),
}

pub struct Parser<'a> {
    lexer: &'a mut Lexer<'a>,
    current_token: Token,
    tree: Arena<Node>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer<'a>) -> Self {
        let current_token = lexer.next_token();

        Self {
            lexer,
            current_token,
            tree: Arena::new(),
        }
    }

    pub fn parse(&mut self) -> Result<(), ParserError> {
        self.parse_expression()
    }

    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    fn parse_expression(&mut self) -> Result<(), ParserError> {
        let left = self.parse_primary()?;

        let mut op = Op::Add;
        match self.current_token.kind {
            TokenKind::Plus => {
                op = Op::Add;
                self.advance();
            }

            _ => (),
        }

        let right = self.parse_primary()?;

        let binary_op = Expr::BinaryOp { op, left, right };

        let (start, _) = self.tree[left].span.unpack_usize();
        let (_, end) = self.tree[right].span.unpack_usize();

        let node = Node {
            expr: binary_op,
            span: Span::new(start, end),
        };

        self.tree.alloc(node);
        self.advance();

        println!("{:?}", self.tree);

        Ok(())
    }

    fn parse_primary(&mut self) -> Result<Idx<Node>, ParserError> {
        match self.current_token.kind {
            TokenKind::Int => {
                let (start, len) = self.current_token.span.unpack_usize();
                let str = &self.lexer.src[start..start + len];
                // We can be certain it's valid UTF-8 & i64. (I think)
                let str = std::str::from_utf8(str).expect("invalid UTF-8");
                let literal: i64 = str.parse().expect("invalid i64");

                let node = Node {
                    expr: Expr::Literal(Literal::Int(literal)),
                    span: self.current_token.span,
                };

                let idx = self.tree.alloc(node);
                self.advance();

                Ok(idx)
            }

            _ => Err(ParserError::UnexpectedPrimaryToken(
                self.current_token.kind.clone(),
            )),
        }
    }
}
