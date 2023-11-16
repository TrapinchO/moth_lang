use crate::{error::Error, exprstmt::*, token::*, visitor::*};

use std::collections::HashSet;

pub fn varcheck(stmt: &Vec<Stmt>) -> Result<Vec<Stmt>, Error> {
    VarCheck {}.check_block(&stmt)?;
    Ok(stmt.clone())
}

struct VarCheck;

impl VarCheck {
    fn check_block(&mut self, block: &Vec<Stmt>) -> Result<(), Error> {
        let mut vars: HashSet<&String> = HashSet::new();
        for s in block {
            match &s.val {
                StmtType::VarDeclStmt(
                    Token {
                        val: TokenType::Identifier(name),
                        ..
                    },
                    ..,
                ) => {
                    vars.insert(name);
                }
                StmtType::AssignStmt(
                    Token {
                        val: TokenType::Identifier(name),
                        ..
                    },
                    ..,
                ) => {
                    if !vars.contains(name) {
                        return Err(Error {
                            msg: "Undeclared variable".to_string(),
                            lines: vec![(s.start, s.end)],
                        });
                    }
                }
                StmtType::BlockStmt(..) => {
                    self.visit_stmt(s)?;
                }
                StmtType::IfStmt(..) => {
                    self.visit_stmt(s)?;
                }
                StmtType::WhileStmt(..) => {
                    self.visit_stmt(s)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl StmtVisitor<Stmt> for VarCheck {
    // for these there is nothing to check (yet)
    fn var_decl(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        Ok(stmt.clone())
    }
    fn assignment(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        Ok(stmt.clone())
    }
    fn expr(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        Ok(stmt.clone())
    }
    // go through
    fn block(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::BlockStmt(block) = &stmt.val else {
            unreachable!()
        };
        self.check_block(block)?;
        Ok(stmt.clone())
    }
    fn if_else(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::IfStmt(blocks) = &stmt.val else {
            unreachable!()
        };
        for block in blocks {
            self.check_block(&block.1)?;
        }
        Ok(stmt.clone())
    }
    fn whiles(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::WhileStmt(cond, block) = &stmt.val else {
            unreachable!()
        };
        self.check_block(block)?;
        Ok(stmt.clone())
    }
    fn print(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        Ok(stmt.clone())
    }
}

impl ExprVisitor<Expr> for VarCheck {
    fn int(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr.clone())
    }
    fn float(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr.clone())
    }
    fn string(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr.clone())
    }
    fn bool(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr.clone())
    }
    fn identifier(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr.clone())
    }
    fn parens(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr.clone())
    }
    fn unary(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr.clone())
    }
    fn binary(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr.clone())
    }
}
