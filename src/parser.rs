use crate::lexer::{Token, TokenType};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    Number(i32),
    String(String),
    ParensExpr(Box<Expr>),
    UnaryOperation(String, Box<Expr>),
    BinaryOperation(Box<Expr>, String, Box<Expr>),
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
        self.parse_expr()
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        println!("{:?}", self.tokens);
        let mut left = self.parse_primary()?;
        //self.idx += 1;
        while let Some(Token {typ: TokenType::Symbol(sym), .. }) = &self.tokens.get(self.idx) {
            let sym = sym.clone();
            self.idx += 1;
            let right = self.parse_primary()?;
            //self.idx += 1;
            left = Expr::BinaryOperation(
                Box::new(left),
                sym,
                Box::new(right)
            );
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let expr = match self.tokens.get(self.idx) {
            None => Err("Expected an element".to_string()),
            Some(tok) => Ok(match &tok.typ {
                TokenType::String(s) => Expr::String(s.to_string()),
                TokenType::Number(n) => Expr::Number(*n),
                _ => return Err(format!("Unknown element: {:?}", tok)),
            })
        };
        self.idx += 1;
        expr
    }
}

pub fn reassoc(expr: Expr) -> Expr {
    println!("rrr {:?}", expr);
    match expr {
        Expr::BinaryOperation(left, op, right) => reassoc_(reassoc(*left), op, reassoc(*right)),
        Expr::ParensExpr(expr) => Expr::ParensExpr(Box::new(reassoc(*expr))),
        expr => expr,
    }
}

fn reassoc_(left: Expr, op: String, right: Expr) -> Expr {
    println!("__ {:?} {:?} {:?}", &left, &op, &right);
    // left = false, right = true
    let prec_table: HashMap<&str, (usize, bool)> = [
        ("+", (1, true)),
        ("-", (1, false)),
        ("*", (2, false)),
    ].iter().cloned().collect();

    match &right {
        Expr::BinaryOperation(left2, op2, right2) => {
            let (prec, assoc) = prec_table.get(op.as_str()).unwrap();
            let (prec2, assoc2) = prec_table.get(op2.as_str()).unwrap();
            println!("{} {} | {} {}", prec, assoc, prec2, assoc2);
            match prec.cmp(prec2) {
                std::cmp::Ordering::Greater => {
                    Expr::BinaryOperation(
                        Box::new(reassoc_(left, op2.clone(), *left2.clone())),
                        op.clone(),
                        right2.clone())
                }
                std::cmp::Ordering::Less => {
                    Expr::BinaryOperation(Box::new(left), op, Box::new(right))
                }
                std::cmp::Ordering::Equal => {
                    match (assoc, assoc2) {
                        (true, true) => Expr::BinaryOperation(
                            Box::new(reassoc_(left, op2.clone(), *left2.clone())),
                            op.clone(),
                            right2.clone()),
                        (false, false) => Expr::BinaryOperation(Box::new(left), op, Box::new(right)),
                        _ => panic!("wrong associativity"),
                    }
                }
            }
        }
        _ => Expr::BinaryOperation(Box::new(left), op, Box::new(right)),
    }
}
