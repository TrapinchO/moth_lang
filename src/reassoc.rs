use std::collections::HashMap;

use crate::error::Error;
use crate::lexer::{Token, TokenType};
use crate::parser::{Expr, ExprType, StmtType, Stmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Associativity {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Precedence {
    precedence: usize,
    associativity: Associativity,
}

impl Precedence {
    pub fn new(prec: usize, assoc: Associativity) -> Self {
        Precedence {
            precedence: prec,
            associativity: assoc,
        }
    }
}

pub fn reassociate(stmt: &Vec<Stmt>) -> Result<Vec<Stmt>, Error> {
    let mut ls = vec![];
    for s in stmt {
        ls.push(match &s.typ {
            StmtType::ExprStmt(expr) => Stmt {
                typ: StmtType::ExprStmt(reassoc_expr(expr)?),
                ..*s
            },
            StmtType::AssingmentStmt(ident, expr) => Stmt {
                typ: StmtType::AssingmentStmt(ident.clone(), reassoc_expr(expr)?),
                ..*s
            }
        })
    }
    Ok(ls)
}

// https://stackoverflow.com/a/67992584
fn reassoc_expr(expr: &Expr) -> Result<Expr, Error> {
    Ok(match &expr.typ {
        ExprType::BinaryOperation(left, op, right) => reassoc_(
            &reassoc_expr(left)?,
            op,
            &reassoc_expr(right)?
        )?,
        ExprType::Parens(expr) => Expr {
            typ: ExprType::Parens(reassoc_expr(expr.as_ref())?.into()),
            ..expr.as_ref().clone()
        },
        ExprType::UnaryOperation(op, expr) => Expr {
            typ: ExprType::UnaryOperation(op.clone(), reassoc_expr(expr.as_ref())?.into()),
            ..expr.as_ref().clone()
        },
        ExprType::Number(_)
            | ExprType::Float(_)
            | ExprType::String(_)
            | ExprType::Identifier(_)
            | ExprType::Bool(_) => expr.clone(),
    })
}

fn reassoc_(left: &Expr, op1: &Token, right: &Expr) -> Result<Expr, Error> {
    let prec_table: HashMap<&str, Precedence> = HashMap::from([
        ("+", Precedence::new(1, Associativity::Left)),
        ("-", Precedence::new(1, Associativity::Left)),
        ("*", Precedence::new(2, Associativity::Left)),
        ("/", Precedence::new(2, Associativity::Left)),
        ("^^", Precedence::new(10, Associativity::Right)),  // analyzer shut up now please its used
    ]);

    // not a binary operation, no need to reassociate it
    let ExprType::BinaryOperation(left2, op2, right2) = &right.typ else {
        return Ok(Expr {
            typ: ExprType::BinaryOperation(
                left.clone().into(),
                op1.clone(),
                right.clone().into()),
            start: left.start,
            end: right.end,
        })
    };

    let TokenType::Symbol(op1_sym) = &op1.typ else {
        panic!("Operator token 1 is not a symbol");
    };
    let prec1 = prec_table.get(op1_sym.as_str()).ok_or(Error {
        msg: format!("Operator not found: {}", op1_sym),
        lines: vec![(op1.start, op1.end)],
    })?;

    let TokenType::Symbol(op2_sym) = &op2.typ else {
        panic!("Operator token 2 is not a symbol");
    };
    let prec2 = prec_table.get(op2_sym.as_str()).ok_or(Error {
        msg: format!("Operator not found: {}", op2_sym),
        lines: vec![(op2.start, op2.end)],
    })?;

    // TODO: make functions like in the answer
    match prec1.precedence.cmp(&prec2.precedence) {
        std::cmp::Ordering::Greater => {
            let left = reassoc_(left, op1, left2)?.into();
            Ok(Expr {
                typ: ExprType::BinaryOperation(left, op2.clone(), right2.clone()),
                start: right2.start,
                end: right2.end,
            })
        }

        std::cmp::Ordering::Less => Ok(Expr {
            typ: ExprType::BinaryOperation(
                left.clone().into(),
                op1.clone(),
                right.clone().into()),
            start: left.start,
            end: right.end,
        }),

        std::cmp::Ordering::Equal => match (prec1.associativity, prec2.associativity) {
            (Associativity::Left, Associativity::Left) => {
                let left = reassoc_(left, op1, left2)?.into();
                Ok(Expr {
                    typ: ExprType::BinaryOperation(left, op2.clone(), right2.clone()),
                    start: right2.start,
                    end: right2.end,
                })
            }
            (Associativity::Right, Associativity::Right) => Ok(Expr {
                typ: ExprType::BinaryOperation(
                    left.clone().into(),
                    op1.clone(),
                    right.clone().into(),
                ),
                start: left.start,
                end: right.end,
            }),
            _ => Err(Error {
                msg: format!(
                    "Incompatible operator precedence: \"{}\" ({:?}) and \"{}\" ({:?}) - both have precedence {}",
                    op1.typ, prec1.associativity, op2.typ, prec2.associativity, prec1.precedence
                ),
                lines: vec![(op1.start, op1.end), (op2.start, op2.end)]
            }),
        },
    }
}
