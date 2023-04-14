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
    Equals,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub pos: usize,
    pub line: i32,
    pub typ: TokenType,
}

const SYMBOLS: &str = "+-*/=<>!|.$&@#";

pub struct Error {
    pub msg: String,
    pub line: i32,
    pub pos: usize,
}

struct Lexer {
    code: Vec<char>,
    idx: usize,
    pos: usize,
}

// TODO: add helper methods
// TODO: decide on "get_current"
impl Lexer {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.chars().collect(),
            idx: 0,
            pos: 0,
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

    pub fn lex(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = vec![];

        let mut line = 1;
        while self.idx < self.code.len() {
            let pos = self.pos;

            let typ = match self.get_current()? {
                ' ' => {
                    self.advance();
                    continue;
                }
                '\n' => {
                    line += 1;
                    self.pos = 0;
                    self.idx += 1;
                    continue;
                }
                num if num.is_digit(10) => {
                    let num = self.lex_number()?;
                    TokenType::Number(num)
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
                    // TODO: fix
                    let sym = self.lex_symbol();
                    match sym.as_str() {
                        "=" => TokenType::Equals,
                        // ignore comments
                        // IMPLEMENTATION DETAIL: "//-" is an operator, not a comment
                        _ if sym.chars().all(|s| s == '/') && sym.len() >= 2 => {
                            while self.idx < self.code.len() && self.get_current().unwrap() != '\n' {
                                self.advance();
                            }
                            continue;
                        }
                        _ => TokenType::Symbol(sym),
                    }
                }
                // +1 to ignore the quote
                '\"' => {
                    let string = self.lex_string()?;
                    TokenType::String(string)
                }
                unknown => {
                    return Err(format!(
                        "Unknown character: \"{}\" at pos {} on line {}",
                        unknown, self.pos, line
                    ))
                }
            };
            tokens.push(Token {
                pos,
                line,
                typ,
            });
        }

        tokens.push(Token {
            pos: self.pos,
            line,
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
            println!("{}, {}", self.idx, self.get_current().unwrap());
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
}

pub fn lex(code: &str) -> Result<Vec<Token>, Error> {
    match Lexer::new(code).lex() {
        Ok(tokens) => Ok(tokens),
        Err(msg) => Err(Error {
            msg,
            pos: 0,
            line: 0
        })
    }
}
