use crate::{environment::Environment, error::Error, exprstmt::*, located::Location, token::*};

use std::collections::HashMap;

pub fn varcheck(builtins: HashMap<String, (Location, bool)>, stmt: &Vec<Stmt>) -> Result<(), (Vec<Error>, Vec<Error>)> {
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
    env: Environment<(Location, bool)>,
    errs: Vec<Error>,
    warns: Vec<Error>,
}

// TODO: because env.contains looks through ALL of the scopes,
// shadowing in a different scope is not possible
impl VarCheck {
    fn declare_item(&mut self, name: &String, loc: Location) {
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
    fn check_block(&mut self, block: &Vec<Stmt>) {
        self.env.add_scope();
        for s in block {
            match &s.val {
                StmtType::VarDeclStmt(t, expr) => {
                    // TODO: I hate this, but destructuring in the match is horrible
                    let Token { val: TokenType::Identifier(name), .. } = t else {
                        unreachable!();
                    };
                    self.visit_expr(expr);

                    self.declare_item(name, t.loc);
                }
                StmtType::FunDeclStmt(t, _, _) => {
                    let Token { val: TokenType::Identifier(name), .. } = t else {
                        unreachable!();
                    };

                    self.declare_item(name, t.loc);

                    self.visit_stmt(s);
                }
                StmtType::AssignStmt(t, expr) => {
                    let Token { val: TokenType::Identifier(name), .. } = t else {
                        unreachable!();
                    };

                    self.visit_expr(expr);
                    if !self.env.contains(name) {
                        self.errs.push(Error {
                            msg: "Undeclared variable".to_string(),
                            lines: vec![s.loc],
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
                    msg: format!("Variable \"{name}\" not used."),
                    lines: vec![used.0],
                })
            }
        }
        self.env.remove_scope();
    }
}

impl VarCheck {
    fn visit_stmt(&mut self, stmt: &Stmt) {
        let loc = stmt.loc;
        match &stmt.val {
            StmtType::ExprStmt(expr) => self.expr(loc, expr),
            StmtType::VarDeclStmt(ident, expr) => self.var_decl(loc, ident, expr),
            StmtType::AssignStmt(ident, expr) => self.assignment(loc, ident, expr),
            StmtType::BlockStmt(block) => self.block(loc, block),
            StmtType::IfStmt(blocks) => self.if_else(loc, blocks),
            StmtType::WhileStmt(cond, block) => self.whiles(loc, cond, block),
            StmtType::FunDeclStmt(name, params, block) => self.fun(loc, name, params, block),
            StmtType::ReturnStmt(expr) => self.retur(loc, expr),
            StmtType::BreakStmt => self.brek(loc),
            StmtType::ContinueStmt => self.cont(loc),
        }
    }
    fn expr(&mut self, _: Location, expr: &Expr) {
        self.visit_expr(expr);
    }
    fn var_decl(&mut self, _: Location, _: &Token, expr: &Expr) {
        self.visit_expr(expr);
    }
    fn assignment(&mut self, _: Location, _: &Token, expr: &Expr) {
        self.visit_expr(expr);
    }
    fn block(&mut self, _: Location, block: &Vec<Stmt>) {
        self.check_block(block);
    }
    fn if_else(&mut self, _: Location, blocks: &Vec<(Expr, Vec<Stmt>)>) {
        for (cond, block) in blocks {
            self.visit_expr(cond);
            self.check_block(block);
        }
    }
    fn whiles(&mut self, _: Location, cond: &Expr, block: &Vec<Stmt>) {
        self.visit_expr(cond);
        self.check_block(block);
    }
    fn fun(&mut self, _: Location, _: &Token, params: &Vec<Token>, block: &Vec<Stmt>) {
        let mut params2: HashMap<String, (Location, bool)> = HashMap::new();
        for p in params.iter() {
            let TokenType::Identifier(name) = &p.val else {
                unreachable!()
            };
            match params2.get(name) {
                Some(original) => {
                    self.errs.push(Error {
                        msg: format!("Duplicate parameter: \"{p}\""),
                        lines: vec![original.0, p.loc],
                    });
                }
                None => {
                    params2.insert(name.clone(), (p.loc, false));
                }
            }
        }
        self.env.add_scope_vars(params2);
        self.check_block(block);
        self.env.remove_scope();
    }
    fn retur(&mut self, _: Location, expr: &Expr) {
        self.visit_expr(expr);
    }
    fn brek(&mut self, _: Location) {}
    fn cont(&mut self, _: Location) {}
}
impl VarCheck {
    fn visit_expr(&mut self, expr: &Expr) {
        let loc = expr.loc;
        match &expr.val {
            ExprType::Unit => self.unit(loc),
            ExprType::Int(n) => self.int(loc, n),
            ExprType::Float(n) => self.float(loc, n),
            ExprType::String(s) => self.string(loc, s),
            ExprType::Bool(b) => self.bool(loc, b),
            ExprType::Identifier(ident) => self.identifier(loc, ident),
            ExprType::Parens(expr1) => self.parens(loc, expr1),
            ExprType::Call(callee, args) => self.call(loc, callee, args),
            ExprType::UnaryOperation(op, expr1) => self.unary(loc, op, expr1),
            ExprType::BinaryOperation(left, op, right) => self.binary(loc, left, op, right),
            ExprType::List(ls) => self.list(loc, ls),
            ExprType::Index(expr2, idx) => self.index(loc, expr2, idx),
        };
    }
    // nothing to check
    fn unit(&mut self, _: Location) {}
    fn int(&mut self, _: Location, _: &i32) {}
    fn float(&mut self, _: Location, _: &f32) {}
    fn string(&mut self, _: Location, _: &String) {}
    fn bool(&mut self, _: Location, _: &bool) {}
    fn identifier(&mut self, loc: Location, ident: &String) {
        match self.env.get(ident) {
            Some(var) => {
                self.env.update(ident, (var.0, true)).unwrap();
            }
            None => {
                self.errs.push(Error {
                    msg: "Undeclared variable".to_string(),
                    lines: vec![loc],
                });
            }
        }
    }
    fn parens(&mut self, _: Location, expr: &Expr) {
        self.visit_expr(expr);
    }
    fn call(&mut self, _: Location, callee: &Expr, args: &Vec<Expr>) {
        self.visit_expr(callee);
        for arg in args {
            self.visit_expr(arg);
        }
    }
    fn unary(&mut self, _: Location, _: &Token, expr: &Expr) {
        self.visit_expr(expr);
    }
    fn binary(&mut self, _: Location, left: &Expr, _: &Token, right: &Expr) {
        self.visit_expr(left);
        self.visit_expr(right);
    }
    fn list(&mut self, _: Location, ls: &Vec<Expr>) {
        for e in ls {
            self.visit_expr(e);
        }
    }
    fn index(&mut self, _: Location, expr2: &Expr, idx: &Expr) {
        self.visit_expr(expr2);
        self.visit_expr(idx);
    }
}
