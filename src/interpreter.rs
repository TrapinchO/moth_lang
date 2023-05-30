use std::collections::HashMap;

use crate::token::*;
use crate::exprstmt::{ExprType, Expr, Stmt};
use crate::error::Error;
use crate::visitor::{ExprVisitor, StmtVisitor};
use crate::value::*;




#[derive(Debug, PartialEq, Clone)]
struct Environment {
    env: HashMap<String, ValueType>
}

impl Environment {
    pub fn insert(&mut self, ident: &Token, val: Value) -> Result<(), Error> {
        let TokenType::Identifier(name) = &ident.typ else { unreachable!() };
        if self.env.contains_key(name) {
            return Err(Error {
                msg: format!("Name \"{}\" already exists", name),
                lines: vec![(ident.start, ident.end)]
            })
        }
        self.env.insert(name.clone(), val.typ);
        Ok(())
    }

    pub fn get(&self, ident: &String, pos: (usize, usize)) -> Result<ValueType, Error> {
        self.env.get(ident).cloned().ok_or(Error {
            msg: format!("Name not found: \"{}\"", ident),
            lines: vec![pos]
        })
    }

    pub fn update(&mut self, ident: &Token, val: Value) ->Result<(), Error> {
        let TokenType::Identifier(name) = &ident.typ else { unreachable!() };
        if !self.env.contains_key(&name.to_string()) {
            return Err(Error {
                msg: format!("Name \"{}\" does not exists", name),
                lines: vec![(ident.start, ident.end)]
            })
        }
        *self.env.get_mut(name).unwrap() = val.typ;
        Ok(())
    }
}


pub fn interpret(stmts: &Vec<Stmt>) -> Result<(), Error> {
    // TODO: solve positions for builtin stuff
    let defaults = HashMap::from(BUILTINS.map(
        |(name, f)| (name.to_string(), ValueType::Function(f))
    ));
    Interpreter::new(defaults).interpret(stmts)
}

struct Interpreter {
    environment: Environment
}

impl Interpreter {
    pub fn new(defaults: HashMap<String, ValueType>) -> Self {
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
        let val = self.visit_expr(&expr)?;
        self.environment.insert(&ident, val)?;
        Ok(())
    }

    fn assignment(&mut self, ident: Token, expr: Expr) -> Result<(), Error> {
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
        let ExprType::Identifier(name) = &expr.typ.clone() else { unreachable!() };
        Ok(Value {
            typ: self.environment.get(name, (expr.start, expr.end))?,
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
        let ValueType::Function(func) = self.environment.get(op_name, (op.start, op.end))? else {
            return Err(Error {
                msg: format!("Symbol\"{}\" is not a function!", op_name),
                lines: vec![(op.start, op.end)]
            })
        };
        Ok(Value {
            typ: func(vec![val]).or_else(|msg| Err(Error { msg, lines: vec![(op.start, expr.end)] }))?,
            start: op.start,
            end: expr.end,
        })
    }
    fn binary(&mut self, left: &Expr, op: &Token, right: &Expr) -> Result<Value, Error> {
        let left2 = self.visit_expr(left)?;
        let right2 = self.visit_expr(right)?;
        let TokenType::Symbol(op_name) = &op.typ else {
            panic!("Expected a symbol, found {:?}", op)
        };
        let ValueType::Function(func) = self.environment.get(op_name, (op.start, op.end))? else {
            return Err(Error {
                msg: format!("Symbol\"{}\" is not a function!", op_name),
                lines: vec![(op.start, op.end)]
            })
        };
        Ok(Value {
            typ: func(vec![left2, right2]).or_else(|msg| Err(Error { msg, lines: vec![(left.start, right.end)] }))?,
            start: left.start,
            end: right.end,
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
