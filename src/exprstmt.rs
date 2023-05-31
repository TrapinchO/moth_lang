use crate::token::Token;

use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum ExprType {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Identifier(String),
    Parens(Rc<Expr>),
    UnaryOperation(Token, Rc<Expr>),
    BinaryOperation(Rc<Expr>, Token, Rc<Expr>),
}

impl ExprType {
    fn format(&self) -> String {
        match self {
            Self::Int(n) => n.to_string(),
            Self::Float(n) => n.to_string(),
            Self::String(s) => format!("\"{}\"", s),
            Self::Bool(b) => b.to_string(),
            Self::Identifier(ident) => ident.to_string(),
            Self::Parens(expr) => format!("({})", expr.typ.format()),
            Self::UnaryOperation(op, expr) => format!("({} {})", op.typ, expr),
            Self::BinaryOperation(left, op, right) => format!("({} {} {})", left, op.typ, right),
        }
    }
}

impl Display for ExprType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

// TODO: consider moving this to the bottom again, as rust seems to be
// affected by position of the arguments, at least regarding the borrow checker
#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub typ: ExprType,
    pub start: usize,
    pub end: usize,
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.typ.format())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum StmtType {
    ExprStmt(Expr),
    // identifier, expression
    VarDeclStmt(Token, Expr),
    AssignStmt(Token, Expr),
}
impl StmtType {
    fn format(&self) -> String {
        match self {
            Self::ExprStmt(expr) => expr.to_string(),
            Self::VarDeclStmt(ident, expr) => format!("let {} = {}", ident, expr),
            Self::AssignStmt(name, expr) => format!("{} = {}", name, expr),
        }
    }
}
impl Display for StmtType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Stmt {
    pub typ: StmtType,
    pub start: usize,
    pub end: usize,
}
impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.typ)
    }
}
