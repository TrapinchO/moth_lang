use crate::{
    error::Error,
    located::{Located, Location},
    token::*,
};

use std::collections::HashMap;

const SYMBOLS: &str = "+-*/=<>!|.$&@#?~^:%";

const KEYWORDS: [(&str, TokenType); 10] = [
    ("let", TokenType::Let),
    ("fun", TokenType::Fun),
    ("true", TokenType::True),
    ("false", TokenType::False),
    ("if", TokenType::If),
    ("else", TokenType::Else),
    ("while", TokenType::While),
    ("return", TokenType::Return),
    ("break", TokenType::Break),
    ("continue", TokenType::Continue),
];

const SPECIAL_SYMBOLS: [(char, TokenType); 8] = [
    ('(', TokenType::LParen),
    (')', TokenType::RParen),
    ('[', TokenType::LBracket),
    (']', TokenType::RBracket),
    ('{', TokenType::LBrace),
    ('}', TokenType::RBrace),
    (';', TokenType::Semicolon),
    (',', TokenType::Comma),
    // TODO: what with Dot?
];

pub fn lex(code: &str) -> Result<Vec<Token>, Error> {
    Lexer::new(code).lex()
}

struct Lexer {
    code: Vec<char>,
    start_idx: usize,
    idx: usize,
}

// I know I could add the tokens in the functions themselves, but I like this more
impl Lexer {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.chars().collect(),
            start_idx: 0,
            idx: 0,
        }
    }

    fn is_at_end(&self) -> bool {
        self.idx >= self.code.len()
    }

    fn get_current(&self) -> char {
        if self.is_at_end() {
            panic!("Attempted to index character out of bounds: {}", self.idx);
        }
        self.code[self.idx]
    }

    fn advance(&mut self) {
        self.idx += 1;
    }

    fn is_char(&self, character: char) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.get_current() == character
        }
    }

    fn error(&self, msg: String) -> Error {
        Error {
            msg,
            lines: vec![Location {
                start: self.start_idx,
                end: self.idx,
            }],
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = vec![];

        while !self.is_at_end() {
            self.start_idx = self.idx; // beginning to lex a new tokens
            let typ = match self.get_current() {
                ' ' | '\t' | '\n' => {
                    self.advance();
                    continue;
                }
                num if num.is_ascii_digit() => {
                    // floats: should be anything that matches <number>.<number>
                    // no spaces, missing whole/decimal part
                    self.lex_number()?
                }
                ident if ident.is_alphabetic() || ident == '_' => {
                    let ident = self.lex_identifier();
                    let keywords = HashMap::from(KEYWORDS);
                    match keywords.get(ident.as_str()) {
                        Some(kw) => kw.clone(),
                        None => TokenType::Identifier(ident),
                    }
                }
                s if SPECIAL_SYMBOLS.map(|x| x.0).contains(&s) => {
                    self.advance();
                    let special_symbols = HashMap::from(SPECIAL_SYMBOLS);
                    special_symbols.get(&s).unwrap().clone()
                }
                sym if SYMBOLS.contains(sym) => {
                    let sym = self.lex_symbol();
                    match sym.as_str() {
                        "=" => TokenType::Equals,
                        "?" => TokenType::QuestionMark,
                        "." => TokenType::Dot,
                        // it is a comment if it stars with /* and has only stars afterwards
                        _ if sym.starts_with("/*") && sym[2..].chars().all(|s| s == '*') => {
                            self.lex_block_comment()?;
                            continue;
                        }
                        // ignore comments
                        // IMPLEMENTATION DETAIL: "//-" is an operator, not a comment
                        _ if sym.chars().all(|s| s == '/') && sym.len() >= 2 => {
                            self.lex_line_comment();
                            continue;
                        }
                        _ => TokenType::Symbol(sym),
                    }
                }
                '\"' => TokenType::String(self.lex_string()?),
                unknown => return Err(self.error(format!("Unknown character: \"{unknown}\""))),
            };
            tokens.push(Located {
                val: typ,
                loc: Location {
                    start: self.start_idx,
                    end: self.idx - 1,
                },
            });
        }

        tokens.push(Located {
            val: TokenType::Eof,
            loc: Location {
                start: self.idx,
                end: self.idx,
            },
        });
        Ok(tokens)
    }

    fn lex_number(&mut self) -> Result<TokenType, Error> {
        let mut num = String::from("");
        let mut is_float = false;

        while !self.is_at_end() {
            let cur_char = self.get_current();
            if cur_char.is_ascii_digit() {
                num.push(cur_char);
            } else if cur_char.is_alphabetic() {
                return Err(self.error(format!("Invalid digit: \"{cur_char}\"")));
            }
            // check if the number is a float
            else if self.is_char('.') {
                // if it is just a decimal point (and not a symbol)
                if self.idx < self.code.len() - 1 && self.code[self.idx + 1].is_ascii_digit() {
                    if is_float {
                        self.advance(); // for prettier error message
                        return Err(self.error("Found two floating point number delimiters".to_string()));
                    }
                    is_float = true;
                    num.push('.');
                } else {
                    break;
                }
            } else {
                break;
            }
            self.advance();
        }
        Ok(if is_float {
            // TODO: overflows behaving funny
            TokenType::Float(
                num.parse::<f32>()
                    .map_err(|_| self.error("Integer overflow".to_string()))?,
            )
        } else {
            TokenType::Int(
                num.parse::<i32>()
                    .map_err(|_| self.error("Integer overflow".to_string()))?,
            )
        })
    }

    fn lex_identifier(&mut self) -> String {
        let mut s = String::from("");

        while !self.is_at_end() {
            let cur_char = self.get_current();
            if !(cur_char.is_alphanumeric() || cur_char == '_') {
                break;
            }
            s.push(cur_char);
            self.advance();
        }
        s
    }

    fn lex_symbol(&mut self) -> String {
        let mut s = String::from("");

        while !self.is_at_end() {
            let cur_char = self.get_current();
            if !SYMBOLS.contains(cur_char) {
                break;
            }
            s.push(cur_char);
            self.advance();
        }
        s
    }

    fn lex_string(&mut self) -> Result<String, Error> {
        let mut s = String::from("");

        // move behind the opening quote
        self.advance();
        while !self.is_at_end() {
            if self.is_char('\"') {
                // move behind the closing quote
                self.advance();
                return Ok(s);
            }
            if self.is_char('\n') {
                return Err(self.error("EOL while parsing string".to_string()));
            }
            s.push(self.get_current());
            self.advance();
        }
        Err(self.error("EOF while parsing string".to_string()))
    }

    fn lex_line_comment(&mut self) {
        while !self.is_at_end() && !self.is_char('\n') {
            self.advance();
        }
    }

    // TODO: stuff like "==*/" at the end
    fn lex_block_comment(&mut self) -> Result<(), Error> {
        while !self.is_at_end() {
            // could be the end of the comment
            // fun fact: clippy hates this, but it is more readable imo
            if self.is_char('*') {
                // at the end of the file
                if (self.idx == self.code.len() - 2 && self.code[self.idx + 1] == '/')
                    // otherwise must check whether it is not an operator instead (e.g. */*)
                    || (self.idx < self.code.len() - 2
                    && self.code[self.idx + 1] == '/'
                    && !SYMBOLS.contains(self.code[self.idx + 2]))
                {
                    self.advance();
                    self.advance();
                    return Ok(());
                }
            }
            self.advance();
        }
        Err(self.error("EOF while lexing block comment".to_string()))
    }
}
