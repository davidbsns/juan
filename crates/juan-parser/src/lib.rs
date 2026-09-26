use juan_ast::{Expr, Literal, ModuleDecl, Node, NodeId, Op, ParsedModule, SyntaxTree, UnaryOp};
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
    expr_roots: Vec<NodeId>,
    paren_depth: u32,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer<'a>) -> Self {
        let current_token = lexer.next_token();

        Self {
            lexer,
            current_token,
            tree: SyntaxTree::new(),
            expr_roots: Vec::new(),
            paren_depth: 0,
        }
    }

    pub fn parse(mut self) -> Result<ParsedModule, ParserError> {
        let decl = self.parse_module()?;

        // TODO: make .expect handle clean ergonomics with slices
        if matches!(self.current_token.kind, TokenKind::Newline | TokenKind::Eof) {
            self.advance();
        } else {
            return Err(ParserError::ExpectedOneOf(
                self.current_token.kind,
                vec![TokenKind::Newline, TokenKind::Eof],
            ));
        }

        loop {
            match self.current_token.kind {
                TokenKind::Int | TokenKind::LParen | TokenKind::Minus => {
                    let id = self.parse_term()?;

                    if self.current_token.kind == TokenKind::Newline {
                        self.advance();
                    } else if self.current_token.kind != TokenKind::Eof {
                        return Err(ParserError::ExpectedOneOf(
                            self.current_token.kind,
                            vec![TokenKind::Newline, TokenKind::Eof],
                        ));
                    }

                    self.expr_roots.push(id);
                    Ok(())
                }

                TokenKind::Newline => {
                    self.advance();
                    continue;
                }

                TokenKind::Eof => break,

                _ => {
                    return Err(ParserError::UnexpectedToken(
                        self.current_token.kind.clone(),
                    ));
                }
            }?;
        }

        Ok(ParsedModule {
            decl,
            tree: self.tree,
            expr_roots: self.expr_roots,
        })
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

    fn skip_newlines(&mut self) {
        loop {
            if self.current_token.kind == TokenKind::Newline {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_module(&mut self) -> Result<ModuleDecl, ParserError> {
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

        Ok(ModuleDecl {
            path_span: Span::new(start, end),
        })
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
        let mut left = self.parse_unary()?;

        loop {
            if self.paren_depth > 0 {
                self.skip_newlines();
            }

            let op = match self.current_token.kind {
                TokenKind::Star => Op::Mul,
                TokenKind::Slash => Op::Div,
                TokenKind::Percent => Op::Rem,
                _ => break,
            };

            self.advance();
            self.skip_newlines();

            let right = self.parse_unary()?;
            left = self.allocate(left, right, op);
        }

        println!("{:?}", self.tree);

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<NodeId, ParserError> {
        let mut left = self.parse_factor()?;

        loop {
            if self.paren_depth > 0 {
                self.skip_newlines();
            }

            let op = match self.current_token.kind {
                TokenKind::Plus => Op::Add,
                TokenKind::Minus => Op::Sub,
                _ => break,
            };

            self.advance();
            self.skip_newlines();

            let right = self.parse_factor()?;
            left = self.allocate(left, right, op);
        }

        println!("{:?}", self.tree);

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<NodeId, ParserError> {
        if self.current_token.kind == TokenKind::Minus {
            let (start, _) = self.current_token.span.unpack();

            self.advance();
            self.skip_newlines();

            let operand = self.parse_unary()?;
            let (op_start, op_len) = self.tree[operand].span.unpack::<usize>();

            let node = Node {
                expr: Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    operand,
                },
                span: Span::new(start, op_start + op_len),
            };

            return Ok(self.tree.alloc(node));
        }

        self.parse_primary()
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
                self.paren_depth += 1;

                self.advance();
                self.skip_newlines();

                let idx = self.parse_term()?;

                self.paren_depth -= 1;
                self.expect(TokenKind::RParen)?;

                Ok(idx)
            }

            _ => Err(ParserError::UnexpectedToken(
                self.current_token.kind.clone(),
            )),
        }
    }
}
