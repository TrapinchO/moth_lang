use crate::{
    associativity::Precedence,
    error::Error,
    exprstmt,
    located::Location,
    visitor::{ExprVisitor, StmtVisitor},
};

use super::lowexprstmt::{LExpr, Expr, LStmt, Stmt};

pub fn simplify(ast: Vec<exprstmt::LStmt>) -> Result<Vec<LStmt>, Error> {
    Simplifier.simplify(ast)
}

struct Simplifier;

impl Simplifier {
    pub fn simplify(&mut self, ast: Vec<exprstmt::LStmt>) -> Result<Vec<LStmt>, Error> {
        let mut ls = vec![];
        for s in ast {
            ls.push(self.visit_stmt(s)?);
        }
        Ok(ls)
    }
}

impl ExprVisitor<LExpr> for Simplifier {
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

    fn list(&mut self, loc: Location, expr: Vec<exprstmt::LExpr>) -> Result<LExpr, Error> {
        let mut ls = vec![];
        for e in expr {
            ls.push(self.visit_expr(e)?);
        }
        Ok(LExpr {
            val: Expr::List(ls),
            loc,
        })
    }

    fn call(&mut self, loc: Location, callee: exprstmt::LExpr, args: Vec<exprstmt::LExpr>) -> Result<LExpr, Error> {
        let callee2 = self.visit_expr(callee)?;
        let mut ls = vec![];
        for e in args {
            ls.push(self.visit_expr(e)?);
        }
        Ok(LExpr {
            val: Expr::Call(callee2.into(), ls),
            loc,
        })
    }

    fn index(&mut self, loc: Location, expr2: exprstmt::LExpr, idx: exprstmt::LExpr) -> Result<LExpr, Error> {
        Ok(LExpr {
            val: Expr::Index(self.visit_expr(expr2)?.into(), self.visit_expr(idx)?.into()),
            loc,
        })
    }

    fn lambda(
        &mut self,
        loc: Location,
        params: Vec<exprstmt::Identifier>,
        body: Vec<exprstmt::LStmt>,
    ) -> Result<LExpr, Error> {
        let mut bl = vec![];
        for s in body {
            bl.push(self.visit_stmt(s)?);
        }
        Ok(LExpr {
            val: Expr::Lambda(params, bl),
            loc,
        })
    }

    // dont do anything anymore
    fn parens(&mut self, _: Location, expr: exprstmt::LExpr) -> Result<LExpr, Error> {
        self.visit_expr(expr)
    }

    // change into a function call
    fn unary(&mut self, loc: Location, op: exprstmt::Symbol, expr: exprstmt::LExpr) -> Result<LExpr, Error> {
        let expr2 = self.visit_expr(expr)?;
        let val = match &expr2.val {
            Expr::Int(n) if op.val.as_str() == "-" => Expr::Int(-n),
            Expr::Float(n) if op.val.as_str() == "-" => Expr::Float(-n),
            Expr::Bool(b) if op.val.as_str() == "!" => Expr::Bool(!b),
            _ => {
                let name = match op.val.as_str() {
                    "!" => "$$not".to_string(),
                    "-" => "$$neg".to_string(),
                    _ => unreachable!(),
                };
                return Ok(LExpr {
                    val: Expr::Call(
                        LExpr {
                            val: Expr::Identifier(name),
                            loc: op.loc,
                        }
                        .into(),
                        vec![expr2],
                    ),
                    loc,
                });
            }
        };
        Ok(LExpr { val, loc })
    }

    fn binary(
        &mut self,
        loc: Location,
        left: exprstmt::LExpr,
        op: exprstmt::Symbol,
        right: exprstmt::LExpr,
    ) -> Result<LExpr, Error> {
        let left2 = self.visit_expr(left)?;
        let right2 = self.visit_expr(right)?;

        // try to fold constant literals
        match (&left2.val, &right2.val) {
            (Expr::Int(n1), Expr::Int(n2)) => {
                let val = match op.val.as_str() {
                    "+" => n1 + n2,
                    "-" => n1 - n2,
                    "*" => n1 * n2,
                    "/" => n1 / n2,
                    "%" => n1 % n2,
                    // it is not a "primitive" operator, cannot be folded
                    _ => {
                        return Ok(LExpr {
                            val: Expr::Call(
                                LExpr {
                                    val: Expr::Identifier(op.val),
                                    loc: op.loc,
                                }
                                .into(),
                                vec![left2, right2],
                            ),
                            loc,
                        })
                    }
                };
                Ok(LExpr {
                    val: Expr::Int(val),
                    loc,
                })
            }
            // TODO: try to merge this into int arm
            (Expr::Float(n1), Expr::Float(n2)) => {
                let val = match op.val.as_str() {
                    "+" => n1 + n2,
                    "-" => n1 - n2,
                    "*" => n1 * n2,
                    "/" => n1 / n2,
                    "%" => n1 % n2,
                    _ => {
                        return Ok(LExpr {
                            val: Expr::Call(
                                LExpr {
                                    val: Expr::Identifier(op.val),
                                    loc: op.loc,
                                }
                                .into(),
                                vec![left2, right2],
                            ),
                            loc,
                        })
                    }
                };
                Ok(LExpr {
                    val: Expr::Float(val),
                    loc,
                })
            }
            // arguments are not numbers, cannot be folded
            _ => Ok(LExpr {
                val: Expr::Call(
                    LExpr {
                        val: Expr::Identifier(op.val),
                        loc: op.loc,
                    }
                    .into(),
                    vec![left2, right2],
                ),
                loc,
            }),
        }
    }
    fn field(&mut self, loc: Location, expr: exprstmt::LExpr, name: exprstmt::Identifier) -> Result<LExpr, Error> {
        Ok(LExpr {
            val: Expr::FieldAccess(self.visit_expr(expr)?.into(), name),
            loc,
        })
    }
    fn method(
        &mut self,
        loc: Location,
        callee: exprstmt::LExpr,
        name: exprstmt::Identifier,
        args: Vec<exprstmt::LExpr>,
    ) -> Result<LExpr, Error> {
        let callee2 = self.visit_expr(callee)?;
        let mut ls = vec![];
        for e in args {
            ls.push(self.visit_expr(e)?);
        }
        Ok(LExpr {
            val: Expr::MethodAccess(callee2.into(), name, ls),
            loc,
        })
    }
}

