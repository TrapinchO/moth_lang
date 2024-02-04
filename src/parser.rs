use std::vec;

use crate::{error::Error, exprstmt::*, token::*};

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
        // must have -1 for EOF
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
                lines: vec![tok.loc()],
            })
        } else {
            self.advance();
            Ok(tok)
        }
    }

    /// if the current token has said type
    fn is_typ(&self, typ: &TokenType) -> bool {
        self.get_current().val.compare_variant(typ)
    }

    fn advance(&mut self) {
        self.idx += 1;
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut ls = vec![];
        while !self.is_at_end()
            && !self.is_typ(&TokenType::Eof)  // apparently needed
            && !self.is_typ(&TokenType::RBrace)
        {
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
            TokenType::While => self.parse_while(),
            TokenType::Fun => self.parse_fun(),
            TokenType::Continue => {
                self.advance();
                self.expect(&TokenType::Semicolon, "Expected a semicolon \";\"")?;
                Ok(Stmt {
                    val: StmtType::ContinueStmt,
                    start: tok.start,
                    end: tok.end,
                })
            },
            TokenType::Break => {
                self.advance();
                self.expect(&TokenType::Semicolon, "Expected a semicolon \";\"")?;
                Ok(Stmt {
                    val: StmtType::BreakStmt,
                    start: tok.start,
                    end: tok.end,
                })
            },
            TokenType::Return => {
                self.advance();
                let val = if self.is_typ(&TokenType::Semicolon) {
                    // phantom value, the location is for the return statement
                    Expr {
                        val: ExprType::Unit,
                        start: tok.start,
                        end: tok.end,
                    }
                } else {
                    self.parse_expression()?
                };
                self.expect(&TokenType::Semicolon, "Expected a semicolon \";\"")?;
                Ok(Stmt {
                    val: StmtType::ReturnStmt(val),
                    start: tok.start,
                    end: tok.end,
                })
            },
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

    fn parse_block(&mut self) -> Result<Stmt, Error> {
        let mut ls = vec![];
        let start = self
            .expect(&TokenType::LBrace, "Expected { at the beginning of the block")?
            .start;
        while !self.is_at_end()
            && !self.is_typ(&TokenType::Eof)  // apparently needed
            && !self.is_typ(&TokenType::RBrace)
        {
            ls.push(self.parse_statement()?);
        }
        let end = self
            .expect(&TokenType::RBrace, "Expected } at the end of the block")?
            .start;

        Ok(Stmt {
            val: StmtType::BlockStmt(ls),
            start,
            end,
        })
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, Error> {
        let ident = self.expect(&TokenType::Identifier("".to_string()), "Expected an identifier")?;
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

        Ok(if self.is_typ(&TokenType::Equals) {
            self.advance();
            let expr = self.parse_expression()?;
            Stmt {
                start: ident.start,
                end: expr.end,
                val: StmtType::AssignStmt(ident, expr),
            }
        } else {
            // since the identifier can be a part of an expression, it has to backtrack a little
            // bit; and since we already moved at least once, it is safe
            self.idx -= 1;
            let expr = self.parse_expression()?;
            Stmt {
                start: expr.start,
                end: expr.end,
                val: StmtType::ExprStmt(expr),
            }
        })
    }

    fn parse_if_else(&mut self) -> Result<Stmt, Error> {
        let mut blocks: Vec<(Expr, Block)> = vec![];

        let start = self.get_current().start;
        self.advance(); // move past if

        let cond = self.parse_expression()?;
        let if_block = self.parse_block()?;
        let StmtType::BlockStmt(bl) = if_block.val else {
            unreachable!();
        };
        blocks.push((cond, bl));
        let mut end = if_block.end;

        while self.is_typ(&TokenType::Else) {
            let else_kw = self.get_current().clone();
            self.advance();
            if self.is_typ(&TokenType::If) {
                self.advance();
                let cond = self.parse_expression()?;
                let if_block = self.parse_block()?;
                let StmtType::BlockStmt(bl) = if_block.val else {
                    unreachable!();
                };
                blocks.push((cond, bl));
                end = if_block.end;
            } else {
                let else_block = self.parse_block()?;
                let StmtType::BlockStmt(bl) = else_block.val else {
                    unreachable!();
                };
                blocks.push((
                    Expr {
                        val: ExprType::Bool(true),
                        start: else_kw.start,
                        end: else_kw.end,
                    },
                    bl,
                ));
                end = else_block.end;
                break;
            }
        }

        Ok(Stmt {
            start,
            end,
            val: StmtType::IfStmt(blocks),
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, Error> {
        let start = self.get_current().start;
        self.advance(); // move past while
        let cond = self.parse_expression()?;
        let block = self.parse_block()?;
        let StmtType::BlockStmt(bl) = block.val else {
            unreachable!();
        };
        Ok(Stmt {
            start,
            end: block.end,
            val: StmtType::WhileStmt(cond, bl),
        })
    }

    fn parse_fun(&mut self) -> Result<Stmt, Error> {
        let fun = self.expect(&TokenType::Fun, "unreachable")?;

        let name = self.expect(&TokenType::Identifier("".to_string()), "Expected an identifier")?;

        self.expect(&TokenType::LParen, "Expected an opening parenthesis")?;
        let mut params = vec![];
        while !self.is_at_end() {
            params.push(self.expect(&TokenType::Identifier("".to_string()), "Expected a parameter name")?);
            if self.is_typ(&TokenType::RParen) {
                let _ = self.expect(&TokenType::RParen, "")?;
                let block = self.parse_block()?;
                // TODO: horrible cheating, but eh
                let StmtType::BlockStmt(bl) = block.val else {
                    unreachable!();
                };
                return Ok(Stmt {
                    val: StmtType::FunDeclStmt(name, params, bl),
                    start: fun.start,
                    end: block.end,
                });
            }
            self.expect(&TokenType::Comma, "Expected a comma \",\" after an argument")?;
        }
        Err(Error {
            msg: "Reached EOF".to_string(), // TODO: idk yet how
            lines: vec![self.get_current().loc()],
        })
    }

    fn parse_expression(&mut self) -> Result<Expr, Error> {
        self.parse_binary()
    }

    fn parse_binary(&mut self) -> Result<Expr, Error> {
        let left = self.parse_unary()?;
        // if it is a symbol, look for nested binary operator
        if let tok @ Token {
            val: TokenType::Symbol(_),
            ..
        } = self.get_current().clone()
        {
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
        let tok @ Token { val: TokenType::Symbol(_), .. } = self.get_current().clone() else {
            return self.parse_call()
        };
        let TokenType::Symbol(sym) = &tok.val else {
            unreachable!()
        };
        if !vec!["-", "!"].contains(&sym.as_str()) {
            return Err(Error {
                msg: format!("Unknown operator: \"{}\"", sym),
                lines: vec![tok.loc()],
            })
        }
        self.advance();
        let expr = self.parse_unary()?;
        Ok(Expr {
            start: tok.start,
            end: expr.end,
            val: ExprType::UnaryOperation(tok, expr.into()),
        })
    }

    fn parse_call(&mut self) -> Result<Expr, Error> {
        let expr = self.parse_primary()?;
        if !self.is_typ(&TokenType::LParen) {
            return Ok(expr);
        }
        let lparen = self.expect(&TokenType::LParen, "")?;
        let mut args = vec![];
        if self.is_typ(&TokenType::RParen) {
            let rparen = self.expect(&TokenType::RParen, "")?;
            return Ok(Expr {
                val: ExprType::Call(expr.into(), args),
                start: lparen.start,
                end: rparen.end,
            });
        }
        while !self.is_at_end() {
            args.push(self.parse_expression()?);
            if self.is_typ(&TokenType::RParen) {
                let rparen = self.expect(&TokenType::RParen, "")?;
                return Ok(Expr {
                    val: ExprType::Call(expr.into(), args),
                    start: lparen.start,
                    end: rparen.end,
                });
            }
            self.expect(&TokenType::Comma, "Expected a comma \",\" after an argument")?;
        }
        let eof = self.get_current();
        Err(Error {
            msg: "Unexpected EOF while parsing function call".to_string(),
            lines: vec![eof.loc()],
        })
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
                // TODO: UNIT expression
                self.advance();
                let expr = self.parse_expression()?;
                // TODO: consider improving
                /*
                return Err(Error {
                    msg: "Expected closing parenthesis".to_string(),
                    lines: vec![tok.loc()]
                });
                */
                let paren = self.expect(&TokenType::RParen, "Expected closing parenthesis")?;
                return Ok(Expr {
                    val: ExprType::Parens(expr.into()),
                    start: tok.start,
                    end: paren.end,
                });
            }
            TokenType::Eof => {
                return Err(Error {
                    msg: "Expected an element but reached EOF".to_string(),
                    lines: vec![tok.loc()],
                })
            }
            _ => {
                return Err(Error {
                    msg: format!("Unknown element: {}", tok.val),
                    lines: vec![tok.loc()],
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
