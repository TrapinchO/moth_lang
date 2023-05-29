use crate::error::Error;
use crate::token::*;
use std::collections::HashMap;


const SYMBOLS: &str = "+-*/=<>!|.$&@#?~^:%";

const KEYWORDS: [(&str, TokenType); 4] = [
    ("let", TokenType::Let),
    ("fun", TokenType::Fun),
    ("true", TokenType::True),
    ("false", TokenType::False),
];

const SPECIAL_SYMBOLS: [(char, TokenType); 7] = [
    ('(', TokenType::LParen),
    (')', TokenType::RParen),
    ('[', TokenType::LBracket),
    (']', TokenType::RBracket),
    ('{', TokenType::LBrace),
    ('}', TokenType::RBrace),
    (';', TokenType::Semicolon),
];

pub fn lex(code: &str) -> Result<Vec<Token>, Error> {
    Lexer::new(code).lex()
}

struct Lexer {
    code: Vec<char>,
    start_idx: usize,
    idx: usize,
}

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
            lines: vec![(self.start_idx, self.idx)],
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = vec![];

        while !self.is_at_end() {
            self.start_idx = self.idx;  // beginning to lex a new tokens
            let typ = match self.get_current() {
                ' ' | '\n' => {
                    self.advance();
                    continue;
                },
                num if num.is_ascii_digit() => {
                    // floats: should be anything that matches <number>.<number>
                    // no spaces, missing whole/decimal part
                    self.lex_number()?
                },
                ident if ident.is_alphanumeric() => {
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
                },
                sym if SYMBOLS.contains(sym) => {
                    let sym = self.lex_symbol();
                    match sym.as_str() {
                        "=" => TokenType::Equals,
                        "?" => TokenType::QuestionMark,
                        "." => TokenType::Dot,
                        "/**/" => continue,
                        "/*" => {
                            // might come useful one day for documentation
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
                // +1 to ignore the quote
                '\"' => TokenType::String(self.lex_string()?),
                unknown => return Err(self.error(format!("Unknown character: \"{}\"", unknown))),
            };
            tokens.push(Token {
                start: self.start_idx,
                end: self.idx - 1, // the lexer is already moved
                typ,
            });
        }

        tokens.push(Token {
            start: self.idx,
            end: self.idx,
            typ: TokenType::Eof,
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
                return Err(self.error(format!("Invalid digit: \"{}\"", cur_char)))
            }
            // check if the number is a float
            else if self.is_char('.') {
                if self.idx < self.code.len() - 1 && self.code[self.idx + 1].is_ascii_digit() {
                    
                    if is_float {
                        self.advance();  // for prettier error message
                        return Err(self.error("Found two floating point number delimiters".to_string()))
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
            TokenType::Float(num.parse::<f32>().unwrap())
        } else {
            TokenType::Int(num.parse::<i32>().unwrap())
        })
    }

    fn lex_identifier(&mut self) -> String {
        let mut s = String::from("");

        while !self.is_at_end() {
            let cur_char = self.get_current();
            if !cur_char.is_alphanumeric() {
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

    // TODO: /*** ... */ and similar edge cases
    fn lex_block_comment(&mut self) -> Result<String, Error> {
        let mut comment = String::new();
        while !self.is_at_end() {
            // necessary to get correct line and pos positions
            if self.is_char('\n') {
                self.advance();
                continue;
            }

            if self.is_char('*') && self.idx < self.code.len() - 1 && self.code[self.idx + 1] == '/' {
                self.advance();
                self.advance();
                return Ok(comment);
            }
            comment.push(self.get_current());
            self.advance();
        }
        Err(self.error("EOF while lexing block comment".to_string()))
    }
}
