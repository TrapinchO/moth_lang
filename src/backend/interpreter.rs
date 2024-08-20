use std::collections::HashMap;

use crate::{
    environment::Environment,
    error::{Error, ErrorType},
    located::{Located, Location},
    mref::{MList, MMap},
};
use super::value::*;
use super::lowexprstmt::*;

pub fn interpret(builtins: HashMap<String, ValueType>, stmts: Vec<Stmt>) -> Result<(), Error> {
    Interpreter::new(builtins).interpret(stmts)
}

#[derive(Debug)]
enum InterpErrorType {
    Error(Error),
    Return(Value),
    Continue,
    Break,
}

type InterpError = Located<InterpErrorType>;

// a miracle
impl From<Error> for InterpError {
    fn from(value: Error) -> Self {
        InterpError {
            loc: *value.lines.first().unwrap(),
            val: InterpErrorType::Error(value),
        }
    }
}

pub struct Interpreter {
    environment: Environment<ValueType>,
}

impl Interpreter {
    pub fn new(defaults: HashMap<String, ValueType>) -> Self {
        Self {
            environment: Environment::new(defaults),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), Error> {
        // not really needed, but might make a bit less mess when debugging
        self.add_scope();
        for s in stmts {
            match self.visit_stmt(s.clone()) {
                Ok(..) => {}
                Err(err) => {
                    let msg = match err.val {
                        InterpErrorType::Error(error) => return Err(error),
                        InterpErrorType::Return(_) => ErrorType::ReturnOutsideFunction,
                        InterpErrorType::Break => ErrorType::BreakOutsideLoop,
                        InterpErrorType::Continue => ErrorType::ContinueOutsideLoop,
                    };
                    return Err(Error {
                        msg,
                        lines: vec![err.loc],
                    });
                }
            };
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn get_val(&self, name: String) -> Option<ValueType> {
        self.environment.get(&name)
    }

    fn add_scope(&mut self) {
        self.environment.add_scope();
    }

    fn remove_scope(&mut self) {
        self.environment.remove_scope();
    }

    fn interpret_block(&mut self, block: Vec<Stmt>) -> Result<(), InterpError> {
        self.add_scope();
        for s in block {
            match self.visit_stmt(s) {
                Ok(()) => {}
                Err(err) => {
                    self.remove_scope();
                    return Err(err);
                }
            }
        }
        self.remove_scope();
        Ok(())
    }
}

impl Interpreter {
    fn visit_stmt(&mut self, stmt: Stmt) -> Result<(), InterpError> {
        let loc = stmt.loc;
        match stmt.val {
            StmtType::ExprStmt(expr) => self.expr(loc, expr),
            StmtType::VarDeclStmt(ident, expr) => self.var_decl(loc, ident, expr),
            StmtType::AssignStmt(ident, expr) => self.assignment(loc, ident, expr),
            StmtType::AssignIndexStmt(ls, idx, val) => self.assignindex(loc, ls, idx, val),
            StmtType::BlockStmt(block) => self.block(loc, block),
            StmtType::IfStmt(blocks) => self.if_else(loc, blocks),
            StmtType::WhileStmt(cond, block) => self.whiles(loc, cond, block),
            StmtType::ReturnStmt(expr) => self.retur(loc, expr),
            StmtType::BreakStmt => self.brek(loc),
            StmtType::ContinueStmt => self.cont(loc),
            StmtType::StructStmt(name, fields) => self.struc(loc, name, fields),
            StmtType::AssignStructStmt(expr1, name, expr2) => self.assignstruc(loc, expr1, name, expr2),
            StmtType::ImplStmt(name, block) => self.imp(loc, name, block)
        }
    }

    fn var_decl(&mut self, _: Location, ident: Identifier, expr: Expr) -> Result<(), InterpError> {
        let name = ident.val;
        let val = self.visit_expr(expr)?;
        if !self.environment.insert(&name, val.val) {
            unreachable!("Item \"{}\" already declared\nLocation: {:?}", name, ident.loc);
        }
        Ok(())
    }

    fn assignment(&mut self, _: Location, ident: Identifier, expr: Expr) -> Result<(), InterpError> {
        let name = ident.val;
        let val = self.visit_expr(expr)?;
        if !self.environment.update(&name, val.val) {
            unreachable!("Item \"{}\" not declared\nLocation: {:?}", name, ident.loc);
        }
        Ok(())
    }

    fn assignindex(&mut self, _: Location, ls: Expr, idx: Expr, val: Expr) -> Result<(), InterpError> {
        let ls_loc = ls.loc;
        let ValueType::List(mut ls2) = self.visit_expr(ls)?.val else {
            return Err(Error {
                msg: ErrorType::ExpectedListIndex,
                lines: vec![ls_loc],
            }
            .into());
        };
        let idx_loc = idx.loc;
        let ValueType::Int(n) = self.visit_expr(idx)?.val else {
            return Err(Error {
                msg: ErrorType::ExpectedIndex,
                lines: vec![idx_loc],
            }.into())
        };
        let n2 = MList::check_index(n, ls2.len()).ok_or_else(|| Error {
            msg: ErrorType::IndexOutOfRange(n, ls2.len()),
            lines: vec![idx_loc],
        })?;
        ls2.modify(n2, self.visit_expr(val)?);
        Ok(())
    }

    fn block(&mut self, _: Location, block: Vec<Stmt>) -> Result<(), InterpError> {
        self.interpret_block(block)?;
        Ok(())
    }

    fn if_else(&mut self, _: Location, blocks: Vec<(Expr, Vec<Stmt>)>) -> Result<(), InterpError> {
        for (cond, block) in blocks {
            let ValueType::Bool(cond2) = self.visit_expr(cond.clone())?.val else {
                return Err(Error {
                    msg: ErrorType::ExpectedBool,
                    lines: vec![cond.loc],
                }.into());
            };
            // do not continue
            if cond2 {
                self.interpret_block(block)?;
                break;
            }
        }

        Ok(())
    }

    fn whiles(&mut self, _: Location, cond: Expr, block: Vec<Stmt>) -> Result<(), InterpError> {
        loop {
            let cond = self.visit_expr(cond.clone())?;
            let ValueType::Bool(b) = cond.val else {
                return Err(Error {
                    msg: ErrorType::ExpectedBool,
                    lines: vec![cond.loc]
                }.into())
            };
            if !b {
                break;
            }
            match self.interpret_block(block.clone()) {
                Ok(()) => {}
                Err(err) => match err.val {
                    InterpErrorType::Error(_) => return Err(err),
                    InterpErrorType::Return(_) => return Err(err),
                    InterpErrorType::Continue => continue,
                    InterpErrorType::Break => break,
                },
            }
        }
        Ok(())
    }

    fn expr(&mut self, _: Location, expr: Expr) -> Result<(), InterpError> {
        // TODO: later check if it is not unit!
        let _ = self.visit_expr(expr)?;
        Ok(())
    }
    fn brek(&mut self, loc: Location) -> Result<(), InterpError> {
        Err(InterpError {
            val: InterpErrorType::Break,
            loc,
        })
    }
    fn cont(&mut self, loc: Location) -> Result<(), InterpError> {
        Err(InterpError {
            val: InterpErrorType::Continue,
            loc,
        })
    }
    fn retur(&mut self, loc: Location, expr: Expr) -> Result<(), InterpError> {
        let val = self.visit_expr(expr)?;
        Err(InterpError {
            val: InterpErrorType::Return(val),
            loc,
        })
    }
    fn struc(&mut self, _: Location, name: Identifier, fields: Vec<Identifier>) -> Result<(), InterpError> {
        if !self.environment.insert(&name.val, ValueType::Struct(name.clone(), fields, HashMap::new().into())) {
            unreachable!("Item \"{}\" already declared\nLocation: {:?}", name.val, name.loc);
        }
        Ok(())
    }
    fn assignstruc(&mut self, _: Location, expr1: Expr, name: Identifier, expr2: Expr) -> Result<(), InterpError> {
        let expr = self.visit_expr(expr1)?;
        let ValueType::Instance(_, mut fields) = expr.val else {
            return Err(Error {
                msg: ErrorType::ExpectedInstance,
                lines: vec![expr.loc],
            }.into());
        };
        if fields.get(&name.val).is_none() { // field not present in the object
            return Err(Error {
                msg: ErrorType::UnknownField(name.val),
                lines: vec![name.loc],
            }.into());
        }
        let val = self.visit_expr(expr2)?;
        fields.insert(name.val, val.val);
        Ok(())
    }
    fn imp(&mut self, _: Location, name: Identifier, block: Vec<Stmt>) -> Result<(), InterpError> {
        // its existence is checked in varcheck
        // and though it may be reassigned, the name still MUST exist
        // it does not have to be a struct anymore though
        let struc = self.environment.get(&name.val).expect("Struct somehow now defined");
        let ValueType::Struct(_, _, mut methods) = struc else {
            return Err(Error {
                msg: ErrorType::OtherError(format!("Value bound to \"{}\" is not a struct", name.val)),
                lines: vec![ name.loc ],
            }.into());
        };
        for s in block {
            let StmtType::VarDeclStmt(name, fun) = s.val else {
                unreachable!("Checked for in varcheck");
            };
            methods.insert(name.val, self.visit_expr(fun)?.val);
        }
        Ok(())
    }
}

impl Interpreter {
    fn visit_expr(&mut self, expr: Expr) -> Result<Value, Error> {
        let loc = expr.loc;
        let val = match expr.val {
            ExprType::Unit => self.unit(),
            ExprType::Int(n) => self.int(n),
            ExprType::Float(n) => self.float(n),
            ExprType::String(s) => self.string(s),
            ExprType::Bool(b) => self.bool(b),
            ExprType::Identifier(ident) => self.identifier(ident, loc),
            ExprType::Call(callee, args) => self.call(*callee, args, loc),
            ExprType::List(ls) => self.list(loc, ls),
            ExprType::Index(expr2, idx) => self.index(loc, *expr2, *idx),
            ExprType::Lambda(params, body) => self.lambda(loc, params, body),
            ExprType::FieldAccess(expr, name) => self.field(loc, *expr, name),
        }?;
        Ok(Value { val, loc: expr.loc })
    }
    fn unit(&mut self) -> Result<ValueType, Error> {
        Ok(ValueType::Unit)
    }
    fn int(&mut self, n: i32) -> Result<ValueType, Error> {
        Ok(ValueType::Int(n))
    }
    fn float(&mut self, n: f32) -> Result<ValueType, Error> {
        Ok(ValueType::Float(n))
    }
    fn identifier(&mut self, ident: String, loc: Location) -> Result<ValueType, Error> {
        self.environment.get(&ident)
            .ok_or_else(|| unreachable!("Item \"{}\" not declared\nLocation: {:?}", ident, loc))
    }
    fn string(&mut self, s: String) -> Result<ValueType, Error> {
        Ok(ValueType::String(s))
    }
    fn bool(&mut self, b: bool) -> Result<ValueType, Error> {
        Ok(ValueType::Bool(b))
    }
    fn call(&mut self, callee: Expr, args: Vec<Expr>, loc: Location) -> Result<ValueType, Error> {
        let mut args2 = vec![];
        for arg in args {
            args2.push(self.visit_expr(arg)?.val);
        }

        let callee = self.visit_expr(callee)?;
        match callee.val {
            ValueType::NativeFunction(func) => self.call_fn_native(func, args2, loc),
            ValueType::Function(params, body, closure) => self.call_fn(params, body, closure, args2, loc),
            ValueType::Struct(name, fields, methods) => self.call_struct(name, fields, args2, methods, loc),
            _ => Err(Error {
                msg: ErrorType::ItemNotCalleable,
                lines: vec![callee.loc],
            }),
        }
    }
    fn list(&mut self, _: Location, ls: Vec<Expr>) -> Result<ValueType, Error> {
        // a nicer version, but requires cloning...
        /*
        let ls2 = ls.into_iter()
            .map(|e| self.visit_expr(e))
            .collect::<Result<Vec<_>, _>>()?;
        */
        let mut ls2 = vec![];
        for e in ls {
            ls2.push(self.visit_expr(e)?);
        }
        Ok(ValueType::List(ls2.into()))
    }
    fn index(&mut self, loc: Location, expr2: Expr, idx: Expr) -> Result<ValueType, Error> {
        let val = self.visit_expr(expr2)?;
        let idx2 = self.visit_expr(idx)?;
        let ValueType::Int(n) = idx2.val else {
            return Err(Error {
                msg: ErrorType::ExpectedIndex,
                lines: vec![idx2.loc],
            });
        };
        match val.val {
            ValueType::List(ls) => {
                let n2 = MList::check_index(n, ls.len()).ok_or_else(|| Error {
                    msg: ErrorType::IndexOutOfRange(n, ls.len()),
                    lines: vec![loc],
                })?;
                Ok(ls.read(|l| l[n2].clone()).val)
            }
            ValueType::String(s) => {
                let n2 = MList::check_index(n, s.len()).ok_or_else(|| Error {
                    msg: ErrorType::IndexOutOfRange(n, s.len()),
                    lines: vec![loc],
                })?;
                Ok(ValueType::String(s.chars().nth(n2).unwrap().to_string()))
            }
            _ => Err(Error {
                msg: ErrorType::ItemNotIndexable,
                lines: vec![val.loc],
            }),
        }
    }
    fn lambda(&mut self, _: Location, params: Vec<Identifier>, body: Vec<Stmt>) -> Result<ValueType, Error> {
        let mut params2 = vec![];
        for p in params {
            params2.push(p.val);
        }
        Ok(ValueType::Function(params2, body, self.environment.scopes.clone()))
    }
    fn field(&mut self, loc: Location, expr: Expr, name: Identifier) -> Result<ValueType, Error> {
        let expr2 = self.visit_expr(expr)?;
        let ValueType::Instance(struct_name, fields) = expr2.val else {
            return Err(Error {
                msg: ErrorType::ExpectedInstance,
                lines: vec![expr2.loc],
            });
        };
        Ok(fields.get(&name.val).ok_or_else(|| Error {
            msg: ErrorType::FieldNotFound(name.val, struct_name),
            lines: vec![loc],
        })?.clone())
    }

    fn call_fn(
        &mut self,
        params: Vec<String>,
        body: Vec<Stmt>,
        closure: Closure,
        args: Vec<ValueType>,
        loc: Location,
    ) -> Result<ValueType, Error> {
        if args.len() != params.len() {
            return Err(Error {
                msg: ErrorType::IncorrectParameterCount(args.len(), params.len()),
                lines: vec![loc],
            });
        }
        // craftinginterpreters seem to do it
        let env = self.environment.clone();
        self.environment = Environment { scopes: closure };
        self.environment.add_scope_vars(
            params
                .iter()
                .zip(args)
                .map(|(n, v)| (n.clone(), v))
                .collect::<HashMap<_, _>>(),
        );

        let val = match self.interpret_block(body) {
            Ok(..) => ValueType::Unit, // hope this doesnt bite me later...
            Err(err) => match err.val {
                InterpErrorType::Error(err) => return Err(err),
                InterpErrorType::Return(val) => val.val,
                InterpErrorType::Break => {
                    return Err(Error {
                        msg: ErrorType::BreakOutsideLoop,
                        lines: vec![err.loc],
                    });
                }
                InterpErrorType::Continue => {
                    return Err(Error {
                        msg: ErrorType::ContinueOutsideLoop,
                        lines: vec![err.loc],
                    });
                }
            },
        };
        self.remove_scope();
        self.environment = env;
        Ok(val)
    }

    // btw the self is technically not needed
    // leaving it here for style for now
    fn call_fn_native(
        &self,
        func: NativeFunction,
        args: Vec<ValueType>,
        loc: Location,
    ) -> Result<ValueType, Error> {
        func(args).map_err(|msg| Error { msg: ErrorType::NativeFunctionError(msg), lines: vec![loc] })
    }

    // btw the self is technically not needed 2
    // leaving it here for style for now
    fn call_struct(
        &self,
        name: Identifier,
        fields: Vec<Identifier>,
        args: Vec<ValueType>,
        methods: MMap<ValueType>,
        loc: Location,
    ) -> Result<ValueType, Error> {
        if args.len() != fields.len() {
            return Err(Error {
                msg: ErrorType::IncorrectParameterCount(args.len(), fields.len()),
                lines: vec![loc],
            });
        }

        let mut m = HashMap::new();
        for (k, v) in methods.iter() {
            if let Some(f) = fields.iter().find(|f| f.val == k) {
                return Err(Error {
                    msg: ErrorType::DuplicateField(k),
                    lines: vec![f.loc],
                });
            }
            m.insert(k, v);
        }

        let m2 = args.iter()
            .zip(fields)
            .map(|(a, f)| (f.val, a.clone()))
            .collect::<HashMap<_, _>>();
        m.extend(m2);

        Ok(ValueType::Instance(name.val, MMap::new(m)))
    }
}
