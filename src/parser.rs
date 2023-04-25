use crate::lexer::{Token, TokenType};
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


pub fn parse(tokens: Vec<Token>) -> Result<Expr, String> {
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
    
    pub fn parse(&mut self) -> Result<Expr, String> {
        self.parse_binary()
    }

    fn parse_binary(&mut self) -> Result<Expr, String> {
        let left = self.parse_primary()?;
        if let Some(Token {typ: TokenType::Symbol(sym), .. }) = &self.tokens.get(self.idx) {
            self.idx += 1;
            return Ok(Expr::BinaryOperation(left.into(), sym.clone(), self.parse_binary()?.into()))
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let expr = match self.tokens.get(self.idx) {
            None => Err("Expected an element".to_string()),
            Some(tok) => Ok(match &tok.typ {
                TokenType::String(s) => Expr::String(s.to_string()),
                TokenType::Number(n) => Expr::Number(*n),
                TokenType::LParen => {
                    self.idx += 1;
                    let expr = self.parse()?;
                    match self.tokens.get(self.idx) {
                        Some(&Token { typ: TokenType::RParen, .. }) => Expr::ParensExpr(expr.into()),
                        _ => return Err("Missing closing parenthesis".to_string()),
                    }
                }
                _ => return Err(format!("Unknown element: {:?}", tok)),
            })
        };
        self.idx += 1;
        expr
    }
}


pub fn reassoc(expr: &Expr) -> Expr {
    //println!("rrr {:?}", expr);
    match expr {
        Expr::BinaryOperation(left, op, right) => reassoc_(&reassoc(&left.clone()), &op, &reassoc(&right.clone())),
        Expr::ParensExpr(expr) => Expr::ParensExpr(reassoc(&expr).into()),
        expr => expr.clone(),
    }
}

// https://stackoverflow.com/a/67992584
fn reassoc_(left: &Expr, op: &String, right: &Expr) -> Expr {
    //println!("__ {:?} {:?} {:?}", &left, &op, &right);
    // left = false, right = true
    let prec_table: HashMap<&str, (usize, bool)> = [
        ("+", (1, false)),
        ("-", (1, false)),
        ("*", (2, false)),
        ("/", (2, false)),
    ].iter().cloned().collect();

    match right {
        Expr::BinaryOperation(left2, op2, right2) => {
            let (prec, assoc) = prec_table.get(op.as_str()).unwrap();
            let (prec2, assoc2) = prec_table.get(op2.as_str()).unwrap();
            //println!(" {} {} {} | {} {} {}", op, prec, assoc, op2, prec2, assoc2);
            match prec.cmp(prec2) {
                std::cmp::Ordering::Greater => {
                    Expr::BinaryOperation(
                        Rc::new(reassoc_(&left, &op, &left2)),
                        op2.clone(),
                        right2.clone())
                }
                std::cmp::Ordering::Less => {
                    Expr::BinaryOperation(left.clone().into(), op.clone(), right.clone().into())
                }
                std::cmp::Ordering::Equal => {
                    match (assoc, assoc2) {
                        (true, true) => Expr::BinaryOperation(
                            reassoc_(left, &op2, &left2).into(),
                            op.clone(),
                            right2.clone()),
                        (false, false) => Expr::BinaryOperation(
                            left.clone().into(),
                            op.clone(),
                            right.clone().into()
                        ),
                        _ => panic!("wrong associativity"),
                    }
                }
            }
        }
        _ => Expr::BinaryOperation(left.clone().into(), op.clone(),right.clone().into()),
    }
}
