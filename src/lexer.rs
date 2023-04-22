use std::collections::HashMap;
use crate::error::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    Number(i32),
    Identifier(String),
    Let,
    Fun,
    String(String),
    Eof,
    Symbol(String),
    Equals,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub pos: usize,
    pub line: i32,
    pub typ: TokenType,
}

const SYMBOLS: &str = "+-*/=<>!|.$&@#";


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

// TODO: add helper methods
// TODO: decide on "get_current"
impl Lexer {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.chars().collect(),
            idx: 0,
            start_pos: 0,
            pos: 0,
            line: 1,  // TODO: make 0
        }
    }

    fn get_current(&self) -> Result<char, String> {
        if self.idx >= self.code.len() {
            return Err("Attempted to index character out of bounds".to_string());
        }
        Ok(self.code[self.idx])
    }

    fn advance(&mut self) {
        self.idx += 1;
        self.pos += 1;
    }
    
    fn error(&self, msg: String) -> Error {
        Error {
            msg,
            line: self.line,
            start: self.start_pos,
            pos: self.pos
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = vec![];

        while self.idx < self.code.len() {
            self.start_pos = self.pos;
            let typ = match self.get_current().unwrap() {
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
                num if num.is_digit(10) => {
                    match self.lex_number() {
                        Err(msg) => return Err(self.error(msg)),
                        Ok(num) => TokenType::Number(num)
                    }
                }
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
                sym if SYMBOLS.contains(sym) => {
                    let sym = self.lex_symbol();
                    match sym.as_str() {
                        "=" => TokenType::Equals,
                        "/*" => {
                            match self.lex_block_comment() {
                                Err(msg) => return Err(self.error(msg)),
                                Ok(comment) => continue,
                            }
                        }
                        // ignore comments
                        // IMPLEMENTATION DETAIL: "//-" is an operator, not a comment
                        // TODO: multiline comments
                        _ if sym.chars().all(|s| s == '/') && sym.len() >= 2 => {
                            self.lex_line_comment();
                            continue;
                        }
                        _ => TokenType::Symbol(sym),
                    }
                }
                // +1 to ignore the quote
                '\"' => {
                    match self.lex_string() {
                        Err(msg) => return Err(self.error(msg)),
                        Ok(string) => TokenType::String(string)
                    }
                }
                unknown => {
                    return Err(self.error(format!("Unknown character: \"{}\"", unknown)))
                }
            };
            tokens.push(Token {
                pos: self.start_pos,
                line: self.line as i32,  // TODO: finish usize x i32
                typ,
            });
        }

        tokens.push(Token {
            pos: self.pos,
            line: self.line as i32,
            typ: TokenType::Eof,
        });
        Ok(tokens)
    }

    fn lex_number(&mut self) -> Result<i32, String> {
        let mut num = String::from("");

        while self.idx < self.code.len() {
            if self.get_current()?.is_digit(10) {
                num.push(self.get_current().unwrap());
            } else if self.get_current()?.is_alphabetic() {
                return Err(format!(
                    "Invalid digit: \"{}\"",
                    self.get_current()?
                ));
            } else {
                break;
            }
            self.advance();
        }
        Ok(num.parse::<i32>().unwrap())
    }

    fn lex_identifier(&mut self) -> String {
        let mut s = String::from("");

        while self.idx < self.code.len() {
            if !self.get_current().unwrap().is_alphanumeric() {
                break;
            }
            s.push(self.get_current().unwrap());
            self.advance();
        }
        s
    }

    fn lex_symbol(&mut self) -> String {
        let mut s = String::from("");

        while self.idx < self.code.len() {
            if !SYMBOLS.contains(self.get_current().unwrap()) {
                break;
            }
            s.push(self.get_current().unwrap());
            self.advance();
        }
        s
    }

    fn lex_string(&mut self) -> Result<String, String> {
        let mut s = String::from("");

        // move behind the opening quote
        self.advance();
        while self.idx < self.code.len() {
            if self.get_current()? == '\"' {
                // move behind the quote
                self.advance();
                return Ok(s);
            }
            if self.get_current()? == '\n' {
                return Err("EOL while parsing string".to_string());
            }
            s.push(self.get_current()?);
            self.advance();
        }
        Err("EOF while parsing string".to_string())
    }

    fn lex_line_comment(&mut self) {
        while self.idx < self.code.len() && self.get_current() != Ok('\n') {
            println!("{:?}", self.get_current());
            self.advance();
        }
    }

    fn lex_block_comment(&mut self) -> Result<String, String> {
        let mut comment = String::new();
        while self.idx < self.code.len() {
            if self.get_current().unwrap() == '*' {
                if self.idx < self.code.len()-1 && self.code[self.idx+1] == '/' {
                    self.advance();
                    self.advance();
                    return Ok(comment);
                }
            }
            comment.push(self.get_current().unwrap());
            self.advance();
        }
        Err("EOF while lexing block comment".to_string())
    }
}
