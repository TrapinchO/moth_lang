use crate::{
    environment::Environment,
    error::Error,
    exprstmt::*,
    token::*,
    value::*,
    visitor::*,
};

use std::collections::{HashMap, HashSet};

pub fn varcheck(builtins: HashMap<String, ValueType>, stmt: Vec<Stmt>) -> Result<Vec<Stmt>, Error> {
    VarCheck {
        env: Environment::new(builtins),
    }
    .check_block(stmt.clone())?;
    Ok(stmt)
}

struct VarCheck {
    env: Environment,
}

impl VarCheck {
    fn check_block(&mut self, block: Vec<Stmt>) -> Result<(), Error> {
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
                    self.visit_expr(expr.clone())?;

                    if self.env.contains(name) {
                        // TODO: functions behave weirdly
                        // TODO: also add the first declaration
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
                },
                StmtType::FunDeclStmt(
                    Token {
                        val: TokenType::Identifier(name),
                        ..
                    },
                    ..
                ) => {
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
                        Value { val: ValueType::Function(vec![], vec![]), start: 0, end: 0 }
                    ).unwrap();

                    self.visit_stmt(s)?;
                },
                StmtType::AssignStmt(
                    Token {
                        val: TokenType::Identifier(name),
                        ..
                    },
                    expr,
                ) => {
                    self.visit_expr(expr.clone())?;
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
                StmtType::ExprStmt(..) => {
                    self.visit_stmt(s)?;
                }
                StmtType::BreakStmt => {},
                StmtType::ContinueStmt => {},
                StmtType::ReturnStmt(..) => {
                    self.visit_stmt(s)?;
                }
                _ => {
                    // TODO: this is like the classic meme "idk what it did so I removed it"...
                    // it broke.
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
    fn var_decl(&mut self, stmt: Stmt) -> Result<Stmt, Error> {
        let StmtType::VarDeclStmt(_, expr) = stmt.val.clone() else {
            unreachable!()
        };
        self.visit_expr(expr)?;
        Ok(stmt)
    }
    fn assignment(&mut self, stmt: Stmt) -> Result<Stmt, Error> {
        let StmtType::AssignStmt(_, expr) = stmt.val.clone() else {
            unreachable!()
        };
        self.visit_expr(expr)?;
        Ok(stmt)
    }
    fn expr(&mut self, stmt: Stmt) -> Result<Stmt, Error> {
        let StmtType::ExprStmt(expr) = stmt.val.clone() else {
            unreachable!()
        };
        self.visit_expr(expr)?;
        Ok(stmt)
    }
    // go through
    fn block(&mut self, stmt: Stmt) -> Result<Stmt, Error> {
        let StmtType::BlockStmt(block) = stmt.val.clone() else {
            unreachable!()
        };
        self.check_block(block)?;
        Ok(stmt)
    }
    fn if_else(&mut self, stmt: Stmt) -> Result<Stmt, Error> {
        let StmtType::IfStmt(blocks) = stmt.val.clone() else {
            unreachable!()
        };
        for (cond, block) in blocks {
            self.visit_expr(cond)?;
            self.check_block(block)?;
        }
        Ok(stmt)
    }
    fn whiles(&mut self, stmt: Stmt) -> Result<Stmt, Error> {
        let StmtType::WhileStmt(cond, block) = stmt.val.clone() else {
            unreachable!()
        };
        self.visit_expr(cond)?;
        self.check_block(block)?;
        Ok(stmt)
    }
    fn fun(&mut self, stmt: Stmt) -> Result<Stmt, Error> {
        let StmtType::FunDeclStmt(_, params, block) = stmt.val.clone() else {
            unreachable!()
        };
        let mut params2 = HashSet::new();
        for p in params.iter() {
            let TokenType::Identifier(name) = &p.val else {
                unreachable!()
            };
            if params2.contains(name) {
                return Err(Error {
                    msg: format!("Found duplicate parameter: \"{}\"", p),
                    lines: vec![p.loc()],
                })
            }
            params2.insert(name.clone());
        }
        self.env.add_scope_vars(
            params2.iter().map(|p| { (p.clone(), ValueType::Unit) }).collect::<HashMap<_, _>>()
        );
        self.check_block(block)?;
        self.env.remove_scope();
        Ok(stmt)
    }
    fn cont(&mut self, stmt: Stmt) -> Result<Stmt, Error> {
        Ok(stmt)
    }
    fn brek(&mut self, stmt: Stmt) -> Result<Stmt, Error> {
        Ok(stmt)
    }
    fn retur(&mut self, stmt: Stmt) -> Result<Stmt, Error> {
        let StmtType::ReturnStmt(expr) = stmt.val.clone() else {
            unreachable!()
        };
        self.visit_expr(expr)?;
        Ok(stmt)
    }
}

impl ExprVisitor<()> for VarCheck {
    fn unit(&mut self, _: Expr) -> Result<(), Error> {
        Ok(())
    }
    fn int(&mut self, _: Expr) -> Result<(), Error> {
        Ok(())
    }
    fn float(&mut self, _: Expr) -> Result<(), Error> {
        Ok(())
    }
    fn string(&mut self, _: Expr) -> Result<(), Error> {
        Ok(())
    }
    fn bool(&mut self, _: Expr) -> Result<(), Error> {
        Ok(())
    }
    fn identifier(&mut self, expr: Expr) -> Result<(), Error> {
        let ExprType::Identifier(name) = expr.val.clone() else {
            unreachable!()
        };
        if !self.env.contains(&name) {
            return Err(Error {
                msg: "Undeclared variable".to_string(),
                lines: vec![expr.loc()],
            });
        }
        Ok(())
    }
    fn parens(&mut self, expr: Expr) -> Result<(), Error> {
        let ExprType::Parens(expr2) = expr.val else {
            unreachable!()
        };
        self.visit_expr(*expr2)?;
        Ok(())
    }
    fn call(&mut self, expr: Expr) -> Result<(), Error> {
        let ExprType::Call(callee, args) = expr.val else {
            unreachable!()
        };
        self.visit_expr(*callee)?;
        for arg in args {
            self.visit_expr(arg)?;
        }
        Ok(())
    }
    fn unary(&mut self, expr: Expr) -> Result<(), Error> {
        let ExprType::UnaryOperation(_, expr2) = expr.val else {
            unreachable!()
        };
        self.visit_expr(*expr2)?;
        Ok(())
    }
    fn binary(&mut self, expr: Expr) -> Result<(), Error> {
        let ExprType::BinaryOperation(left, _, right) = expr.val else {
            unreachable!()
        };
        self.visit_expr(*left)?;
        self.visit_expr(*right)?;
        Ok(())
    }
}
