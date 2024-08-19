use std::collections::HashMap;

use crate::{associativity::{Associativity, Precedence}, error::Error, error::ErrorType, exprstmt::*, located::Location, visitor::{ExprVisitor, StmtVisitor}};

pub fn reassociate(ops: HashMap<String, Precedence>, stmt: Vec<Stmt>) -> Result<Vec<Stmt>, Error> {
    let mut reassoc = Reassociate { ops };
    let mut ls = vec![];
    for s in stmt {
        ls.push(reassoc.reassociate(s)?);
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
    fn reassoc(&mut self, left: Expr, op1: Symbol, right: Expr) -> Result<Expr, Error> {
        let left = self.visit_expr(left)?;
        let right = self.visit_expr(right)?;
        // not a binary operation, no need to reassociate it
        let ExprType::BinaryOperation(left2, op2, right2) = right.val.clone() else {
            return Ok(Expr {
                loc: Location {
                    start: left.loc.start,
                    end: right.loc.end,
                },
                val: ExprType::BinaryOperation(left.into(), op1, right.into()),
            });
        };

        let op1_sym = &op1.val;
        let prec1 = self.ops.get(op1_sym).ok_or(Error {
            msg: ErrorType::OperatorNotFound(op1.val.clone()),
            lines: vec![op1.loc],
        })?;

        let op2_sym = &op2.val;
        let prec2 = self.ops.get(op2_sym).ok_or(Error {
            msg: ErrorType::OperatorNotFound(op2.val.clone()),
            lines: vec![op2.loc],
        })?;
        // TODO: make functions like in the SO answer?
        match prec1.prec.cmp(&prec2.prec) {
            std::cmp::Ordering::Greater => {
                let left = self.reassoc(left, op1, *left2)?.into();
                Ok(Expr {
                    loc: right2.loc,
                    val: ExprType::BinaryOperation(left, op2, right2),
                })
            }

            std::cmp::Ordering::Less => Ok(Expr {
                loc: Location {
                    start: left.loc.start,
                    end: right.loc.end,
                },
                val: ExprType::BinaryOperation(left.into(), op1, right.into()),
            }),

            std::cmp::Ordering::Equal => match (prec1.assoc, prec2.assoc) {
                (Associativity::Left, Associativity::Left) => {
                    let left = self.reassoc(left, op1, *left2)?.into();
                    Ok(Expr {
                        loc: right2.loc,
                        val: ExprType::BinaryOperation(left, op2, right2),
                    })
                }
                (Associativity::Right, Associativity::Right) => Ok(Expr {
                    loc: Location {
                        start: left.loc.start,
                        end: right.loc.end,
                    },
                    val: ExprType::BinaryOperation(left.into(), op1, right.into()),
                }),
                _ => Err(Error {
                    msg: ErrorType::IncompatiblePrecedence(op1.val, *prec1, op2.val, *prec2),
                    lines: vec![op1.loc, op2.loc],
                }),
            },
        }
    }
}

impl StmtVisitor<Stmt> for Reassociate {
    fn expr(&mut self, loc: Location, expr: Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::ExprStmt(self.visit_expr(expr)?),
            loc,
        })
    }
    fn var_decl(&mut self, loc: Location, ident: Identifier, expr: Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::VarDeclStmt(ident, self.visit_expr(expr)?),
            loc,
        })
    }
    fn assignment(&mut self, loc: Location, ident: Identifier, expr: Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::AssignStmt(ident, self.visit_expr(expr)?),
            loc,
        })
    }
    fn assignindex(&mut self, loc: Location, ls: Expr, idx: Expr, val: Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::AssignIndexStmt(
                self.visit_expr(ls)?,
                self.visit_expr(idx)?,
                self.visit_expr(val)?
            ),
            loc,
        })
    }
    fn block(&mut self, loc: Location, block: Vec<Stmt>) -> Result<Stmt, Error> {
        let mut block2: Vec<Stmt> = vec![];
        for s in block {
            block2.push(self.visit_stmt(s)?);
        }
        Ok(Stmt {
            val: StmtType::BlockStmt(block2),
            loc,
        })
    }
    fn if_else(&mut self, loc: Location, blocks: Vec<(Expr, Vec<Stmt>)>) -> Result<Stmt, Error> {
        let mut blocks_result = vec![];
        for (cond, stmts) in blocks {
            let mut block = vec![];
            for s in stmts {
                block.push(self.visit_stmt(s)?);
            }
            blocks_result.push((self.visit_expr(cond)?, block));
        }

        Ok(Stmt {
            val: StmtType::IfStmt(blocks_result),
            loc,
        })
    }
    fn whiles(&mut self, loc: Location, cond: Expr, block: Vec<Stmt>) -> Result<Stmt, Error> {
        let cond = self.visit_expr(cond)?;
        let mut block2 = vec![];
        for s in block {
            block2.push(self.visit_stmt(s)?);
        }
        Ok(Stmt {
            val: StmtType::WhileStmt(cond, block2),
            loc,
        })
    }
    fn fun(
        &mut self,
        loc: Location,
        name: Identifier,
        params: Vec<Identifier>,
        block: Vec<Stmt>,
    ) -> Result<Stmt, Error> {
        let mut block2 = vec![];
        for s in block {
            block2.push(self.visit_stmt(s)?);
        }
        Ok(Stmt {
            val: StmtType::FunDeclStmt(name, params, block2),
            loc,
        })
    }
    fn operator(
        &mut self,
        loc: Location,
        name: Symbol,
        params: (Identifier, Identifier),
        block: Vec<Stmt>,
        prec: Precedence,
    ) -> Result<Stmt, Error> {
        let s = name.val.clone();
        self.ops.insert(s, prec);
        let mut block2 = vec![];
        for s in block {
            block2.push(self.visit_stmt(s)?);
        }
        Ok(Stmt {
            val: StmtType::OperatorDeclStmt(name, params, block2, prec),
            loc,
        })
    }
    fn retur(&mut self, loc: Location, expr: Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::ReturnStmt(self.visit_expr(expr)?),
            loc,
        })
    }
    fn cont(&mut self, loc: Location) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::ContinueStmt,
            loc,
        })
    }
    fn brek(&mut self, loc: Location) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::ContinueStmt,
            loc,
        })
    }
    fn struc(&mut self, loc: Location, name: Identifier, fields: Vec<Identifier>) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::StructStmt(name, fields),
            loc,
        })
    }
    fn assignstruc(&mut self, loc: Location, expr1: Expr, name: Identifier, expr2: Expr) -> Result<Stmt, Error> {
        let expr1 = self.visit_expr(expr1)?;
        let expr2 = self.visit_expr(expr2)?;
        Ok(Stmt {
            val: StmtType::AssignStructStmt(expr1, name, expr2),
            loc,
        })
    }
}

