use juan_ast::{Expr, Literal, Node, NodeId, Op, SyntaxTree};
use juan_lexer::{
    Lexer,
    tokens::{Token, TokenKind},
};
use juan_span::Span;
use thiserror::Error;

// TODO: movee into separate file once big enough
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Unexpected Primary Token: {0:?}")]
    UnexpectedPrimaryToken(TokenKind),

    #[error("Unexpected Token To Parse: {0:?}")]
    UnexpectedTokenParsed(TokenKind),

    #[error("Unexpected Token gotten: {0:?}, wanted: {0:?}")]
    UnexpectedTokenParsedAndWanted(TokenKind, TokenKind),
}

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
        // Modules
        if self.current_token.kind != TokenKind::Module {
            return Err(ParserError::UnexpectedTokenParsedAndWanted(
                self.current_token.kind.clone(),
                TokenKind::Module,
            ));
        }
        self.parse_module()?;

        loop {
            match self.current_token.kind {
                TokenKind::Int => self.parse_expression(),

                TokenKind::Eof => break,

                _ => {
                    return Err(ParserError::UnexpectedTokenParsed(
                        self.current_token.kind.clone(),
                    ));
                }
            }?
        }

        Ok(())
    }

    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    fn parse_module(&mut self) -> Result<(), ParserError> {
        if self.current_token.kind != TokenKind::Module {
            return Err(ParserError::UnexpectedTokenParsedAndWanted(
                self.current_token.kind.clone(),
                TokenKind::Module,
            ));
        }

        self.advance();

        if self.current_token.kind != TokenKind::Identifier {
            return Err(ParserError::UnexpectedTokenParsedAndWanted(
                self.current_token.kind.clone(),
                TokenKind::Identifier,
            ));
        }

        let (start, len) = self.current_token.span.unpack();
        let mut end = start + len;

        self.advance();

        while self.current_token.kind == TokenKind::Dot {
            self.advance();

            if self.current_token.kind != TokenKind::Identifier {
                return Err(ParserError::UnexpectedTokenParsedAndWanted(
                    self.current_token.kind.clone(),
                    TokenKind::Identifier,
                ));
            }

            let (start_offset, len_offset) = self.current_token.span.unpack::<usize>();
            end = start_offset + len_offset;

            self.advance();
        }

        let node = Node {
            expr: Expr::Module,
            span: Span::new(start, end),
        };

        self.tree.alloc(node);

        Ok(())
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

        let (left_start, _) = self.tree[left].span.unpack();
        let (right_start, len) = self.tree[right].span.unpack::<usize>();

        let node = Node {
            expr: binary_op,
            span: Span::new(left_start, right_start + len),
        };

        self.tree.alloc(node);
        self.advance();

        println!("{:?}", self.tree);

        Ok(())
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

            _ => Err(ParserError::UnexpectedPrimaryToken(
                self.current_token.kind.clone(),
            )),
        }
    }
}
