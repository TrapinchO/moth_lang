use std::collections::HashMap;

use crate::{
    environment::Environment,
    error::{Error, ErrorType},
    exprstmt::{Expr, ExprType, Stmt, StmtType},
    token::*,
    value::*,
    visitor::ExprVisitor,
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
                    }.to_string();
                    return Err(Error {
                        msg,
                        lines: vec![s.loc()] // TODO: add locations
                    });
                }
            };
        }
        Ok(())
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
        match stmt.val {
            StmtType::VarDeclStmt(..) => self.var_decl(stmt),
            StmtType::AssignStmt(..) => self.assignment(stmt),
            StmtType::ExprStmt(..) => self.expr(stmt),
            StmtType::BlockStmt(..) => self.block(stmt),
            StmtType::IfStmt(..) => self.if_else(stmt),
            StmtType::WhileStmt(..) => self.whiles(stmt),
            StmtType::FunDeclStmt(..) => self.fun(stmt),
            StmtType::ContinueStmt => self.cont(stmt),
            StmtType::BreakStmt => self.brek(stmt),
            StmtType::ReturnStmt(..) => self.retur(stmt),
        }
    }
    fn var_decl(&mut self, stmt: Stmt) -> Result<(), ErrorType> {
        let StmtType::VarDeclStmt(ident, expr) = stmt.val else {
            unreachable!()
        };
        let val = self.visit_expr(expr)?;
        self.environment.insert(&ident, val.val)?;
        Ok(())
    }

    fn assignment(&mut self, stmt: Stmt) -> Result<(), ErrorType> {
        let StmtType::AssignStmt(ident, expr) = stmt.val else {
            unreachable!()
        };
        let val = self.visit_expr(expr)?;
        self.environment.update(&ident, val.val)?;
        Ok(())
    }

    fn block(&mut self, stmt: Stmt) -> Result<(), ErrorType> {
        let StmtType::BlockStmt(block) = stmt.val else {
            unreachable!()
        };
        self.interpret_block(block)?;
        Ok(())
    }

    fn if_else(&mut self, stmt: Stmt) -> Result<(), ErrorType> {
        let StmtType::IfStmt(blocks) = stmt.val else {
            unreachable!()
        };
        for (cond, block) in blocks {
            let ValueType::Bool(cond2) = self.visit_expr(cond.clone())?.val else {
                return Err(ErrorType::Error(Error {
                    msg: format!("Expected bool, got {}", cond.val),
                    lines: vec![cond.loc()],
                }));
            };
            // do not continue
            if cond2 {
                self.interpret_block(block)?;
                break;
            }
        }

        Ok(())
    }

    fn whiles(&mut self, stmt: Stmt) -> Result<(), ErrorType> {
        let StmtType::WhileStmt(cond, block) = stmt.val else {
            unreachable!()
        };
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

    fn expr(&mut self, stmt: Stmt) -> Result<(), ErrorType> {
        let StmtType::ExprStmt(expr) = stmt.val else {
            unreachable!()
        };
        // TODO: later check if it is not unit!
        let _ = self.visit_expr(expr)?;
        Ok(())
    }
    fn fun(&mut self, stmt: Stmt) -> Result<(), ErrorType> {
        let StmtType::FunDeclStmt(ident, params, block) = stmt.val else {
            unreachable!()
        };
        let mut params2 = vec![];
        for p in params {
            let TokenType::Identifier(n) = &p.val else {
                unreachable!()
            };
            params2.push(n.clone());
        }
        self.environment.insert(
            &ident,
            ValueType::Function(params2, block)
        )?;
        // TODO: nothing here yet
        Ok(())
    }
    fn brek(&mut self, _stmt: Stmt) -> Result<(), ErrorType> {
        Err(ErrorType::Break)
    }
    fn cont(&mut self, _stmt: Stmt) -> Result<(), ErrorType> {
        Err(ErrorType::Continue)
    }
    fn retur(&mut self, stmt: Stmt) -> Result<(), ErrorType> {
        let StmtType::ReturnStmt(expr) = stmt.val else {
            unreachable!()
        };
        let val = self.visit_expr(expr)?;
        Err(ErrorType::Return(val))
    }
}

