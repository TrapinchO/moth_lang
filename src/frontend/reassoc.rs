use std::collections::HashMap;

use crate::{
    associativity::{Associativity, Precedence},
    error::Error,
    error::ErrorType,
    exprstmt::*,
    located::Location,
    visitor::{ExprVisitor, StmtVisitor},
};

pub fn reassociate(ops: HashMap<String, Precedence>, stmt: Vec<LStmt>) -> Result<Vec<LStmt>, Error> {
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
    pub fn reassociate(&mut self, stmt: LStmt) -> Result<LStmt, Error> {
        self.visit_stmt(stmt)
    }

    // the one method this file exists for
    // binary operator reassociation
    // https://stackoverflow.com/a/67992584
    // TODO: play with references and stuff once I dare again
    fn reassoc(&mut self, left: LExpr, op1: Symbol, right: LExpr) -> Result<LExpr, Error> {
        let left = self.visit_expr(left)?;
        let right = self.visit_expr(right)?;
        // not a binary operation, no need to reassociate it
        let Expr::BinaryOperation(left2, op2, right2) = right.val.clone() else {
            return Ok(LExpr {
                loc: Location {
                    start: left.loc.start,
                    end: right.loc.end,
                },
                val: Expr::BinaryOperation(left.into(), op1, right.into()),
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
                Ok(LExpr {
                    loc: right2.loc,
                    val: Expr::BinaryOperation(left, op2, right2),
                })
            }

            std::cmp::Ordering::Less => Ok(LExpr {
                loc: Location {
                    start: left.loc.start,
                    end: right.loc.end,
                },
                val: Expr::BinaryOperation(left.into(), op1, right.into()),
            }),

            std::cmp::Ordering::Equal => match (prec1.assoc, prec2.assoc) {
                (Associativity::Left, Associativity::Left) => {
                    let left = self.reassoc(left, op1, *left2)?.into();
                    Ok(LExpr {
                        loc: right2.loc,
                        val: Expr::BinaryOperation(left, op2, right2),
                    })
                }
                (Associativity::Right, Associativity::Right) => Ok(LExpr {
                    loc: Location {
                        start: left.loc.start,
                        end: right.loc.end,
                    },
                    val: Expr::BinaryOperation(left.into(), op1, right.into()),
                }),
                _ => Err(Error {
                    msg: ErrorType::IncompatiblePrecedence(op1.val, *prec1, op2.val, *prec2),
                    lines: vec![op1.loc, op2.loc],
                }),
            },
        }
    }
}

impl StmtVisitor<LStmt> for Reassociate {
    fn expr(&mut self, loc: Location, expr: LExpr) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::Expr(self.visit_expr(expr)?),
            loc,
        })
    }
    fn var_decl(&mut self, loc: Location, ident: Identifier, expr: LExpr) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::VarDecl(ident, self.visit_expr(expr)?),
            loc,
        })
    }
    fn assignment(&mut self, loc: Location, ident: Identifier, expr: LExpr) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::Assign(ident, self.visit_expr(expr)?),
            loc,
        })
    }
    fn assignindex(&mut self, loc: Location, ls: LExpr, idx: LExpr, val: LExpr) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::AssignIndex(self.visit_expr(ls)?, self.visit_expr(idx)?, self.visit_expr(val)?),
            loc,
        })
    }
    fn block(&mut self, loc: Location, block: Vec<LStmt>) -> Result<LStmt, Error> {
        let mut block2: Vec<LStmt> = vec![];
        for s in block {
            block2.push(self.visit_stmt(s)?);
        }
        Ok(LStmt {
            val: Stmt::Block(block2),
            loc,
        })
    }
    fn if_else(&mut self, loc: Location, blocks: Vec<(LExpr, Vec<LStmt>)>, els: Option<Block>) -> Result<LStmt, Error> {
        let mut blocks_result = vec![];
        for (cond, stmts) in blocks {
            let mut block = vec![];
            for s in stmts {
                block.push(self.visit_stmt(s)?);
            }
            blocks_result.push((self.visit_expr(cond)?, block));
        }

        let els2 = els
            .map(|b| b.into_iter()
                .map(|s| self.visit_stmt(s))
                .collect::<Result<Vec<_>, _>>())
            .transpose()?;

        Ok(LStmt {
            val: Stmt::If(blocks_result, els2),
            loc,
        })
    }
    fn whiles(&mut self, loc: Location, cond: LExpr, block: Vec<LStmt>) -> Result<LStmt, Error> {
        let cond = self.visit_expr(cond)?;
        let mut block2 = vec![];
        for s in block {
            block2.push(self.visit_stmt(s)?);
        }
        Ok(LStmt {
            val: Stmt::While(cond, block2),
            loc,
        })
    }
    fn fun(
        &mut self,
        loc: Location,
        name: Identifier,
        params: Vec<Identifier>,
        block: Vec<LStmt>,
    ) -> Result<LStmt, Error> {
        let mut block2 = vec![];
        for s in block {
            block2.push(self.visit_stmt(s)?);
        }
        Ok(LStmt {
            val: Stmt::FunDecl(name, params, block2),
            loc,
        })
    }
    fn operator(
        &mut self,
        loc: Location,
        name: Symbol,
        params: (Identifier, Identifier),
        block: Vec<LStmt>,
        prec: Precedence,
    ) -> Result<LStmt, Error> {
        let s = name.val.clone();
        self.ops.insert(s, prec);
        let mut block2 = vec![];
        for s in block {
            block2.push(self.visit_stmt(s)?);
        }
        Ok(LStmt {
            val: Stmt::OperatorDecl(name, params, block2, prec),
            loc,
        })
    }
    fn retur(&mut self, loc: Location, expr: LExpr) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::Return(self.visit_expr(expr)?),
            loc,
        })
    }
    fn cont(&mut self, loc: Location) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::Continue,
            loc,
        })
    }
    fn brek(&mut self, loc: Location) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::Continue,
            loc,
        })
    }
    fn struc(&mut self, loc: Location, name: Identifier, fields: Vec<Identifier>) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::Struct(name, fields),
            loc,
        })
    }
    fn assignstruc(&mut self, loc: Location, expr1: LExpr, name: Identifier, expr2: LExpr) -> Result<LStmt, Error> {
        let expr1 = self.visit_expr(expr1)?;
        let expr2 = self.visit_expr(expr2)?;
        Ok(LStmt {
            val: Stmt::AssignStruct(expr1, name, expr2),
            loc,
        })
    }
    fn imp(&mut self, loc: Location, name: Identifier, block: Vec<LStmt>) -> Result<LStmt, Error> {
        let mut block2 = vec![];
        for s in block {
            block2.push(self.visit_stmt(s)?);
        }
        Ok(LStmt {
            val: Stmt::Impl(name, block2),
            loc,
        })
    }
}

