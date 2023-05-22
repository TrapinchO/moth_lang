use std::collections::HashMap;

use crate::lexer::{Token, TokenType};
use crate::parser::{ExprType, StmtType, Stmt};
use crate::{error::Error, parser::Expr};

#[derive(Debug, PartialEq, Clone)]
enum Value {
    String(String),
    Bool(bool),
    Int(i32),
    Float(f32),
}

// TODO: cannot have Eq because of the float
#[derive(Debug, PartialEq, Clone)]
struct Environment {
    env: HashMap<String, Value>
}

impl Environment {
    pub fn insert(&mut self, name: String, val: Value) -> Result<(), Error> {
        if self.env.contains_key(&name) {
            return Err(Error {
                msg: format!("Name \"{}\" already exists", name),
                lines: vec![(0, 0)]  // TODO: fix
            })
        }
        self.env.insert(name, val);
        Ok(())
    }

    pub fn get(&self, name: &String) -> Result<Value, Error> {
        self.env.get(name).cloned().ok_or(Error {
            msg: format!("Name not found: \"{}\"", name),
            lines: vec![(0, 0)] // TODO: fix
        })
    }

    pub fn update(&mut self, name: &String, val: Value) ->Result<(), Error> {
        if !self.env.contains_key(&name.to_string()) {
            return Err(Error {
                msg: format!("Name \"{}\" does not exists", name),
                lines: vec![(0, 0)]  // TODO: fix
            })
        }
        *self.env.get_mut(name).unwrap() = val;
        Ok(())
    }
}

pub fn interpret(stmt: &Vec<Stmt>) -> Result<(), Error> {
    Interpreter::new().interpret(stmt)
}

// TODO: use visitor patter? make a trait?
struct Interpreter {
    environment: Environment
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter { environment: Environment {env: HashMap::new() } }
    }

    pub fn interpret(&mut self, stmt: &Vec<Stmt>) -> Result<(), Error> {
        for s in stmt {
            match &s.typ {
                StmtType::AssingmentStmt(ident, expr) => self.assignmentstmt(ident, expr)?,
                StmtType::ExprStmt(expr) => { println!("exprstmt: {:?}", self.expr(expr)?); },
            }
        }
        println!("{:?}", self.environment);
        Ok(())
    }

    fn assignmentstmt(&mut self, ident: &Token, expr: &Expr) -> Result<(), Error> {
        let TokenType::Identifier(name) = &ident.typ else {
            panic!("Expected an identifier");
        };
        self.environment.insert(name.to_string(), self.expr(expr)?)
    }

    pub fn expr(&self, expr: &Expr) -> Result<Value, Error> {
        match &expr.typ {
            ExprType::Int(n) => Ok(Value::Float(*n as f32)),
            ExprType::Float(n) => Ok(Value::Float(*n)),
            ExprType::String(s) => Ok(Value::String(s.clone())),
            ExprType::Bool(b) => Ok(Value::Bool(*b)),
            ExprType::Identifier(ident) => self.environment.get(ident),
            ExprType::Parens(expr) => self.expr(expr),
            ExprType::UnaryOperation(op, expr) => self.unary(op, self.expr(expr)?),
            ExprType::BinaryOperation(left, op, right) => self.binary(self.expr(left)?, op, self.expr(right)?),
        }
    }

    fn unary(&self, sym: &Token, val: Value) -> Result<Value, Error> {
        let TokenType::Symbol(op) = &sym.typ else {
            panic!("Expected a symbol, found {:?}", sym);
        };
        Ok(match val {
            Value::Float(n) => {
                match op.as_str() {
                    "-" => Value::Float(-n),  // TODO fix Int vs Float
                    _ => return operator_error(sym),
                }
            },
            Value::Int(n) => {
                match op.as_str() {
                    "-" => Value::Int(-n),  // TODO fix Int vs Float
                    _ => return operator_error(sym),
                }
            },
            _ => todo!("Not yet implemented!")
        })
    }

    fn binary(&self, left: Value, sym: &Token, right: Value) -> Result<Value, Error> {
        let TokenType::Symbol(op) = &sym.typ else {
            panic!("Expected a symbol, found {:?}", sym)
        };
        Ok(match (left, right) {
            (Value::Float(f1), Value::Float(f2)) => Value::Float(match op.as_str() {
                "+" => f1 + f2,
                "-" => f1 - f2,
                "*" => f1 * f2,
                "/" => {
                    if f2 == 0.0 {
                        // rust gives "inf" TODO: make better
                        return Err(Error {
                            msg: "Cannot divide by zero".to_string(),
                            lines: vec![(sym.start, sym.end)],
                        });
                    }
                    f1 / f2
                }
                _ => return operator_error(sym),
            }),
            (Value::String(s1), Value::String(s2)) => Value::String(match op.as_str() {
                "+" => s1 + &s2,
                _ => return operator_error(sym),
            }),
            (val1, val2) => todo!("Values {:?} and {:?} not supported", val1, val2)
        })
    }
}


fn operator_error<T>(sym: &Token) -> Result<T, Error> {
    let TokenType::Symbol(op) = &sym.typ else {
        panic!("Expected a symbol, found {:?}", sym)
    };
    Err(Error {
        msg: format!("Operator \"{}\" not found", op),
        lines: vec![(sym.start, sym.end)]
    })
}
