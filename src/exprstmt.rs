use crate::{located::Located, token::Token};

use std::{fmt::Display, rc::Rc};

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
            Self::Parens(expr) => format!("({})", expr.val.format()),
            Self::UnaryOperation(op, expr) => format!("({} {})", op.val, expr),
            Self::BinaryOperation(left, op, right) => format!("({} {} {})", left, op.val, right),
        }
    }
}

impl Display for ExprType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

pub type Expr = Located<ExprType>;

#[derive(Debug, PartialEq, Clone)]
pub enum StmtType {
    ExprStmt(Expr),
    // identifier, expression
    VarDeclStmt(Token, Expr),
    AssignStmt(Token, Expr),
    BlockStmt(Vec<Stmt>),
    IfStmt(Vec<(Expr, Vec<Stmt>)>),
    WhileStmt(Expr, Vec<Stmt>),
    PrintStmt(Expr),
}
impl StmtType {
    fn format(&self) -> String {
        match self {
            Self::ExprStmt(expr) => expr.to_string() + ";",
            Self::VarDeclStmt(ident, expr) => format!("let {} = {};", ident, expr),
            Self::AssignStmt(name, expr) => format!("{} = {};", name, expr),
            Self::BlockStmt(block) => block.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n"),
            Self::IfStmt(blocks) => {
                let first = blocks.first().unwrap(); // always present
                let rest = &blocks[1..]
                    .iter()
                    .map(|(cond, stmts)| {
                        format!(
                            "else if {} {{\n{}\n}}",
                            cond.val,
                            stmts.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n")
                        )
                    })
                    .collect::<Vec<_>>();
                format!(
                    "if {} {{\n{}\n}} {}",
                    first.0,
                    first.1.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n"),
                    rest.join("")
                )
            }
            Self::WhileStmt(cond, block) => format!(
                "while {} {{{}}}",
                cond,
                block.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n")
            ),
            Self::PrintStmt(expr) => format!("print {}", expr),
        }
    }
}

impl Display for StmtType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

pub type Stmt = Located<StmtType>;
pub type Block = Vec<Stmt>;
