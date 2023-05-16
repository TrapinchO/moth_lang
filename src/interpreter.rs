use std::collections::HashMap;

use crate::lexer::{Token, TokenType};
use crate::parser::{ExprType, StmtType, Stmt};
use crate::{error::Error, parser::Expr};

pub fn interpret(stmt: &Vec<Stmt>) -> Result<(), Error> {
    Interpreter::new().interpret(stmt)
}

// TODO: use visitor patter? make a trait?
struct Interpreter {
    environment: HashMap<String, f64>
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter { environment: HashMap::new() }
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
        if self.environment.contains_key(&name.to_string()) {
            return Err(Error {
                msg: format!("Variable \"{}\" already exists!", name),
                lines: vec![(0, 0)]  // TODO: fix
            })
        }
        self.environment.insert(name.to_string(), self.expr(expr)?);
        Ok(())
    }

    pub fn expr(&self, expr: &Expr) -> Result<f64, Error> {
        match &expr.typ {
            ExprType::Number(n) => Ok((*n).into()),
            ExprType::String(_) => todo!("strings are not implemented yet!"),
            ExprType::Identifier(ident) => self.environment.get(ident).cloned().ok_or(Error {
                msg: format!("Identifier not found: \"{}\"", ident),
                lines: vec![(0, 0)] // TODO: fix
            }),
            ExprType::Parens(expr) => self.expr(expr),
            ExprType::UnaryOperation(op, expr) => self.unary(op, self.expr(expr)?),
            ExprType::BinaryOperation(left, op, right) => self.binary(self.expr(left)?, op, self.expr(right)?),
        }
    }

    fn unary(&self, _op: &Token, val: f64) -> Result<f64, Error> {
        let TokenType::Symbol(op) = &_op.typ else {
            panic!("Expected a symbol!")
        };
        match op.as_str() {
            "-" => Ok(-val),
            _ => Err(Error {
                msg: format!("Operator \"{}\" does not exist", op.as_str()),
                lines: vec![(_op.start, _op.end)],
            }),
        }
    }

    fn binary(&self, left: f64, _op: &Token, right: f64) -> Result<f64, Error> {
        let TokenType::Symbol(op) = &_op.typ else {
            panic!("Expected a symbol!")
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
                        lines: vec![(_op.start, _op.end)],
                    });
                }
                left / right
            }
            _ => todo!("Unary not implemented yet!"),
        })
    }
}