impl ExprVisitor<LExpr> for Reassociate {
    fn unit(&mut self, loc: Location) -> Result<LExpr, Error> {
        Ok(LExpr {
            val: Expr::Unit,
            loc,
        })
    }
    fn int(&mut self, loc: Location, n: i32) -> Result<LExpr, Error> {
        Ok(LExpr {
            val: Expr::Int(n),
            loc,
        })
    }
    fn float(&mut self, loc: Location, n: f32) -> Result<LExpr, Error> {
        Ok(LExpr {
            val: Expr::Float(n),
            loc,
        })
    }
    fn string(&mut self, loc: Location, s: String) -> Result<LExpr, Error> {
        Ok(LExpr {
            val: Expr::String(s),
            loc,
        })
    }
    fn bool(&mut self, loc: Location, b: bool) -> Result<LExpr, Error> {
        Ok(LExpr {
            val: Expr::Bool(b),
            loc,
        })
    }
    fn identifier(&mut self, loc: Location, ident: String) -> Result<LExpr, Error> {
        Ok(LExpr {
            val: Expr::Identifier(ident),
            loc,
        })
    }
    fn parens(&mut self, loc: Location, expr: LExpr) -> Result<LExpr, Error> {
        Ok(LExpr {
            val: Expr::Parens(self.visit_expr(expr)?.into()),
            loc,
        })
    }
    fn call(&mut self, loc: Location, callee: LExpr, args: Vec<LExpr>) -> Result<LExpr, Error> {
        let mut args2 = vec![];
        for arg in args {
            args2.push(self.visit_expr(arg)?);
        }
        Ok(LExpr {
            val: Expr::Call(self.visit_expr(callee)?.into(), args2),
            loc,
        })
    }
    fn unary(&mut self, loc: Location, op: Symbol, expr: LExpr) -> Result<LExpr, Error> {
        Ok(LExpr {
            val: Expr::UnaryOperation(op, self.visit_expr(expr)?.into()),
            loc,
        })
    }
    fn binary(&mut self, _: Location, left: LExpr, op: Symbol, right: LExpr) -> Result<LExpr, Error> {
        self.reassoc(left, op, right)
    }
    fn list(&mut self, loc: Location, ls: Vec<LExpr>) -> Result<LExpr, Error> {
        let mut ls2 = vec![];
        for e in ls {
            ls2.push(self.visit_expr(e)?);
        }
        Ok(LExpr {
            val: Expr::List(ls2),
            loc,
        })
    }
    fn index(&mut self, loc: Location, expr2: LExpr, idx: LExpr) -> Result<LExpr, Error> {
        Ok(LExpr {
            val: Expr::Index(self.visit_expr(expr2)?.into(), self.visit_expr(idx)?.into()),
            loc,
        })
    }
    fn lambda(&mut self, loc: Location, params: Vec<Identifier>, body: Vec<LStmt>) -> Result<LExpr, Error> {
        let mut body2 = vec![];
        for s in body {
            body2.push(self.visit_stmt(s)?);
        }
        Ok(LExpr {
            val: Expr::Lambda(params, body2),
            loc,
        })
    }
    fn field(&mut self, loc: Location, expr: LExpr, name: Identifier) -> Result<LExpr, Error> {
        let expr2 = self.visit_expr(expr)?;
        Ok(LExpr {
            val: Expr::FieldAccess(expr2.into(), name),
            loc,
        })
    }
    fn method(&mut self, loc: Location, callee: LExpr, name: Identifier, args: Vec<LExpr>) -> Result<LExpr, Error> {
        let callee2 = self.visit_expr(callee)?;
        let mut args2 = vec![];
        for e in args {
            args2.push(self.visit_expr(e)?);
        }
        Ok(LExpr {
            val: Expr::MethodAccess(callee2.into(), name, args2),
            loc,
        })
    }
}
