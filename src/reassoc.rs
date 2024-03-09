use std::collections::HashMap;

use crate::{
    error::Error,
    exprstmt::*,
    token::*, visitor::ExprVisitor, visitor::{Location, StmtVisitor},
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
        let prec1 = self.ops.get(op1_sym).ok_or(Error {
            msg: format!("Operator not found: {}", op1_sym),
            lines: vec![op1.loc()],
        })?;

        let TokenType::Symbol(op2_sym) = &op2.val else {
            unreachable!()
        };
        let prec2 = self.ops.get(op2_sym).ok_or(Error {
            msg: format!("Operator not found: {}", op2_sym),
            lines: vec![op2.loc()],
        })?;
        // TODO: make functions like in the SO answer?
        match prec1.prec.cmp(&prec2.prec) {
            std::cmp::Ordering::Greater => {
                let left = self.reassoc(left, op1, *left2)?.into();
                Ok(Expr {
                    start: right2.start,
                    end: right2.end,
                    val: ExprType::BinaryOperation(left, op2, right2),
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

impl StmtVisitor<Stmt> for Reassociate {
    fn expr(&mut self, loc: Location, expr: Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::ExprStmt(self.visit_expr(expr)?),
            start: loc.0,
            end: loc.1,
        })
    }
    fn var_decl(&mut self, loc: Location, ident: Token, expr: Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::VarDeclStmt(ident, self.visit_expr(expr)?),
            start: loc.0,
            end: loc.1,
        })
    }
    fn assignment(&mut self, loc: Location, ident: Token, expr: Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::AssignStmt(ident, self.visit_expr(expr)?),
            start: loc.0,
            end: loc.1,
        })
    }
    fn block(&mut self, loc: Location, block: Vec<Stmt>) -> Result<Stmt, Error> {
                let mut block2: Vec<Stmt> = vec![];
                for s in block {
                    block2.push(self.visit_stmt(s)?)
                }
                Ok(Stmt {
                    val: StmtType::BlockStmt(block2),
            start: loc.0,
            end: loc.1,
                })
    }
    fn if_else(&mut self, loc: Location, blocks: Vec<(Expr, Vec<Stmt>)>) -> Result<Stmt, Error> {
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
            start: loc.0,
            end: loc.1,
                })
    }
    fn whiles(&mut self, loc: Location, cond: Expr, block: Vec<Stmt>) -> Result<Stmt, Error> {
                let cond = self.visit_expr(cond)?;
                let mut block2: Block = vec![];
                for s in block {
                    block2.push(self.visit_stmt(s)?)
                }
                Ok(Stmt {
                    val: StmtType::WhileStmt(cond, block2),
            start: loc.0,
            end: loc.1,
                })
    }
    fn fun(&mut self, loc: Location, name: Token, params: Vec<Token>, block: Vec<Stmt>) -> Result<Stmt, Error> {
                let mut block2: Block = vec![];
                for s in block {
                    block2.push(self.visit_stmt(s)?)
                }
                Ok(Stmt {
                    val: StmtType::FunDeclStmt(name, params, block2),
            start: loc.0,
            end: loc.1,
                })
    }
    fn retur(&mut self, loc: Location, expr: Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::ReturnStmt(self.visit_expr(expr)?),
            start: loc.0,
            end: loc.1,
        })
    }
    fn cont(&mut self, loc: Location) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::ContinueStmt,
            start: loc.0,
            end: loc.1,
        })
    }
    fn brek(&mut self, loc: Location) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::ContinueStmt,
            start: loc.0,
            end: loc.1,
        })
    }
}

impl ExprVisitor<Expr> for Reassociate {
    fn unit(&mut self, loc: Location) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Unit,
            start: loc.0,
            end: loc.1,
        })
    }
    fn int(&mut self, loc: Location, n: i32) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Int(n),
            start: loc.0,
            end: loc.1,
        })
    }
    fn float(&mut self, loc: Location, n: f32) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Float(n),
            start: loc.0,
            end: loc.1,
        })
    }
    fn string(&mut self, loc: Location, s: String) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::String(s),
            start: loc.0,
            end: loc.1,
        })
    }
    fn bool(&mut self, loc: Location, b: bool) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Bool(b),
            start: loc.0,
            end: loc.1,
        })
    }
    fn identifier(&mut self, loc: Location, ident: String) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Identifier(ident),
            start: loc.0,
            end: loc.1,
        })
    }
    fn parens(&mut self, loc: Location, expr: Expr) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Parens(self.visit_expr(expr)?.into()),
            start: loc.0,
            end: loc.1,
        })
    }
    fn call(&mut self, loc: Location, callee: Expr, args: Vec<Expr>) -> Result<Expr, Error> {
        let mut args2 = vec![];
        for arg in args {
            args2.push(self.visit_expr(arg)?);
        }
        Ok(Expr {
            val: ExprType::Call(self.visit_expr(callee)?.into(), args2),
            start: loc.0,
            end: loc.1,
        })
    }
    fn unary(&mut self, loc: Location, op: Token, expr: Expr) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::UnaryOperation(op, self.visit_expr(expr)?.into()),
            start: loc.0,
            end: loc.1,
        })
    }
    fn binary(&mut self, _: Location, left: Expr, op: Token, right: Expr) -> Result<Expr, Error> {
        self.reassoc(left, op, right)
    }
}
