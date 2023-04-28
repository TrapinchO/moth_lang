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
    UnaryOperation(Token, Rc<Expr>),
    BinaryOperation(Rc<Expr>, Token, Rc<Expr>),
}

impl Expr {
    fn format(&self) -> String {
        let s = match self {
            Self::Number(n) => n.to_string(),
            Self::String(s) => format!("\"{}\"", s),
            Self::ParensExpr(expr) => format!("({})", expr.format()),
            Self::UnaryOperation(op, expr) => format!("({} {})", op.typ, expr),
            Self::BinaryOperation(left, op, right) => format!("({} {} {})", left, op.typ, right)
        };
        format!("{}", s)
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
        if let Some(tok @ Token {typ: TokenType::Symbol(_), .. }) = &self.tokens.get(self.idx) {
            self.idx += 1;
            return Ok(Expr::BinaryOperation(left.into(), tok.clone().to_owned(), self.parse_binary()?.into()))
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

struct OperatorPrecedence {
    precedence: usize,
    associativity: Associativity,
}
impl Precedence {
    pub fn new(&self, prec: usize, assoc: Associativity) {
        Precedence {
            precedence: prec,
            associativity: assoc,
        }
    }
}

// https://stackoverflow.com/a/67992584
pub fn reassoc(expr: &Expr) -> Result<Expr, Error> {
    Ok(match expr {
        Expr::BinaryOperation(left, op, right) => reassoc_(&reassoc(&left.clone())?, &op, &reassoc(&right.clone())?)?,
        Expr::ParensExpr(expr) => Expr::ParensExpr(reassoc(&expr)?.into()),
        Expr::UnaryOperation(op, expr) => Expr::UnaryOperation(op.clone().to_owned(), reassoc(expr)?.into()),
        expr => expr.clone(),
    })
}

fn reassoc_(left: &Expr, op: &Token, right: &Expr) -> Result<Expr, Error> {
    let prec_table: HashMap<&str, (usize, Precedence)> = [
        ("+", Precedence::new(1, Associativity::Left)),
        ("-", Precedence::new(1, Associativity::Left)),
        ("*", Precedence::new(2, Associativity::Left)),
        ("/", Precedence::new(2, Associativity::Left)),
    ].iter().cloned().collect();

    // not a binary operation, no need to reassociate it
    let Expr::BinaryOperation(left2, op2, right2) = right else {
       return Ok(Expr::BinaryOperation(left.clone().into(), op.clone(),right.clone().into()))
    };

    let Token {typ: TokenType::Symbol(op1_sym), ..} = op.clone() else {
        panic!("Operator token 1 is not a symbol");
    };
    let prec1 = prec_table.get(op1_sym.as_str())
        .ok_or(Error {
            msg: format!("Operator not found: {}", op1_sym),
            line: op.line,
            start: op.start,
            end: op.end,
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

    match prec1.precedence.cmp(prec2.precedence) {
        std::cmp::Ordering::Greater => Ok(Expr::BinaryOperation(
            reassoc_(&left, &op, &left2)?.into(),
            op2.clone(),
            right2.clone()
        )),

        std::cmp::Ordering::Less => Ok(Expr::BinaryOperation(
            left.clone().into(),
            op.clone(),
            right.clone().into()
        )),

        std::cmp::Ordering::Equal => match (prec1.associativity, prec2.associativity) {
            (Associativity::Right, Associativity::Right) => Ok(Expr::BinaryOperation(
                reassoc_(left, &op2, &left2)?.into(),
                op.clone(),
                right2.clone()
                )),
            (Associativity::Left, Associativity::Left) => Ok(Expr::BinaryOperation(
                left.clone().into(),
                op.clone(),
                right.clone().into()
            )),
            _ => Err(Error {
                msg: format!("Wrong associativity: {:?}: {:?}; {:?}: {:?}", op, prec1, op2, prec2),
                line: op.line,
                start: op.start,
                end: op2.end,
            }),
        }
    }
}
