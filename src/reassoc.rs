use std::collections::HashMap;

use crate::{
    error::Error,
    exprstmt::*,
    token::*,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Precedence {
    pub prec: usize,
    pub assoc: Associativity,
}

pub fn reassociate(ops: HashMap<String, Precedence>, stmt: Vec<Stmt>) -> Result<Vec<Stmt>, Error> {
    let mut reassoc = Reassociate { ops };
    let mut ls = vec![];
    for s in stmt {
        ls.push(reassoc.reassociate(s)?)
    }
    Ok(ls)
}

struct Reassociate {
    ops: HashMap<String, Precedence>,
}
impl Reassociate {
    pub fn reassociate(&mut self, stmt: Stmt) -> Result<Stmt, Error> {
        self.visit_stmt(stmt)
    }

    // the one method this file exists for
    // binary operator reassociation
    // https://stackoverflow.com/a/67992584
    // TODO: play with references and stuff once I dare again
    fn reassoc(&mut self, left: Expr, op1: Token, right: Expr) -> Result<Expr, Error> {
        let left = self.visit_expr(left)?;
        let right = self.visit_expr(right)?;
        // not a binary operation, no need to reassociate it
        let ExprType::BinaryOperation(left2, op2, right2) = right.val.clone() else {
            return Ok(Expr {
                start: left.start,
                end: right.end,
                val: ExprType::BinaryOperation(left.into(), op1, right.into()),
            });
        };

        let TokenType::Symbol(op1_sym) = &op1.val else {
            unreachable!()
        };
        let prec1 = self.ops.get(op1_sym.as_str()).ok_or(Error {
            msg: format!("Operator not found: {}", op1_sym),
            lines: vec![op1.loc()],
        })?;

        let TokenType::Symbol(op2_sym) = &op2.val else {
            unreachable!()
        };
        let prec2 = self.ops.get(op2_sym.as_str()).ok_or(Error {
            msg: format!("Operator not found: {}", op2_sym),
            lines: vec![op2.loc()],
        })?;
        // TODO: make functions like in the SO answer?
        match prec1.prec.cmp(&prec2.prec) {
            std::cmp::Ordering::Greater => {
                let left = self.reassoc(left, op1, *left2)?.into();
                Ok(Expr {
                    val: ExprType::BinaryOperation(left, op2.clone(), right2.clone()),
                    start: right2.start,
                    end: right2.end,
                })
            }

            std::cmp::Ordering::Less => Ok(Expr {
                start: left.start,
                end: right.end,
                val: ExprType::BinaryOperation(left.into(), op1, right.into()),
            }),

            std::cmp::Ordering::Equal => match (prec1.assoc, prec2.assoc) {
                (Associativity::Left, Associativity::Left) => {
                    let left = self.reassoc(left, op1, *left2)?.into();
                    Ok(Expr {
                        start: right2.start,
                        end: right2.end,
                        val: ExprType::BinaryOperation(left, op2, right2),
                    })
                }
                (Associativity::Right, Associativity::Right) => Ok(Expr {
                    start: left.start,
                    end: right.end,
                    val: ExprType::BinaryOperation(left.into(), op1, right.into()),
                }),
                _ => Err(Error {
                    msg: format!(
                        "Incompatible operator precedence: \"{}\" ({:?}) and \"{}\" ({:?}) - both have precedence {}",
                        op1.val, prec1.assoc, op2.val, prec2.assoc, prec1.prec
                    ),
                    lines: vec![op1.loc(), op2.loc()],
                }),
            },
        }
    }
}

impl Reassociate {
    fn visit_stmt(&mut self, stmt: Stmt) -> Result<Stmt, Error> {
        Ok(match stmt.val {
            StmtType::VarDeclStmt(ident, expr) => Stmt {
                val: StmtType::VarDeclStmt(ident, self.visit_expr(expr)?),
                ..stmt
            },
            StmtType::AssignStmt(ident, expr) => {
                Stmt {
                    val: StmtType::AssignStmt(ident, self.visit_expr(expr)?),
                    ..stmt
                }
            },
            StmtType::ExprStmt(expr) => {
                Stmt {
                    val: StmtType::ExprStmt(self.visit_expr(expr)?),
                    ..stmt
                }
            },
            StmtType::BlockStmt(block) => {
                let mut block2: Vec<Stmt> = vec![];
                for s in block {
                    block2.push(self.visit_stmt(s)?)
                }
                Stmt {
                    val: StmtType::BlockStmt(block2),
                    ..stmt
                }
            },
            StmtType::IfStmt(blocks) => {
                let mut blocks_result: Vec<(Expr, Block)> = vec![];
                for (cond, stmts) in blocks {
                    let mut block: Block = vec![];
                    for s in stmts {
                        block.push(self.visit_stmt(s)?)
                    }
                    blocks_result.push((self.visit_expr(cond)?, block))
                }

                Stmt {
                    val: StmtType::IfStmt(blocks_result),
                    ..stmt
                }
            }
            StmtType::WhileStmt(cond, block) => {
                let cond = self.visit_expr(cond)?;
                let mut block2: Block = vec![];
                for s in block {
                    block2.push(self.visit_stmt(s)?)
                }
                Stmt {
                    val: StmtType::WhileStmt(cond, block2),
                    ..stmt
                }
            },
            StmtType::FunDeclStmt(ident, params, block) => {
                let mut block2: Block = vec![];
                for s in block {
                    block2.push(self.visit_stmt(s)?)
                }
                Stmt {
                    val: StmtType::FunDeclStmt(ident, params, block2),
                    ..stmt
                }
            },
            StmtType::ContinueStmt => stmt,
            StmtType::BreakStmt => stmt,
            StmtType::ReturnStmt(expr) => {
                Stmt {
                    val: StmtType::ReturnStmt(self.visit_expr(expr)?),
                    ..stmt
                }
            },
        })
    }

    fn visit_expr(&mut self, expr: Expr) -> Result<Expr, Error> {
        Ok(match expr.val {
            ExprType::Unit => expr,
            ExprType::Int(_) => expr,
            ExprType::Float(_) => expr,
            ExprType::String(_) => expr,
            ExprType::Bool(_) => expr,
            ExprType::Identifier(_) => expr,
            ExprType::Parens(expr2) => {
                Expr {
                    val: ExprType::Parens(self.visit_expr(*expr2)?.into()),
                    ..expr
                }
            },
            ExprType::Call(callee, args) => {
                let mut args2 = vec![];
                for arg in args {
                    args2.push(self.visit_expr(arg)?);
                }
                Expr {
                    val: ExprType::Call(self.visit_expr(*callee)?.into(), args2),
                    ..expr
                }
            },
            ExprType::UnaryOperation(op, expr2) => {
                Expr {
                    val: ExprType::UnaryOperation(op, self.visit_expr(*expr2)?.into()),
                    ..expr
                }
            },
            ExprType::BinaryOperation(left, op, right) => {
                self.reassoc(*left, op, *right)?
            },

        })
    }
}
