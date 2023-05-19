use std::collections::HashMap;

use crate::lexer::{Token, TokenType};
use crate::parser::{ExprType, StmtType, Stmt};
use crate::{error::Error, parser::Expr};

// TODO: cannot have Eq because of the float
#[derive(Debug, PartialEq, Clone)]
struct Environment {
    env: HashMap<String, f64>
}

impl Environment {
    pub fn insert(&mut self, name: String, val: f64) -> Result<(), Error> {
        if self.env.contains_key(&name.to_string()) {
            return Err(Error {
                msg: format!("Name \"{}\" already exists", name),
                lines: vec![(0, 0)]  // TODO: fix
            })
        }
        self.env.insert(name, val);
        Ok(())
    }

    pub fn get(&self, name: &String) -> Result<f64, Error> {
        self.env.get(name).cloned().ok_or(Error {
            msg: format!("Name not found: \"{}\"", name),
            lines: vec![(0, 0)] // TODO: fix
        })
    }

    pub fn update(&mut self, name: &String, val: f64) ->Result<(), Error> {
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
                StmtType::AssingmentStmt(ident, expr) => self.assignmentstmt(ident, expr),
                StmtType::ExprStmt(expr) => Ok({ println!("exprstmt: {:?}", self.expr(expr)?); }),
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

    pub fn expr(&self, expr: &Expr) -> Result<f64, Error> {
        match &expr.typ {
            ExprType::Number(n) => Ok((*n).into()),
            ExprType::String(_) => todo!("strings are not implemented yet!"),
            ExprType::Identifier(ident) => self.environment.get(ident),
            ExprType::Parens(expr) => self.expr(expr),
            ExprType::UnaryOperation(op, expr) => self.unary(op, self.expr(expr)?),
            ExprType::BinaryOperation(left, op, right) => self.binary(self.expr(left)?, op, self.expr(right)?),
        }
    }

    fn unary(&self, sym: &Token, val: f64) -> Result<f64, Error> {
        let TokenType::Symbol(op) = &sym.typ else {
            panic!("Expected a symbol, found {:?}", sym);
        };
        match op.as_str() {
            "-" => Ok(-val),
            _ => Err(Error {
                msg: format!("Operator \"{}\" does not exist", op),
                lines: vec![(sym.start, sym.end)],
            }),
        }
    }

    fn binary(&self, left: f64, sym: &Token, right: f64) -> Result<f64, Error> {
        let TokenType::Symbol(op) = &sym.typ else {
            panic!("Expected a symbol, found {:?}", sym)
        };
        Ok(match op.as_str() {
            "+" => left + right,
            "-" => left - right,
            "*" => left * right,
            "/" => {
                if right == 0.0 {
                    // rust gives "inf" TODO: make better
                    return Err(Error {
                        msg: "Cannot divide by zero".to_string(),
                        lines: vec![(sym.start, sym.end)],
                    });
                }
                left / right
            }
            _ => todo!("Unary not implemented yet!"),
        })
    }
}
