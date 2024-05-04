use std::{mem, vec};

use crate::{error::Error, exprstmt::*, located::Location, token::*};

macro_rules! check_variant {
    ($self:ident, $variant:ident $( ( $($pattern:pat),+ ) )?, $msg:literal) => {
        {
            let tok = $self.get_current();
            match tok.val {
                TokenType::$variant $( ( $($pattern),+ ) )? => { let tok = tok.clone(); $self.advance(); Ok(tok) },
                _ => Err(Error {
                    //msg: concat!("Expected ", stringify!($variant)).to_string(),
                    msg: $msg.to_string(),
                    lines: vec![tok.loc],
                })
            }
        }
    };
}

macro_rules! is_typ {
    ($self:ident, $variant:ident $( ( $($pattern:pat),+ ) )?) => {
        matches!($self.get_current().val, TokenType::$variant $( ( $($pattern),+ ) )?)
    };
}

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

    fn advance(&mut self) {
        self.idx += 1;
    }

    // TODO: accept beginning and separator
    // TODO: allow trailing separator
    fn sep<R>(&mut self, f: fn(&mut Self) -> Result<R, Error>, end_tok: TokenType) -> Result<Vec<R>, Error> {
        // TODO: fix hack
        // funnily enough, my new system with macros broke this one
        fn cmp(this: &TokenType, other: &TokenType) -> bool {
            mem::discriminant(this) == mem::discriminant(other)
        }
        let mut items = vec![];
        if cmp(&self.get_current().val, &end_tok) {
            return Ok(items);
        }
        while !self.is_at_end() {
            items.push(f(self)?);
            if cmp(&self.get_current().val, &end_tok) {
                return Ok(items);
            }
            check_variant!(self, Comma, "Expected a comma \",\" after an item")?;
        }
        let eof = self.get_current();
        Err(Error {
            msg: "Unexpected EOF while parsing function call".to_string(),
            lines: vec![eof.loc],
        })
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut ls = vec![];
        while !self.is_at_end()
            && !is_typ!(self, Eof)  // apparently needed
            && !is_typ!(self, RBrace)
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
                check_variant!(self, Semicolon, "Expected a semicolon \";\"")?;
                Ok(stmt)
            }
            TokenType::If => self.parse_if_else(),
            TokenType::While => self.parse_while(),
            TokenType::Fun => self.parse_fun(),
            TokenType::Continue => {
                self.advance();
                check_variant!(self, Semicolon, "Expected a semicolon \";\"")?;
                Ok(Stmt {
                    val: StmtType::ContinueStmt,
                    loc: tok.loc,
                })
            }
            TokenType::Break => {
                self.advance();
                check_variant!(self, Semicolon, "Expected a semicolon \";\"")?;
                Ok(Stmt {
                    val: StmtType::BreakStmt,
                    loc: tok.loc,
                })
            }
            TokenType::Return => {
                self.advance();
                let val = if !is_typ!(self, Semicolon) {
                    self.parse_expression()?
                } else {
                    // phantom value, the location is for the return statement
                    Expr {
                        val: ExprType::Unit,
                        loc: tok.loc,
                    }
                };
                check_variant!(self, Semicolon, "Expected a semicolon \";\"")?;
                Ok(Stmt {
                    loc: Location {
                        start: tok.loc.start,
                        end: val.loc.end,
                    },
                    val: StmtType::ReturnStmt(val),
                })
            }
            TokenType::LBrace => self.parse_block(),
            _ => {
                let expr = self.parse_expression()?;
                if !is_typ!(self, Equals) {
                    check_variant!(self, Semicolon, "Expected a semicolon \";\"")?;
                    Ok(Stmt {
                        loc: expr.loc,
                        val: StmtType::ExprStmt(expr),
                    })
                } else {
                    let loc = expr.loc;
                    match expr.val.clone() {
                        ExprType::Identifier(ident) => {
                            self.advance(); // consume the equals
                            let val = self.parse_expression()?;
                            check_variant!(self, Semicolon, "Expected a semicolon \";\"")?;
                            Ok(Stmt {
                                loc: Location {
                                    start: loc.start,
                                    end: val.loc.end,
                                },
                                val: StmtType::AssignStmt(Token { val: TokenType::Identifier(ident), loc: expr.loc }, val),
                            })
                        }
                        ExprType::Index(ls, idx) => {
                            self.advance();
                            let val = self.parse_expression()?;
                            check_variant!(self, Semicolon, "Expected a semicolon \";\"")?;
                            Ok(Stmt {
                                loc: Location {
                                    start: loc.start,
                                    end: val.loc.end,
                                },
                                val: StmtType::AssignIndexStmt(*ls, *idx, val),
                            })
                        }
                        _ => { unreachable!() },
                    }
                }
            }
        }
    }

    fn parse_block(&mut self) -> Result<Stmt, Error> {
        // maybe can be changed into get + advance?
        let start = check_variant!(self, LBrace, "Expected { at the beginning of the block")?
            .loc
            .start;

        let mut ls = vec![];
        while !self.is_at_end()
            && !is_typ!(self, Eof)  // apparently needed
            && !is_typ!(self, RBrace)
        {
            ls.push(self.parse_statement()?);
        }
        let end = check_variant!(self, RBrace, "Expected } at the end of the block")?
            .loc
            .end;

        Ok(Stmt {
            val: StmtType::BlockStmt(ls),
            loc: Location { start, end },
        })
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, Error> {
        let ident = check_variant!(self, Identifier(_), "Expected an identifier")?;
        check_variant!(self, Equals, "Expected an equals symbol")?;
        let expr = self.parse_expression()?;
        Ok(Stmt {
            loc: Location {
                start: ident.loc.start,
                end: expr.loc.end,
            },
            val: StmtType::VarDeclStmt(ident, expr),
        })
    }

    fn parse_if_else(&mut self) -> Result<Stmt, Error> {
        let start = self.get_current().loc.start;
        self.advance(); // move past if

        let mut blocks = vec![];

        let cond = self.parse_expression()?;
        let if_block = self.parse_block()?;
        let StmtType::BlockStmt(bl) = if_block.val else {
            unreachable!();
        };
        blocks.push((cond, bl));
        let mut end = if_block.loc.end;
        let mut exit = false;
        while is_typ!(self, Else) {
            let else_kw = self.get_current().clone();
            self.advance();

            let cond = if is_typ!(self, If) {
                self.advance();
                self.parse_expression()?
            } else {
                exit = true;
                Expr {
                    val: ExprType::Bool(true),
                    loc: else_kw.loc,
                }
            };
            let if_block = self.parse_block()?;
            let StmtType::BlockStmt(bl) = if_block.val else {
                unreachable!();
            };
            blocks.push((cond, bl));
            end = if_block.loc.end;
            if exit {
                break;
            }
        }

        Ok(Stmt {
            val: StmtType::IfStmt(blocks),
            loc: Location { start, end },
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, Error> {
        let start = self.get_current().loc.start;
        self.advance(); // move past while
        let cond = self.parse_expression()?;
        let block = self.parse_block()?;
        let StmtType::BlockStmt(bl) = block.val else {
            unreachable!();
        };
        Ok(Stmt {
            val: StmtType::WhileStmt(cond, bl),
            loc: Location {
                start,
                end: block.loc.end,
            },
        })
    }

    fn parse_fun(&mut self) -> Result<Stmt, Error> {
        let start = self.get_current().loc.start;
        self.advance();

        let ident = check_variant!(self, Identifier(_), "Expected an identifier")?;

        check_variant!(self, LParen, "Expected an opening parenthesis")?;

        // TODO: monstrosity, but checks out
        if is_typ!(self, RParen) {
            self.advance();
            let block = self.parse_block()?;
            // TODO: horrible cheating, but eh
            let StmtType::BlockStmt(bl) = block.val else {
                unreachable!();
            };
            return Ok(Stmt {
                val: StmtType::FunDeclStmt(ident, vec![], bl),
                loc: Location { start, end: block.loc.end },
            })
        }
        let mut params = vec![];
        while !self.is_at_end() {
            params.push(check_variant!(self, Identifier(_), "Expected a parameter name")?);
            // TODO: turn the condition around
            if is_typ!(self, RParen) {
                self.advance();
                //check_variant!(self, RParen, "");
                let block = self.parse_block()?;
                // TODO: horrible cheating, but eh
                let StmtType::BlockStmt(bl) = block.val else {
                    unreachable!();
                };
                return Ok(Stmt {
                    val: StmtType::FunDeclStmt(ident, params, bl),
                    loc: Location {
                        start,
                        end: block.loc.end,
                    },
                });
            }
            check_variant!(self, Comma, "Expected a comma \",\" after an argument")?;
        }
        Err(Error {
            msg: "Reached EOF".to_string(), // TODO: idk yet how
            lines: vec![self.get_current().loc],
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
                loc: Location {
                    start: left.loc.start,
                    end: right.loc.end,
                },
                val: ExprType::BinaryOperation(left.into(), tok, right.into()),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, Error> {
        let tok @ Token { val: TokenType::Symbol(_), .. } = self.get_current().clone() else {
            return self.parse_suffix()
        };
        let TokenType::Symbol(sym) = &tok.val else {
            unreachable!()
        };
        if !["-", "!"].contains(&sym.as_str()) {
            return Err(Error {
                msg: format!("Unknown operator: \"{sym}\""),
                lines: vec![tok.loc],
            });
        }
        self.advance();
        let expr = self.parse_unary()?;
        Ok(Expr {
            loc: Location {
                start: tok.loc.start,
                end: expr.loc.end,
            },
            val: ExprType::UnaryOperation(tok, expr.into()),
        })
    }

    fn parse_suffix(&mut self) -> Result<Expr, Error> {
        let mut expr = self.parse_primary()?;
        // a condition is not actually needed
        // the parser should figure out by itself
        loop {
            match self.get_current().val {
                TokenType::LParen => {
            self.advance();  // move past the paren
            let start = expr.loc.start;
            let args = self.sep(Parser::parse_expression, TokenType::RParen)?;
            let end = check_variant!(self, RParen, "")?.loc.end;
            expr = Expr {
                loc: Location { start, end },
                val: ExprType::Call(expr.into(), args),
            };
                },
                TokenType::LBracket => {

            self.advance();  // move past the bracket
            let start = expr.loc.start;
            let idx = self.parse_expression()?;
            let end = check_variant!(self, RBracket, "Expected closing bracket.")?.loc.end;
            expr = Expr {
                loc: Location { start, end },
                val: ExprType::Index(expr.into(), idx.into()),
            };
                },
                _ => { break; }
            }
        };
        Ok(expr)
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
                let val = if is_typ!(self, RParen) {
                    ExprType::Unit
                } else {
                    ExprType::Parens(self.parse_expression()?.into())
                };
                let end = check_variant!(self, RParen, "Expected closing parenthesis")?.loc.end;
                return Ok(Expr {
                    val,
                    loc: Location {
                        start: tok.loc.start,
                        end,
                    },
                });
            }
            TokenType::LBracket => {
                let start = tok.loc.start;
                self.advance();
                let items = self.sep(Parser::parse_expression, TokenType::RBracket)?;
                let end = check_variant!(self, RBracket, "")?.loc.end;
                return Ok(Expr {
                    loc: Location { start, end },
                    val: ExprType::List(items),
                });
            }
            TokenType::Eof => {
                return Err(Error {
                    msg: "Expected an element but reached EOF".to_string(),
                    lines: vec![tok.loc],
                })
            }
            _ => {
                return Err(Error {
                    msg: format!("Unknown element: {}", tok.val),
                    lines: vec![tok.loc],
                })
            }
        };
        Ok(Expr {
            val: expr,
            loc: tok.loc,
        })
    }
}
