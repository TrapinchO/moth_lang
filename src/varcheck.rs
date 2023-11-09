use crate::error::Error;
use crate::exprstmt::*;
use crate::token::*;
use crate::visitor::*;

use std::collections::HashSet;

pub fn varcheck(stmt: &Vec<Stmt>) -> Result<Vec<Stmt>, Error> {
    let mut ls = vec![];
    for s in stmt {
        ls.push(VarCheck{}.visit_stmt(s)?)
    }
    Ok(ls)
}


struct VarCheck;

impl VarCheck {
    fn check_block(&mut self, block: Vec<Stmt>) -> Result<(), Error> {
        let mut vars = HashSet::new();
        for s in block {
            match &s.val {
                StmtType::VarDeclStmt(Token { val: TokenType::Identifier(name) }, start: _, end: ..) => vars.insert(name),
                StmtType::AssignStmt(Token { val: TokenType::Identifier(name) }, start: _, end: ..) => {
                    if !vars.contains(name) {
                        return Err(Error {
                            msg: "Unknown variable".to_string(),
                            lines: vec![],
                        });
                    }
                },
                s2 @ StmtType::BlockStmt(..) => self.visit_stmt(s2)?,
                s2 @ StmtType::IfStmt(..) => self.visit_stmt(s2)?,
                s2 @ StmtType::WhileStmt(..) => self.visit_stmt(s2)?,
                _ => {}
            }
        }
        Ok(())
    }
}

impl StmtVisitor<Stmt> for VarCheck {
    // for these there is nothing to check (yet)
    fn var_decl(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        Ok(stmt)
    }
    fn assignment(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        Ok(stmt)
    }
    fn expr(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        Ok(stmt)
    }
    // go through
    fn block(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::BlockStmt(block) = &stmt.val else { unreachable!() };
        self.check_block(block)?;
        Ok(stmt)
    }
    fn if_else(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::IfStmt(blocks) = &stmt.val else { unreachable!() };
        for block in blocks {
            self.check_block(block)?;
        }
        Ok(stmt)
    }
    fn whiles(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::WhileStmt(cond, block) = &stmt.val else { unreachable!() };
        self.check_block(block)?;
        Ok(stmt)
    }
    fn print(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        Ok(stmt)
    }
}

impl ExprVisitor<Expr> for VarCheck {
    fn int(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
    fn float(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
    fn string(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
    fn bool(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
    fn identifier(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
    fn parens(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
    fn unary(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
    fn binary(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
}
