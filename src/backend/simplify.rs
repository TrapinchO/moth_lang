use crate::{associativity::Precedence, error::Error, exprstmt, located::Location, visitor::{ExprVisitor, StmtVisitor}};

use super::lowexprstmt::{Expr, ExprType, Stmt, StmtType};


pub fn simplify(ast: Vec<exprstmt::Stmt>) -> Result<Vec<Stmt>, Error> {
    Simplifier.simplify(ast)
}

struct Simplifier;

impl Simplifier {
    pub fn simplify(&mut self, ast: Vec<exprstmt::Stmt>) -> Result<Vec<Stmt>, Error> {
        let mut ls = vec![];
        for s in ast {
            ls.push(self.visit_stmt(s)?);
        }
        Ok(ls)
    }
}

impl ExprVisitor<Expr> for Simplifier {
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

    fn list(&mut self, loc: Location, expr: Vec<exprstmt::Expr>) -> Result<Expr, Error> {
        let mut ls = vec![];
        for e in expr {
            ls.push(self.visit_expr(e)?);
        }
        Ok(Expr {
            val: ExprType::List(ls),
            loc,
        })
    }

    fn call(&mut self, loc: Location, callee: exprstmt::Expr, args: Vec<exprstmt::Expr>) -> Result<Expr, Error> {
        let mut ls = vec![];
        for e in args {
            ls.push(self.visit_expr(e)?);
        }
        Ok(Expr {
            val: ExprType::Call(self.visit_expr(callee)?.into(), ls),
            loc,
        })
    }

    fn index(&mut self, loc: Location, expr2: exprstmt::Expr, idx: exprstmt::Expr) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Index(self.visit_expr(expr2)?.into(), self.visit_expr(idx)?.into()),
            loc,
        })
    }

    fn lambda(&mut self, loc: Location, params: Vec<exprstmt::Identifier>, body: Vec<exprstmt::Stmt>) -> Result<Expr, Error> {
        let mut bl = vec![];
        for s in body {
            bl.push(self.visit_stmt(s)?);
        }
        Ok(Expr {
            val: ExprType::Lambda(params, bl),
            loc,
        })
    }

    // dont do anything anymore
    fn parens(&mut self, _: Location, expr: exprstmt::Expr) -> Result<Expr, Error> {
        self.visit_expr(expr)
    }

    // change into a function call
    fn unary(&mut self, loc: Location, op: exprstmt::Symbol, expr: exprstmt::Expr) -> Result<Expr, Error> {
        let name = match op.val.as_str() {
            "!" => "$$not".to_string(),
            "-" => "$$neg".to_string(),
            _ => unreachable!()
        };
        Ok(Expr {
            val: ExprType::Call(
                Expr { val: ExprType::Identifier(name), loc: op.loc }.into(),
                vec![self.visit_expr(expr)?]),
            loc,
        })
    }

    fn binary(&mut self, loc: Location, left: exprstmt::Expr, op: exprstmt::Symbol, right: exprstmt::Expr) -> Result<Expr, Error> {
        Ok(Expr {
            val: ExprType::Call(
                Expr { val: ExprType::Identifier(op.val), loc: op.loc }.into(),
                vec![self.visit_expr(left)?, self.visit_expr(right)?]),
            loc,
        })
    }
}


impl StmtVisitor<Stmt> for Simplifier {
    fn expr(&mut self, loc: Location, expr: exprstmt::Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::ExprStmt(self.visit_expr(expr)?),
            loc,
        })
    }

    fn var_decl(&mut self, loc: Location, ident: exprstmt::Identifier, expr: exprstmt::Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::VarDeclStmt(ident, self.visit_expr(expr)?),
            loc,
        })
    }

    fn assignment(&mut self, loc: Location, ident: exprstmt::Identifier, expr: exprstmt::Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::AssignStmt(ident, self.visit_expr(expr)?),
            loc,
        })
    }

    fn assignindex(&mut self, loc: Location, ls: exprstmt::Expr, idx: exprstmt::Expr, val: exprstmt::Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::AssignIndexStmt(
                self.visit_expr(ls)?,
                self.visit_expr(idx)?,
                self.visit_expr(val)?),
            loc,
        })
    }

    fn block(&mut self, loc: Location, block: Vec<exprstmt::Stmt>) -> Result<Stmt, Error> {
        let mut bl = vec![];
        for s in block {
            bl.push(self.visit_stmt(s)?);
        }
        Ok(Stmt {
            val: StmtType::BlockStmt(bl),
            loc,
        })
    }

    fn if_else(&mut self, loc: Location, blocks: Vec<(exprstmt::Expr, Vec<exprstmt::Stmt>)>) -> Result<Stmt, Error> {
        let mut bl = vec![];
        for (c, b) in blocks {
            let mut block = vec![];
            for s in b {
                block.push(self.visit_stmt(s)?);
            }
            bl.push((self.visit_expr(c)?, block));
        }
        Ok(Stmt {
            val: StmtType::IfStmt(bl),
            loc,
        })
    }

    fn whiles(&mut self, loc: Location, cond: exprstmt::Expr, block: Vec<exprstmt::Stmt>) -> Result<Stmt, Error> {
        let mut bl = vec![];
        for s in block {
            bl.push(self.visit_stmt(s)?);
        }
        Ok(Stmt {
            val: StmtType::WhileStmt(self.visit_expr(cond)?, bl),
            loc,
        })
    }

    fn retur(&mut self, loc: Location, expr: exprstmt::Expr) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::ReturnStmt(self.visit_expr(expr)?),
            loc,
        })
    }

    fn brek(&mut self, loc: Location) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::BreakStmt,
            loc,
        })
    }

    fn cont(&mut self, loc: Location) -> Result<Stmt, Error> {
        Ok(Stmt {
            val: StmtType::ContinueStmt,
            loc,
        })
    }

    fn fun(&mut self, loc: Location, name: exprstmt::Identifier, params: Vec<exprstmt::Identifier>, block: Vec<exprstmt::Stmt>) -> Result<Stmt, Error> {
        let mut bl = vec![];
        for s in block {
            bl.push(self.visit_stmt(s)?);
        }
        Ok(Stmt {
            val: StmtType::VarDeclStmt(name, Expr { val: ExprType::Lambda(params, bl), loc }),
            loc,
        })
    }

    fn operator(
            &mut self,
            loc: Location,
            name: exprstmt::Symbol,
            params: (exprstmt::Identifier, exprstmt::Identifier),
            block: Vec<exprstmt::Stmt>,
            _: Precedence,
        ) -> Result<Stmt, Error> {
        self.fun(loc, name, vec![params.0, params.1], block)
    }
}
