use std::collections::HashMap;

use crate::lexer::{Token, TokenType};
use crate::parser::{ExprType, Stmt};
use crate::{error::Error, parser::Expr};
use crate::visitor::{ExprVisitor, StmtVisitor};

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



#[derive(Debug, PartialEq, Clone)]
struct Environment {
    env: HashMap<String, Value>
}

impl Environment {
    // TODO: fix error positions not being displayed properly
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
        // TODO: add proper positions for the argument list
        let [left, right] = &args[..] else { return Err(Error { msg: format!("Wrong number of arguemtns {}", args.len()), lines: vec![(0, 0)] }) };
        let typ = match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a + b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a + b),
            (ValueType::String(a), ValueType::String(b)) => ValueType::String(a.clone() + &b),
            _ => return Err(Error {
                msg: format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right),
                lines: vec![(left.start, right.end)]
            })
        };
        Ok(Value {
            typ,
            start: left.start,
            end: right.end,
        })
    }),
    ("-", |args| {
        let [left, right] = &args[..] else { return Err(Error { msg: format!("Wrong number of arguemtns {}", args.len()), lines: vec![(0, 0)] }) };
        Ok(Value {
            typ: match (&left.typ, &right.typ) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a - b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a - b),
                _ => return Err(Error {
                    msg: format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right),
                lines: vec![(left.start, right.end)]
                })
            },
            start: left.start,
            end: right.end,
        })
    }),
];

pub fn interpret(stmts: &Vec<Stmt>) -> Result<(), Error> {
    // TODO: solve positions for builtin stuff
    let defaults = HashMap::from(BUILTINS.map(
        |(name, f)| (name.to_string(), Value { typ: ValueType::Function(f), start: 0, end: 0 })
    ));
    Interpreter::new(defaults).interpret(&stmts)
}

struct Interpreter {
    environment: Environment
}

impl Interpreter {
    pub fn new(defaults: HashMap<String, Value>) -> Self {
        Interpreter { environment: Environment {env: defaults } }
    }

    pub fn interpret(&mut self, stmts: &Vec<Stmt>) -> Result<(), Error> {
        for s in stmts {
            // TODO: change to references later
            self.visit_stmt(s.clone())?;
        }
        Ok(())
    }
}

impl StmtVisitor<()> for Interpreter {
    fn var_decl(&mut self, ident: Token, expr: Expr) -> Result<(), Error> {
        let TokenType::Identifier(name) = &ident.typ else {
            panic!("Expected an identifier");
        };
        let val = self.visit_expr(&expr)?;
        self.environment.insert(name.to_string(), val)?;
        Ok(())
    }

    fn assignment(&mut self, ident: String, expr: Expr) -> Result<(), Error> {
        let val = self.visit_expr(&expr)?;
        self.environment.update(&ident, val)?;
        Ok(())
    }

    fn expr(&mut self, expr: Expr) -> Result<(), Error> {
        let val = self.visit_expr(&expr)?;
        println!("{:?}", val);
        Ok(())
    }
}

impl ExprVisitor<Value> for Interpreter {
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
            typ: ValueType::String(s.clone()),
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
        self.visit_expr(expr)
    }
    fn unary(&mut self, op: &Token, expr: &Expr) -> Result<Value, Error> {
        let val = self.visit_expr(expr)?;

        let TokenType::Symbol(op_name) = &op.typ else {
            panic!("Expected a symbol, found {:?}", op);
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
        let left2 = self.visit_expr(left)?;
        let right2 = self.visit_expr(right)?;
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


fn operator_error<T>(sym: &Token) -> Result<T, Error> {
    let TokenType::Symbol(op) = &sym.typ else {
        panic!("Expected a symbol, found {:?}", sym)
    };
    Err(Error {
        msg: format!("Operator \"{}\" not found", op),
        lines: vec![(sym.start, sym.end)]
    })
}
