use crate::error::Error;
use crate::lexer::{Token, TokenType};
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExprType {
    Number(i32),
    String(String),
    Parens(Rc<Expr>),
    UnaryOperation(Token, Rc<Expr>),
    BinaryOperation(Rc<Expr>, Token, Rc<Expr>),
}

impl ExprType {
    fn format(&self) -> String {
        match self {
            Self::Number(n) => n.to_string(),
            Self::String(s) => format!("\"{}\"", s),
            Self::Parens(expr) => format!("({})", expr.typ.format()),
            Self::UnaryOperation(op, expr) => format!("({} {})", op.typ, expr),
            Self::BinaryOperation(left, op, right) => format!("({} {} {})", left, op.typ, right),
        }
    }
}

impl Display for ExprType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

// TODO: Stuff will probably break when multiline expressions because of the indexes
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Expr {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub typ: ExprType,
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.typ.format())
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
        if tokens.is_empty() || tokens.len() == 1 && tokens[0].typ == TokenType::Eof {
            panic!("Expected code to parse");
        }
        Parser { tokens, idx: 0 }
    }

    fn is_at_end(&self) -> bool {
        self.idx >= self.tokens.len() // last token should be EOF
    }

    fn get_current(&self) -> &Token {
        if self.is_at_end() {
            panic!("Attempted to index token out ouf bounds: {} (length {})", self.idx, self.tokens.len());
        }
        &self.tokens[self.idx]
    }
    fn advance(&mut self) {
        self.idx += 1;
    }

    pub fn parse(&mut self) -> Result<Expr, Error> {
        self.parse_expression()
    }

    fn parse_expression(&mut self) -> Result<Expr, Error> {
        self.parse_binary()
    }

    fn parse_binary(&mut self) -> Result<Expr, Error> {
        let left = self.parse_primary()?;
        if let tok @ Token {typ: TokenType::Symbol(_), .. } = self.get_current().clone() {
            self.advance();

            let right = self.parse_binary()?;
            return Ok(Expr {
                start: left.start,
                end: right.end,
                line: tok.line,
                typ: ExprType::BinaryOperation(left.into(), tok, right.into()),
            });
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, Error> {
        let tok = self.get_current().clone();
        let expr = match &tok.typ {
            TokenType::String(s) => ExprType::String(s.to_string()),
            TokenType::Number(n) => ExprType::Number(*n),
            TokenType::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                let tok = self.get_current();
                if tok.typ != TokenType::RParen {
                    return Err(Error {
                        msg: "Expected closing parenthesis".to_string(),
                        line: tok.line,
                        start: tok.start,
                        end: tok.end,
                    });
                }
                ExprType::Parens(expr.into())
            }
            TokenType::RParen => return Err(Error {
                msg: "Expected an expression".to_string(),
                line: tok.line,
                start: tok.start,
                end: tok.end
            }),

            TokenType::Eof => return Err(Error {
                msg: "Expected an element".to_string(),
                line: tok.line,
                start: tok.start,
                end: tok.end
            }),
            _ => return Err(Error {
                msg: format!("Unknown element: {:?}", tok),
                line: tok.line,
                start: tok.start,
                end: tok.end
            }),
        };
        self.advance();
        Ok(Expr {
            start: tok.start,
            end: tok.end,
            line: tok.line,
            typ: expr,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Associativity {
    Left,
    Right,
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
pub fn reassoc(expr: &Expr) -> Result<Expr, Error> {
    Ok(match &expr.typ {
        ExprType::BinaryOperation(left, op, right) => reassoc_(
            &reassoc(&left.clone())?,
            op,
            &reassoc(&right.clone())?
        )?,
        ExprType::Parens(expr) => Expr {
            typ: ExprType::Parens(reassoc(expr.as_ref())?.into()),
            ..expr.as_ref().clone()
        },
        ExprType::UnaryOperation(op, expr) => Expr {
            typ: ExprType::UnaryOperation(op.clone(), reassoc(expr.as_ref())?.into()),
            ..expr.as_ref().clone()
        },
        ExprType::Number(_) => expr.clone(),
        ExprType::String(_) => expr.clone(),
    })
}

fn reassoc_(left: &Expr, op1: &Token, right: &Expr) -> Result<Expr, Error> {
    let prec_table: HashMap<&str, Precedence> = [
        ("+", Precedence::new(1, Associativity::Left)),
        ("-", Precedence::new(1, Associativity::Left)),
        ("*", Precedence::new(2, Associativity::Left)),
        ("/", Precedence::new(2, Associativity::Left)),
        ("^^", Precedence::new(10, Associativity::Right)),  // analyzer shut up now please its used
    ].iter().cloned().collect();

    // not a binary operation, no need to reassociate it
    let ExprType::BinaryOperation(left2, op2, right2) = &right.typ else {
        return Ok(Expr {
            typ: ExprType::BinaryOperation(
                left.clone().into(),
                op1.clone(),
                right.clone().into()),
            line: op1.line,
            start: left.start,
            end: right.end,
        })
    };

    let Token {typ: TokenType::Symbol(op1_sym), ..} = op1.clone() else {
        panic!("Operator token 1 is not a symbol");
    };
    let prec1 = prec_table.get(op1_sym.as_str()).ok_or(Error {
        msg: format!("Operator not found: {}", op1_sym),
        line: op1.line,
        start: op1.start,
        end: op1.end,
    })?;

    let Token {typ: TokenType::Symbol(op2_sym), ..} = op2.clone() else {
        panic!("Operator token 2 is not a symbol");
    };
    let prec2 = prec_table.get(op2_sym.as_str()).ok_or(Error {
        msg: format!("Operator not found: {}", op2_sym),
        line: op2.line,
        start: op2.start,
        end: op2.end,
    })?;

    match prec1.precedence.cmp(&prec2.precedence) {
        std::cmp::Ordering::Greater => {
            let left = reassoc_(left, op1, left2)?.into();
            Ok(Expr {
                typ: ExprType::BinaryOperation(left, op2.clone(), right2.clone()),
                line: op2.line,
                start: right2.start,
                end: right2.end,
            })
        }

        std::cmp::Ordering::Less => Ok(Expr {
            typ: ExprType::BinaryOperation(
                left.clone().into(),
                op1.clone(),
                right.clone().into()),
            line: op1.line,
            start: left.start,
            end: right.end,
        }),

        std::cmp::Ordering::Equal => match (prec1.associativity, prec2.associativity) {
            (Associativity::Left, Associativity::Left) => {
                let left = reassoc_(left, op1, left2)?.into();
                Ok(Expr {
                    typ: ExprType::BinaryOperation(left, op2.clone(), right2.clone()),
                    line: op2.line,
                    start: right2.start,
                    end: right2.end,
                })
            }
            (Associativity::Right, Associativity::Right) => Ok(Expr {
                typ: ExprType::BinaryOperation(
                    left.clone().into(),
                    op1.clone(),
                    right.clone().into(),
                ),
                line: op1.line,
                start: left.start,
                end: right.end,
            }),
            _ => Err(Error {
                msg: format!(
                    "Incompatible operator precedence: {} ({:?}) and {} ({:?})",
                    op1.typ, prec1, op2.typ, prec2
                ),
                line: op1.line,
                start: op1.start,
                end: op2.end,
            }),
        },
    }
}
