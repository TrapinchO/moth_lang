use crate::lexer::{Token, TokenType};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    Number(i32),
    String(String),
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
        self.idx += 1;
        while let Some(Token {typ: TokenType::Symbol(sym), .. }) = self.tokens.get(self.idx) {
            self.idx += 1;
            let right = self.parse_primary()?;
            self.idx += 1;
            left = Expr::BinaryOperation(
                Box::new(left),
                sym.clone(),
                Box::new(right)
            )
        }
        Ok(left)
    }

    fn parse_primary(&self) -> Result<Expr, String> {
        let expr = match self.tokens.get(self.idx) {
            None => Err("Expected an element".to_string()),
            Some(tok) => Ok(match &tok.typ {
                TokenType::String(s) => Expr::String(s.to_string()),
                TokenType::Number(n) => Expr::Number(*n),
                _ => return Err(format!("Unknown element: {:?}", tok)),
            })
        };
        //self.idx += 1;
        expr
    }
}
