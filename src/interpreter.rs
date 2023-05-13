use crate::lexer::{Token, TokenType};
use crate::parser::ExprType;
use crate::{error::Error, parser::Expr};

pub fn interpret(expr: &Expr) -> Result<f64, Error> {
    match &expr.typ {
        ExprType::Number(n) => Ok((*n).into()),
        ExprType::String(_) => todo!("not implemented yet!"),
        ExprType::Parens(expr) => interpret(expr),
        ExprType::UnaryOperation(op, expr) => unary(op, interpret(expr)?),
        ExprType::BinaryOperation(left, op, right) => binary(interpret(left)?, op, interpret(right)?),
    }
}

fn unary(_op: &Token, val: f64) -> Result<f64, Error> {
    let TokenType::Symbol(op) = &_op.typ else {
        panic!("Expected a symbol!")
    };
    match op.as_str() {
        "-" => Ok(-val),
        _ => Err(Error {
            msg: format!("Operator \"{}\" does not exist", op.as_str()),
            lines: vec![(_op.line, _op.start, _op.end)],
        }),
    }
}

fn binary(left: f64, _op: &Token, right: f64) -> Result<f64, Error> {
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
                    lines: vec![(_op.line, _op.start, _op.end)],
                });
            }
            left / right
        }
        _ => todo!("Unary not implemented yet!"),
    })
}
