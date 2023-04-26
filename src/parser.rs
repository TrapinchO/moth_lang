use crate::lexer::{Token, TokenType};
use crate::error::Error;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    Number(i32),
    String(String),
    ParensExpr(Rc<Expr>),
    UnaryOperation(String, Rc<Expr>),
    BinaryOperation(Rc<Expr>, String, Rc<Expr>),
}

impl Expr {
    fn format(&self) -> String {
        let s = match self {
            Self::Number(n) => n.to_string(),
            Self::String(s) => format!("\"{}\"", s),
            Self::ParensExpr(expr) => expr.format(),
            Self::UnaryOperation(op, expr) => format!("{} {}", op, expr),
            Self::BinaryOperation(left, op, right) => format!("{} {} {}", left, op, right)
        };
        format!("({})", s)
    }
}
impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}


pub fn parse(tokens: Vec<Token>) -> Result<Expr, Error> {
    Parser::new(tokens).parse()
}


struct Parser {
    tokens: Vec<Token>,
    idx: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            idx: 0,
        }
    }
    
    pub fn parse(&mut self) -> Result<Expr, Error> {
        self.parse_binary()
    }

    fn parse_binary(&mut self) -> Result<Expr, Error> {
        let left = self.parse_primary()?;
        if let Some(Token {typ: TokenType::Symbol(sym), .. }) = &self.tokens.get(self.idx) {
            self.idx += 1;
            return Ok(Expr::BinaryOperation(left.into(), sym.clone(), self.parse_binary()?.into()))
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, Error> {
        let tok = self.tokens.get(self.idx).ok_or(Error {
            msg: "Expected an element".to_string(),
            line: 0,
            end: 0,
            start: 0,
        })?;
        let expr = match &tok.typ {
            TokenType::String(s) => Expr::String(s.to_string()),
            TokenType::Number(n) => Expr::Number(*n),
            TokenType::LParen => {
                self.idx += 1;
                let expr = self.parse()?;
                let tok = self.tokens.get(self.idx)
                    .expect(&format!("Parser accessed an element beyond the token vector at index {}", self.idx));
                match tok {
                    &Token { typ: TokenType::RParen, .. } => Expr::ParensExpr(expr.into()),
                    tok => return Err(Error {
                        msg: "Expected closing parenthesis".to_string(),
                        line: tok.line,
                        start: tok.start,
                        end: tok.end,
                    }),
                }
            }
            _ => return Err(Error {
                msg: format!("Unknown element: {:?}", tok),
                line: tok.line,
                start: tok.start,
                end: tok.end
            }),
        };
        self.idx += 1;
        Ok(expr)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Associativity {
    Left,
    Right
}

// https://stackoverflow.com/a/67992584
pub fn reassoc(expr: &Expr) -> Result<Expr, String> {
    Ok(match expr {
        Expr::BinaryOperation(left, op, right) => reassoc_(&reassoc(&left.clone())?, &op, &reassoc(&right.clone())?)?,
        Expr::ParensExpr(expr) => Expr::ParensExpr(reassoc(&expr)?.into()),
        expr => expr.clone(),
    })
}

fn reassoc_(left: &Expr, op: &String, right: &Expr) -> Result<Expr, String> {
    let prec_table: HashMap<&str, (usize, Associativity)> = [
        ("+", (1, Associativity::Left)),
        ("-", (1, Associativity::Right)),
        ("*", (2, Associativity::Right)),
        ("/", (2, Associativity::Right)),
    ].iter().cloned().collect();

    match right {
        Expr::BinaryOperation(left2, op2, right2) => {
            let (prec, assoc) = prec_table.get(op.as_str())
                .ok_or(format!("Operator not found: {}", op))?;
            let (prec2, assoc2) = prec_table.get(op2.as_str())
                .ok_or(format!("Operator not found: {}", op2))?;

            match prec.cmp(prec2) {
                std::cmp::Ordering::Greater => Ok(Expr::BinaryOperation(
                    Rc::new(reassoc_(&left, &op, &left2)?),
                    op2.clone(),
                    right2.clone())),

                std::cmp::Ordering::Less => Ok(Expr::BinaryOperation(
                        left.clone().into(),
                        op.clone(),
                        right.clone().into())),

                std::cmp::Ordering::Equal => match (assoc, assoc2) {
                    (Associativity::Right, Associativity::Right) => Ok(Expr::BinaryOperation(
                        reassoc_(left, &op2, &left2)?.into(),
                        op.clone(),
                        right2.clone())),
                    (Associativity::Left, Associativity::Left) => Ok(Expr::BinaryOperation(
                        left.clone().into(),
                        op.clone(),
                        right.clone().into()
                    )),
                    _ => Err(format!("Wrong associativity: {}: {} ({:?}); {}: {} ({:?})",
                                     op, prec, assoc, op2, prec2, assoc2)),
                }
            }
        }
        _ => Ok(Expr::BinaryOperation(left.clone().into(), op.clone(),right.clone().into())),
    }
}
