use std::collections::HashMap;

use crate::lexer::{Token, TokenType};
use crate::parser::{ExprType, StmtType, Stmt};
use crate::{error::Error, parser::Expr};

#[derive(Debug, PartialEq, Clone)]
enum ValueType {
    String(String),
    Bool(bool),
    Int(i32),
    Float(f32),
    Function(fn(Vec<Value>)->Result<Value, Error>),
}

#[derive(Debug, PartialEq, Clone)]
struct Value {
    typ: ValueType,
    start: usize,
    end: usize,
}

trait StmtVisitor<T> {
    fn visit(&mut self, stmt: Stmt) -> Result<T, Error> {
        match stmt.typ {
            StmtType::AssingmentStmt(ident, expr) => self.assignment(ident, expr),
            StmtType::ExprStmt(expr) => self.expr(expr),
        }
    }

    fn assignment(&mut self, ident: Token, expr: Expr) -> Result<T, Error>;
    fn expr(&mut self, expr: Expr) -> Result<T, Error>;
}

trait ExprVisitor<T> {
    fn visit(&mut self, expr: Expr) -> Result<T, Error> {
        match expr.typ {
            ExprType::Int(_) => self.int(&expr),
            ExprType::Float(_) => self.float(&expr),
            ExprType::String(_) => self.string(&expr),
            ExprType::Bool(_) => self.bool(&expr),
            ExprType::Identifier(_) => self.identifier(&expr),
            ExprType::Parens(expr) => self.parens(&expr),
            ExprType::UnaryOperation(op, expr) => self.unary(&op, &expr),
            ExprType::BinaryOperation(left, op, right) => self.binary(&left, &op, &right),
        }
    }
    fn int(&mut self, expr: &Expr) -> Result<T, Error>;
    fn float(&mut self, expr: &Expr) -> Result<T, Error>;
    fn string(&mut self, expr: &Expr) -> Result<T, Error>;
    fn bool(&mut self, expr: &Expr) -> Result<T, Error>;
    fn identifier(&mut self, expr: &Expr) -> Result<T, Error>;
    fn parens(&mut self, expr: &Expr) -> Result<T, Error>;
    fn unary(&mut self, op: &Token, expr: &Expr) -> Result<T, Error>;
    fn binary(&mut self, left: &Expr, op: &Token, right: &Expr) -> Result<T, Error>;
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
                lines: vec![(val.start, val.end)]
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
                lines: vec![(val.start, val.end)]
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
        let typ = match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a + b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a + b),
            (ValueType::String(a), ValueType::String(b)) => ValueType::String(a.clone() + &b),
            _ => return Err(Error {
                msg: format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right),
                lines: vec![(0, 0)]
            })
        };
        Ok(Value {
            typ,
            start: left.start,
            end: right.end,
        })
    }),
    ("-", |args| {
        // TODO: add proper positions
        let [left, right] = &args[..] else { return Err(Error { msg: format!("Wrong number of arguemtns {}", args.len()), lines: vec![(0, 0)] }) };
        Ok(Value {
            typ: match (&left.typ, &right.typ) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a - b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a - b),
                _ => return Err(Error {
                    msg: format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right),
                    lines: vec![(0, 0)]
                })
            },
            start: left.start,
            end: right.end,
        })
    }),
];
pub fn interpret(stmt: &Vec<Stmt>) -> Result<(), Error> {
    // TODO: solve positions for builtin stuff
    let defaults = HashMap::from(BUILTINS.map(|(name, f)| (name.to_string(), Value { typ: ValueType::Function(f), start: 0, end: 0 })));
    Interpreter::new(defaults).interpret(stmt)
}

// TODO: use visitor patter? make a trait?
struct Interpreter {
    environment: Environment
}

struct Interpreter2 {
    environment: Environment
}

