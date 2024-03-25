use crate::{environment::Environment, error::Error, exprstmt::*, token::*, value::*, located::Location};

use std::collections::{HashMap, HashSet};

pub fn varcheck(builtins: HashMap<String, ValueType>, stmt: &Vec<Stmt>) -> Result<(), Vec<Error>> {
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

                    if self.env.contains(name) {
                        // TODO: functions behave weirdly
                        // TODO: also add the first declaration
                        self.errs.push(Error {
                            msg: "Already declared variable".to_string(),
                            lines: vec![s.loc],
                        });
                        continue;
                    }
                    // give dummy values
                    // it is always going to succeed (as I already check for the existence)
                    self.env.insert(
                        &Token { val: TokenType::Identifier(name.to_string()), loc: Location { start: 0, end: 0 } },
                        Value { val: ValueType::Unit, loc: Location { start: 0, end: 0 } }
                    ).unwrap();
                },
                StmtType::FunDeclStmt(t, _, _) => {
                    let Token { val: TokenType::Identifier(name), .. } = t else {
                        unreachable!();
                    };

                    if self.env.contains(name) {
                        self.errs.push(Error {
                            msg: "Already declared variable".to_string(),
                            lines: vec![s.loc],
                        });
                    }
                    // give dummy values
                    // it is always going to succeed (as I already check for the existence)
                    self.env.insert(
                        &Token { val: TokenType::Identifier(name.to_string()), loc: Location { start: 0, end: 0 } },
                        Value { val: ValueType::Function(vec![], vec![]), loc: Location { start: 0, end: 0 } }
                    ).unwrap();

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
        let mut params2 = HashSet::new();
        for p in params {
            let TokenType::Identifier(name) = &p.val else {
                unreachable!()
            };
            if params2.contains(name) {
                self.errs.push(Error {
                    msg: format!("Found duplicate parameter: \"{}\"", p),
                    lines: vec![p.loc],
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
    fn retur(&mut self, _: Location, expr: &Expr) {
        self.visit_expr(expr);
    }
    fn brek(&mut self, _: Location) {
    }
    fn cont(&mut self, _: Location) {
    }
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
        };
    }
    // nothing to check
    fn unit(&mut self, _: Location) {
    }
    fn int(&mut self, _: Location, _: &i32) {
    }
    fn float(&mut self, _: Location, _: &f32) {
    }
    fn string(&mut self, _: Location, _: &String) {
    }
    fn bool(&mut self, _: Location, _: &bool) {
    }
    fn identifier(&mut self, loc: Location, ident: &String) {
        if !self.env.contains(ident) {
            self.errs.push(Error {
                msg: "Undeclared variable".to_string(),
                lines: vec![loc],
            });
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
}
