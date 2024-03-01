use crate::{
    environment::Environment,
    error::Error,
    exprstmt::*,
    token::*,
    value::*,
};

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
                StmtType::VarDeclStmt(
                    Token {
                        val: TokenType::Identifier(name),
                        ..
                    },
                    expr,
                ) => {
                    self.visit_expr(expr.clone());

                    if self.env.contains(name) {
                        // TODO: functions behave weirdly
                        // TODO: also add the first declaration
                        self.errs.push(Error {
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
                },
                StmtType::AssignStmt(
                    Token {
                        val: TokenType::Identifier(name),
                        ..
                    },
                    expr,
                ) => {
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
                StmtType::BreakStmt => {},
                StmtType::ContinueStmt => {},
                StmtType::ReturnStmt(..) => {
                    self.visit_stmt(s);
                }
                _ => {
                    // TODO: this is like the classic meme "idk what it did so I removed it"...
                    // it broke.
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
            StmtType::VarDeclStmt(..) => self.var_decl(stmt),
            StmtType::AssignStmt(..) => self.assignment(stmt),
            StmtType::ExprStmt(..) => self.expr(stmt),
            StmtType::BlockStmt(..) => self.block(stmt),
            StmtType::IfStmt(..) => self.if_else(stmt),
            StmtType::WhileStmt(..) => self.whiles(stmt),
            StmtType::FunDeclStmt(..) => self.fun(stmt),
            StmtType::ContinueStmt => {},
            StmtType::BreakStmt => {},
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
        let mut params2 = HashSet::new();
        for p in params.iter() {
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

    fn unit(&mut self, _: Expr) {
    }
    fn int(&mut self, _: Expr) {
    }
    fn float(&mut self, _: Expr) {
    }
    fn string(&mut self, _: Expr) {
    }
    fn bool(&mut self, _: Expr) {
    }
    fn identifier(&mut self, expr: Expr) {
        let ExprType::Identifier(name) = expr.val.clone() else {
            unreachable!()
        };
        if !self.env.contains(&name) {
            self.errs.push(Error {
                msg: "Undeclared variable".to_string(),
                lines: vec![expr.loc()],
            });
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