impl ExprVisitor<Value> for Interpreter2 {
    fn int(&mut self, expr: &Expr) -> Result<Value, Error> {
        let ExprType::Int(n) = &expr.typ.clone() else { unreachable!() };
        Ok(Value {
            typ: ValueType::Int(*n),
            start: expr.start,
            end: expr.end,
        })
    }
    fn float(&mut self, expr: &Expr) -> Result<Value, Error> {
        let ExprType::Float(f) = &expr.typ.clone() else { unreachable!() };
        Ok(Value {
            typ: ValueType::Float(*f),
            start: expr.start,
            end: expr.end,
        })
    }
    fn string(&mut self, expr: &Expr) -> Result<Value, Error> {
        let ExprType::String(s) = &expr.typ.clone() else { unreachable!() };
        Ok(Value {
            typ: ValueType::String(*s),
            start: expr.start,
            end: expr.end,
        })
    }
    fn identifier(&mut self, expr: &Expr) -> Result<Value, Error> {
        let ExprType::Identifier(ident) = &expr.typ.clone() else { unreachable!() };
        Ok(Value {
            typ: self.environment.get(ident)?.typ,
            start: expr.start,
            end: expr.end,
        })
    }
    fn bool(&mut self, expr: &Expr) -> Result<Value, Error> {
        let ExprType::Bool(b) = &expr.typ.clone() else { unreachable!() };
        Ok(Value {
            typ: ValueType::Bool(*b),
            start: expr.start,
            end: expr.end,
        })
    }
    fn parens(&mut self, expr: &Expr) -> Result<Value, Error> {
        self.visit(&expr)
    }
    fn unary(&mut self, op: &Token, expr: &Expr) -> Result<Value, Error> {
        let val = self.visit(expr)?;

        let TokenType::Symbol(op_name) = &op.typ else {
            panic!("Expected a symbol, found {:?}", sym);
        };
        Ok(Value {
            typ: match val.typ {
                ValueType::Float(n) => {
                  match op_name.as_str() {
                       "-" => ValueType::Float(-n),  // TODO fix Int vs Float
                       _ => return operator_error(op),
                  }
                },
                ValueType::Int(n) => {
                    match op_name.as_str() {
                        "-" => ValueType::Int(-n),  // TODO fix Int vs Float
                        _ => return operator_error(op),
                    }
                },
                _ => todo!("Not yet implemented!")
            },
            start: op.start,
            end: val.end
        })
    }
    fn binary(&mut self, left: &Expr, op: &Token, right: &Expr) -> Result<Value, Error> {
        let left2 = self.visit(left)?;
        let right2 = self.visit(right)?;
        let TokenType::Symbol(op_name) = &op.typ else {
            panic!("Expected a symbol, found {:?}", op)
        };
        let ValueType::Function(func) = self.environment.get(op_name)?.typ else {
            return Err(Error {
                msg: format!("Symbol\"{}\" is not a function!", op_name),
                lines: vec![(op.start, op.end)]
            })
        };
        func(vec![left2, right2])
    }
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
        let typ = match &expr.typ {
            ExprType::Int(n) => ValueType::Float(*n as f32),
            ExprType::Float(n) => ValueType::Float(*n),
            ExprType::String(s) => ValueType::String(s.clone()),
            ExprType::Bool(b) => ValueType::Bool(*b),
            // TODO: fix the arms below, they discard positions
            // keep? discard? store somewhere else?
            ExprType::Identifier(ident) => self.environment.get(ident)?.typ,
            ExprType::Parens(expr) => self.expr(expr)?.typ,
            ExprType::UnaryOperation(op, expr) => self.unary(op, self.expr(expr)?)?.typ,
            ExprType::BinaryOperation(left, op, right) => self.binary(self.expr(left)?, op, self.expr(right)?)?.typ,
        };
        Ok(Value { typ, start: expr.start, end: expr.end })
    }

    fn unary(&self, sym: &Token, val: Value) -> Result<Value, Error> {
        let TokenType::Symbol(op) = &sym.typ else {
            panic!("Expected a symbol, found {:?}", sym);
        };
        Ok(Value {
            typ: match val.typ {
                ValueType::Float(n) => {
                  match op.as_str() {
                       "-" => ValueType::Float(-n),  // TODO fix Int vs Float
                       _ => return operator_error(sym),
                  }
                },
                ValueType::Int(n) => {
                    match op.as_str() {
                        "-" => ValueType::Int(-n),  // TODO fix Int vs Float
                        _ => return operator_error(sym),
                    }
                },
                _ => todo!("Not yet implemented!")
            },
            start: sym.start,
            end: val.end
        })
    }

    fn binary(&self, left: Value, sym: &Token, right: Value) -> Result<Value, Error> {
        let TokenType::Symbol(op_name) = &sym.typ else {
            panic!("Expected a symbol, found {:?}", sym)
        };
        let ValueType::Function(op) = self.environment.get(op_name)?.typ else {
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