impl ExprVisitor<Value> for Interpreter {
    fn unit(&mut self, expr: Expr) -> Result<Value, Error> {
        Ok(Value {
            val: ValueType::Unit,
            start: expr.start,
            end: expr.end,
        })
    }
    fn int(&mut self, expr: Expr) -> Result<Value, Error> {
        let ExprType::Int(n) = expr.val else { unreachable!() };
        Ok(Value {
            val: ValueType::Int(n),
            start: expr.start,
            end: expr.end,
        })
    }
    fn float(&mut self, expr: Expr) -> Result<Value, Error> {
        let ExprType::Float(f) = expr.val else { unreachable!() };
        Ok(Value {
            val: ValueType::Float(f),
            start: expr.start,
            end: expr.end,
        })
    }
    fn string(&mut self, expr: Expr) -> Result<Value, Error> {
        let ExprType::String(s) = expr.val else { unreachable!() };
        Ok(Value {
            val: ValueType::String(s),
            start: expr.start,
            end: expr.end,
        })
    }
    fn identifier(&mut self, expr: Expr) -> Result<Value, Error> {
        let ExprType::Identifier(name) = expr.val else {
            unreachable!()
        };
        Ok(Value {
            val: self.environment.get(&name, (expr.start, expr.end))?,
            start: expr.start,
            end: expr.end,
        })
    }
    fn bool(&mut self, expr: Expr) -> Result<Value, Error> {
        let ExprType::Bool(b) = expr.val else { unreachable!() };
        Ok(Value {
            val: ValueType::Bool(b),
            start: expr.start,
            end: expr.end,
        })
    }
    fn parens(&mut self, expr: Expr) -> Result<Value, Error> {
        let ExprType::Parens(expr2) = expr.val else {
            unreachable!()
        };
        Ok(Value {
            start: expr.start,
            end: expr.end,
            val: self.visit_expr(*expr2)?.val,
        })
    }
    fn call(&mut self, expr: Expr) -> Result<Value, Error> {
        let ExprType::Call(callee, args) = expr.val.clone() else {
            unreachable!()
        };
        let mut args2 = vec![];
        for arg in args {
            args2.push(self.visit_expr(arg)?);
        }

        let callee = self.visit_expr(*callee)?;
        match callee.val {
            ValueType::NativeFunction(func) => Ok(Value {
                val: func(args2).map_err(|msg| Error {
                    msg,
                    lines: vec![expr.loc()],
                })?,
                start: expr.start,
                end: expr.end,
            }),
            ValueType::Function(params, block) => {
                if args2.len() != params.len() {
                    return Err(Error {
                        msg: format!(
                            "the number of arguments ({}) must match the number of parameters ({})",
                            args2.len(),
                            params.len()
                        ),
                        lines: vec![expr.loc()],
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
                                lines: vec![expr.loc()], // TODO: add locations
                            })
                        }
                        ErrorType::Continue => {
                            return Err(Error {
                                msg: "Cannot use break outside of loop".to_string(),
                                lines: vec![expr.loc()], // TODO: add locations
                            })
                        }
                    },
                };
                self.remove_scope();
                Ok(Value {
                    val,
                    start: expr.start,
                    end: expr.end,
                })
            }
            _ => Err(Error {
                msg: format!("\"{}\" is not calleable", callee.val),
                lines: vec![callee.loc()],
            }),
        }
    }
    fn unary(&mut self, expr: Expr) -> Result<Value, Error> {
        let ExprType::UnaryOperation(op, expr2) = expr.val.clone() else {
            unreachable!()
        };
        let val = self.visit_expr(*expr2)?;
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
                        lines: vec![val.loc()],
                    })
                }
            },
            "!" => match val.val {
                ValueType::Bool(b) => ValueType::Bool(!b),
                _ => {
                    return Err(Error {
                        msg: format!("Incorrect type: {}", val.val),
                        lines: vec![val.loc()],
                    })
                }
            },
            sym => {
                unreachable!("unknown binary operator interpreted: {}", sym);
            }
        };

        Ok(Value {
            val: new_val,
            start: expr.start,
            end: expr.end,
        })
    }
    fn binary(&mut self, expr: Expr) -> Result<Value, Error> {
        let ExprType::BinaryOperation(left, op, right) = expr.val else {
            unreachable!()
        };
        let left2 = self.visit_expr(*left)?;
        let right2 = self.visit_expr(*right.clone())?;
        let TokenType::Symbol(op_name) = &op.val else {
            panic!("Expected a symbol, found {}", op.val)
        };
        let ValueType::NativeFunction(func) = self.environment.get(op_name, (op.start, op.end))? else {
            return Err(Error {
                msg: format!("Symbol\"{}\" is not a function", op_name),
                lines: vec![op.loc()],
            });
        };
        Ok(Value {
            start: left2.start,
            end: right2.end,
            val: func(vec![left2, right2]).map_err(|msg| Error {
                msg,
                lines: vec![right.loc()],
            })?,
        })
    }
}
