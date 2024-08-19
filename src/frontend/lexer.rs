use crate::{
    error::{Error, ErrorType},
    located::{Located, Location},
};
use super::token::{Token, TokenType};

const SYMBOLS: &str = "+-*/=<>!|.$&@#?~^:%";

const KEYWORDS: [(&str, TokenType); 13] = [
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
    ("infixr", TokenType::Infixr),
    ("infixl", TokenType::Infixl),
    ("struct", TokenType::Struct),
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
];

pub fn lex(code: &str) -> Result<Vec<Token>, Vec<Error>> {
    let mut lexer = Lexer::new(code);
    match lexer.lex() {
        Ok(tok) => Ok(tok),
        Err(()) => Err(lexer.errs),
    }
}

struct Lexer {
    code: Vec<char>,
    start_idx: usize,
    idx: usize,
    errs: Vec<Error>,
}

// I know I could add the tokens in the functions themselves, but I like this more
impl Lexer {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.chars().collect(),
            start_idx: 0,
            idx: 0,
            errs: vec![],
        }
    }

    fn is_at_end(&self) -> bool {
        self.idx >= self.code.len()
    }

    fn get_current(&self) -> char {
        assert!(!self.is_at_end(),
                "Attempted to index character out of bounds: {}", self.idx);
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

    fn error(&self, msg: ErrorType) -> Error {
        Error {
            msg,
            lines: vec![Location {
                start: self.start_idx,
                end: self.idx,
            }],
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, ()> {
        let mut tokens = vec![];

        while !self.is_at_end() {
            self.start_idx = self.idx; // beginning to lex a new tokens
            let typ = match self.get_current() {
                ' ' | '\t' | '\n' => {
                    self.advance();
                    continue;
                }
                '\"' => {
                    match self.lex_string() {
                        Ok(s) => TokenType::String(s),
                        Err(err) => {
                            self.errs.push(err);
                            continue
                        }
                    }
                },
                num if num.is_ascii_digit() => {
                    // floats: should be anything that matches <number>.<number>
                    // no spaces, missing whole/decimal part
                    match self.lex_number() {
                        Ok(n) => n,
                        Err(err) => { self.errs.push(err); continue},
                    }
                }
                ident if ident.is_alphabetic() || ident == '_' => {
                    let ident = self.lex_identifier();
                    KEYWORDS
                        .iter()
                        .find(|x| x.0 == ident)
                        .unwrap_or(&("", TokenType::Identifier(ident)))
                        .clone()
                        .1
                }
                s if SPECIAL_SYMBOLS.map(|x| x.0).contains(&s) => {
                    self.advance();
                    SPECIAL_SYMBOLS.iter().find(|x| x.0 == s).unwrap().clone().1
                }
                sym if SYMBOLS.contains(sym) => {
                    let sym = self.lex_symbol();
                    match sym.as_str() {
                        "=" => TokenType::Equals,
                        "?" => TokenType::QuestionMark,
                        "." => TokenType::Dot,
                        "|" => TokenType::Pipe,
                        // TODO: error underlines the following symbol as well
                        _ if sym.ends_with("*/") && sym[..sym.len() - 2].chars().all(|s| s == '*') => {
                            self.errs.push(self.error(ErrorType::CommentSymbol));
                            continue
                        }
                        // it is a comment if it stars with /* and has only stars afterwards
                        _ if sym.starts_with("/*") && sym[2..].chars().all(|s| s == '*') => {
                            if let Err(err) = self.lex_block_comment() {
                                self.errs.push(err);
                            }
                            continue;
                        }
                        // ignore comments
                        // IMPLEMENTATION DETAIL: "//-" is an operator, not a comment
                        _ if sym.len() >= 2 && sym.chars().all(|s| s == '/') => {
                            self.lex_line_comment();
                            continue;
                        }
                        _ => TokenType::Symbol(sym),
                    }
                }
                unknown => {
                    self.errs.push(self.error(ErrorType::UnknownCharacter(unknown)));
                    self.advance();
                    continue
                }
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
        if self.errs.is_empty() {
            Ok(tokens)
        } else {
            // cannot move the errors out from this function without cloning
            // so just mark the error and get them from the outside
            Err(())
        }
    }

    fn lex_number(&mut self) -> Result<TokenType, Error> {
        let mut num = String::new();
        let mut is_float = false;

        while !self.is_at_end() {
            let cur_char = self.get_current();
            if cur_char.is_ascii_digit() {
                num.push(cur_char);
            } else if cur_char.is_alphabetic() {
                return Err(self.error(ErrorType::InvalidDigit(cur_char)));
            }
            // check if the number is a float
            else if self.is_char('.') {
                // if it is just a decimal point (and not a symbol)
                if self.idx < self.code.len() - 1 && !SYMBOLS.contains(self.code[self.idx + 1]) {
                    if is_float {
                        self.advance(); // for prettier error message
                        return Err(self.error(ErrorType::TwoDecimalPoints));
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
                    .map_err(|_| self.error(ErrorType::IntegerOverflow))?,
            )
        } else {
            TokenType::Int(
                num.parse::<i32>()
                    .map_err(|_| self.error(ErrorType::IntegerOverflow))?,
            )
        })
    }

    fn lex_identifier(&mut self) -> String {
        let mut s = String::new();

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
        let mut s = String::new();

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
        let mut s = String::new();

        // move behind the opening quote
        self.advance();
        while !self.is_at_end() {
            match self.get_current() {
                '\"' => {
                    // move behind the closing quote
                    self.advance();
                    return Ok(s);
                },
                '\n' => {
                    return Err(self.error(ErrorType::StringEol));
                },
                '\\' => {
                    self.advance();
                    if self.is_at_end() {
                        return Err(self.error(ErrorType::StringEof))
                    }
                    let escaped = match self.get_current() {
                        'n' => '\n',
                        't' => '\t',
                        '\"' => '\"',
                        '\'' => '\'',
                        '\\' => '\\',
                        c => {
                            // makes sure newlines etc. do not behave funny
                            let c = c.escape_debug().to_string();
                            return Err(self.error(ErrorType::InvalidEscapeChar(c)));
                        }
                    };
                    s.push(escaped);
                }
                c => {
                    s.push(c);
                }
            }
            self.advance();
        }
        Err(self.error(ErrorType::StringEof))
    }

    fn lex_line_comment(&mut self) {
        while !self.is_at_end() && !self.is_char('\n') {
            self.advance();
        }
    }

    fn lex_block_comment(&mut self) -> Result<(), Error> {
        let mut state = 0;
        while !self.is_at_end() {
            // could be the end of the comment
            if self.is_char('*') && state <= 1 {
                state = 1;
            } else if state == 1 && self.is_char('/') {
                state = 2;
            } else if state == 2 && !SYMBOLS.contains(self.get_current()) {
                //end of the comment
                self.advance();
                return Ok(());
            } else if state > 0 {
                // not the end of the comment actually
                state = 0;
            } else if SYMBOLS.contains(self.get_current()) {
                // if state is 0 (didnt find a star) and the char is a Symbol
                // set the state so that the following (potential) star is ignored
                // to prevent false positives like -*/
                state = 5;
            }
            self.advance();
        }
        // the comment end might be the last thing in the file
        if state == 2 {
            return Ok(());
        }
        Err(self.error(ErrorType::CommentEof))
    }
}
