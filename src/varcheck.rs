use crate::{environment::Environment, error::Error, exprstmt::*, token::*, value::*};

use std::collections::{HashMap, HashSet};

pub fn varcheck(builtins: HashMap<String, ValueType>, stmt: Vec<Stmt>) -> Result<(), Vec<Error>> {
    let mut var_check = VarCheck {
        env: Environment::new(builtins),
        errs: vec![],
    };
    var_check.check_block(stmt);
    if !var_check.errs.is_empty() {
        Err(var_check.errs)
    } else {
        Ok(())
    }
}

struct VarCheck {
    env: Environment,
    errs: Vec<Error>,
}

impl VarCheck {
    fn check_block(&mut self, block: Vec<Stmt>) {
        self.env.add_scope();
        for s in block {
            match &s.val {
                StmtType::VarDeclStmt(t, expr) => {
                    // TODO: I hate this, but destructuring in the match is horrible
                    let Token { val: TokenType::Identifier(name), .. } = t else {
                        unreachable!();
                    };
                    self.visit_expr(expr.clone());

                    if self.env.contains(name) {
                        // TODO: functions behave weirdly
                        // TODO: also add the first declaration
                        self.errs.push(Error {
                            msg: "Already declared variable".to_string(),
                            lines: vec![s.loc()],
                        });
                        continue;
                    }
                    // give dummy values
                    // it is always going to succeed (as I already check for the existence)
                    self.env.insert(
                        &Token { val: TokenType::Identifier(name.to_string()), start: 0, end: 0 },
                        Value { val: ValueType::Unit, start: 0, end: 0 }
                    ).unwrap();
                },
                StmtType::FunDeclStmt(t, _, _) => {
                    let Token { val: TokenType::Identifier(name), .. } = t else {
                        unreachable!();
                    };

                    if self.env.contains(name) {
                        self.errs.push(Error {
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

                    self.visit_stmt(s);
                }
                StmtType::AssignStmt(t, expr) => {
                    let Token { val: TokenType::Identifier(name), .. } = t else {
                        unreachable!();
                    };

                    self.visit_expr(expr.clone());
                    if !self.env.contains(name) {
                        self.errs.push(Error {
                            msg: "Undeclared variable".to_string(),
                            lines: vec![s.loc()],
                        });
                    }
                }
                StmtType::BlockStmt(..) => {
                    self.visit_stmt(s);
                }
                StmtType::IfStmt(..) => {
                    self.visit_stmt(s);
                }
                StmtType::WhileStmt(..) => {
                    self.visit_stmt(s);
                }
                StmtType::ExprStmt(..) => {
                    self.visit_stmt(s);
                }
                StmtType::BreakStmt => {}
                StmtType::ContinueStmt => {}
                StmtType::ReturnStmt(..) => {
                    self.visit_stmt(s);
                }
            }
        }
        self.env.remove_scope();
    }
}

impl VarCheck {
    fn visit_stmt(&mut self, stmt: Stmt) {
        match stmt.val {
            // for these there is nothing to check (yet)
            StmtType::VarDeclStmt(_, expr) => {
                self.visit_expr(expr);
            },
            StmtType::AssignStmt(_, expr) => {
                self.visit_expr(expr);
            },
            StmtType::ExprStmt(expr) => {
                self.visit_expr(expr);
            },
            StmtType::BlockStmt(block) => {
                // go through
                self.check_block(block);
            },
            StmtType::IfStmt(blocks) => {
                for (cond, block) in blocks {
                    self.visit_expr(cond);
                    self.check_block(block);
                }
            },
            StmtType::WhileStmt(cond, block) => {
                self.visit_expr(cond);
                self.check_block(block);
            },
            StmtType::FunDeclStmt(_, params, block) => {
                let mut params2 = HashSet::new();
                for p in params {
                    let TokenType::Identifier(name) = &p.val else {
                        unreachable!()
                    };
                    if params2.contains(name) {
                        self.errs.push(Error {
                            msg: format!("Found duplicate parameter: \"{}\"", p),
                            lines: vec![p.loc()],
                        });
                    }
                    params2.insert(name.clone());
                }
                self.env.add_scope_vars(
                    params2.iter().map(|p| { (p.clone(), ValueType::Unit) }).collect::<HashMap<_, _>>()
                    );
                self.check_block(block);
                self.env.remove_scope();
            },
            StmtType::ContinueStmt => {}
            StmtType::BreakStmt => {}
            StmtType::ReturnStmt(expr) => {
                self.visit_expr(expr);
            },
        }
    }
}

impl VarCheck {
    fn visit_expr(&mut self, expr: Expr) {
        match expr.val {
            ExprType::Unit => {},
            ExprType::Int(..) => {},
            ExprType::Float(..) => {},
            ExprType::String(..) => {},
            ExprType::Bool(..) => {},
            ExprType::Identifier(ref name) => {
                if !self.env.contains(name) {
                    self.errs.push(Error {
                        msg: "Undeclared variable".to_string(),
                        lines: vec![expr.loc()],
                    });
                }
            },
            ExprType::Parens(expr2) => self.visit_expr(*expr2),
            ExprType::Call(callee, args) => {
                self.visit_expr(*callee);
                for arg in args {
                    self.visit_expr(arg);
                }
            },
            ExprType::UnaryOperation(_, expr2) => self.visit_expr(*expr2),
            ExprType::BinaryOperation(left, _, right) => {
                self.visit_expr(*left);
                self.visit_expr(*right);
            },
        }
    }
}
