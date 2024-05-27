use std::fmt::Display;

use crate::located::Located;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Int(i32),
    Float(f32),
    Identifier(String),
    True,
    False,
    Let,
    Fun,
    Return,
    If,
    Else,
    While,
    Break,
    Continue,
    Infixl,
    Infixr,
    String(String),
    // NOTE: EOF is needed as a buffer for some stuff in the parser
    // specifically for expressions, I think
    // probably not worth removing it, at least for now
    Eof,
    Symbol(String),
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Equals,
    QuestionMark,
    Semicolon,
    Dot,
    Comma,
}

impl TokenType {
    fn format(&self) -> String {
        match self {
            Self::Int(n) => n.to_string(),
            Self::Float(n) => n.to_string(),
            Self::String(s) => format!("\"{}\"", s),
            Self::Symbol(s) => s.to_string(),
            Self::Identifier(i) => i.to_string(),
            typ => format!("{:?}", typ),
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

pub type Token = Located<TokenType>;
