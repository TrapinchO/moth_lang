use std::fmt::Display;

/// a lower representation of the AST
/// as desugared and simplified as possible
/// i.e. operators (both unary and binary) are changed into function calls
/// function and operator declarations are changed into lambdas assigned to their respective
/// identifier
use crate::located::Located;

pub type Identifier = Located<String>;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Unit,
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Identifier(String),
    Call(Box<LExpr>, Vec<LExpr>), // callee(arg1, arg2, arg3)
    List(Vec<LExpr>),
    Index(Box<LExpr>, Box<LExpr>), // expr[idx]
    Lambda(Vec<Identifier>, Vec<LStmt>), // |params| { block }
    FieldAccess(Box<LExpr>, Identifier),
    MethodAccess(Box<LExpr>, Identifier, Vec<LExpr>), // expr.name(args)
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
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
            Self::FieldAccess(expr, name) => format!("{expr}.{name}"),
            Self::MethodAccess(callee, name, args) => format!(
                "{callee}.{name}({args})",
                args = args.iter().map(|e| { format!("{e}") }).collect::<Vec<_>>().join(", ")
            ),
        };
        write!(f, "{s}")
    }
}


pub type LExpr = Located<Expr>;

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expr(LExpr),
    VarDecl(Identifier, LExpr),
    Assign(Identifier, LExpr),
    AssignIndex(LExpr, LExpr, LExpr), // expr[expr] = expr
    Block(Vec<LStmt>),
    If(Vec<(LExpr, Vec<LStmt>)>),
    While(LExpr, Vec<LStmt>),
    Return(LExpr),
    Break,
    Continue,
    Struct(Identifier, Vec<Identifier>),
    AssignStruct(LExpr, Identifier, LExpr), // expr.name = expr
    Impl(Identifier, Vec<LStmt>),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Expr(expr) => expr.to_string() + ";",
            Self::VarDecl(ident, expr) => format!("let {ident} = {expr};"),
            Self::Assign(ident, expr) => format!("{ident} = {expr};"),
            Self::AssignIndex(ls, idx, val) => format!("{ls}[{idx}] = {val};"),
            Self::Block(block) => format!(
                "{{\n{block}\n}}",
                block = block.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n")
            ),
            Self::If(blocks) => {
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
            Self::While(cond, block) => format!(
                "while {cond} {{{block}}}",
                block = block.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n")
            ),
            Self::Return(expr) => format!("return {expr};"),
            Self::Break => "break;".to_string(),
            Self::Continue => "continue;".to_string(),
            Self::Struct(name, fields) => format!(
                "struct {name} {{ {} }}",
                fields.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ")
            ),
            Self::AssignStruct(expr1, name, expr2) => format!("{expr1}.{} = {expr2}", name.val),
            Self::Impl(name, block) => format!(
                "impl {name} {{\n{block}\n}}",
                block = block.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n")
            ),
        };
        write!(f, "{s}")
    }
}

pub type LStmt = Located<Stmt>;
