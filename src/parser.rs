use crate::error::Error;
use crate::lexer::{Token, TokenType};
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExprType {
    Number(i32),
    String(String),
    Identifier(String),
    Parens(Rc<Expr>),
    UnaryOperation(Token, Rc<Expr>),
    BinaryOperation(Rc<Expr>, Token, Rc<Expr>),
}

impl ExprType {
    fn format(&self) -> String {
        match self {
            Self::Number(n) => n.to_string(),
            Self::String(s) => format!("\"{}\"", s),
            Self::Identifier(ident) => ident.to_string(),
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
// TODO: consider moving this to the bottom again, as rust seems to be
// affected by position of the arguments, at least regarding the borrow checker
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Expr {
    pub typ: ExprType,
    pub start: usize,
    pub end: usize,
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.typ.format())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StmtType {
    ExprStmt(Expr),
    AssingmentStmt(Token, Expr),
}
impl StmtType {
    fn format(&self) -> String {
        match self {
            Self::ExprStmt(expr) => expr.to_string(),
            Self::AssingmentStmt(ident, expr) => format!("let {} = {}", ident, expr)
        }
    }
}
impl Display for StmtType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Stmt {
    pub typ: StmtType,
    pub start: usize,
    pub end: usize,
}
impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.typ)
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, Error> {
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
            panic!(
                "Attempted to index token out ouf bounds: {} (length {})",
                self.idx,
                self.tokens.len()
            );
        }
        &self.tokens[self.idx]
    }

    fn expect(&mut self, typ: &TokenType, msg: &str) -> Result<Token, Error> {
        let tok = self.get_current().clone();
        if !tok.typ.compare_variant(typ) {
            Err(Error {
                msg: msg.to_string(),
                lines: vec![(tok.start, tok.end)]
            })
        } else {
            self.advance();
            Ok(tok)
        }
    }

    fn advance(&mut self) {
        self.idx += 1;
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Error> {
        self.parse_block()
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut ls = vec![];
        while !self.is_at_end() && !self.get_current().typ.compare_variant(&TokenType::Eof) {
            ls.push(self.parse_statement()?);
            self.expect(&TokenType::Semicolon, "Expected a semicolon \";\"")?;
        }

        Ok(ls)
    }

    fn parse_statement(&mut self) -> Result<Stmt, Error> {
        let tok = self.get_current().clone();
        match tok.typ {
            TokenType::Let => {
                self.advance();
                let (ident, expr) = self.parse_assignment()?;
                Ok(Stmt {
                    start: tok.start,
                    end: expr.end,
                    typ: StmtType::AssingmentStmt(ident, expr),
                })
            },
            _ => {
                let expr = self.parse_expression()?;
                Ok(Stmt {
                    start: expr.start,
                    end: expr.end,
                    typ: StmtType::ExprStmt(expr),
                })
            },
        }
    }

    fn parse_assignment(&mut self) -> Result<(Token, Expr), Error> {
        let ident = self.expect(&TokenType::Identifier("".to_string()), "Expected an identifier")?;
        self.expect(&TokenType::Equals, "Expected an equals symbol")?;
        Ok((ident, self.parse_expression()?))
    }

    fn parse_expression(&mut self) -> Result<Expr, Error> {
        self.parse_binary()
    }

    fn parse_binary(&mut self) -> Result<Expr, Error> {
        let left = self.parse_unary()?;
        if let tok @ Token {typ: TokenType::Symbol(_), .. } = self.get_current().clone() {
            self.advance();

            let right = self.parse_binary()?;
            return Ok(Expr {
                start: left.start,
                end: right.end,
                typ: ExprType::BinaryOperation(left.into(), tok, right.into()),
            });
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, Error> {
        if let tok @ Token {typ: TokenType::Symbol(_), .. } = self.get_current().clone() {
            self.advance();

            let expr = self.parse_unary()?;
            Ok(Expr {
                start: tok.start,
                end: expr.end,
                typ: ExprType::UnaryOperation(tok, expr.into()),
            })
        } else {
            Ok(self.parse_primary()?)
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, Error> {
        let tok = self.get_current().clone();
        let expr = match &tok.typ {
            TokenType::String(s) => {
                let expr = ExprType::String(s.to_string());
                self.advance();
                expr
            },
            TokenType::Number(n) => {
                let expr = ExprType::Number(*n);
                self.advance();
                expr
            },
            TokenType::Identifier(ident) => {
                let expr = ExprType::Identifier(ident.to_string());
                self.advance();
                expr
            },
            TokenType::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&TokenType::RParen, "Expected closing parenthesis")?;
                ExprType::Parens(expr.into())
            },
            TokenType::Eof => {
                return Err(Error {
                    msg: "Expected an element".to_string(),
                    lines: vec![(tok.start, tok.end)],
                })
            },
            _ => {
                return Err(Error {
                    msg: format!("Unknown element: {}", tok),
                    lines: vec![(tok.start, tok.end)],
                })
            }
        };
        Ok(Expr {
            start: tok.start,
            end: tok.end,
            typ: expr,
        })
    }
}
