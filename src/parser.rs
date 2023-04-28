use crate::lexer::{Token, TokenType};
use crate::error::Error;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExprType {
    Number(i32),
    String(String),
    Parens(Rc<ExprType>),
    UnaryOperation(Token, Rc<ExprType>),
    BinaryOperation(Rc<ExprType>, Token, Rc<ExprType>),
}

impl ExprType {
    fn format(&self) -> String {
        match self {
            Self::Number(n) => n.to_string(),
            Self::String(s) => format!("\"{}\"", s),
            Self::Parens(expr) => format!("({})", expr.format()),
            Self::UnaryOperation(op, expr) => format!("({} {})", op.typ, expr),
            Self::BinaryOperation(left, op, right) => format!("({} {} {})", left, op.typ, right)
        }
    }
}
impl Display for ExprType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}


pub fn parse(tokens: Vec<Token>) -> Result<ExprType, Error> {
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

    pub fn parse(&mut self) -> Result<ExprType, Error> {
        self.parse_binary()
    }

    fn parse_binary(&mut self) -> Result<ExprType, Error> {
        let left = self.parse_primary()?;
        if let Some(tok @ Token {typ: TokenType::Symbol(_), .. }) = &self.tokens.get(self.idx) {
            self.idx += 1;
            let tok = tok.clone().to_owned();
            let right = self.parse_binary()?;
            return Ok(ExprType::BinaryOperation(left.into(), tok, right.into()))
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<ExprType, Error> {
        let tok = self.tokens.get(self.idx).ok_or(Error {
            msg: "Expected an element".to_string(),
            line: 0,
            end: 0,
            start: 0,
        })?;
        let expr = match &tok.typ {
            TokenType::String(s) => ExprType::String(s.to_string()),
            TokenType::Number(n) => ExprType::Number(*n),
            TokenType::LParen => {
                self.idx += 1;
                let expr = self.parse()?;
                let tok = self.tokens.get(self.idx)
                    .unwrap_or_else(|| panic!("Parser accessed an element beyond the token vector at index {}", self.idx));
                match tok {
                    &Token { typ: TokenType::RParen, .. } => ExprType::Parens(expr.into()),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Precedence {
    precedence: usize,
    associativity: Associativity,
}

impl Precedence {
    pub fn new(prec: usize, assoc: Associativity) -> Self {
        Precedence {
            precedence: prec,
            associativity: assoc,
        }
    }
}

// https://stackoverflow.com/a/67992584
pub fn reassoc(expr: &ExprType) -> Result<ExprType, Error> {
    Ok(match expr {
        ExprType::BinaryOperation(left, op, right) => reassoc_(&reassoc(&left.clone())?, op, &reassoc(&right.clone())?)?,
        ExprType::Parens(expr) => ExprType::Parens(reassoc(expr)?.into()),
        ExprType::UnaryOperation(op, expr) => ExprType::UnaryOperation(op.clone(), reassoc(expr)?.into()),
        expr => expr.clone(),
    })
}

fn reassoc_(left: &ExprType, op1: &Token, right: &ExprType) -> Result<ExprType, Error> {
    let prec_table: HashMap<&str, Precedence> = [
        ("+", Precedence::new(1, Associativity::Left)),
        ("-", Precedence::new(1, Associativity::Left)),
        ("*", Precedence::new(2, Associativity::Left)),
        ("/", Precedence::new(2, Associativity::Left)),
    ].iter().cloned().collect();

    // not a binary operation, no need to reassociate it
    let ExprType::BinaryOperation(left2, op2, right2) = right else {
        return Ok(ExprType::BinaryOperation(
            left.clone().into(),
            op1.clone(),
            right.clone().into()
        ))
    };

    let Token {typ: TokenType::Symbol(op1_sym), ..} = op1.clone() else {
        panic!("Operator token 1 is not a symbol");
    };
    let prec1 = prec_table.get(op1_sym.as_str())
        .ok_or(Error {
            msg: format!("Operator not found: {}", op1_sym),
            line: op1.line,
            start: op1.start,
            end: op1.end,
        })?;

    let Token {typ: TokenType::Symbol(op2_sym), ..} = op2.clone() else {
        panic!("Operator token 2 is not a symbol");
    };
    let prec2 = prec_table.get(op2_sym.as_str())
        .ok_or(Error {
            msg: format!("Operator not found: {}", op2_sym),
            line: op2.line,
            start: op2.start,
            end: op2.end,
        })?;

    match prec1.precedence.cmp(&prec2.precedence) {
        std::cmp::Ordering::Greater => Ok(ExprType::BinaryOperation(
            reassoc_(left, op1, left2)?.into(),
            op2.clone(),
            right2.clone()
        )),

        std::cmp::Ordering::Less => Ok(ExprType::BinaryOperation(
            left.clone().into(),
            op1.clone(),
            right.clone().into()
        )),

        std::cmp::Ordering::Equal => match (prec1.associativity, prec2.associativity) {
            (Associativity::Right, Associativity::Right) => Ok(ExprType::BinaryOperation(
                reassoc_(left, op2, left2)?.into(),
                op1.clone(),
                right2.clone()
                )),
            (Associativity::Left, Associativity::Left) => Ok(ExprType::BinaryOperation(
                left.clone().into(),
                op1.clone(),
                right.clone().into()
            )),
            _ => Err(Error {
                msg: format!("Incompatible operator precedence: {} ({:?}) and {} ({:?})", op1.typ, prec1, op2.typ, prec2),
                line: op1.line,
                start: op1.start,
                end: op2.end,
            }),
        }
    }
}
