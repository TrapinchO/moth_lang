use crate::{located::Located, token::Token};

use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum ExprType {
    Unit,
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Identifier(String),
    Parens(Box<Expr>),
    Call(Box<Expr>, Vec<Expr>), // calle, args (calle(arg1, arg2, arg3))
    UnaryOperation(Token, Box<Expr>),
    BinaryOperation(Box<Expr>, Token, Box<Expr>),
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
            Self::Parens(expr) => format!("({expr})", expr = expr.val.format()),
            Self::Call(callee, args) => format!(
                "{callee}({args})",
                args = args.iter().map(|e| { format!("{e}") }).collect::<Vec<_>>().join(", ")
            ),
            Self::UnaryOperation(op, expr) => format!("({op} {expr})", op = op.val),
            Self::BinaryOperation(left, op, right) => format!("({left} {op} {right})", op = op.val),
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
    // name, parameters, body
    FunDeclStmt(Token, Vec<Token>, Vec<Stmt>),
    ReturnStmt(Expr),
    BreakStmt,
    ContinueStmt,
}
impl StmtType {
    fn format(&self) -> String {
        match self {
            Self::ExprStmt(expr) => expr.to_string() + ";",
            Self::VarDeclStmt(ident, expr) => format!("let {ident} = {expr};"),
            Self::AssignStmt(ident, expr) => format!("{ident} = {expr};"),
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
            Self::FunDeclStmt(ident, params, block) => format!(
                "fun {ident}({params}){block}",
                params = params.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", "),
                block = block.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n")
            ),
            Self::ReturnStmt(expr) => format!("return {expr};"),
            Self::BreakStmt => "break;".to_string(),
            Self::ContinueStmt => "continue;".to_string(),
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
