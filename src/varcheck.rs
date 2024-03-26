use crate::{environment::Environment, error::Error, exprstmt::*, token::*};

use std::collections::HashMap;

pub fn varcheck(builtins: HashMap<String, ((usize, usize), bool)>, stmt: Vec<Stmt>) -> Result<(), (Vec<Error>, Vec<Error>)> {
    let mut var_check = VarCheck {
        env: Environment::new(builtins),
        errs: vec![],
        warns: vec![],
    };
    var_check.check_block(stmt);
    if !var_check.errs.is_empty() || !var_check.warns.is_empty() {
        Err((var_check.warns, var_check.errs))
    } else {
        Ok(())
    }
}

struct VarCheck {
    env: Environment<((usize, usize), bool)>,
    errs: Vec<Error>,
    warns: Vec<Error>,
}

impl VarCheck {
    fn declare_item(&mut self, name: &String, loc: (usize, usize)) {
        match self.env.get(name) {
            Some(val) => {
                self.errs.push(Error {
                    msg: "Already declared variable".to_string(),
                    lines: vec![val.0, loc],
                });
            }
            None => {
                // we dont care about the error, we know it
                // give dummy values
                // it is always going to succeed (as I already check for the existence)
                self.env.insert(name, (loc, false)).unwrap();
            }
        };
    }
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

                    self.declare_item(name, t.loc());
                },
                StmtType::FunDeclStmt(t, _, _) => {
                    let Token { val: TokenType::Identifier(name), .. } = t else {
                        unreachable!();
                    };

                    self.declare_item(name, t.loc());

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
        // TODO: no error positions existence
        // idea - take the positions when declared as an option and none them when found
        for (name, used) in self.env.scopes.last().unwrap() {
            if !used.1 {
                self.warns.push(Error {
                    msg: format!("Variable \"{}\" not used.", name),
                    lines: vec![used.0],
                })
            }
        }
        self.env.remove_scope();
    }
}

impl VarCheck {
    fn visit_stmt(&mut self, stmt: Stmt) {
        match stmt.val {
            StmtType::VarDeclStmt(..) => self.var_decl(stmt),
            StmtType::AssignStmt(..) => self.assignment(stmt),
            StmtType::ExprStmt(..) => self.expr(stmt),
            StmtType::BlockStmt(..) => self.block(stmt),
            StmtType::IfStmt(..) => self.if_else(stmt),
            StmtType::WhileStmt(..) => self.whiles(stmt),
            StmtType::FunDeclStmt(..) => self.fun(stmt),
            StmtType::ContinueStmt => {}
            StmtType::BreakStmt => {}
            StmtType::ReturnStmt(..) => self.retur(stmt),
        }
    }

    // for these there is nothing to check (yet)
    fn var_decl(&mut self, stmt: Stmt) {
        let StmtType::VarDeclStmt(_, expr) = stmt.val.clone() else {
            unreachable!()
        };
        self.visit_expr(expr);
    }
    fn assignment(&mut self, stmt: Stmt) {
        let StmtType::AssignStmt(_, expr) = stmt.val.clone() else {
            unreachable!()
        };
        self.visit_expr(expr);
    }
    fn expr(&mut self, stmt: Stmt) {
        let StmtType::ExprStmt(expr) = stmt.val.clone() else {
            unreachable!()
        };
        self.visit_expr(expr);
    }
    // go through
    fn block(&mut self, stmt: Stmt) {
        let StmtType::BlockStmt(block) = stmt.val.clone() else {
            unreachable!()
        };
        self.check_block(block);
    }
    fn if_else(&mut self, stmt: Stmt) {
        let StmtType::IfStmt(blocks) = stmt.val.clone() else {
            unreachable!()
        };
        for (cond, block) in blocks {
            self.visit_expr(cond);
            self.check_block(block);
        }
    }
    fn whiles(&mut self, stmt: Stmt) {
        let StmtType::WhileStmt(cond, block) = stmt.val.clone() else {
            unreachable!()
        };
        self.visit_expr(cond);
        self.check_block(block);
    }
    fn fun(&mut self, stmt: Stmt) {
        let StmtType::FunDeclStmt(_, params, block) = stmt.val.clone() else {
            unreachable!()
        };
        let mut params2: HashMap<String, ((usize, usize), bool)> = HashMap::new();
        for p in params.iter() {
            let TokenType::Identifier(name) = &p.val else {
                unreachable!()
            };
            match params2.get(name) {
                Some(original) => {
                    self.errs.push(Error {
                        msg: format!("Duplicate parameter: \"{}\"", p),
                        lines: vec![original.0, p.loc()],
                    });
                },
                None => {
                    params2.insert(name.clone(), (p.loc(), false));
                }
            }
        }
        self.env.add_scope_vars(params2);
        self.check_block(block);
        self.env.remove_scope();
    }
    fn retur(&mut self, stmt: Stmt) {
        let StmtType::ReturnStmt(expr) = stmt.val.clone() else {
            unreachable!()
        };
        self.visit_expr(expr);
    }
}

impl VarCheck {
    fn visit_expr(&mut self, expr: Expr) {
        match &expr.val {
            ExprType::Unit => self.unit(expr),
            ExprType::Int(..) => self.int(expr),
            ExprType::Float(..) => self.float(expr),
            ExprType::String(..) => self.string(expr),
            ExprType::Bool(..) => self.bool(expr),
            ExprType::Identifier(..) => self.identifier(expr),
            ExprType::Parens(..) => self.parens(expr),
            ExprType::Call(..) => self.call(expr),
            ExprType::UnaryOperation(..) => self.unary(expr),
            ExprType::BinaryOperation(..) => self.binary(expr),
        }
    }

    fn unit(&mut self, _: Expr) {}
    fn int(&mut self, _: Expr) {}
    fn float(&mut self, _: Expr) {}
    fn string(&mut self, _: Expr) {}
    fn bool(&mut self, _: Expr) {}
    fn identifier(&mut self, expr: Expr) {
        let ExprType::Identifier(name) = expr.val.clone() else {
            unreachable!()
        };
        match self.env.get(&name) {
            Some(var) => {
                self.env.update(
                    &name,
                    (var.0, true)
                    ).unwrap();
            },
            None => {
                self.errs.push(Error {
                    msg: "Undeclared variable".to_string(),
                    lines: vec![expr.loc()],
                });
            }
        }
    }
    fn parens(&mut self, expr: Expr) {
        let ExprType::Parens(expr2) = expr.val else {
            unreachable!()
        };
        self.visit_expr(*expr2);
    }
    fn call(&mut self, expr: Expr) {
        let ExprType::Call(callee, args) = expr.val else {
            unreachable!()
        };
        self.visit_expr(*callee);
        for arg in args {
            self.visit_expr(arg);
        }
    }
    fn unary(&mut self, expr: Expr) {
        let ExprType::UnaryOperation(_, expr2) = expr.val else {
            unreachable!()
        };
        self.visit_expr(*expr2);
    }
    fn binary(&mut self, expr: Expr) {
        let ExprType::BinaryOperation(left, _, right) = expr.val else {
            unreachable!()
        };
        self.visit_expr(*left);
        self.visit_expr(*right);
    }
}
