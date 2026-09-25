use juan_ast::{Expr, Literal, Node, NodeId, Op, SyntaxTree};
use juan_lexer::{
    Lexer,
    tokens::{Token, TokenKind},
};
use juan_span::Span;

use crate::error::ParserError;

pub mod error;

pub struct Parser<'a> {
    lexer: &'a mut Lexer<'a>,
    current_token: Token,
    tree: SyntaxTree,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer<'a>) -> Self {
        let current_token = lexer.next_token();

        Self {
            lexer,
            current_token,
            tree: SyntaxTree::new(),
        }
    }

    pub fn tree(self) -> SyntaxTree {
        self.tree
    }

    pub fn parse(&mut self) -> Result<(), ParserError> {
        self.parse_module()?;

        loop {
            match self.current_token.kind {
                TokenKind::Int => self.parse_term(),
                TokenKind::LParen => self.parse_term(),

                TokenKind::Eof => break,

                _ => {
                    return Err(ParserError::UnexpectedToken(
                        self.current_token.kind.clone(),
                    ));
                }
            }?;
        }

        Ok(())
    }

    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParserError> {
        let token = self.current_token.clone();
        if token.kind != kind {
            return Err(ParserError::TokenMismatch(token.kind, kind));
        }

        self.advance();

        Ok(token)
    }

    fn parse_module(&mut self) -> Result<(), ParserError> {
        self.expect(TokenKind::Module)?;

        let token = self.expect(TokenKind::Identifier)?;

        let (start, len) = token.span.unpack();
        let mut end = start + len;

        while self.current_token.kind == TokenKind::Dot {
            self.advance();

            let token = self.expect(TokenKind::Identifier)?;

            let (start_offset, len_offset) = token.span.unpack::<usize>();
            end = start_offset + len_offset;
        }

        let node = Node {
            expr: Expr::Module,
            span: Span::new(start, end),
        };

        self.tree.alloc(node);

        Ok(())
    }

    fn allocate(&mut self, left: NodeId, right: NodeId, op: Op) -> NodeId {
        let (left_start, _) = self.tree[left].span.unpack();
        let (right_start, len) = self.tree[right].span.unpack::<usize>();

        self.tree.alloc(Node {
            expr: Expr::BinaryOp { op, left, right },
            span: Span::new(left_start, right_start + len),
        })
    }

    fn parse_factor(&mut self) -> Result<NodeId, ParserError> {
        let mut left = self.parse_primary()?;

        while let Some(op) = match self.current_token.kind {
            TokenKind::Star => Some(Op::Mul),
            TokenKind::Slash => Some(Op::Div),
            _ => None,
        } {
            self.advance();

            let right = self.parse_primary()?;
            left = self.allocate(left, right, op);
        }

        println!("{:?}", self.tree);

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<NodeId, ParserError> {
        let mut left = self.parse_factor()?;

        while let Some(op) = match self.current_token.kind {
            TokenKind::Plus => Some(Op::Add),
            TokenKind::Minus => Some(Op::Sub),
            _ => None,
        } {
            self.advance();

            let right = self.parse_factor()?;
            left = self.allocate(left, right, op);
        }

        println!("{:?}", self.tree);

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<NodeId, ParserError> {
        match self.current_token.kind {
            TokenKind::Int => {
                let (start, len) = self.current_token.span.unpack();
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

            TokenKind::LParen => {
                self.advance();

                let idx = self.parse_term()?;

                self.expect(TokenKind::RParen)?;

                Ok(idx)
            }

            _ => Err(ParserError::UnexpectedToken(
                self.current_token.kind.clone(),
            )),
        }
    }
}