impl StmtVisitor<LStmt> for Simplifier {
    fn expr(&mut self, loc: Location, expr: exprstmt::LExpr) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::Expr(self.visit_expr(expr)?),
            loc,
        })
    }

    fn var_decl(&mut self, loc: Location, ident: exprstmt::Identifier, expr: exprstmt::LExpr) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::VarDecl(ident, self.visit_expr(expr)?),
            loc,
        })
    }

    fn assignment(&mut self, loc: Location, ident: exprstmt::Identifier, expr: exprstmt::LExpr) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::Assign(ident, self.visit_expr(expr)?),
            loc,
        })
    }

    fn assignindex(
        &mut self,
        loc: Location,
        ls: exprstmt::LExpr,
        idx: exprstmt::LExpr,
        val: exprstmt::LExpr,
    ) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::AssignIndex(self.visit_expr(ls)?, self.visit_expr(idx)?, self.visit_expr(val)?),
            loc,
        })
    }

    fn block(&mut self, loc: Location, block: Vec<exprstmt::LStmt>) -> Result<LStmt, Error> {
        let mut bl = vec![];
        for s in block {
            bl.push(self.visit_stmt(s)?);
        }
        Ok(LStmt {
            val: Stmt::Block(bl),
            loc,
        })
    }

    fn if_else(&mut self, loc: Location, blocks: Vec<(exprstmt::LExpr, Vec<exprstmt::LStmt>)>, els: Option<Vec<exprstmt::LStmt>>) -> Result<LStmt, Error> {
        let mut bl = vec![];
        for (c, b) in blocks {
            let mut block = vec![];
            for s in b {
                block.push(self.visit_stmt(s)?);
            }
            bl.push((self.visit_expr(c)?, block));
        }
        if let Some(else_bl) = els {
            let mut block = vec![];
            for s in else_bl {
                block.push(self.visit_stmt(s)?);
            }
            bl.push((LExpr { val: Expr::Bool(true), loc }, block));
        }
        Ok(LStmt {
            val: Stmt::If(bl),
            loc,
        })
    }

    fn whiles(&mut self, loc: Location, cond: exprstmt::LExpr, block: Vec<exprstmt::LStmt>) -> Result<LStmt, Error> {
        let mut bl = vec![];
        for s in block {
            bl.push(self.visit_stmt(s)?);
        }
        Ok(LStmt {
            val: Stmt::While(self.visit_expr(cond)?, bl),
            loc,
        })
    }

    fn retur(&mut self, loc: Location, expr: exprstmt::LExpr) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::Return(self.visit_expr(expr)?),
            loc,
        })
    }

    fn brek(&mut self, loc: Location) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::Break,
            loc,
        })
    }

    fn cont(&mut self, loc: Location) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::Continue,
            loc,
        })
    }

    fn fun(
        &mut self,
        loc: Location,
        name: exprstmt::Identifier,
        params: Vec<exprstmt::Identifier>,
        block: Vec<exprstmt::LStmt>,
    ) -> Result<LStmt, Error> {
        let mut bl = vec![];
        for s in block {
            bl.push(self.visit_stmt(s)?);
        }
        Ok(LStmt {
            val: Stmt::VarDecl(
                name,
                LExpr {
                    val: Expr::Lambda(params, bl),
                    loc,
                },
            ),
            loc,
        })
    }

    fn operator(
        &mut self,
        loc: Location,
        name: exprstmt::Symbol,
        params: (exprstmt::Identifier, exprstmt::Identifier),
        block: Vec<exprstmt::LStmt>,
        _: Precedence,
    ) -> Result<LStmt, Error> {
        self.fun(loc, name, vec![params.0, params.1], block)
    }

    fn struc(
        &mut self,
        loc: Location,
        name: exprstmt::Identifier,
        fields: Vec<exprstmt::Identifier>,
    ) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::Struct(name, fields),
            loc,
        })
    }

    fn assignstruc(
        &mut self,
        loc: Location,
        expr1: exprstmt::LExpr,
        name: exprstmt::Identifier,
        expr2: exprstmt::LExpr,
    ) -> Result<LStmt, Error> {
        Ok(LStmt {
            val: Stmt::AssignStruct(self.visit_expr(expr1)?, name, self.visit_expr(expr2)?),
            loc,
        })
    }
    fn imp(&mut self, loc: Location, name: exprstmt::Identifier, block: Vec<exprstmt::LStmt>) -> Result<LStmt, Error> {
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
