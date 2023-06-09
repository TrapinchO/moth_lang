use std::collections::HashMap;

use crate::error::Error;
use crate::exprstmt::*;
use crate::token::*;
use crate::visitor::{ExprVisitor, StmtVisitor};

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

pub fn reassociate(ops: HashMap<String, Precedence>, stmt: &Vec<Stmt>) -> Result<Vec<Stmt>, Error> {
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
    pub fn reassociate(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        self.visit_stmt(stmt)
    }
}
impl StmtVisitor<Stmt> for Reassociate {
    fn expr(&mut self, expr: &Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::ExprStmt(self.visit_expr(expr)?),
            start: expr.start,
            end: expr.end,
        })
    }
    fn var_decl(&mut self, ident: &Token, expr: &Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::VarDeclStmt(ident.clone(), self.visit_expr(expr)?),
            start: ident.start,
            end: expr.end,
        })
    }
    fn assignment(&mut self, ident: &Token, expr: &Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::AssignStmt(ident.clone(), self.visit_expr(expr)?),
            start: 0,
            end: 0,
        })
    }

    fn block(&mut self, block: &Vec<Stmt>) -> Result<T, Error> {
        let mut block2: Vec<Stmt> = vec![];
        for s in block {
            block2.push(self.visit_stmt(s)?)
        }
        Ok(Stmt {
            val: StmtType::BlockStmt(block2),
            start: 0,
            end: 0,
        })
    }

    fn if_else(&mut self, blocks: &Vec<(Expr, Vec<Stmt>)>) -> Result<Stmt, Error> {
        let mut blocks_result: Vec<(Expr, Vec<Stmt>)> = vec![];
        for (cond, stmts) in blocks {
            let mut block: Vec<Stmt> = vec![];
            for s in stmts {
                block.push(self.visit_stmt(s)?)
            }
            blocks_result.push((self.visit_expr(cond)?, block))
        }

        Ok(Stmt {
            val: StmtType::IfStmt(blocks_result),
            start: 0,
            end: 0,
        })
    }
    fn whiles(&mut self, cond: &Expr, block: &Vec<Stmt>) -> Result<Stmt, Error> {
        let cond = self.visit_expr(cond)?;
        let mut block2: Vec<Stmt> = vec![];
        for s in block {
            block2.push(self.visit_stmt(s)?)
        }
        Ok(Stmt {
            val: StmtType::WhileStmt(cond, block2),
            start: 0,
            end: 0,
        })
    }
}
impl ExprVisitor<Expr> for Reassociate {
    fn int(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr.clone())
    }
    fn bool(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr.clone())
    }
    fn float(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr.clone())
    }
    fn string(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr.clone())
    }
    fn identifier(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr.clone())
    }
    fn parens(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Parens(self.visit_expr(expr)?.into()),
            ..*expr
        })
    }
    fn unary(&mut self, op: &Token, expr: &Expr) -> Result<Expr, Error> {
        let expr = self.visit_expr(expr)?;
        Ok(Expr {
            start: op.start,
            end: expr.end,
            val: ExprType::UnaryOperation(op.clone(), expr.into()),
        })
    }
    // the one method this file exists for
    // https://stackoverflow.com/a/67992584
    fn binary(&mut self, left: &Expr, op1: &Token, right: &Expr) -> Result<Expr, Error> {
        let left = self.visit_expr(left)?;
        let right = self.visit_expr(right)?;
        // not a binary operation, no need to reassociate it
        let ExprType::BinaryOperation(left2, op2, right2) = &right.val else {
            return Ok(Expr {
                val: ExprType::BinaryOperation(
                    left.clone().into(),
                    op1.clone(),
                    right.clone().into()),
                start: left.start,
                end: right.end,
            })
        };

        let TokenType::Symbol(op1_sym) = &op1.val else {
            panic!("Operator token 1 is not a symbol");
        };
        let prec1 = self.ops.get(op1_sym.as_str()).ok_or(Error {
            msg: format!("Operator not found: {}", op1_sym),
            lines: vec![(op1.start, op1.end)],
        })?;

        let TokenType::Symbol(op2_sym) = &op2.val else {
            panic!("Operator token 2 is not a symbol");
        };
        let prec2 = self.ops.get(op2_sym.as_str()).ok_or(Error {
            msg: format!("Operator not found: {}", op2_sym),
            lines: vec![(op2.start, op2.end)],
        })?;
        // TODO: make functions like in the SO answer?
        match prec1.prec.cmp(&prec2.prec) {
            std::cmp::Ordering::Greater => {
                let left = self.binary(&left, op1, left2)?.into();
                Ok(Expr {
                    val: ExprType::BinaryOperation(left, op2.clone(), right2.clone()),
                    start: right2.start,
                    end: right2.end,
                })
            }

            std::cmp::Ordering::Less => Ok(Expr {
                val: ExprType::BinaryOperation(
                    left.clone().into(),
                    op1.clone(),
                    right.clone().into()),
                start: left.start,
                end: right.end,
            }),

            std::cmp::Ordering::Equal => match (prec1.assoc, prec2.assoc) {
                (Associativity::Left, Associativity::Left) => {
                    let left = self.binary(&left, op1, left2)?.into();
                    Ok(Expr {
                        val: ExprType::BinaryOperation(left, op2.clone(), right2.clone()),
                        start: right2.start,
                        end: right2.end,
                    })
                }
                (Associativity::Right, Associativity::Right) => Ok(Expr {
                    val: ExprType::BinaryOperation(
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
                        op1.val, prec1.assoc, op2.val, prec2.assoc, prec1.prec
                    ),
                    lines: vec![(op1.start, op1.end), (op2.start, op2.end)]
                }),
            },
        }
    }
}
