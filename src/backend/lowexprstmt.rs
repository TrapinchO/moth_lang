use std::fmt::Display;

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

impl ExprType {
    fn format(&self) -> String {
        match self {
            Self::Unit => "()".to_string(),
            Self::Int(n) => n.to_string(),
            Self::Float(n) => n.to_string(),
            Self::String(s) => format!("\"{s}\""),
            Self::Bool(b) => b.to_string(),
            Self::Identifier(ident) => ident.to_string(),
            Self::Call(callee, args) => format!(
                "{callee}({args})",
                args = args.iter().map(|e| { format!("{e}") }).collect::<Vec<_>>().join(", ")
            ),
            Self::List(ls) => format!(
                "[{}]",
                ls.iter().map(|e| { format!("{e}") }).collect::<Vec<_>>().join(", ")
            ),
            Self::Index(expr, idx) => format!("{}[{}]", expr.val, idx.val),
            Self::Lambda(params, block) => format!(
                "lambda({params}){block}",
                params = params.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", "),
                block = block.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n")
            ),
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
    VarDeclStmt(Identifier, Expr),
    AssignStmt(Identifier, Expr),
    AssignIndexStmt(Expr, Expr, Expr), // expr[expr] = expr
    BlockStmt(Vec<Stmt>),
    IfStmt(Vec<(Expr, Vec<Stmt>)>),
    WhileStmt(Expr, Vec<Stmt>),
    ReturnStmt(Expr),
    BreakStmt,
    ContinueStmt,
    StructStmt(Identifier, Vec<Identifier>),
}

impl StmtType {
    fn format(&self) -> String {
        match self {
            Self::ExprStmt(expr) => expr.to_string() + ";",
            Self::VarDeclStmt(ident, expr) => format!("let {ident} = {expr};"),
            Self::AssignStmt(ident, expr) => format!("{ident} = {expr};"),
            Self::AssignIndexStmt(ls, idx, val) => format!("{ls}[{idx}] = {val};"),
            Self::BlockStmt(block) => format!(
                "{{\n{block}\n}}",
                block = block.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n")
            ),
            Self::IfStmt(blocks) => {
                let first = blocks.first().unwrap(); // always present
                let rest = &blocks[1..]
                    .iter()
                    .map(|(cond, stmts)| {
                        format!(
                            "else if {cond} {{\n{stmts}\n}}",
                            cond = cond.val,
                            stmts = stmts.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n")
                        )
                    })
                    .collect::<Vec<_>>();
                format!(
                    "if {cond} {{\n{block}\n}} {rest}",
                    cond = first.0,
                    block = first.1.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n"),
                    rest = rest.join("")
                )
            }
            Self::WhileStmt(cond, block) => format!(
                "while {cond} {{{block}}}",
                block = block.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n")
            ),
            Self::ReturnStmt(expr) => format!("return {expr};"),
            Self::BreakStmt => "break;".to_string(),
            Self::ContinueStmt => "continue;".to_string(),
            Self::StructStmt(name, fields) => format!("struct {name} {{ {} }}", fields.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", "))
        }
    }
}

impl Display for StmtType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

pub type Stmt = Located<StmtType>;