impl ExprVisitor<Expr> for Reassociate {
    fn unit(&mut self, loc: Location) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Unit,
            loc,
        })
    }
    fn int(&mut self, loc: Location, n: i32) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Int(n),
            loc,
        })
    }
    fn float(&mut self, loc: Location, n: f32) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Float(n),
            loc,
        })
    }
    fn string(&mut self, loc: Location, s: String) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::String(s),
            loc,
        })
    }
    fn bool(&mut self, loc: Location, b: bool) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Bool(b),
            loc,
        })
    }
    fn identifier(&mut self, loc: Location, ident: String) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Identifier(ident),
            loc,
        })
    }
    fn parens(&mut self, loc: Location, expr: Expr) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Parens(self.visit_expr(expr)?.into()),
            loc,
        })
    }
    fn call(&mut self, loc: Location, callee: Expr, args: Vec<Expr>) -> Result<Expr, Error> {
        let mut args2 = vec![];
        for arg in args {
            args2.push(self.visit_expr(arg)?);
        }
        Ok(Expr {
            val: ExprType::Call(self.visit_expr(callee)?.into(), args2),
            loc,
        })
    }
    fn unary(&mut self, loc: Location, op: Symbol, expr: Expr) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::UnaryOperation(op, self.visit_expr(expr)?.into()),
            loc,
        })
    }
    fn binary(&mut self, _: Location, left: Expr, op: Symbol, right: Expr) -> Result<Expr, Error> {
        self.reassoc(left, op, right)
    }
    fn list(&mut self, loc: Location, ls: Vec<Expr>) -> Result<Expr, Error> {
        let mut ls2 = vec![];
        for e in ls {
            ls2.push(self.visit_expr(e)?);
        }
        Ok(Expr {
            val: ExprType::List(ls2),
            loc,
        })
    }
    fn index(&mut self, loc: Location, expr2: Expr, idx: Expr) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Index(
                 self.visit_expr(expr2)?.into(),
                 self.visit_expr(idx)?.into()
            ),
            loc,
        })
    }
    fn lambda(&mut self, loc: Location, params: Vec<Identifier>, body: Vec<Stmt>) -> Result<Expr, Error> {
        let mut body2 = vec![];
        for s in body {
            body2.push(self.visit_stmt(s)?);
        }
        Ok(Expr {
            val: ExprType::Lambda(params, body2),
            loc,
        })
    }
    fn field(&mut self, loc: Location, expr: Expr, name: Identifier) -> Result<Expr, Error> {
        let expr2 = self.visit_expr(expr)?;
        Ok(Expr {
            val: ExprType::FieldAccess(expr2.into(), name),
            loc,
        })
    }
}
