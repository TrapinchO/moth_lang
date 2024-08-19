#![allow(clippy::ptr_arg)]
use crate::{associativity::Precedence, environment::Environment, error::{ErrorType, Error}, exprstmt::*, located::Location};

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

// TODO: consider cutting down everything unused, e.g. brek and cont methods
// as well as some unused parameters
// these are leftovers of Visitor (which is no longer used), maybe they will come handy some time?
impl VarCheck {
    fn declare_item(&mut self, name: &String, loc: Location) {
        // there should always be a scope
        match self.env.scopes.last().unwrap().get(name) {
            Some(val) => {
                self.errs.push(Error {
                    msg: ErrorType::AlreadyDeclaredItem,
                    lines: vec![val.0, loc],
                });
            }
            None => {
                self.env.insert(name, (loc, false));
            }
        };
    }
    fn check_block(&mut self, block: &Vec<Stmt>) {
        self.env.add_scope();
        for s in block {
            match &s.val {
                StmtType::VarDeclStmt(t, expr) => {
                    self.visit_expr(expr);

                    self.declare_item(&t.val, t.loc);
                }
                StmtType::FunDeclStmt(t, _, _) | StmtType::OperatorDeclStmt(t, _, _, _) => {
                    self.declare_item(&t.val, t.loc);

                    self.visit_stmt(s);
                }
                StmtType::AssignStmt(t, expr) => {
                    self.visit_expr(expr);
                    if !self.env.contains(&t.val) {
                        self.errs.push(Error {
                            msg: ErrorType::UndeclaredItem,
                            lines: vec![s.loc],
                        });
                    }
                }
                StmtType::BreakStmt | StmtType::ContinueStmt => {}
                StmtType::StructStmt(name, _) => {
                    self.declare_item(&name.val, name.loc);
                }
                // necessary for pattern matching
                StmtType::AssignIndexStmt(..)
                    | StmtType::BlockStmt(..)
                    | StmtType::IfStmt(..)
                    | StmtType::WhileStmt(..)
                    | StmtType::ReturnStmt(..)
                    | StmtType::ExprStmt(..)
                    | StmtType::AssignStructStmt(..) => {
                        self.visit_stmt(s);
                }
            }
        }

        for (name, used) in self.env.scopes.last().unwrap().iter() {
            if !used.1 {
                self.warns.push(Error {
                    msg: ErrorType::ItemNotUsed(name),
                    lines: vec![used.0],
                });
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
            StmtType::AssignIndexStmt(ls, idx, val) => self.assignindex(loc, ls, idx, val),
            StmtType::BlockStmt(block) => self.block(loc, block),
            StmtType::IfStmt(blocks) => self.if_else(loc, blocks),
            StmtType::WhileStmt(cond, block) => self.whiles(loc, cond, block),
            StmtType::FunDeclStmt(name, params, block) => self.fun(loc, name, params, block),
            StmtType::OperatorDeclStmt(name, params, block, prec) => self.operator(loc, name, params, block, prec),
            StmtType::ReturnStmt(expr) => self.retur(loc, expr),
            StmtType::BreakStmt => self.brek(loc),
            StmtType::ContinueStmt => self.cont(loc),
            StmtType::StructStmt(name, fields) => self.struc(loc, name, fields),
            StmtType::AssignStructStmt(expr1, name, expr2) => self.assignstruc(loc, expr1, name, expr2),
        }
    }
    fn expr(&mut self, _: Location, expr: &Expr) {
        self.visit_expr(expr);
    }
    fn var_decl(&mut self, _: Location, _: &Identifier, expr: &Expr) {
        self.visit_expr(expr);
    }
    fn assignment(&mut self, _: Location, _: &Identifier, expr: &Expr) {
        self.visit_expr(expr);
    }
    fn assignindex(&mut self, _: Location, ls: &Expr, idx: &Expr, val: &Expr) {
        self.visit_expr(ls);
        self.visit_expr(idx);
        self.visit_expr(val);
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
    fn fun(&mut self, _: Location, _: &Identifier, params: &Vec<Identifier>, block: &Vec<Stmt>) {
        let mut params2: HashMap<String, (Location, bool)> = HashMap::new();
        for p in params {
            let name = p.val.clone();
            match params2.get(&name) {
                Some(original) => {
                    self.errs.push(Error {
                        msg: ErrorType::DuplicateParameter(name),
                        lines: vec![original.0, p.loc],
                    });
                }
                None => {
                    params2.insert(name, (p.loc, false));
                }
            }
        }
        self.env.add_scope_vars(params2);
        self.check_block(block);
        for (name, used) in self.env.scopes.last().unwrap().iter() {
            if !used.1 {
                self.warns.push(Error {
                    msg: ErrorType::ItemNotUsed(name),
                    lines: vec![used.0],
                });
            }
        }
        self.env.remove_scope();
    }
    fn operator(
        &mut self,
        location: Location,
        name: &Symbol,
        params: &(Identifier, Identifier),
        block: &Vec<Stmt>,
        _: &Precedence,
    ) {
        self.fun(location, name, &vec![params.0.clone(), params.1.clone()], block);
    }
    fn retur(&mut self, _: Location, expr: &Expr) {
        self.visit_expr(expr);
    }
    fn brek(&mut self, _: Location) {}
    fn cont(&mut self, _: Location) {}
    fn struc(&mut self, _: Location, _: &Identifier, _: &Vec<Identifier>) {}
    fn assignstruc(&mut self, _: Location, expr1: &Expr, _: &Identifier, expr2: &Expr) {
        self.visit_expr(expr1);
        self.visit_expr(expr2);
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
            ExprType::List(ls) => self.list(loc, ls),
            ExprType::Index(expr2, idx) => self.index(loc, expr2, idx),
            ExprType::Lambda(params, body) => self.lambda(loc, params, body),
            ExprType::FieldAccess(expr, name) => self.field(loc, expr, name),
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
                self.env.update(ident, (var.0, true));
            }
            None => {
                self.errs.push(Error {
                    msg: ErrorType::UndeclaredItem,
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
    fn unary(&mut self, _: Location, _: &Symbol, expr: &Expr) {
        self.visit_expr(expr);
    }
    fn binary(&mut self, loc: Location, left: &Expr, op: &Symbol, right: &Expr) {
        self.visit_expr(left);
        let s = &op.val;
        match self.env.get(s) {
            Some(var) => {
                self.env.update(s, (var.0, true));
            }
            None => self.errs.push(Error {
                msg: ErrorType::UndeclaredItem,
                lines: vec![loc],
            }),
        }
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
    fn lambda(&mut self, loc: Location, params: &Vec<Identifier>, body: &Vec<Stmt>) {
        self.fun(loc, &Identifier { val: "".to_string(), loc }, params, body);
    }
    fn field(&mut self, _: Location, expr: &Expr, _: &Identifier) {
        self.visit_expr(expr);
        // TODO: check for fields
    }
}
