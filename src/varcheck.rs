use crate::{environment::Environment, error::Error, exprstmt::*, token::*, value::*, visitor::*};

use std::collections::HashMap;

pub fn varcheck(builtins: HashMap<String, ValueType>, stmt: &Vec<Stmt>) -> Result<Vec<Stmt>, Error> {
    VarCheck {
        env: Environment::new(builtins),
    }
    .check_block(stmt)?;
    Ok(stmt.clone())
}

struct VarCheck {
    env: Environment,
}

impl VarCheck {
    fn check_block(&mut self, block: &Vec<Stmt>) -> Result<(), Error> {
        self.env.add_scope();
        for s in block {
            match &s.val {
                StmtType::VarDeclStmt(
                    Token {
                        val: TokenType::Identifier(name),
                        ..
                    },
                    expr,
                ) => {
                    self.visit_expr(expr)?;
                    if self.env.contains(name) {
                        return Err(Error {
                            msg: "Already declared variable".to_string(),
                            lines: vec![s.loc()],
                        });
                    }
                    // give dummy values
                    // it is always going to succeed (as I already check for the existence)
                    self.env.insert(
                        &Token { val: TokenType::Identifier(name.to_string()), start: 0, end: 0 },
                        Value { val: ValueType::Unit, start: 0, end: 0 }
                    ).unwrap();
                }
                StmtType::AssignStmt(
                    Token {
                        val: TokenType::Identifier(name),
                        ..
                    },
                    expr,
                ) => {
                    self.visit_expr(expr)?;
                    if !self.env.contains(name) {
                        return Err(Error {
                            msg: "Undeclared variable".to_string(),
                            lines: vec![s.loc()],
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
                _ => {
                    self.visit_stmt(s)?;
                }
            }
        }
        self.env.remove_scope();
        Ok(())
    }
}

impl StmtVisitor<Stmt> for VarCheck {
    // for these there is nothing to check (yet)
    fn var_decl(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::VarDeclStmt(_, expr) = &stmt.val else {
            unreachable!()
        };
        self.visit_expr(expr)?;
        Ok(stmt.clone())
    }
    fn assignment(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::AssignStmt(_, expr) = &stmt.val else {
            unreachable!()
        };
        self.visit_expr(expr)?;
        Ok(stmt.clone())
    }
    fn expr(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::ExprStmt(expr) = &stmt.val else {
            unreachable!()
        };
        self.visit_expr(expr)?;
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
            self.visit_expr(&block.0)?;
            self.check_block(&block.1)?;
        }
        Ok(stmt.clone())
    }
    fn whiles(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::WhileStmt(cond, block) = &stmt.val else {
            unreachable!()
        };
        self.visit_expr(cond)?;
        self.check_block(block)?;
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
        let ExprType::Identifier(name) = &expr.val else {
            unreachable!()
        };
        if !self.env.contains(name) {
            return Err(Error {
                msg: "Undeclared variable".to_string(),
                lines: vec![expr.loc()],
            });
        }
        Ok(expr.clone())
    }
    fn parens(&mut self, expr: &Expr) -> Result<Expr, Error> {
        let ExprType::Parens(expr2) = &expr.val else {
            unreachable!()
        };
        self.visit_expr(expr2)?;
        Ok(expr.clone())
    }
    fn call(&mut self, expr: &Expr) -> Result<Expr, Error> {
        let ExprType::Call(callee, args) = &expr.val else {
            unreachable!()
        };
        self.visit_expr(callee)?;
        for arg in args {
            self.visit_expr(arg)?;
        }
        Ok(expr.clone())
    }
    fn unary(&mut self, expr: &Expr) -> Result<Expr, Error> {
        let ExprType::UnaryOperation(_, expr2) = &expr.val else {
            unreachable!()
        };
        self.visit_expr(expr2)?;
        Ok(expr.clone())
    }
    fn binary(&mut self, expr: &Expr) -> Result<Expr, Error> {
        let ExprType::BinaryOperation(left, _, right) = &expr.val else {
            unreachable!()
        };
        self.visit_expr(left)?;
        self.visit_expr(right)?;
        Ok(expr.clone())
    }
}
