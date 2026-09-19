use juan_span::Span;
use la_arena::{Arena, Idx};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    Add,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Literal {
    Int(i64),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr {
    Literal(Literal),
    BinaryOp {
        op: Op,
        left: Idx<Node>,
        right: Idx<Node>,
    },
    Module,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    pub expr: Expr,
    pub span: Span,
}

pub type SyntaxTree = Arena<Node>;
pub type NodeId = Idx<Node>;
