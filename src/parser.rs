use crate::error::Error;
use crate::exprstmt::*;
use crate::token::*;

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, Error> {
    if tokens.is_empty() || tokens.len() == 1 && tokens[0].val == TokenType::Eof {
        return Ok(vec![]);
    }
    Parser::new(tokens).parse()
}

struct Parser {
    tokens: Vec<Token>,
    idx: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        if tokens.is_empty() || tokens.len() == 1 && tokens[0].val == TokenType::Eof {
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
        if !tok.val.compare_variant(typ) {
            Err(Error {
                msg: msg.to_string(),
                lines: vec![(tok.start, tok.end)],
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
        while !self.is_at_end()
            && !self.get_current().val.compare_variant(&TokenType::Eof)  // TODO: apparently needed
            && !self.get_current().val.compare_variant(&TokenType::RBrace) {
            ls.push(self.parse_statement()?);
        }

        Ok(ls)
    }

    fn parse_statement(&mut self) -> Result<Stmt, Error> {
        let tok = self.get_current().clone();
        match tok.val {
            TokenType::Let => {
                self.advance();
                let stmt = self.parse_var_decl()?;
                self.expect(&TokenType::Semicolon, "Expected a semicolon \";\"")?;
                Ok(stmt)
            }
            TokenType::Identifier(_) => {
                let stmt = self.parse_assign()?;
                self.expect(&TokenType::Semicolon, "Expected a semicolon \";\"")?;
                Ok(stmt)
            }
            TokenType::If => self.parse_if_else(),
            _ => {
                let expr = self.parse_expression()?;
                let stmt = Stmt {
                    start: expr.start,
                    end: expr.end,
                    val: StmtType::ExprStmt(expr),
                };
                self.expect(&TokenType::Semicolon, "Expected a semicolon \";\"")?;
                Ok(stmt)
            }
        }
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, Error> {
        let ident = self.expect(
            &TokenType::Identifier("".to_string()),
            "Expected an identifier",
        )?;
        self.expect(&TokenType::Equals, "Expected an equals symbol")?;
        let expr = self.parse_expression()?;
        Ok(Stmt {
            start: ident.start,
            end: expr.end,
            val: StmtType::VarDeclStmt(ident, expr),
        })
    }

    fn parse_assign(&mut self) -> Result<Stmt, Error> {
        let ident = self.get_current().clone();
        self.advance();

        Ok(if self.get_current().val.compare_variant(&TokenType::Equals) {
            self.advance();
            let expr = self.parse_expression()?;
            Stmt { start: ident.start, end: expr.end, val: StmtType::AssignStmt(ident, expr) }
        } else {
            // since the identifier can be a part of an expression, it has to backtrack a little
            // bit; and since we already moved at least once, it is safe
            self.idx -= 1;
            let expr = self.parse_expression()?;
            Stmt { start: expr.start, end: expr.end, val: StmtType::ExprStmt(expr) }
        })
    }

    fn parse_if_else(&mut self) -> Result<Stmt, Error> {
        let start = self.get_current().start;
        self.advance();
        let cond = self.parse_expression()?;
        self.expect(&TokenType::LBrace, "Expeted { after condition")?;
        let if_block = self.parse_block()?;
        self.expect(&TokenType::RBrace, "Expeted } at the end of the block")?;
        let else_block = if self.expect(&TokenType::Else, "").is_ok() {
            self.expect(&TokenType::LBrace, "Expeted { after else")?;
            let else_block = self.parse_block()?;
            self.expect(&TokenType::RBrace, "Expeted } at the end of the block")?;
            Some(else_block)
        } else {
            None
        };

        Ok(Stmt {
            val: StmtType::IfStmt(cond, if_block, else_block),
            start,
            end: 0,
        })
    }

    fn parse_expression(&mut self) -> Result<Expr, Error> {
        self.parse_binary()
    }

    fn parse_binary(&mut self) -> Result<Expr, Error> {
        let left = self.parse_unary()?;
        // if it is a symbol, look for nested binary operator
        if let tok @ Token {val: TokenType::Symbol(_), .. } = self.get_current().clone() {
            self.advance();

            let right = self.parse_binary()?;
            Ok(Expr {
                start: left.start,
                end: right.end,
                val: ExprType::BinaryOperation(left.into(), tok, right.into()),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, Error> {
        // if it is a symbol, look for nested unary operator
        if let tok @ Token {val: TokenType::Symbol(_), .. } = self.get_current().clone() {
            self.advance();

            let expr = self.parse_unary()?;
            Ok(Expr {
                start: tok.start,
                end: expr.end,
                val: ExprType::UnaryOperation(tok, expr.into()),
            })
        } else {
            Ok(self.parse_primary()?)
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, Error> {
        let tok = self.get_current().clone();
        let expr = match &tok.val {
            TokenType::String(s) => {
                let expr = ExprType::String(s.to_string());
                self.advance();
                expr
            }
            TokenType::Int(n) => {
                let expr = ExprType::Int(*n);
                self.advance();
                expr
            }
            TokenType::Float(n) => {
                let expr = ExprType::Float(*n);
                self.advance();
                expr
            }
            TokenType::Identifier(ident) => {
                let expr = ExprType::Identifier(ident.to_string());
                self.advance();
                expr
            }
            TokenType::True => {
                let expr = ExprType::Bool(true);
                self.advance();
                expr
            }
            TokenType::False => {
                let expr = ExprType::Bool(false);
                self.advance();
                expr
            }
            TokenType::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                let paren = self.expect(&TokenType::RParen, "Expected closing parenthesis")?;
                return Ok(Expr {
                    val: ExprType::Parens(expr.into()),
                    start: tok.start,
                    end: paren.end,
                })
            }
            TokenType::Eof => {
                return Err(Error {
                    msg: "Expected an element but reached EOF".to_string(),
                    lines: vec![(tok.start, tok.end)],
                })
            }
            _ => {
                return Err(Error {
                    msg: format!("Unknown element: {}", tok.val),
                    lines: vec![(tok.start, tok.end)],
                })
            }
        };
        Ok(Expr {
            start: tok.start,
            end: tok.end,
            val: expr,
        })
    }
}
