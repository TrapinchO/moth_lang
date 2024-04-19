use core::panic;
use std::collections::HashMap;

use crate::{
    environment::Environment,
    error::{Error, ErrorType},
    exprstmt::{Expr, ExprType, Stmt, StmtType},
    located::Location,
    token::*,
    value::*,
};

pub fn interpret(builtins: HashMap<String, ValueType>, stmts: Vec<Stmt>) -> Result<(), Error> {
    Interpreter::new(builtins).interpret(stmts)
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
                        ErrorType::Error(error) => return Err(error),
                        ErrorType::Return(_) => "Cannot use return outside of a function",
                        ErrorType::Break => "Cannot use break outside of a loop",
                        ErrorType::Continue => "Cannot use continue outside of a loop",
                    }
                    .to_string();
                    return Err(Error {
                        msg,
                        lines: vec![s.loc], // TODO: add locations
                    });
                }
            };
        }
        Ok(())
    }

    #[cfg(any(test))]
    pub fn get_val(&self, name: String) -> Option<ValueType> {
        self.environment.get(&name)
    }

    fn add_scope(&mut self) {
        self.environment.add_scope()
    }

    fn remove_scope(&mut self) {
        self.environment.remove_scope();
    }

    fn interpret_block(&mut self, block: Vec<Stmt>) -> Result<(), ErrorType> {
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
    fn visit_stmt(&mut self, stmt: Stmt) -> Result<(), ErrorType> {
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
            StmtType::ReturnStmt(expr) => self.retur(loc, expr),
            StmtType::BreakStmt => self.brek(loc),
            StmtType::ContinueStmt => self.cont(loc),
        }
    }
    fn var_decl(&mut self, _: Location, ident: Token, expr: Expr) -> Result<(), ErrorType> {
        let TokenType::Identifier(name) = &ident.val else {
            unreachable!()
        };
        let val = self.visit_expr(expr)?;
        self.environment.insert(name, val.val).ok_or(Error {
            msg: format!("Name \"{name}\" already exists"),
            lines: vec![ident.loc],
        })?;
        Ok(())
    }

    fn assignment(&mut self, _: Location, ident: Token, expr: Expr) -> Result<(), ErrorType> {
        let TokenType::Identifier(name) = &ident.val else {
            unreachable!()
        };
        let val = self.visit_expr(expr)?;
        self.environment.update(name, val.val).ok_or_else(|| Error {
            msg: format!("Name not found: \"{name}\""),
            lines: vec![ident.loc],
        })?;
        Ok(())
    }

    fn assignindex(&mut self, _: Location, ls: Expr, idx: Expr, val: Expr) -> Result<(), ErrorType> {
        let ValueType::List(mut ls2) = self.visit_expr(ls.clone())?.val else {
            return Err(Error {
                msg: "Expected a list index".to_string(),
                lines: vec![ls.loc],
            }.into())
        };
        let ValueType::Int(n) = self.visit_expr(idx.clone())?.val else {
            return Err(Error {
                msg: "Expected an index".to_string(),
                lines: vec![idx.loc],
            }.into())
        };
        let n2 = MList::check_index(n, ls2.read(|l| l.len())).ok_or_else(|| Error {
            msg: format!("Index out of range: {}", n),
            lines: vec![idx.loc]
        })?;
        ls2.modify(n2, self.visit_expr(val)?);
        Ok(())
    }

    fn block(&mut self, _: Location, block: Vec<Stmt>) -> Result<(), ErrorType> {
        self.interpret_block(block)?;
        Ok(())
    }

    fn if_else(&mut self, _: Location, blocks: Vec<(Expr, Vec<Stmt>)>) -> Result<(), ErrorType> {
        for (cond, block) in blocks {
            let ValueType::Bool(cond2) = self.visit_expr(cond.clone())?.val else {
                return Err(Error {
                    msg: format!("Expected bool, got {}", cond.val),
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

    fn whiles(&mut self, _: Location, cond: Expr, block: Vec<Stmt>) -> Result<(), ErrorType> {
        while let ValueType::Bool(true) = self.visit_expr(cond.clone())?.val {
            match self.interpret_block(block.clone()) {
                Ok(_) => {}
                Err(err) => match err {
                    ErrorType::Error(_) => return Err(err),
                    ErrorType::Return(_) => return Err(err),
                    ErrorType::Continue => continue,
                    ErrorType::Break => break,
                },
            }
        }
        Ok(())
    }

    fn expr(&mut self, _: Location, expr: Expr) -> Result<(), ErrorType> {
        // TODO: later check if it is not unit!
        let _ = self.visit_expr(expr)?;
        Ok(())
    }
    fn fun(&mut self, _: Location, name: Token, params: Vec<Token>, block: Vec<Stmt>) -> Result<(), ErrorType> {
        let TokenType::Identifier(name2) = &name.val else {
            unreachable!()
        };
        let mut params2 = vec![];
        for p in params {
            let TokenType::Identifier(n) = &p.val else {
                unreachable!()
            };
            params2.push(n.clone());
        }
        self.environment
            .insert(name2, ValueType::Function(params2, block))
            .ok_or(Error {
                msg: format!("Name \"{name2}\" already exists"),
                lines: vec![name.loc],
            })?;
        // TODO: nothing here yet
        Ok(())
    }
    fn brek(&mut self, _: Location) -> Result<(), ErrorType> {
        Err(ErrorType::Break)
    }
    fn cont(&mut self, _: Location) -> Result<(), ErrorType> {
        Err(ErrorType::Continue)
    }
    fn retur(&mut self, _: Location, expr: Expr) -> Result<(), ErrorType> {
        let val = self.visit_expr(expr)?;
        Err(ErrorType::Return(val))
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
            ExprType::BinaryOperation(left, op, right) => self.binary(*left, op, *right),
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
            msg: format!("Name not found: \"{ident}\""),
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
            args2.push(self.visit_expr(arg)?);
        }

        let callee = self.visit_expr(callee)?;
        match callee.val {
            // TODO: the ok and ? can be removed
            ValueType::NativeFunction(func) => Ok(func(args2).map_err(|msg| Error { msg, lines: vec![loc] })?),
            ValueType::Function(params, block) => {
                if args2.len() != params.len() {
                    return Err(Error {
                        msg: format!(
                            "the number of arguments ({}) must match the number of parameters ({})",
                            args2.len(),
                            params.len()
                        ),
                        lines: vec![loc],
                    });
                }
                self.environment.add_scope_vars(
                    params
                        .iter()
                        .zip(args2)
                        .map(|(n, v)| (n.clone(), v.val))
                        .collect::<HashMap<_, _>>(),
                );
                let val = match self.interpret_block(block) {
                    Ok(..) => ValueType::Unit, // hope this doesnt bite me later...
                    Err(err) => match err {
                        // TODO: ERRORRRRRR
                        ErrorType::Error(err) => return Err(err),
                        ErrorType::Return(val) => val.val,
                        ErrorType::Break => {
                            return Err(Error {
                                msg: "Cannot use break outside of loop".to_string(),
                                lines: vec![loc], // TODO: add locations
                            });
                        }
                        ErrorType::Continue => {
                            return Err(Error {
                                msg: "Cannot use break outside of loop".to_string(),
                                lines: vec![loc], // TODO: add locations
                            });
                        }
                    },
                };
                self.remove_scope();
                Ok(val)
            }
            _ => Err(Error {
                msg: format!("\"{}\" is not calleable", callee.val),
                lines: vec![callee.loc],
            }),
        }
    }
    fn unary(&mut self, op: Token, expr: Expr) -> Result<ValueType, Error> {
        let val = self.visit_expr(expr)?;
        let TokenType::Symbol(op_name) = &op.val else {
            panic!("Expected a symbol, found {}", op.val);
        };
        let new_val = match op_name.as_str() {
            "-" => match val.val {
                ValueType::Int(n) => ValueType::Int(-n),
                ValueType::Float(n) => ValueType::Float(-n),
                _ => {
                    return Err(Error {
                        msg: format!("Incorrect type: {}", val.val),
                        lines: vec![val.loc],
                    })
                }
            },
            "!" => match val.val {
                ValueType::Bool(b) => ValueType::Bool(!b),
                _ => {
                    return Err(Error {
                        msg: format!("Incorrect type: {}", val.val),
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
    fn binary(&mut self, left: Expr, op: Token, right: Expr) -> Result<ValueType, Error> {
        let right_loc = right.loc;
        let left2 = self.visit_expr(left)?;
        let right2 = self.visit_expr(right)?;
        let TokenType::Symbol(op_name) = &op.val else {
            panic!("Expected a symbol, found {}", op.val)
        };
        let ValueType::NativeFunction(func) = self.environment.get(op_name).ok_or(Error {
            msg: format!("Name not found: \"{op_name}\""),
            lines: vec![op.loc],
        })?
        else {
            return Err(Error {
                msg: format!("Symbol\"{op_name}\" is not a function"),
                lines: vec![op.loc],
            });
        };
        func(vec![left2, right2]).map_err(|msg| Error {
            msg,
            lines: vec![right_loc],
        })
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
                msg: format!("Expected an integer, got {}", idx2.val),
                lines: vec![idx2.loc]
            })
        };
        let ValueType::List(ls) = val.val else {
            return Err(Error {
                msg: format!("Expected a list, got {}", val.val),
                lines: vec![val.loc]
            })
        };
        let n2 = MList::check_index(n, ls.read(|l| l.len())).ok_or_else(|| Error {
            msg: format!("Index out of range: {}", n),
            lines: vec![loc]
        })?;
        Ok(ls.read(|l| l[n2].clone()).val)
    }
}
