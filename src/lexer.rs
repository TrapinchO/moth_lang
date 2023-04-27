use crate::error::Error;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    Number(i32),
    Identifier(String),
    Let,
    Fun,
    String(String),
    Eof,
    Symbol(String),
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Equals,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub typ: TokenType,
}

const SYMBOLS: &str = "+-*/=<>!|.$&@#?~";

pub fn lex(code: &str) -> Result<Vec<Token>, Error> {
    Lexer::new(code).lex()
}

struct Lexer {
    code: Vec<char>,
    idx: usize,
    start_pos: usize,
    pos: usize,
    line: usize,
}

impl Lexer {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.chars().collect(),
            idx: 0,
            start_pos: 0,
            pos: 0,
            line: 0,
        }
    }

    fn is_at_end(&self) -> bool {
        self.idx >= self.code.len()
    }

    fn get_current(&self) -> Result<char, String> {
        if self.is_at_end() {
            return Err("Attempted to index character out of bounds".to_string());
        }
        Ok(self.code[self.idx])
    }

    fn advance(&mut self) {
        self.idx += 1;
        self.pos += 1;
    }

    fn is_char(&self, character: char) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.get_current().unwrap() == character
        }
    }

    fn error(&self, msg: String) -> Error {
        Error {
            msg,
            line: self.line,
            start: self.start_pos,
            end: self.pos,
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = vec![];

        while !self.is_at_end() {
            self.start_pos = self.pos;
            let typ = match self.get_current()
                .expect(&format!("Lexer accessed an element beyond the character vector at index {}", self.pos)) {
                ' ' => {
                    self.advance();
                    continue;
                }
                '\n' => {
                    self.line += 1;
                    self.pos = 0;
                    self.idx += 1;
                    continue;
                }
                // TODO: floats, different bases
                num if num.is_digit(10) => match self.lex_number() {
                    Err(msg) => return Err(self.error(msg)),
                    Ok(num) => TokenType::Number(num),
                },
                ident if ident.is_alphanumeric() => {
                    let keywords: HashMap<&str, TokenType> = [
                        ("let", TokenType::Let),
                        ("fun", TokenType::Fun)
                    ].iter().cloned().collect();

                    let ident = self.lex_identifier();
                    // NOTE: I am absolutely not sure about the map stuff
                    // but it works, so... yeah...
                    match keywords.get(&ident.as_str()) {
                        Some(kw) => kw.clone(),
                        None => TokenType::Identifier(ident),
                    }
                }
                // TODO: tests:
                '(' => {self.advance(); TokenType::LParen},
                ')' => {self.advance(); TokenType::RParen},
                '[' => {self.advance(); TokenType::LBracket},
                ']' => {self.advance(); TokenType::RBracket},
                '{' => {self.advance(); TokenType::LBrace},
                '}' => {self.advance(); TokenType::RBrace},
                sym if SYMBOLS.contains(sym) => {
                    let sym = self.lex_symbol();
                    match sym.as_str() {
                        "=" => TokenType::Equals,
                        "/**/" => continue,
                        "/*" => {
                            // might come useful one day for documentation
                            match self.lex_block_comment() {
                                Err(msg) => return Err(self.error(msg)),
                                Ok(_) => continue,
                            }
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
                '\"' => match self.lex_string() {
                    Err(msg) => return Err(self.error(msg)),
                    Ok(string) => TokenType::String(string),
                },
                unknown => return Err(self.error(format!("Unknown character: \"{}\"", unknown))),
            };
            tokens.push(Token {
                start: self.start_pos,
                end: self.pos-1,  // the lexer is already moved
                line: self.line,
                typ,
            });
        }

        tokens.push(Token {
            start: self.pos,
            end: self.pos,
            line: self.line,
            typ: TokenType::Eof,
        });
        Ok(tokens)
    }

    fn lex_number(&mut self) -> Result<i32, String> {
        let mut num = String::from("");

        while !self.is_at_end() {
            let cur_char = self.get_current()?;
            if cur_char.is_digit(10) {
                num.push(cur_char);
            } else if cur_char.is_alphabetic() {
                return Err(format!("Invalid digit: \"{}\"", cur_char));
            } else {
                break;
            }
            self.advance();
        }
        Ok(num.parse::<i32>().unwrap())
    }

    fn lex_identifier(&mut self) -> String {
        let mut s = String::from("");

        while !self.is_at_end() {
            let cur_char = self.get_current()
                .expect(&format!("Lexer accessed an element beyond the character vector at index {}", self.pos));
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
            let cur_char = self.get_current()
                .expect(&format!("Lexer accessed an element beyond the character vector at index {}", self.pos));
            if !SYMBOLS.contains(cur_char) {
                break;
            }
            s.push(cur_char);
            self.advance();
        }
        s
    }

    fn lex_string(&mut self) -> Result<String, String> {
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
                return Err("EOL while parsing string".to_string());
            }
            s.push(self.get_current()?);
            self.advance();
        }
        Err("EOF while parsing string".to_string())
    }

    fn lex_line_comment(&mut self) {
        while !self.is_at_end() && !self.is_char('\n') {
            self.advance();
        }
    }

    // TODO: /*** ... */ and similar edge cases
    fn lex_block_comment(&mut self) -> Result<String, String> {
        let mut comment = String::new();
        while !self.is_at_end() {
            // necessary to get correct line and pos positions
            if self.is_char('\n') {
                self.advance();
                self.line += 1;
                self.pos = 0;
                continue;
            }

            if self.is_char('*') {
                if self.idx < self.code.len() - 1 && self.code[self.idx + 1] == '/' {
                    self.advance();
                    self.advance();
                    return Ok(comment);
                }
            }
            comment.push(self.get_current()?);
            self.advance();
        }
        Err("EOF while lexing block comment".to_string())
    }
}
