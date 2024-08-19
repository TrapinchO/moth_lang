use std::{mem, vec};

use crate::{
    associativity::{Associativity, Precedence},
    error::{Error, ErrorType},
    exprstmt::*,
    located::{Located, Location},
};
use super::token::{Token, TokenType};

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
        assert!(!(tokens.is_empty() || tokens.len() == 1 && tokens[0].val == TokenType::Eof),
                "Expected code to parse");
        Self { tokens, idx: 0 }
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
            self.tokens.len());

        &self.tokens[self.idx]
    }

    fn peek(&self, n: usize) -> Option<&Token> {
        if self.idx + n >= self.tokens.len() {
            None
        } else {
            Some(&self.tokens[self.idx+n])
        }
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
            TokenType::Struct => self.parse_struct(),
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
                let val = if is_typ!(self, Semicolon) {
                    // phantom value, the location is for the return statement
                    Expr {
                        val: ExprType::Unit,
                        loc: tok.loc,
                    }
                } else {
                    self.parse_expression()?
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
            TokenType::LBrace => {
                let bl = self.parse_block()?;
                Ok(Stmt {
                    val: StmtType::BlockStmt(bl.val),
                    loc: bl.loc,
                })
            },
            _ => self.parse_assignment(),
        }
    }

    fn parse_block(&mut self) -> Result<Located<Vec<Stmt>>, Error> {
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

        Ok(Located {
            val: ls,
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

        blocks.push((cond, if_block.val));
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

            blocks.push((cond, if_block.val));
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

        Ok(Stmt {
            val: StmtType::WhileStmt(cond, block.val),
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
            Self::parse_param,
        )?;
        let block = self.parse_block()?;
        // TODO: horrible cheating, but eh
        if !op {
            Ok(Stmt {
                val: StmtType::FunDeclStmt(
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
            Ok(Stmt {
                val: StmtType::OperatorDeclStmt(
                    Symbol { val: name, loc: tok.loc },
                    (param1.clone(), param2.clone()), block.val,
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

    fn parse_struct(&mut self) -> Result<Stmt, Error> {
        let start = self.get_current().loc.start;
        self.advance();
        let name_tok = self.get_current().clone();
        let TokenType::Identifier(name) = name_tok.val else {
            return Err(Error {
                msg: ErrorType::ExpectedStructName,
                lines: vec![name_tok.loc],
            });
        };
        self.advance(); // move past name
        let fields = self.sep(TokenType::LBrace, TokenType::RBrace, |s| {
            let tok = s.get_current().clone();
            if let TokenType::Identifier(ident) = tok.val {
                s.advance();
                Ok(Identifier { val: ident, loc: tok.loc })
            } else {
                Err(Error {
                    msg: ErrorType::ExpectedFieldName,
                    lines: vec![tok.loc]
                })
            }
        })?;
        Ok(Stmt {
            val: StmtType::StructStmt(Identifier { val: name, loc: name_tok.loc }, fields.0),
            loc: Location { start, end: fields.1.end },
        })
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
            return self.parse_lambda(false)
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
        let mut expr = self.parse_field()?;
        let start = expr.loc.start;
        // a condition is not actually needed
        // the parser should figure out by itself
        loop {
            match self.get_current().val {
                TokenType::LParen => {
                    let (args, loc) = self.sep(
                        TokenType::LParen, TokenType::RParen,
                        Self::parse_expression
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
    /// in that case it is marked with has_params: false and we do not need to match the symbol again
    fn parse_lambda(&mut self, has_params: bool) -> Result<Expr, Error> {
        let start = self.get_current().loc.start;
        let params = if !has_params {
            self.advance();
            vec![]
        } else {
            self.sep(
                TokenType::Pipe, TokenType::Pipe,
                Self::parse_param
            )?.0
        };
        // can be either a block or a single body
        let (body, end_loc) = if is_typ!(self, LBrace) {
            let block = self.parse_block()?;
            (block.val, block.loc.end)
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
            loc: Location { start, end: end_loc }
        })
    }

    fn parse_field(&mut self) -> Result<Expr, Error> {
        let mut expr = self.parse_primary()?;
        while is_typ!(self, Dot) {
            self.advance();
            let name_tok = self.get_current().clone();
            let TokenType::Identifier(name) = name_tok.val else {
                return Err(Error {
                    msg: ErrorType::ExpectedFieldName,
                    lines: vec![name_tok.loc],
                });
            };
            self.advance();
            expr = Expr {
                loc: Location { start: expr.loc.start, end: name_tok.loc.end },
                val: ExprType::FieldAccess(expr.into(), Identifier { val: name, loc: name_tok.loc }),
            };
        }
        Ok(expr)
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
                    TokenType::Symbol(sym) if matches!(self.peek(1), Some(Token { val: TokenType::RParen, .. })) => {
                        self.advance();
                        ExprType::Identifier(sym)
                    },
                    _ => {
                        ExprType::Parens(self.parse_expression()?.into())
                    }
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
                    Self::parse_expression
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
                return self.parse_lambda(true);
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
