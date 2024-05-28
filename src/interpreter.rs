use std::collections::HashMap;

use crate::{
    associativity::Precedence,
    environment::Environment,
    error::{Error, ErrorType},
    exprstmt::*,
    located::Location,
    mref::MList,
    value::*,
};

pub fn interpret(builtins: HashMap<String, ValueType>, stmts: Vec<Stmt>) -> Result<(), Error> {
    Interpreter::new(builtins).interpret(stmts)
}

#[derive(Debug)]
enum InterpError {
    Error(Error),
    Return(Value),
    Continue,
    Break,
}

// a miracle
impl From<Error> for InterpError {
    fn from(value: Error) -> Self {
        InterpError::Error(value)
    }
}

pub struct Interpreter {
    environment: Environment<ValueType>,
}

impl Interpreter {
    pub fn new(defaults: HashMap<String, ValueType>) -> Self {
        Interpreter {
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
                    let msg = match err {
                        InterpError::Error(error) => return Err(error),
                        InterpError::Return(_) => ErrorType::ReturnOutsideFunction,
                        InterpError::Break => ErrorType::BreakOutsideLoop,
                        InterpError::Continue => ErrorType::ContinueOutsideLoop,
                    };
                    return Err(Error {
                        msg,
                        lines: vec![s.loc], // TODO: add locations
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
        self.environment.add_scope()
    }

    fn remove_scope(&mut self) {
        self.environment.remove_scope();
    }

    fn interpret_block(&mut self, block: Vec<Stmt>) -> Result<(), InterpError> {
        self.add_scope();
        for s in block {
            match self.visit_stmt(s) {
                Ok(_) => {}
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
            StmtType::FunDeclStmt(name, params, block) => self.fun(loc, name, params, block),
            StmtType::OperatorDeclStmt(name, params, block, prec) => self.operator(loc, name, params, block, prec),
            StmtType::ReturnStmt(expr) => self.retur(loc, expr),
            StmtType::BreakStmt => self.brek(loc),
            StmtType::ContinueStmt => self.cont(loc),
        }
    }
    fn var_decl(&mut self, _: Location, ident: Identifier, expr: Expr) -> Result<(), InterpError> {
        let name = ident.val;
        let val = self.visit_expr(expr)?;
        if !self.environment.insert(&name, val.val) {
            return Err(Error {
                msg: ErrorType::AlreadyDeclaredItem,
                lines: vec![ident.loc],
            }.into());
        }
        Ok(())
    }

    fn assignment(&mut self, _: Location, ident: Identifier, expr: Expr) -> Result<(), InterpError> {
        let name = ident.val;
        let val = self.visit_expr(expr)?;
        if !self.environment.update(&name, val.val) {
            return Err(Error {
                msg: ErrorType::UndeclaredItem,
                lines: vec![ident.loc],
            }.into());
        }
        Ok(())
    }

    fn assignindex(&mut self, _: Location, ls: Expr, idx: Expr, val: Expr) -> Result<(), InterpError> {
        let ValueType::List(mut ls2) = self.visit_expr(ls.clone())?.val else {
            return Err(Error {
                msg: ErrorType::ExpectedListIndex,
                lines: vec![ls.loc],
            }
            .into());
        };
        let ValueType::Int(n) = self.visit_expr(idx.clone())?.val else {
            return Err(Error {
                msg: ErrorType::ExpectedIndex,
                lines: vec![idx.loc],
            }.into())
        };
        let n2 = MList::check_index(n, ls2.len()).ok_or_else(|| Error {
            msg: ErrorType::IndexOutOfRange(n, ls2.len()),
            lines: vec![idx.loc],
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
            let val = self.visit_expr(cond.clone())?;
            let ValueType::Bool(b) = val.val else {
                return Err(Error {
                    msg: ErrorType::ExpectedBool,
                    lines: vec![val.loc]
                }.into())
            };
            if !b {
                break;
            }
            match self.interpret_block(block.clone()) {
                Ok(_) => {}
                Err(err) => match err {
                    InterpError::Error(_) => return Err(err),
                    InterpError::Return(_) => return Err(err),
                    InterpError::Continue => continue,
                    InterpError::Break => break,
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
    fn fun(
        &mut self,
        _: Location,
        name: Identifier,
        params: Vec<Identifier>,
        block: Vec<Stmt>,
    ) -> Result<(), InterpError> {
        let mut params2 = vec![];
        for p in params {
            params2.push(p.val);
        }
        if !self.environment.insert(
            &name.val,
            ValueType::Function(params2, block, self.environment.scopes.clone()),
            ) {
             return Err(Error {
                msg: ErrorType::AlreadyDeclaredItem,
                lines: vec![name.loc],
             }.into());
         }
        Ok(())
    }
    fn operator(
        &mut self,
        loc: Location,
        name: Symbol,
        params: (Identifier, Identifier),
        block: Vec<Stmt>,
        _: Precedence,
    ) -> Result<(), InterpError> {
        self.fun(loc, name, vec![params.0, params.1], block)
    }
    fn brek(&mut self, _: Location) -> Result<(), InterpError> {
        Err(InterpError::Break)
    }
    fn cont(&mut self, _: Location) -> Result<(), InterpError> {
        Err(InterpError::Continue)
    }
    fn retur(&mut self, _: Location, expr: Expr) -> Result<(), InterpError> {
        let val = self.visit_expr(expr)?;
        Err(InterpError::Return(val))
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
            ExprType::Parens(expr1) => self.parens(*expr1),
            ExprType::Call(callee, args) => self.call(*callee, args, loc),
            ExprType::UnaryOperation(op, expr1) => self.unary(op, *expr1),
            ExprType::BinaryOperation(left, op, right) => self.binary(*left, op, *right, loc),
            ExprType::List(ls) => self.list(loc, ls),
            ExprType::Index(expr2, idx) => self.index(loc, *expr2, *idx),
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
        self.environment.get(&ident).ok_or_else(|| Error {
            msg: ErrorType::UndeclaredItem,
            lines: vec![loc],
        })
    }
    fn string(&mut self, s: String) -> Result<ValueType, Error> {
        Ok(ValueType::String(s))
    }
    fn bool(&mut self, b: bool) -> Result<ValueType, Error> {
        Ok(ValueType::Bool(b))
    }
    fn parens(&mut self, expr: Expr) -> Result<ValueType, Error> {
        Ok(self.visit_expr(expr)?.val)
    }
    fn call(&mut self, callee: Expr, args: Vec<Expr>, loc: Location) -> Result<ValueType, Error> {
        let mut args2 = vec![];
        for arg in args {
            args2.push(self.visit_expr(arg)?.val);
        }

        let callee = self.visit_expr(callee)?;
        match callee.val {
            // TODO: the ok and ? can be removed
            ValueType::NativeFunction(func) => self.call_fn_native(func, args2, loc),
            ValueType::Function(params, body, closure) => self.call_fn(params, body, closure, args2, loc),
            _ => Err(Error {
                msg: ErrorType::ItemNotCalleable,
                lines: vec![callee.loc],
            }),
        }
    }
    fn unary(&mut self, op: Symbol, expr: Expr) -> Result<ValueType, Error> {
        let val = self.visit_expr(expr)?;
        let op_name = op.val;
        let new_val = match op_name.as_str() {
            "-" => match val.val {
                ValueType::Int(n) => ValueType::Int(-n),
                ValueType::Float(n) => ValueType::Float(-n),
                _ => {
                    return Err(Error {
                        msg: ErrorType::ExpectedUnaryNumber,
                        lines: vec![val.loc],
                    })
                }
            },
            "!" => match val.val {
                ValueType::Bool(b) => ValueType::Bool(!b),
                _ => {
                    return Err(Error {
                        msg: ErrorType::ExpectedUnaryBool,
                        lines: vec![val.loc],
                    })
                }
            },
            sym => {
                unreachable!("unknown binary operator interpreted: {sym}");
            }
        };

        Ok(new_val)
    }
    fn binary(&mut self, left: Expr, op: Symbol, right: Expr, loc: Location) -> Result<ValueType, Error> {
        let right_loc = right.loc;
        let left2 = self.visit_expr(left)?;
        let right2 = self.visit_expr(right)?;
        let op_name = &op.val;
        let val = self.environment.get(op_name).ok_or(Error {
            msg: ErrorType::UndeclaredItem,
            lines: vec![op.loc],
        })?;
        match val {
            ValueType::NativeFunction(func) => self.call_fn_native(func, vec![left2.val, right2.val], right_loc),
            ValueType::Function(params, body, closure) => {
                self.call_fn(params, body, closure, vec![left2.val, right2.val], loc)
            }
            _ => Err(Error {
                msg: ErrorType::ItemNotCalleable,
                lines: vec![op.loc],
            }),
        }
    }
    fn list(&mut self, _: Location, ls: Vec<Expr>) -> Result<ValueType, Error> {
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
            Err(err) => match err {
                // TODO: ERRORRRRRR
                InterpError::Error(err) => return Err(err),
                InterpError::Return(val) => val.val,
                InterpError::Break => {
                    return Err(Error {
                        msg: ErrorType::BreakOutsideLoop,
                        lines: vec![loc], // TODO: add locations
                    });
                }
                InterpError::Continue => {
                    return Err(Error {
                        msg: ErrorType::ContinueOutsideLoop,
                        lines: vec![loc], // TODO: add locations
                    });
                }
            },
        };
        self.remove_scope();
        self.environment = env;
        Ok(val)
    }

    fn call_fn_native(
        &mut self,
        func: NativeFunction,
        args: Vec<ValueType>,
        loc: Location,
    ) -> Result<ValueType, Error> {
        func(args).map_err(|msg| Error { msg: ErrorType::NativeFunctionError(msg), lines: vec![loc] })
    }
}
