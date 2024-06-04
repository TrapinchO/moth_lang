use std::{mem, vec};

use crate::{
    associativity::{Associativity, Precedence},
    error::{Error, ErrorType},
    exprstmt::*,
    located::Location,
    token::*,
};

macro_rules! check_variant {
    ($self:ident, $variant:ident $( ( $($pattern:pat),+ ) )?, $msg:literal) => {
        {
            let tok = $self.get_current();
            match tok.val {
                TokenType::$variant $( ( $($pattern),+ ) )? => { let tok = tok.clone(); $self.advance(); Ok(tok) },
                _ => Err(Error {
                    //msg: concat!("Expected ", stringify!($variant)).to_string(),
                    msg: ErrorType::ExpectedToken($msg.to_string()),
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

    /// parses an array of items surrounded by opening and closing tokens and delimited by a comma
    /// supports trailing comma
    /// 
    /// "()" // ok
    /// "(a)" // ok
    /// "(a, b)" // ok
    /// "(a, )" // ok
    /// "(a a)" // error
    /// "(,)" // error
    ///
    /// "[]" // ok
    /// "[1+1]" // ok
    fn sep<R>(&mut self, start_tok: TokenType, end_tok: TokenType, f: fn(&mut Self) -> Result<R, Error>) -> Result<(Vec<R>, Location), Error> {
        // TODO: fix hack
        // funnily enough, my new system with macros broke this one
        fn cmp(this: &TokenType, other: &TokenType) -> bool {
            mem::discriminant(this) == mem::discriminant(other)
        }

        // move past starting token
        if !cmp(&self.get_current().val, &start_tok) {
            return Err(Error {
                msg: ErrorType::ExpectedOpeningToken(start_tok),
                lines: vec![self.get_current().loc],
            });
        }
        let start = self.get_current().loc.start;
        self.advance();

        // no items inbetween
        if cmp(&self.get_current().val, &end_tok) {
            let end = self.get_current().loc.end;
            self.advance();
            return Ok((vec![], Location { start, end }));
        }

        let mut items = vec![];
        loop {
            items.push(f(self)?);
            //self.advance();
            if !is_typ!(self, Comma) {
                break;
            }
            self.advance();
            if cmp(&self.get_current().val, &end_tok) {
                break;
            }
        }

        let cur = self.get_current();
        if !cmp(&cur.val, &end_tok) {
            return Err(Error {
                msg: ErrorType::ExpectedClosingToken(end_tok),
                lines: vec![self.get_current().loc],
            });
        }
        let end = cur.loc.end;
        self.advance(); // move past ending token
        Ok((items, Location { start, end }))
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut ls = vec![];
        while !self.is_at_end()
            && !is_typ!(self, Eof)  // apparently needed
        {
            ls.push(self.parse_statement()?);
        }

        Ok(ls)
    }

    /// used to avoid a lambda in function and operator declarations
    fn parse_param(&mut self) -> Result<Identifier, Error> {
        let cur = self.get_current().clone();
        let Token { val: TokenType::Identifier(name), loc } = cur else {
            return Err(Error {
                msg: ErrorType::ExpectedParameterName,
                lines: vec![cur.loc],
            });
        };
        self.advance();
        Ok(Identifier { val: name, loc })
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
            TokenType::Fun => self.parse_fun(false),
            TokenType::Infixl | TokenType::Infixr => self.parse_operator(),
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
            _ => self.parse_assignment(),
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
        let TokenType::Identifier(name) = ident.val else {
            unreachable!()
        };
        check_variant!(self, Equals, "Expected an equals symbol")?;
        let expr = self.parse_expression()?;
        Ok(Stmt {
            loc: Location {
                start: ident.loc.start,
                end: expr.loc.end,
            },
            val: StmtType::VarDeclStmt(
                Identifier { val: name, loc: ident.loc },
                expr),
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
    fn parse_fun(&mut self, force_operator: bool) -> Result<Stmt, Error> {
        let start = self.get_current().loc.start;
        self.advance(); // move past keyword

        let tok = self.get_current().clone();
        let (op, name) = match tok.val {
            TokenType::Identifier(name) => {
                if force_operator {
                    return Err(Error {
                        msg: ErrorType::InvalidOperatorname,
                        lines: vec![tok.loc]
                    })
                }
                (false, name)
            },
            TokenType::Symbol(name) => (true, name),
            _ => {
                return Err(Error {
                    msg: ErrorType::InvalidFunctionName,
                    lines: vec![tok.loc],
                })
            }
        };
        self.advance();

        let (params, _) = self.sep(
            TokenType::LParen, TokenType::RParen,
            Parser::parse_param,
        )?;
        let block = self.parse_block()?;
        // TODO: horrible cheating, but eh
        let StmtType::BlockStmt(bl) = block.val else {
            unreachable!();
        };
        if !op {
            Ok(Stmt {
                val: StmtType::FunDeclStmt(
                    Identifier { val: name.to_string(), loc: tok.loc },
                    params, bl
                ),
                loc: Location { start, end: block.loc.end }
            })
        } else {
            let [param1, param2] = &*params else {
                return Err(Error {
                    msg: ErrorType::IncorrectOperatorParameterCount(params.len()),
                    lines: vec![tok.loc],
                });
            };
            Ok(Stmt {
                val: StmtType::OperatorDeclStmt(
                    Symbol { val: name.to_string(), loc: tok.loc },
                    (param1.clone(), param2.clone()), bl,
                    Precedence { prec: 0, assoc: Associativity::Left }
                ),
                loc: Location { start, end: block.loc.end }
            })
            
        }
    }

    fn parse_operator(&mut self) -> Result<Stmt, Error> {
        let kw = self.get_current().clone();
        let assoc = match kw.val {
            TokenType::Infixr => Associativity::Right,
            TokenType::Infixl => Associativity::Left,
            _ => unreachable!(),
        };
        self.advance();
        // TODO: better matching for errors
        let prec = self.get_current().clone();
        let prec2 = match prec.val {
            TokenType::Int(n @ 0..=10) => { n },
            TokenType::Int(n) => return Err(Error {
                msg: ErrorType::PrecedenceOutOfRange(n),
                lines: vec![prec.loc],
            }),
            _ => return Err(Error {
                msg: ErrorType::InvalidPrecedence,
                lines: vec![prec.loc],
            })
        };
        self.advance();
        // because we set the flag we know it WILL be an operator
        // basically all we need to do is replace the associativity and starting location
        let Stmt { val: StmtType::OperatorDeclStmt(name, params, block, _), loc } = self.parse_fun(true)? else { unreachable!() };
        Ok(Stmt {
            val: StmtType::OperatorDeclStmt(
                    name, params, block,
                    Precedence { prec: prec2 as u8, assoc }),
            loc: Location { start: kw.loc.start, end: loc.end }
        })
    }

    fn parse_assignment(&mut self) -> Result<Stmt, Error> {
        let expr = self.parse_expression()?;
        // just an expression
        if !is_typ!(self, Equals) {
            check_variant!(self, Semicolon, "Expected a semicolon \";\"")?;
            return Ok(Stmt {
                loc: expr.loc,
                val: StmtType::ExprStmt(expr),
            });
        }
        // parse the rest
        self.advance();
        let val = self.parse_expression()?;
        check_variant!(self, Semicolon, "Expected a semicolon \";\"")?;
        let loc = Location {
            start: expr.loc.start,
            end: val.loc.end,
        };
        // return to the left side (aka check it)
        // TODO: this will stop being a problem after the parse is able to report multiple errors
        match expr.val {
            ExprType::Identifier(ident) => Ok(Stmt {
                val: StmtType::AssignStmt(
                    Identifier { val: ident, loc: expr.loc },
                    val
                ),
                loc,
            }),
            ExprType::Index(ls, idx) => Ok(Stmt {
                val: StmtType::AssignIndexStmt(*ls, *idx, val),
                loc,
            }),
            _ => Err(Error {
                msg: ErrorType::InvalidAssignmentTarget,
                lines: vec![expr.loc],
            }),
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, Error> {
        self.parse_binary()
    }

    fn parse_binary(&mut self) -> Result<Expr, Error> {
        let left = self.parse_unary()?;
        // if it is a symbol, look for nested binary operator
        if let Token { val: TokenType::Symbol(sym_name), loc } = self.get_current().clone() {
            self.advance();

            let right = self.parse_binary()?;
            Ok(Expr {
                loc: Location {
                    start: left.loc.start,
                    end: right.loc.end,
                },
                val: ExprType::BinaryOperation(
                    left.into(),
                    Symbol { val: sym_name, loc },
                    right.into()
                ),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, Error> {
        let Token { val: TokenType::Symbol(sym), loc } = self.get_current().clone() else {
            return self.parse_suffix()
        };
        if sym.as_str() == "||" {
            return self.parse_lambda()
        }
        else if !["-", "!"].contains(&sym.as_str()) {
            return Err(Error {
                msg: ErrorType::UnknownUnaryOperator,
                lines: vec![loc],
            });
        }
        self.advance();
        let expr = self.parse_unary()?;
        Ok(Expr {
            loc: Location {
                start: loc.start,
                end: expr.loc.end,
            },
            val: ExprType::UnaryOperation(
                Symbol { val: sym, loc },
                expr.into()),
        })
    }

    /// things like function call and indexing
    fn parse_suffix(&mut self) -> Result<Expr, Error> {
        let mut expr = self.parse_primary()?;
        let start = expr.loc.start;
        // a condition is not actually needed
        // the parser should figure out by itself
        loop {
            match self.get_current().val {
                TokenType::LParen => {
                    let (args, loc) = self.sep(
                        TokenType::LParen, TokenType::RParen,
                        Parser::parse_expression
                    )?;
                    expr = Expr {
                        loc: Location { start, end: loc.end },
                        val: ExprType::Call(expr.into(), args),
                    };
                }
                TokenType::LBracket => {
                    self.advance(); // move past the bracket
                    let idx = self.parse_expression()?;
                    let end = check_variant!(self, RBracket, "Expected closing bracket.")?.loc.end;
                    expr = Expr {
                        loc: Location { start, end },
                        val: ExprType::Index(expr.into(), idx.into()),
                    };
                }
                _ => {
                    break;
                }
            }
        }
        Ok(expr)
    }

    /// notes:
    /// no-parameter lambda is in unary because it catches a symbol
    fn parse_lambda(&mut self) -> Result<Expr, Error> {
        let tok = self.get_current().clone();
        let params = match &tok.val {
            TokenType::Pipe => {
                self.sep(
                    TokenType::Pipe, TokenType::Pipe,
                    Parser::parse_param
                )?.0
            },
            TokenType::Symbol(s) => {
                if s.as_str() == "||" { self.advance(); vec![] }
                else {
                    return Err(Error {
                        lines: vec![tok.loc],
                        msg: ErrorType::UnknownElement(tok.val),
                    });
                }
            },
            _ => unreachable!(), // it is called from two places and we know the possible tokens
        };
        // can be either a block or a single body
        let (body, end_loc) = if is_typ!(self, LBrace) {
            let block = self.parse_block()?;
            let StmtType::BlockStmt(bl) = block.val else { unreachable!() };
            (bl, block.loc.end)
        } else {
            let expr = self.parse_expression()?;
            let loc = expr.loc.end;
            // same as { return expr; }
            (vec![Stmt {
                loc: expr.loc,
                val: StmtType::ReturnStmt(expr),
            }], loc)
        };
        Ok(Expr {
            val: ExprType::Lambda(params, body),
            loc: Location { start: tok.loc.start, end: end_loc }
        })
    }
    fn parse_primary(&mut self) -> Result<Expr, Error> {
        let tok = self.get_current().clone();
        let expr = match &tok.val {
            TokenType::String(s) => {
                self.advance();
                ExprType::String(s.to_string())
            }
            TokenType::Int(n) => {
                self.advance();
                ExprType::Int(*n)
            }
            TokenType::Float(n) => {
                self.advance();
                ExprType::Float(*n)
            }
            TokenType::Identifier(ident) => {
                self.advance();
                ExprType::Identifier(ident.to_string())
            }
            TokenType::True => {
                self.advance();
                ExprType::Bool(true)
            }
            TokenType::False => {
                self.advance();
                ExprType::Bool(false)
            }
            TokenType::LParen => {
                self.advance();
                let next = self.get_current().clone();
                let val = match next.val {
                    TokenType::RParen => {
                        ExprType::Unit
                    },
                    TokenType::Symbol(sym_name) => {
                        self.advance();
                        // either a symbol reference or unary operation
                        if is_typ!(self, RParen) {
                            // TODO: location includes the parenthesis too, not sure if I like it
                            ExprType::Identifier(sym_name)
                        } else {
                            let expr = self.parse_unary()?;
                            ExprType::Parens(
                                Expr {
                                    loc: Location { start: next.loc.start, end: expr.loc.end },
                                    val: ExprType::UnaryOperation(
                                        Symbol { val: sym_name, loc: next.loc },
                                        expr.into()),
                                }.into()
                            )
                        }
                    },
                    _ => {
                        ExprType::Parens(self.parse_expression()?.into())
                    },
                };
                let end = check_variant!(self, RParen, "Expected a closing parenthesis")?.loc.end;
                return Ok(Expr {
                    val,
                    loc: Location {
                        start: tok.loc.start,
                        end,
                    },
                });
            }
            TokenType::LBracket => {
                let (items, loc) = self.sep(
                    TokenType::LBracket, TokenType::RBracket,
                    Parser::parse_expression
                )?;
                return Ok(Expr {
                    loc,
                    val: ExprType::List(items),
                });
            }
            TokenType::Eof => {
                return Err(Error {
                    msg: ErrorType::UnexpectedEof,
                    lines: vec![tok.loc],
                })
            }
            TokenType::Pipe => {
                return self.parse_lambda();
            }
            _ => {
                return Err(Error {
                    msg: ErrorType::UnknownElement(tok.val),
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
