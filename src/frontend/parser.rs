use std::mem;

use super::token::{Token, TokenType};
use crate::{
    associativity::{Associativity, Precedence},
    error::{Error, ErrorType},
    exprstmt::*,
    located::{Located, Location},
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

pub fn parse(tokens: Vec<Token>) -> Result<Vec<LStmt>, Vec<Error>> {
    if tokens.is_empty() || tokens.len() == 1 && tokens[0].val == TokenType::Eof {
        return Ok(vec![]);
    }
    Parser::new(tokens).parse()
}

struct Parser {
    tokens: Vec<Token>,
    idx: usize,
    errs: Vec<Error>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        assert!(
            !(tokens.is_empty() || tokens.len() == 1 && tokens[0].val == TokenType::Eof),
            "Expected code to parse"
        );
        Self {
            tokens,
            idx: 0,
            errs: vec![],
        }
    }

    fn is_at_end(&self) -> bool {
        // must have -1 for EOF
        self.idx >= self.tokens.len() // last token should be EOF
    }

    fn get_current(&self) -> &Token {
        assert!(
            !self.is_at_end(),
            "Attempted to index token out ouf bounds: {} (length {})",
            self.idx,
            self.tokens.len()
        );

        &self.tokens[self.idx]
    }

    fn peek(&self, n: usize) -> Option<&Token> {
        if self.idx + n >= self.tokens.len() {
            None
        } else {
            Some(&self.tokens[self.idx + n])
        }
    }

    fn advance(&mut self) {
        self.idx += 1;
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() && !is_typ!(self, Eof) {
            if matches!(self.get_current().val, TokenType::Semicolon | TokenType::RBrace) {
                self.advance();
                return;
            }
            self.advance();
        }
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
    fn sep<R>(
        &mut self,
        start_tok: TokenType,
        end_tok: TokenType,
        f: fn(&mut Self) -> Result<R, Error>,
    ) -> Result<(Vec<R>, Location), Error> {
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

        // no items in between
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

    /// used to avoid a lambda in function and operator declarations
    fn parse_ident(&mut self) -> Result<Identifier, Error> {
        let cur = self.get_current().clone();
        let Token { val: TokenType::Identifier(name), loc } = cur else {
            return Err(Error {
                msg: ErrorType::ExpectedIdentifier,
                lines: vec![cur.loc],
            });
        };
        self.advance();
        Ok(Identifier { val: name, loc })
    }

    pub fn parse(&mut self) -> Result<Vec<LStmt>, Vec<Error>> {
        let mut ls = vec![];
        while !self.is_at_end()
            && !is_typ!(self, Eof)  // apparently needed
        {
            ls.push(match self.parse_statement() {
                Ok(s) => s,
                Err(err) => {
                    self.errs.push(err);
                    self.synchronize();
                    continue;
                }
            });
        }

        if !self.errs.is_empty() {
            Err(self.errs.clone())
        } else {
            Ok(ls)
        }
    }

    fn parse_statement(&mut self) -> Result<LStmt, Error> {
        let tok = self.get_current().clone();
        match tok.val {
            TokenType::Let => self.parse_var_decl(),
            TokenType::If => self.parse_if_else(),
            TokenType::While => self.parse_while(),
            TokenType::Fun => self.parse_fun(false),
            TokenType::Infixl | TokenType::Infixr => self.parse_operator(),
            TokenType::Struct => self.parse_struct(),
            TokenType::Impl => self.parse_impl(),
            TokenType::Continue => {
                self.advance();
                check_variant!(self, Semicolon, "Expected a semicolon \";\"")?;
                Ok(LStmt {
                    val: Stmt::Continue,
                    loc: tok.loc,
                })
            }
            TokenType::Break => {
                self.advance();
                check_variant!(self, Semicolon, "Expected a semicolon \";\"")?;
                Ok(LStmt {
                    val: Stmt::Break,
                    loc: tok.loc,
                })
            }
            TokenType::Return => {
                self.advance();
                let val = if is_typ!(self, Semicolon) {
                    // phantom value, the location is for the return statement
                    LExpr {
                        val: Expr::Unit,
                        loc: tok.loc,
                    }
                } else {
                    self.parse_expression()?
                };
                check_variant!(self, Semicolon, "Expected a semicolon \";\"")?;
                Ok(LStmt {
                    loc: Location {
                        start: tok.loc.start,
                        end: val.loc.end,
                    },
                    val: Stmt::Return(val),
                })
            }
            TokenType::LBrace => {
                let bl = self.parse_block()?;
                Ok(LStmt {
                    val: Stmt::Block(bl.val),
                    loc: bl.loc,
                })
            }
            _ => self.parse_assignment(),
        }
    }

    fn parse_block(&mut self) -> Result<Located<Vec<LStmt>>, Error> {
        // maybe can be changed into get + advance?
        let start = check_variant!(self, LBrace, "Expected { at the beginning of the block")?
            .loc
            .start;

        let mut ls = vec![];
        while !self.is_at_end()
            && !is_typ!(self, Eof)  // apparently needed
            && !is_typ!(self, RBrace)
        {
            ls.push(match self.parse_statement() {
                Ok(s) => s,
                Err(err) => {
                    self.errs.push(err);
                    self.synchronize();
                    continue;
                }
            });
        }
        let end = check_variant!(self, RBrace, "Expected } at the end of the block")?
            .loc
            .end;

        Ok(Located {
            val: ls,
            loc: Location { start, end },
        })
    }

    fn parse_var_decl(&mut self) -> Result<LStmt, Error> {
        let start = self.get_current().loc.start;
        self.advance();

        let name = self.parse_ident()?;

        check_variant!(self, Equals, "Expected an equals symbol")?;
        let expr = self.parse_expression()?;
        check_variant!(self, Semicolon, "Expected a semicolon \";\"")?;
        Ok(LStmt {
            loc: Location {
                start,
                end: expr.loc.end,
            },
            val: Stmt::VarDecl(name, expr),
        })
    }

    fn parse_if_else(&mut self) -> Result<LStmt, Error> {
        let start = self.get_current().loc.start;
        self.advance(); // move past keyword

        let mut blocks = vec![];

        let cond = self.parse_expression()?;
        let if_block = self.parse_block()?;

        blocks.push((cond, if_block.val));
        let mut end = if_block.loc.end;
        let mut els = None;
        while is_typ!(self, Else) {
            let _else_kw = self.get_current().clone();
            self.advance();

            let cond = if is_typ!(self, If) {
                self.advance();
                self.parse_expression()?
            } else {
                let bl = self.parse_block()?;
                end = bl.loc.end;
                els = Some(bl.val);
                break;
            };
            let if_block = self.parse_block()?;

            blocks.push((cond, if_block.val));
            end = if_block.loc.end;
        }

        Ok(LStmt {
            val: Stmt::If(blocks, els),
            loc: Location { start, end },
        })
    }

    fn parse_while(&mut self) -> Result<LStmt, Error> {
        let start = self.get_current().loc.start;
        self.advance(); // move past keyword
        let cond = self.parse_expression()?;
        let block = self.parse_block()?;

        Ok(LStmt {
            val: Stmt::While(cond, block.val),
            loc: Location {
                start,
                end: block.loc.end,
            },
        })
    }
    fn parse_fun(&mut self, force_operator: bool) -> Result<LStmt, Error> {
        let start = self.get_current().loc.start;
        self.advance(); // move past keyword

        let tok = self.get_current().clone();
        let (op, name) = match tok.val {
            TokenType::Identifier(name) => {
                if force_operator {
                    return Err(Error {
                        msg: ErrorType::InvalidOperatorname,
                        lines: vec![tok.loc],
                    });
                }
                (false, name)
            }
            TokenType::Symbol(name) => (true, name),
            _ => {
                return Err(Error {
                    msg: ErrorType::InvalidFunctionName,
                    lines: vec![tok.loc],
                })
            }
        };
        self.advance();

        let (params, _) = self.sep(TokenType::LParen, TokenType::RParen, Self::parse_ident)?;
        let block = self.parse_block()?;
        // TODO: horrible cheating, but eh
        if !op {
            Ok(LStmt {
                val: Stmt::FunDecl(
                    Identifier { val: name, loc: tok.loc },
                    params, block.val
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
            Ok(LStmt {
                val: Stmt::OperatorDecl(
                    Symbol {
                        val: name,
                        loc: tok.loc,
                    },
                    (param1.clone(), param2.clone()),
                    block.val,
                    Precedence {
                        prec: 0,
                        assoc: Associativity::Left,
                    },
                ),
                loc: Location { start, end: block.loc.end }
            })
        }
    }

    fn parse_operator(&mut self) -> Result<LStmt, Error> {
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
            TokenType::Int(n @ 0..=10) => n,
            TokenType::Int(n) => {
                return Err(Error {
                    msg: ErrorType::PrecedenceOutOfRange(n),
                    lines: vec![prec.loc],
                })
            }
            _ => {
                return Err(Error {
                    msg: ErrorType::InvalidPrecedence,
                    lines: vec![prec.loc],
                })
            }
        };
        self.advance();
        // because we set the flag we know it WILL be an operator
        // basically all we need to do is replace the associativity and starting location
        let LStmt {
            val: Stmt::OperatorDecl(name, params, block, _),
            loc,
        } = self.parse_fun(true)?
        else {
            unreachable!()
        };
        Ok(LStmt {
            val: Stmt::OperatorDecl(
                name,
                params,
                block,
                Precedence {
                    prec: prec2 as u8,
                    assoc,
                },
            ),
            loc: Location {
                start: kw.loc.start,
                end: loc.end,
            },
        })
    }

    fn parse_assignment(&mut self) -> Result<LStmt, Error> {
        let expr = self.parse_expression()?;
        // just an expression
        if !is_typ!(self, Equals) {
            check_variant!(self, Semicolon, "Expected a semicolon \";\"")?;
            return Ok(LStmt {
                loc: expr.loc,
                val: Stmt::Expr(expr),
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
        match expr.val {
            Expr::Identifier(ident) => Ok(LStmt {
                val: Stmt::Assign(
                    Identifier { val: ident, loc: expr.loc },
                    val
                ),
                loc,
            }),
            Expr::Index(ls, idx) => Ok(LStmt {
                val: Stmt::AssignIndex(*ls, *idx, val),
                loc,
            }),
            Expr::FieldAccess(expr, name) => Ok(LStmt {
                val: Stmt::AssignStruct(*expr, name, val),
                loc,
            }),
            _ => Err(Error {
                msg: ErrorType::InvalidAssignmentTarget,
                lines: vec![expr.loc],
            }),
        }
    }

    fn parse_struct(&mut self) -> Result<LStmt, Error> {
        let start = self.get_current().loc.start;
        self.advance();

        let name = self.parse_ident()?;

        // TODO: does not give ExpectedFieldName error
        let fields = self.sep(TokenType::LBrace, TokenType::RBrace, Self::parse_ident)?;

        Ok(LStmt {
            val: Stmt::Struct(name, fields.0),
            loc: Location { start, end: fields.1.end },
        })
    }

    fn parse_impl(&mut self) -> Result<LStmt, Error> {
        let start = self.get_current().loc.start;
        self.advance();

        let name = self.parse_ident()?;

        let block = self.parse_block()?;
        for s in block.val.iter() {
            if !matches!(s.val, Stmt::FunDecl(..)) {
                return Err(Error {
                    msg: ErrorType::NonFunStmtInImpl,
                    lines: vec![s.loc],
                });
            }
        }
        Ok(LStmt {
            val: Stmt::Impl(name, block.val),
            loc: Location { start, end: block.loc.end },
        })
    }

    fn parse_expression(&mut self) -> Result<LExpr, Error> {
        self.parse_binary()
    }

    fn parse_binary(&mut self) -> Result<LExpr, Error> {
        let left = self.parse_unary()?;
        // if it is a symbol, look for nested binary operator
        if let Token {
            val: TokenType::Symbol(sym_name),
            loc,
        } = self.get_current().clone()
        {
            self.advance();

            let right = self.parse_binary()?;
            Ok(LExpr {
                loc: Location {
                    start: left.loc.start,
                    end: right.loc.end,
                },
                val: Expr::BinaryOperation(left.into(), Symbol { val: sym_name, loc }, right.into()),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_unary(&mut self) -> Result<LExpr, Error> {
        let Token {
            val: TokenType::Symbol(sym),
            loc,
        } = self.get_current().clone()
        else {
            return self.parse_suffix();
        };
        if sym.as_str() == "||" {
            return self.parse_lambda(false);
        } else if !["-", "!"].contains(&sym.as_str()) {
            return Err(Error {
                msg: ErrorType::UnknownUnaryOperator,
                lines: vec![loc],
            });
        }
        self.advance();
        let expr = self.parse_unary()?;
        Ok(LExpr {
            loc: Location {
                start: loc.start,
                end: expr.loc.end,
            },
            val: Expr::UnaryOperation(Symbol { val: sym, loc }, expr.into()),
        })
    }

    /// things like function call and indexing
    fn parse_suffix(&mut self) -> Result<LExpr, Error> {
        let mut expr = self.parse_primary()?;
        let start = expr.loc.start;
        // a condition is not actually needed
        // the parser should figure out by itself
        loop {
            match self.get_current().val {
                TokenType::LParen => {
                    let (args, loc) = self.sep(TokenType::LParen, TokenType::RParen, Self::parse_expression)?;
                    expr = LExpr {
                        loc: Location { start, end: loc.end },
                        val: Expr::Call(expr.into(), args),
                    };
                }
                TokenType::LBracket => {
                    self.advance(); // move past the bracket
                    let idx = self.parse_expression()?;
                    let end = check_variant!(self, RBracket, "Expected closing bracket.")?.loc.end;
                    expr = LExpr {
                        loc: Location { start, end },
                        val: Expr::Index(expr.into(), idx.into()),
                    };
                }
                TokenType::Dot => {
                    self.advance();
                    let name = self.parse_ident()?;
                    expr = if is_typ!(self, LParen) {
                        // check if it is a method (needs special treatment)
                        let (params, end_loc) =
                            self.sep(TokenType::LParen, TokenType::RParen, Self::parse_expression)?;
                        LExpr {
                            loc: Location { start, end: end_loc.end },
                            val: Expr::MethodAccess(expr.into(), name, params),
                        }
                    } else {
                        LExpr {
                            loc: Location { start, end: name.loc.end },
                            val: Expr::FieldAccess(expr.into(), name),
                        }
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
    /// in that case it is marked with has_params: false and we do not need to match the symbol again
    fn parse_lambda(&mut self, has_params: bool) -> Result<LExpr, Error> {
        let start = self.get_current().loc.start;
        let params = if has_params {
            self.sep(TokenType::Pipe, TokenType::Pipe, Self::parse_ident)?.0
        } else {
            self.advance(); // go past the ||
            vec![]
        };
        // can be either a block or a single body
        let (body, end_loc) = if is_typ!(self, LBrace) {
            let block = self.parse_block()?;
            (block.val, block.loc.end)
        } else {
            let expr = self.parse_expression()?;
            let loc = expr.loc.end;
            // same as { return expr; }
            (
                vec![LStmt {
                    loc: expr.loc,
                    val: Stmt::Return(expr),
                }],
                loc,
            )
        };
        Ok(LExpr {
            val: Expr::Lambda(params, body),
            loc: Location { start, end: end_loc },
        })
    }

    fn parse_primary(&mut self) -> Result<LExpr, Error> {
        let tok = self.get_current().clone();
        let expr = match &tok.val {
            TokenType::String(s) => {
                self.advance();
                Expr::String(s.to_string())
            }
            TokenType::Int(n) => {
                self.advance();
                Expr::Int(*n)
            }
            TokenType::Float(n) => {
                self.advance();
                Expr::Float(*n)
            }
            TokenType::Identifier(ident) => {
                self.advance();
                Expr::Identifier(ident.clone())
            }
            TokenType::True => {
                self.advance();
                Expr::Bool(true)
            }
            TokenType::False => {
                self.advance();
                Expr::Bool(false)
            }
            TokenType::LParen => {
                self.advance();
                let next = self.get_current().clone();
                let val = match next.val {
                    TokenType::RParen => Expr::Unit,
                    TokenType::Symbol(sym)
                        if matches!(
                            self.peek(1),
                            Some(Token {
                                val: TokenType::RParen,
                                ..
                            })
                        ) =>
                    {
                        self.advance();
                        Expr::Identifier(sym)
                    }
                    _ => Expr::Parens(self.parse_expression()?.into()),
                };
                let end = check_variant!(self, RParen, "Expected a closing parenthesis")?.loc.end;
                return Ok(LExpr {
                    val,
                    loc: Location {
                        start: tok.loc.start,
                        end,
                    },
                });
            }
            TokenType::LBracket => {
                let (items, loc) = self.sep(TokenType::LBracket, TokenType::RBracket, Self::parse_expression)?;
                return Ok(LExpr {
                    loc,
                    val: Expr::List(items),
                });
            }
            TokenType::Eof => {
                return Err(Error {
                    msg: ErrorType::UnexpectedEof,
                    lines: vec![tok.loc],
                })
            }
            TokenType::Pipe => {
                return self.parse_lambda(true);
            }
            _ => {
                return Err(Error {
                    msg: ErrorType::UnknownElement(tok.val),
                    lines: vec![tok.loc],
                })
            }
        };
        Ok(LExpr {
            val: expr,
            loc: tok.loc,
        })
    }
}
