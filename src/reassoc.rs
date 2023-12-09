use std::collections::HashMap;

use crate::{
    error::Error,
    exprstmt::*,
    token::*,
    visitor::{ExprVisitor, StmtVisitor},
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

    // the one method this file exists for
    // binary operator reassociation
    // https://stackoverflow.com/a/67992584
    fn reassoc(&mut self, left: &Expr, op1: &Token, right: &Expr) -> Result<Expr, Error> {
        let left = self.visit_expr(left)?;
        let right = self.visit_expr(right)?;
        // not a binary operation, no need to reassociate it
        let ExprType::BinaryOperation(left2, op2, right2) = &right.val else {
            return Ok(Expr {
                val: ExprType::BinaryOperation(left.clone().into(), op1.clone(), right.clone().into()),
                start: left.start,
                end: right.end,
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
                let left = self.reassoc(&left, op1, left2)?.into();
                Ok(Expr {
                    val: ExprType::BinaryOperation(left, op2.clone(), right2.clone()),
                    start: right2.start,
                    end: right2.end,
                })
            }

            std::cmp::Ordering::Less => Ok(Expr {
                val: ExprType::BinaryOperation(left.clone().into(), op1.clone(), right.clone().into()),
                start: left.start,
                end: right.end,
            }),

            std::cmp::Ordering::Equal => match (prec1.assoc, prec2.assoc) {
                (Associativity::Left, Associativity::Left) => {
                    let left = self.reassoc(&left, op1, left2)?.into();
                    Ok(Expr {
                        val: ExprType::BinaryOperation(left, op2.clone(), right2.clone()),
                        start: right2.start,
                        end: right2.end,
                    })
                }
                (Associativity::Right, Associativity::Right) => Ok(Expr {
                    val: ExprType::BinaryOperation(left.clone().into(), op1.clone(), right.clone().into()),
                    start: left.start,
                    end: right.end,
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
impl StmtVisitor<Stmt> for Reassociate {
    fn expr(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::ExprStmt(expr) = &stmt.val else {
            unreachable!()
        };
        Ok(Stmt {
            val: StmtType::ExprStmt(self.visit_expr(expr)?),
            start: expr.start,
            end: expr.end,
        })
    }
    fn var_decl(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::VarDeclStmt(ident, expr) = &stmt.val else {
            unreachable!()
        };
        Ok(Stmt {
            val: StmtType::VarDeclStmt(ident.clone(), self.visit_expr(expr)?),
            start: stmt.start,
            end: stmt.end,
        })
    }
    fn assignment(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::AssignStmt(ident, expr) = &stmt.val else {
            unreachable!()
        };
        Ok(Stmt {
            val: StmtType::AssignStmt(ident.clone(), self.visit_expr(expr)?),
            start: stmt.start,
            end: stmt.end,
        })
    }

    fn block(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::BlockStmt(block) = &stmt.val else {
            unreachable!()
        };
        let mut block2: Vec<Stmt> = vec![];
        for s in block {
            block2.push(self.visit_stmt(s)?)
        }
        Ok(Stmt {
            val: StmtType::BlockStmt(block2),
            start: stmt.start,
            end: stmt.end,
        })
    }

    fn if_else(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::IfStmt(blocks) = &stmt.val else {
            unreachable!()
        };
        let mut blocks_result: Vec<(Expr, Block)> = vec![];
        for (cond, stmts) in blocks {
            let mut block: Block = vec![];
            for s in stmts {
                block.push(self.visit_stmt(s)?)
            }
            blocks_result.push((self.visit_expr(cond)?, block))
        }

        Ok(Stmt {
            val: StmtType::IfStmt(blocks_result),
            start: stmt.start,
            end: stmt.end,
        })
    }
    fn whiles(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::WhileStmt(cond, block) = &stmt.val else {
            unreachable!()
        };
        let cond = self.visit_expr(cond)?;
        let mut block2: Block = vec![];
        for s in block {
            block2.push(self.visit_stmt(s)?)
        }
        Ok(Stmt {
            val: StmtType::WhileStmt(cond, block2),
            start: stmt.start,
            end: stmt.end,
        })
    }
    fn fun(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::FunDeclStmt(ident, params, block) = &stmt.val else {
            unreachable!()
        };
        let mut block2: Block = vec![];
        for s in block {
            block2.push(self.visit_stmt(s)?)
        }
        Ok(Stmt {
            val: StmtType::FunDeclStmt(ident.clone(), params.clone(), block2),
            ..*stmt
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
        let ExprType::Parens(expr2) = &expr.val else {
            unreachable!()
        };
        Ok(Expr {
            val: ExprType::Parens(self.visit_expr(expr2)?.into()),
            ..*expr
        })
    }
    fn call(&mut self, expr: &Expr) -> Result<Expr, Error> {
        let ExprType::Call(callee, args) = &expr.val else {
            unreachable!()
        };
        let mut args2 = vec![];
        for arg in args {
            args2.push(self.visit_expr(arg)?);
        }
        Ok(Expr {
            val: ExprType::Call(self.visit_expr(callee)?.into(), args2),
            ..*expr
        })
    }
    fn unary(&mut self, expr: &Expr) -> Result<Expr, Error> {
        let ExprType::UnaryOperation(op, expr2) = &expr.val else {
            unreachable!()
        };
        let expr2 = self.visit_expr(expr2)?;
        Ok(Expr {
            start: op.start,
            end: expr2.end,
            val: ExprType::UnaryOperation(op.clone(), expr2.into()),
        })
    }
    fn binary(&mut self, expr: &Expr) -> Result<Expr, Error> {
        let ExprType::BinaryOperation(left, op, right) = &expr.val else {
            unreachable!()
        };
        self.reassoc(left, op, right)
    }
}
