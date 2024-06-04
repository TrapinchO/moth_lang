/// a lower representation of the AST
/// as desugared and simplified as possible
/// i.e. operators (both unary and binary) are changed into function calls
/// function and operator declarations are changed into lambdas assigned to their respective
/// identifier

use crate::located::Located;


pub type Identifier = Located<String>;

#[derive(Debug, PartialEq, Clone)]
pub enum ExprType {
    Unit,
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Identifier(String),
    Call(Box<Expr>, Vec<Expr>), // callee(arg1, arg2, arg3)
    List(Vec<Expr>),
    Index(Box<Expr>, Box<Expr>), // expr[idx]
    Lambda(Vec<Identifier>, Vec<Stmt>), // |params| { block }
}

pub type Expr = Located<ExprType>;

#[derive(Debug, PartialEq, Clone)]
pub enum StmtType {
    ExprStmt(Expr),
    VarDeclStmt(Identifier, Expr),
    AssignStmt(Identifier, Expr),
    AssignIndexStmt(Expr, Expr, Expr), // expr[expr] = expr
    BlockStmt(Vec<Stmt>),
    IfStmt(Vec<(Expr, Vec<Stmt>)>),
    WhileStmt(Expr, Vec<Stmt>),
    ReturnStmt(Expr),
    BreakStmt,
    ContinueStmt,
}

pub type Stmt = Located<StmtType>;
