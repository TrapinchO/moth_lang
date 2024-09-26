#![allow(clippy::ptr_arg)]
use crate::{
    associativity::Precedence,
    environment::Environment,
    error::{Error, ErrorType},
    exprstmt::*,
    located::Location,
};

use std::collections::HashMap;

pub fn varcheck(builtins: HashMap<String, (Location, bool)>, stmt: &Vec<LStmt>) -> Result<(), (Vec<Error>, Vec<Error>)> {
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
    fn check_block(&mut self, block: &Vec<LStmt>) {
        self.env.add_scope();
        for (i, s) in block.iter().enumerate() {
            match &s.val {
                Stmt::VarDecl(t, expr) => {
                    self.visit_expr(expr);

                    self.declare_item(&t.val, t.loc);
                }
                Stmt::FunDecl(t, _, _) | Stmt::OperatorDecl(t, _, _, _) => {
                    self.declare_item(&t.val, t.loc);

                    self.visit_stmt(s);
                }
                Stmt::Assign(t, expr) => {
                    self.visit_expr(expr);
                    if !self.env.contains(&t.val) {
                        self.errs.push(Error {
                            msg: ErrorType::UndeclaredItem,
                            lines: vec![s.loc],
                        });
                    }
                }
                Stmt::Struct(name, _) => {
                    self.declare_item(&name.val, name.loc);
                    self.visit_stmt(s);
                }
                Stmt::Break | Stmt::Continue | Stmt::Return(_) => {
                    if i == block.len() - 1 {
                        break;
                    }
                    let start = block[i + 1].loc.start;
                    let end = block.last().unwrap().loc.end;
                    self.warns.push(Error {
                        msg: ErrorType::DeadCode,
                        lines: vec![s.loc, Location { start, end }],
                    });
                    break;
                }
                // necessary for pattern matching
                Stmt::AssignIndex(..)
                | Stmt::Block(..)
                | Stmt::If(..)
                | Stmt::While(..)
                | Stmt::Expr(..)
                | Stmt::Impl(..)
                | Stmt::AssignStruct(..) => {
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
    fn visit_stmt(&mut self, stmt: &LStmt) {
        let loc = stmt.loc;
        match &stmt.val {
            Stmt::Expr(expr) => self.expr(loc, expr),
            Stmt::VarDecl(ident, expr) => self.var_decl(loc, ident, expr),
            Stmt::Assign(ident, expr) => self.assignment(loc, ident, expr),
            Stmt::AssignIndex(ls, idx, val) => self.assignindex(loc, ls, idx, val),
            Stmt::Block(block) => self.block(loc, block),
            Stmt::If(blocks, els) => self.if_else(loc, blocks, els),
            Stmt::While(cond, block) => self.whiles(loc, cond, block),
            Stmt::FunDecl(name, params, block) => self.fun(loc, name, params, block),
            Stmt::OperatorDecl(name, params, block, prec) => self.operator(loc, name, params, block, prec),
            Stmt::Return(expr) => self.retur(loc, expr),
            Stmt::Break => self.brek(loc),
            Stmt::Continue => self.cont(loc),
            Stmt::Struct(name, fields) => self.struc(loc, name, fields),
            Stmt::AssignStruct(expr1, name, expr2) => self.assignstruc(loc, expr1, name, expr2),
            Stmt::Impl(name, block) => self.imp(loc, name, block),
        }
    }
    fn expr(&mut self, _: Location, expr: &LExpr) {
        self.visit_expr(expr);
    }
    fn var_decl(&mut self, _: Location, _: &Identifier, expr: &LExpr) {
        self.visit_expr(expr);
    }
    fn assignment(&mut self, _: Location, _: &Identifier, expr: &LExpr) {
        self.visit_expr(expr);
    }
    fn assignindex(&mut self, _: Location, ls: &LExpr, idx: &LExpr, val: &LExpr) {
        self.visit_expr(ls);
        self.visit_expr(idx);
        self.visit_expr(val);
    }
    fn block(&mut self, _: Location, block: &Vec<LStmt>) {
        self.check_block(block);
    }
    fn if_else(&mut self, _: Location, blocks: &Vec<(LExpr, Vec<LStmt>)>, els: &Option<Block>) {
        for (cond, block) in blocks.iter() {
            // check for dead code
            if cond.val == Expr::Bool(true) {
                self.warns.push(Error {
                    msg: ErrorType::IfAlwaysExecutes,
                    lines: vec![cond.loc]
                });
            } else if cond.val  == Expr::Bool(false) {
                self.warns.push(Error {
                    msg: ErrorType::IfNeverExecutes,
                    lines: vec![cond.loc],
                });
            }
            self.visit_expr(cond);
            self.check_block(block);
        }
        if let Some(bl) = els {
            self.check_block(bl);
        }
    }
    fn whiles(&mut self, _: Location, cond: &LExpr, block: &Vec<LStmt>) {
        if cond.val == Expr::Bool(false) {
            self.warns.push(Error {
                msg: ErrorType::LoopNeverExecutes,
                lines: vec![cond.loc],
            });
        }
        // TODO: do not forget about "true"
        // but since I have no advanced checks yet (or loop loop), no need ig
        self.visit_expr(cond);
        self.check_block(block);
    }
    fn fun(&mut self, _: Location, _: &Identifier, params: &Vec<Identifier>, block: &Vec<LStmt>) {
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
        block: &Vec<LStmt>,
        _: &Precedence,
    ) {
        self.fun(location, name, &vec![params.0.clone(), params.1.clone()], block);
    }
    fn retur(&mut self, _: Location, expr: &LExpr) {
        self.visit_expr(expr);
    }
    fn brek(&mut self, _: Location) {}
    fn cont(&mut self, _: Location) {}
    fn struc(&mut self, _: Location, _: &Identifier, fields: &Vec<Identifier>) {
        let mut m: HashMap<String, Location> = HashMap::new();
        for f in fields {
            if let Some(field) = m.get(&f.val) {
                self.errs.push(Error {
                    msg: ErrorType::DuplicateField(f.val.clone()),
                    lines: vec![*field, f.loc],
                });
            } else {
                m.insert(f.val.clone(), f.loc);
            }
        }
    }
    fn assignstruc(&mut self, _: Location, expr1: &LExpr, _: &Identifier, expr2: &LExpr) {
        self.visit_expr(expr1);
        self.visit_expr(expr2);
    }
    fn imp(&mut self, _: Location, name: &Identifier, block: &Vec<LStmt>) {
        if !self.env.contains(&name.val) {
            self.errs.push(Error {
                msg: ErrorType::ImplWithoutStruct(name.val.clone()),
                lines: vec![name.loc],
            });
        }
        self.check_block(block);
    }
}
impl VarCheck {
    fn visit_expr(&mut self, expr: &LExpr) {
        let loc = expr.loc;
        match &expr.val {
            Expr::Unit => self.unit(loc),
            Expr::Int(n) => self.int(loc, n),
            Expr::Float(n) => self.float(loc, n),
            Expr::String(s) => self.string(loc, s),
            Expr::Bool(b) => self.bool(loc, b),
            Expr::Identifier(ident) => self.identifier(loc, ident),
            Expr::Parens(expr1) => self.parens(loc, expr1),
            Expr::Call(callee, args) => self.call(loc, callee, args),
            Expr::UnaryOperation(op, expr1) => self.unary(loc, op, expr1),
            Expr::BinaryOperation(left, op, right) => self.binary(loc, left, op, right),
            Expr::List(ls) => self.list(loc, ls),
            Expr::Index(expr2, idx) => self.index(loc, expr2, idx),
            Expr::Lambda(params, body) => self.lambda(loc, params, body),
            Expr::FieldAccess(expr, name) => self.field(loc, expr, name),
            Expr::MethodAccess(expr, name, args) => self.method(loc, expr, name, args),
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
    fn parens(&mut self, _: Location, expr: &LExpr) {
        self.visit_expr(expr);
    }
    fn call(&mut self, _: Location, callee: &LExpr, args: &Vec<LExpr>) {
        self.visit_expr(callee);
        for arg in args {
            self.visit_expr(arg);
        }
    }
    fn unary(&mut self, _: Location, _: &Symbol, expr: &LExpr) {
        self.visit_expr(expr);
    }
    fn binary(&mut self, loc: Location, left: &LExpr, op: &Symbol, right: &LExpr) {
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
    fn list(&mut self, _: Location, ls: &Vec<LExpr>) {
        for e in ls {
            self.visit_expr(e);
        }
    }
    fn index(&mut self, _: Location, expr2: &LExpr, idx: &LExpr) {
        self.visit_expr(expr2);
        self.visit_expr(idx);
    }
    fn lambda(&mut self, loc: Location, params: &Vec<Identifier>, body: &Vec<LStmt>) {
        self.fun(
            loc,
            &Identifier {
                val: "".to_string(),
                loc,
            },
            params,
            body,
        );
    }
    fn field(&mut self, _: Location, expr: &LExpr, _: &Identifier) {
        self.visit_expr(expr);
        // TODO: check for fields
    }
    fn method(&mut self, _: Location, callee: &LExpr, _: &Identifier, args: &Vec<LExpr>) {
        self.visit_expr(callee);
        for arg in args {
            self.visit_expr(arg);
        }
    }
}
