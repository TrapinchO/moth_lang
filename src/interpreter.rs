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
    Function(fn(Vec<Value>)->Result<Value, Error>),
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

const BUILTINS: [(&str, fn(Vec<Value>)->Result<Value, Error>); 2] = [
    ("+", |args| {
        // TODO: add proper positions
        let [left, right] = &args[..] else { return Err(Error { msg: format!("Wrong number of arguemtns {}", args.len()), lines: vec![(0, 0)] }) };
        Ok(match (left, right) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
            (Value::String(a), Value::String(b)) => Value::String(a.clone() + b),
            _ => return Err(Error {
                msg: format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right),
                lines: vec![(0, 0)]
            })
        })
    }),
    ("-", |args| {
        // TODO: add proper positions
        let [left, right] = &args[..] else { return Err(Error { msg: format!("Wrong number of arguemtns {}", args.len()), lines: vec![(0, 0)] }) };
        Ok(match (left, right) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
            _ => return Err(Error {
                msg: format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right),
                lines: vec![(0, 0)]
            })
        })
    }),
];
pub fn interpret(stmt: &Vec<Stmt>) -> Result<(), Error> {
    let defaults = HashMap::from(BUILTINS.map(|(name, f)| (name.to_string(), Value::Function(f))));
    Interpreter::new(defaults).interpret(stmt)
}

// TODO: use visitor patter? make a trait?
struct Interpreter {
    environment: Environment
}

impl Interpreter {
    pub fn new(defaults: HashMap<String, Value>) -> Self {
        Interpreter { environment: Environment {env: defaults } }
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
        let TokenType::Symbol(op_name) = &sym.typ else {
            panic!("Expected a symbol, found {:?}", sym)
        };
        let Value::Function(op) = self.environment.get(op_name)? else {
            return Err(Error {
                msg: format!("Symbol\"{}\" is not a function!", op_name),
                lines: vec![(sym.start, sym.end)]
            })
        };
        op(vec![left, right])
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
