use juan_span::Span;
use la_arena::{Arena, Idx};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum UnaryOp {
    Neg,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Literal {
    Int(i64),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Expr {
    Literal(Literal),
    BinaryOp {
        op: Op,
        left: NodeId,
        right: NodeId,
    },
    UnaryOp {
        op: UnaryOp,
        operand: NodeId,
    },
    // TODO: once statements are in, change NodeId to like a statementid
    Block {
        statements: Vec<NodeId>,
        tail: Option<NodeId>,
    },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    pub expr: Expr,
    pub span: Span,
}

pub type SyntaxTree = Arena<Node>;
pub type NodeId = Idx<Node>;

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleDecl {
    pub path_span: Span,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FunctionDecl {
    pub name: Span,
    pub span: Span,
    pub body: NodeId,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParsedModule {
    pub tree: SyntaxTree,
    pub decl: ModuleDecl,
    pub functions: Vec<FunctionDecl>,
}
